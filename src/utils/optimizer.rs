use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasReport {
    pub size_bytes: usize,
    pub sha256: String,
    pub score: u32,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeResult {
    pub input_size_bytes: usize,
    pub output_size_bytes: usize,
    pub output_path: PathBuf,
    pub tool: String,
}

impl OptimizeResult {
    pub fn reduction_bytes(&self) -> isize {
        self.input_size_bytes as isize - self.output_size_bytes as isize
    }

    pub fn reduction_percent(&self) -> f64 {
        if self.input_size_bytes == 0 {
            0.0
        } else {
            self.reduction_bytes() as f64 / self.input_size_bytes as f64 * 100.0
        }
    }
}

pub fn analyze_wasm(path: &Path) -> Result<GasReport> {
    let bytes = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let sha256 = hex::encode(Sha256::digest(&bytes));
    let size = bytes.len();

    // Heuristics only: keep this lightweight and deterministic.
    let mut suggestions = Vec::new();
    if size > 500_000 {
        suggestions.push(
            "Wasm is large; consider stripping symbols and removing unused features.".to_string(),
        );
    }
    if bytes.windows(4).any(|w| w == b"panic") {
        suggestions.push(
            "Panic strings detected; consider `panic = \"abort\"` and removing verbose messages."
                .to_string(),
        );
    }
    if bytes.windows(7).any(|w| w == b"println") {
        suggestions.push("Debug printing detected; remove logs for production builds.".to_string());
    }

    // A simple, stable scoring function.
    let score = (1_000_000usize.saturating_sub(size)).min(1_000_000) as u32;

    Ok(GasReport {
        size_bytes: size,
        sha256,
        score,
        suggestions,
    })
}

pub fn optimize_wasm(input: &Path, output: &Path) -> Result<OptimizeResult> {
    let bytes = fs::read(input).with_context(|| format!("Failed to read {}", input.display()))?;

    if let Some(parent) = output.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
    }

    let tool = match run_external_optimizer(input, output) {
        Ok(tool) => tool,
        Err(_) => {
            fs::write(output, &bytes)
                .with_context(|| format!("Failed to write {}", output.display()))?;
            "copy-fallback".to_string()
        }
    };
    let output_size = fs::metadata(output)
        .with_context(|| format!("Failed to stat {}", output.display()))?
        .len() as usize;

    Ok(OptimizeResult {
        input_size_bytes: bytes.len(),
        output_size_bytes: output_size,
        output_path: output.to_path_buf(),
        tool,
    })
}

fn run_external_optimizer(input: &Path, output: &Path) -> Result<String> {
    let attempts: [(&str, Vec<String>); 3] = [
        (
            "soroban-optimize",
            vec![
                input.display().to_string(),
                "-o".to_string(),
                output.display().to_string(),
            ],
        ),
        (
            "soroban",
            vec![
                "contract".to_string(),
                "optimize".to_string(),
                "--wasm".to_string(),
                input.display().to_string(),
                "--wasm-out".to_string(),
                output.display().to_string(),
            ],
        ),
        (
            "stellar",
            vec![
                "contract".to_string(),
                "optimize".to_string(),
                "--wasm".to_string(),
                input.display().to_string(),
                "--wasm-out".to_string(),
                output.display().to_string(),
            ],
        ),
    ];

    let mut errors = Vec::new();
    for (program, args) in attempts {
        match Command::new(program).args(&args).output() {
            Ok(output_result) if output_result.status.success() => return Ok(program.to_string()),
            Ok(output_result) => {
                errors.push(format!(
                    "{} failed: {}",
                    program,
                    String::from_utf8_lossy(&output_result.stderr).trim()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                errors.push(format!("{} not found", program));
            }
            Err(error) => errors.push(format!("{} failed to start: {}", program, error)),
        }
    }

    anyhow::bail!(
        "No external Soroban optimizer completed successfully ({})",
        errors.join("; ")
    )
}
