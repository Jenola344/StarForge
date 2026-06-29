use crate::utils::{config, optimizer, print as p, profiler};
use anyhow::Result;
use clap::Subcommand;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, Color, Table};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum GasCommands {
    /// Analyze a compiled Soroban contract for gas/cpu opportunities
    Analyze {
        /// Path to the compiled wasm
        wasm: PathBuf,
        /// Network context (used for fee heuristics)
        #[arg(long)]
        network: Option<String>,
    },
    /// Emit an "optimized" wasm (lightweight, heuristic-based)
    Optimize {
        /// Path to the input wasm
        #[arg(long)]
        target: PathBuf,
        /// Output path for optimized wasm
        #[arg(long)]
        output: PathBuf,
    },
    /// Compare two wasm builds and diff estimated simulation costs
    Diff {
        /// Path to the baseline wasm
        old_wasm: PathBuf,
        /// Path to the candidate wasm
        new_wasm: PathBuf,
    },
}

pub async fn handle(cmd: GasCommands) -> Result<()> {
    match cmd {
        GasCommands::Analyze { wasm, network } => analyze(wasm, network),
        GasCommands::Optimize { target, output } => optimize(target, output),
        GasCommands::Diff { old_wasm, new_wasm } => diff(old_wasm, new_wasm),
    }
}

// ── helpers ────────────────────────────────────────────────────────────────

fn base_table() -> Table {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    t
}

fn header_cell(text: &str) -> Cell {
    Cell::new(text)
        .add_attribute(Attribute::Bold)
        .fg(Color::Cyan)
}

fn value_cell(text: &str) -> Cell {
    Cell::new(text)
}

fn good_cell(text: &str) -> Cell {
    Cell::new(text).fg(Color::Green)
}

fn warn_cell(text: &str) -> Cell {
    Cell::new(text).fg(Color::Yellow)
}

fn bad_cell(text: &str) -> Cell {
    Cell::new(text).fg(Color::Red)
}

fn estimate_simulation_cost(size_bytes: usize) -> u64 {
    2_000 + (size_bytes as u64 / 8)
}

// ── subcommands ────────────────────────────────────────────────────────────

fn analyze(wasm: PathBuf, network: Option<String>) -> Result<()> {
    config::validate_file_path(&wasm, Some("wasm"))?;

    let cfg = config::load()?;
    let network = network.unwrap_or(cfg.network);
    config::validate_network(&network)?;

    p::header("Gas & Compute Visualizer — Analyze");
    p::kv("Network", &network);
    p::kv("Wasm", &wasm.display().to_string());

    let t = profiler::Timer::start();
    let report = optimizer::analyze_wasm(&wasm)?;
    let elapsed = t.elapsed();

    let est_cost = estimate_simulation_cost(report.size_bytes);

    // ── Cost breakdown table ──────────────────────────────────────────────
    println!();
    let mut table = base_table();
    table.set_header(vec![
        header_cell("Metric"),
        header_cell("Value"),
    ]);
    table.add_row(vec![
        value_cell("WASM size (bytes)"),
        value_cell(&report.size_bytes.to_string()),
    ]);
    table.add_row(vec![
        value_cell("WASM size (KB)"),
        value_cell(&format!("{:.2} KB", report.size_bytes as f64 / 1024.0)),
    ]);
    table.add_row(vec![
        value_cell("SHA-256"),
        value_cell(&report.sha256),
    ]);
    table.add_row(vec![
        value_cell("Heuristic score"),
        if report.score >= 80 {
            good_cell(&report.score.to_string())
        } else if report.score >= 50 {
            warn_cell(&report.score.to_string())
        } else {
            bad_cell(&report.score.to_string())
        },
    ]);
    table.add_row(vec![
        value_cell("Est. simulation cost (stroops)"),
        value_cell(&est_cost.to_string()),
    ]);
    table.add_row(vec![
        value_cell("Est. ledger footprint reads"),
        value_cell(&format!("{}", report.size_bytes / 4096 + 1)),
    ]);
    table.add_row(vec![
        value_cell("Est. auth cost (stroops)"),
        value_cell(&format!("{}", est_cost / 10)),
    ]);
    table.add_row(vec![
        value_cell("Analysis duration"),
        value_cell(&format!("{:?}", elapsed)),
    ]);
    println!("{table}");

    // ── Suggestions ───────────────────────────────────────────────────────
    p::separator();
    p::kv_accent("Size (bytes)", &report.size_bytes.to_string());
    p::kv("SHA256", &report.sha256);
    p::kv("Heuristic score", &report.score.to_string());
    p::kv("Risk", &format!("{:?}", report.risk));
    p::kv(
        "Estimated CPU",
        &format!("{} instructions", report.gas.cpu_instructions),
    );
    p::kv("Estimated memory", &format!("{} bytes", report.gas.memory_bytes));
    p::kv("Estimated storage", &format!("{} bytes", report.gas.storage_bytes));
    p::kv("Estimated fee", &format!("{} stroops", report.gas.fee_stroops));
    p::kv("Host calls", &report.resources.host_calls.to_string());
    p::kv(
        "Control flow ops",
        &report.resources.control_flow_ops.to_string(),
    );
    if !report.suggestions.is_empty() {
        println!();
        p::info("Optimization suggestions:");
        let mut stbl = base_table();
        stbl.set_header(vec![header_cell("#"), header_cell("Suggestion")]);
        for (i, s) in report.suggestions.iter().enumerate() {
            stbl.add_row(vec![
                warn_cell(&(i + 1).to_string()),
                value_cell(s),
            ]);
        }
        println!("{stbl}");
    } else {
        println!();
        p::success("No optimization suggestions — contract looks lean.");
    }

    Ok(())
}

fn optimize(target: PathBuf, output: PathBuf) -> Result<()> {
    config::validate_file_path(&target, Some("wasm"))?;

    p::header("Gas & Compute Visualizer — Optimize");
    p::kv("Input", &target.display().to_string());
    p::kv("Output", &output.display().to_string());

    let t = profiler::Timer::start();
    let result = optimizer::optimize_wasm(&target, &output)?;
    let elapsed = t.elapsed();

    let old_cost = estimate_simulation_cost(result.input_size_bytes);
    let new_cost = estimate_simulation_cost(result.output_size_bytes);
    let cost_delta = new_cost as i64 - old_cost as i64;

    println!();
    let mut table = base_table();
    table.set_header(vec![
        header_cell("Metric"),
        header_cell("Before"),
        header_cell("After"),
        header_cell("Delta"),
    ]);
    table.add_row(vec![
        value_cell("Size (bytes)"),
        value_cell(&result.input_size_bytes.to_string()),
        value_cell(&result.output_size_bytes.to_string()),
        if result.reduction_bytes() > 0 {
            good_cell(&format!("-{} bytes", result.reduction_bytes()))
        } else {
            warn_cell("0 bytes")
        },
    ]);
    table.add_row(vec![
        value_cell("Size (KB)"),
        value_cell(&format!("{:.2}", result.input_size_bytes as f64 / 1024.0)),
        value_cell(&format!("{:.2}", result.output_size_bytes as f64 / 1024.0)),
        good_cell(&format!("{:+.2}%", result.reduction_percent())),
    ]);
    table.add_row(vec![
        value_cell("Est. sim cost (stroops)"),
        value_cell(&old_cost.to_string()),
        value_cell(&new_cost.to_string()),
        if cost_delta < 0 {
            good_cell(&format!("{:+}", cost_delta))
        } else {
            warn_cell(&format!("{:+}", cost_delta))
        },
    ]);
    table.add_row(vec![
        value_cell("Optimizer"),
        value_cell(&result.tool),
        value_cell("—"),
        value_cell("—"),
    ]);
    table.add_row(vec![
        value_cell("Duration"),
        value_cell(&format!("{:?}", elapsed)),
        value_cell("—"),
        value_cell("—"),
    ]);
    println!("{table}");

    println!();
    p::success("Optimization complete — output written successfully.");

    Ok(())
}

fn diff(old_wasm: PathBuf, new_wasm: PathBuf) -> Result<()> {
    config::validate_file_path(&old_wasm, Some("wasm"))?;
    config::validate_file_path(&new_wasm, Some("wasm"))?;

    p::header("Gas & Compute Visualizer — Diff");
    p::kv("Baseline", &old_wasm.display().to_string());
    p::kv("Candidate", &new_wasm.display().to_string());

    let mut profile = profiler::Profiler::start();
    let old_report = optimizer::analyze_wasm(&old_wasm)?;
    profile.mark("analyze_old");
    let new_report = optimizer::analyze_wasm(&new_wasm)?;
    profile.mark("analyze_new");

    let old_cost = estimate_simulation_cost(old_report.size_bytes);
    let new_cost = estimate_simulation_cost(new_report.size_bytes);
    let cost_delta = new_cost as i64 - old_cost as i64;
    let cost_pct = if old_cost == 0 {
        0.0
    } else {
        (cost_delta as f64 / old_cost as f64) * 100.0
    };

    let size_delta = new_report.size_bytes as i64 - old_report.size_bytes as i64;
    let size_pct = if old_report.size_bytes == 0 {
        0.0
    } else {
        (size_delta as f64 / old_report.size_bytes as f64) * 100.0
    };
    let comparison = optimizer::compare_gas_reports(&old_report, &new_report);

    let old_auth = old_cost / 10;
    let new_auth = new_cost / 10;
    let auth_delta = new_auth as i64 - old_auth as i64;

    let old_reads = old_report.size_bytes / 4096 + 1;
    let new_reads = new_report.size_bytes / 4096 + 1;
    let reads_delta = new_reads as i64 - old_reads as i64;

    println!();
    let mut table = base_table();
    table.set_header(vec![
        header_cell("Metric"),
        header_cell("Baseline"),
        header_cell("Candidate"),
        header_cell("Delta"),
        header_cell("Change %"),
    ]);

    // Size row
    table.add_row(vec![
        value_cell("WASM size (bytes)"),
        value_cell(&old_report.size_bytes.to_string()),
        value_cell(&new_report.size_bytes.to_string()),
        if size_delta <= 0 {
            good_cell(&format!("{:+}", size_delta))
        } else {
            bad_cell(&format!("{:+}", size_delta))
        },
        if size_pct <= 0.0 {
            good_cell(&format!("{:+.2}%", size_pct))
        } else {
            bad_cell(&format!("{:+.2}%", size_pct))
        },
    ]);

    // Sim cost row
    table.add_row(vec![
        value_cell("Est. sim cost (stroops)"),
        value_cell(&old_cost.to_string()),
        value_cell(&new_cost.to_string()),
        if cost_delta <= 0 {
            good_cell(&format!("{:+}", cost_delta))
        } else {
            bad_cell(&format!("{:+}", cost_delta))
        },
        if cost_pct <= 0.0 {
            good_cell(&format!("{:+.2}%", cost_pct))
        } else {
            bad_cell(&format!("{:+.2}%", cost_pct))
        },
    ]);

    // Auth cost row
    table.add_row(vec![
        value_cell("Est. auth cost (stroops)"),
        value_cell(&old_auth.to_string()),
        value_cell(&new_auth.to_string()),
        if auth_delta <= 0 {
            good_cell(&format!("{:+}", auth_delta))
        } else {
            bad_cell(&format!("{:+}", auth_delta))
        },
        value_cell("—"),
    ]);

    // Ledger reads row
    table.add_row(vec![
        value_cell("Est. ledger footprint reads"),
        value_cell(&old_reads.to_string()),
        value_cell(&new_reads.to_string()),
        if reads_delta <= 0 {
            good_cell(&format!("{:+}", reads_delta))
        } else {
            bad_cell(&format!("{:+}", reads_delta))
        },
        value_cell("—"),
    ]);

    // Heuristic score row
    table.add_row(vec![
        value_cell("Heuristic score"),
        value_cell(&old_report.score.to_string()),
        value_cell(&new_report.score.to_string()),
        if new_report.score >= old_report.score {
            good_cell(&format!("{:+}", new_report.score as i32 - old_report.score as i32))
    p::separator();
    p::kv("Old size (bytes)", &old_report.size_bytes.to_string());
    p::kv("New size (bytes)", &new_report.size_bytes.to_string());
    p::kv(
        "Old est. fee",
        &comparison.baseline_fee_stroops.to_string(),
    );
    p::kv(
        "New est. fee",
        &comparison.candidate_fee_stroops.to_string(),
    );
    p::kv(
        "Old est. CPU",
        &old_report.gas.cpu_instructions.to_string(),
    );
    p::kv(
        "New est. CPU",
        &new_report.gas.cpu_instructions.to_string(),
    );
    p::kv("Old risk", &format!("{:?}", old_report.risk));
    p::kv("New risk", &format!("{:?}", new_report.risk));
    p::kv(
        "Estimated delta",
        &format!(
            "{} ({:+.2}%)",
            if comparison.delta_stroops >= 0 {
                format!("+{}", comparison.delta_stroops)
            } else {
                comparison.delta_stroops.to_string()
            },
            comparison.delta_percent
        ),
    );
    p::kv(
        "Result",
        if comparison.delta_stroops < 0 {
            "Improved (lower estimated cost)"
        } else if comparison.regression {
            "Regressed (estimated fee increased by more than 5%)"
        } else if comparison.delta_stroops > 0 {
            "Regressed (higher estimated cost)"
        } else {
            bad_cell(&format!("{:+}", new_report.score as i32 - old_report.score as i32))
        },
        value_cell("—"),
    ]);

    println!("{table}");

    // ── Verdict ───────────────────────────────────────────────────────────
    println!();
    if cost_delta < 0 {
        p::success(&format!(
            "Candidate is BETTER — saves {} stroops ({:+.2}%)",
            cost_delta.abs(),
            cost_pct
        ));
    } else if cost_delta > 0 {
        p::warn(&format!(
            "Candidate REGRESSED — costs {} more stroops ({:+.2}%)",
            cost_delta,
            cost_pct
        ));
    } else {
        p::info("No change in estimated compute cost.");
    }

    // ── Profile table ─────────────────────────────────────────────────────
    println!();
    let mut ptbl = base_table();
    ptbl.set_header(vec![header_cell("Step"), header_cell("Elapsed")]);
    for point in profile.points() {
        ptbl.add_row(vec![
            value_cell(&point.label),
            value_cell(&format!("{:?}", point.elapsed)),
        ]);
    }
    ptbl.add_row(vec![
        value_cell("Total"),
        value_cell(&format!("{:?}", profile.total_elapsed())),
    ]);
    println!("{ptbl}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_simulation_cost_zero() {
        assert_eq!(estimate_simulation_cost(0), 2_000);
    }

    #[test]
    fn estimate_simulation_cost_nonzero() {
        // 8 bytes → 2000 + 1 = 2001
        assert_eq!(estimate_simulation_cost(8), 2_001);
    }

    #[test]
    fn estimate_simulation_cost_large() {
        // 80_000 bytes → 2000 + 10000 = 12000
        assert_eq!(estimate_simulation_cost(80_000), 12_000);
    }

    #[test]
    fn base_table_has_utf8_preset() {
        let table = base_table();
        // Just ensure it constructs without panic
        let _ = table.to_string();
    }
}
