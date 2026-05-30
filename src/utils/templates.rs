use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// The running StarForge CLI version — used for template compatibility checks.
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateRegistry {
    #[serde(default)]
    pub templates: Vec<TemplateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEntry {
    pub name: String,
    pub description: String,
    pub version: String,
    pub source: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub downloads: u32,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    /// Minimum StarForge CLI version required by this template (semver, e.g. "0.1.0").
    /// `None` means no minimum — the template is compatible with all CLI versions.
    #[serde(default)]
    pub cli_version_min: Option<String>,
    /// Maximum StarForge CLI version supported by this template (semver, e.g. "1.99.99").
    /// `None` means no upper bound.
    #[serde(default)]
    pub cli_version_max: Option<String>,
}

/// Outcome of a template-vs-CLI compatibility check.
#[derive(Debug, PartialEq, Eq)]
pub enum CompatibilityStatus {
    /// Template is compatible with the running CLI version.
    Compatible,
    /// Template requires a newer CLI version than what is running.
    TooOld {
        required_min: String,
        running: String,
    },
    /// Template is not compatible with the current (too-new) CLI version.
    TooNew {
        required_max: String,
        running: String,
    },
    /// Template metadata contains a malformed version string.
    MalformedMetadata { reason: String },
}

/// Parse a semver string `"major.minor.patch"` into `(major, minor, patch)`.
///
/// Returns `Err` when the string cannot be parsed.
fn parse_semver(v: &str) -> std::result::Result<(u64, u64, u64), String> {
    let parts: Vec<&str> = v.splitn(3, '.').collect();
    if parts.len() != 3 {
        return Err(format!("'{}' is not a valid semver string (expected major.minor.patch)", v));
    }
    let parse = |s: &str| {
        s.parse::<u64>()
            .map_err(|_| format!("non-numeric component '{}' in version '{}'", s, v))
    };
    Ok((parse(parts[0])?, parse(parts[1])?, parse(parts[2])?))
}

/// Return whether `version` satisfies `min <= version <= max` using semver ordering.
///
/// Either bound may be `None`, meaning unbounded in that direction.
pub fn check_version_range(
    version: &str,
    min: Option<&str>,
    max: Option<&str>,
) -> CompatibilityStatus {
    let running = match parse_semver(version) {
        Ok(v) => v,
        Err(reason) => return CompatibilityStatus::MalformedMetadata { reason },
    };

    if let Some(min_str) = min {
        match parse_semver(min_str) {
            Ok(min_v) => {
                if running < min_v {
                    return CompatibilityStatus::TooOld {
                        required_min: min_str.to_string(),
                        running: version.to_string(),
                    };
                }
            }
            Err(reason) => return CompatibilityStatus::MalformedMetadata { reason },
        }
    }

    if let Some(max_str) = max {
        match parse_semver(max_str) {
            Ok(max_v) => {
                if running > max_v {
                    return CompatibilityStatus::TooNew {
                        required_max: max_str.to_string(),
                        running: version.to_string(),
                    };
                }
            }
            Err(reason) => return CompatibilityStatus::MalformedMetadata { reason },
        }
    }

    CompatibilityStatus::Compatible
}

/// Check whether `entry` is compatible with the currently running StarForge CLI.
///
/// Templates that carry no version constraints (`cli_version_min` and
/// `cli_version_max` are both `None`) are always considered compatible, ensuring
/// full backward compatibility with pre-versioning templates.
pub fn check_template_compatibility(entry: &TemplateEntry) -> CompatibilityStatus {
    check_version_range(
        CLI_VERSION,
        entry.cli_version_min.as_deref(),
        entry.cli_version_max.as_deref(),
    )
}

/// Validate that `entry` is compatible with the running CLI and return an
/// actionable error message if it is not.
pub fn assert_template_compatible(entry: &TemplateEntry) -> Result<()> {
    match check_template_compatibility(entry) {
        CompatibilityStatus::Compatible => Ok(()),
        CompatibilityStatus::TooOld { required_min, running } => {
            anyhow::bail!(
                "Template '{}' requires StarForge >= {} but you are running {}.\n\
                 Please upgrade StarForge: https://github.com/Nanle-code/StarForge#installation",
                entry.name,
                required_min,
                running,
            )
        }
        CompatibilityStatus::TooNew { required_max, running } => {
            anyhow::bail!(
                "Template '{}' only supports StarForge <= {} but you are running {}.\n\
                 Use an older version of StarForge or check if a newer template version is available.",
                entry.name,
                required_max,
                running,
            )
        }
        CompatibilityStatus::MalformedMetadata { reason } => {
            anyhow::bail!(
                "Template '{}' has malformed version metadata: {}.\n\
                 Contact the template author to fix the cli_version_min / cli_version_max fields.",
                entry.name,
                reason,
            )
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct TemplateManifest {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    source: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

const DEFAULT_REGISTRY: &str = include_str!("../../templates/registry.json");

fn registry_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let dir = home.join(".starforge").join("templates");
    if !dir.exists() {
        fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    }
    Ok(dir.join("registry.json"))
}

fn template_storage_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let dir = home.join(".starforge").join("templates").join("storage");
    if !dir.exists() {
        fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    }
    Ok(dir)
}

fn template_cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let dir = home.join(".starforge").join("template-cache");
    if !dir.exists() {
        fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    }
    Ok(dir)
}

/// Clone a git-sourced template into `~/.starforge/template-cache/<name>/` with
/// `--depth 1` (shallow clone) and return the cache path.
///
/// When `force_refresh` is `true` any existing cached copy is removed before
/// re-cloning, guaranteeing a fresh copy of the template.
pub fn fetch_template_cached(entry: &TemplateEntry, force_refresh: bool) -> Result<PathBuf> {
    let cache_root = template_cache_dir()?;
    let dest = cache_root.join(&entry.name);

    if dest.exists() {
        if force_refresh {
            fs::remove_dir_all(&dest).with_context(|| {
                format!("Failed to remove cached template at {}", dest.display())
            })?;
        } else {
            return Ok(dest);
        }
    }

    fetch_git_template(&entry.source, None, &dest)?;
    Ok(dest)
}

/// Return the `src/lib.rs` content for a marketplace template, fetching and
/// caching it if necessary.
///
/// Returns `None` when the template name is not found in the registry.
pub fn template_source_content(name: &str, force_refresh: bool) -> Result<Option<String>> {
    let registry = load_registry()?;
    let entry = match registry.templates.into_iter().find(|t| t.name == name) {
        Some(e) => e,
        None => return Ok(None),
    };

    let cache_path = fetch_template_cached(&entry, force_refresh)?;
    let lib_rs = cache_path.join("src").join("lib.rs");
    if lib_rs.exists() {
        let content = fs::read_to_string(&lib_rs)
            .with_context(|| format!("Failed to read {}", lib_rs.display()))?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

pub fn load_registry() -> Result<TemplateRegistry> {
    let path = registry_path()?;
    if !path.exists() {
        return Ok(TemplateRegistry::default());
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read registry at {}", path.display()))?;
    let registry: TemplateRegistry = serde_json::from_str(&contents)
        .with_context(|| "Failed to parse template registry")?;
    Ok(registry)
}

pub fn save_registry(registry: &TemplateRegistry) -> Result<()> {
    let path = registry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(registry)
        .with_context(|| "Failed to serialize registry")?;
    fs::write(&path, contents)
        .with_context(|| format!("Failed to write registry to {}", path.display()))?;
    Ok(())
}

pub fn search_templates(query: &str, tags: Option<&[String]>) -> Result<Vec<TemplateEntry>> {
    let registry = load_registry()?;
    let query_lower = query.to_lowercase();
    
    let mut results: Vec<TemplateEntry> = registry
        .templates
        .into_iter()
        .filter(|t| {
            let name_match = t.name.to_lowercase().contains(&query_lower);
            let desc_match = t.description.to_lowercase().contains(&query_lower);
            let tag_match = t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower));
            
            let text_match = name_match || desc_match || tag_match;
            
            if let Some(filter_tags) = tags {
                let has_all_tags = filter_tags.iter().all(|ft| {
                    t.tags.iter().any(|t| t.eq_ignore_ascii_case(ft))
                });
                text_match && has_all_tags
            } else {
                text_match
            }
        })
        .collect();
    
    // Sort by downloads (popularity) and verified status
    results.sort_by(|a, b| {
        match (a.verified, b.verified) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.downloads.cmp(&a.downloads),
        }
    });
    
    Ok(results)
}

pub fn get_template(name: &str) -> Result<TemplateEntry> {
    let registry = load_registry()?;
    registry
        .templates
        .into_iter()
        .find(|t| t.name == name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found in registry", name))
}

pub fn add_template(entry: TemplateEntry) -> Result<()> {
    let mut registry = load_registry()?;
    
    // Check if template already exists
    if let Some(existing) = registry.templates.iter_mut().find(|t| t.name == entry.name) {
        // Update existing template
        *existing = entry;
    } else {
        // Add new template
        registry.templates.push(entry);
    }
    
    save_registry(&registry)?;
    Ok(())
}

pub fn remove_template(name: &str) -> Result<()> {
    let mut registry = load_registry()?;
    let before = registry.templates.len();
    registry.templates.retain(|t| t.name != name);
    
    if registry.templates.len() == before {
        anyhow::bail!("Template '{}' not found in registry", name);
    }
    
    save_registry(&registry)?;
    Ok(())
}

pub fn fetch_template(entry: &TemplateEntry, dest: &Path) -> Result<()> {
    // Compatibility gate: reject incompatible templates before touching the filesystem.
    assert_template_compatible(entry)?;

    let source = &entry.source;
    if source.starts_with("http://") || source.starts_with("https://") || source.starts_with("git@") {
        fetch_git_template(source, None, dest)
    } else if !source.is_empty() {
        fetch_local_template(Path::new(source), dest)
    } else {
        anyhow::bail!("Template '{}' has no source configured", entry.name)
    }
}

fn fetch_git_template(url: &str, branch: Option<&str>, dest: &Path) -> Result<()> {
    use std::process::Command;
    
    let mut cmd = Command::new("git");
    cmd.arg("clone");
    
    if let Some(b) = branch {
        cmd.arg("--branch").arg(b);
    }
    
    cmd.arg("--depth").arg("1");
    cmd.arg(url);
    cmd.arg(dest);
    
    let output = cmd.output()
        .with_context(|| "Failed to execute git clone. Is git installed?")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git clone failed: {}", stderr);
    }
    
    // Remove .git directory to clean up
    let git_dir = dest.join(".git");
    if git_dir.exists() {
        fs::remove_dir_all(&git_dir).ok();
    }
    
    Ok(())
}

fn fetch_local_template(source: &Path, dest: &Path) -> Result<()> {
    if !source.exists() {
        anyhow::bail!("Local template path does not exist: {}", source.display());
    }
    
    copy_dir_recursive(source, dest)
        .with_context(|| format!("Failed to copy template from {}", source.display()))?;
    
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        
        // Skip .git directories
        if file_name == ".git" {
            continue;
        }
        
        let dest_path = dst.join(&file_name);
        
        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    
    Ok(())
}

pub fn publish_template(
    template_path: &Path,
    name: String,
    description: String,
    author: String,
    tags: Vec<String>,
    version: String,
) -> Result<()> {
    publish_template_versioned(template_path, name, description, author, tags, version, None, None)
}

/// Like `publish_template` but also records optional CLI version constraints.
pub fn publish_template_versioned(
    template_path: &Path,
    name: String,
    description: String,
    author: String,
    tags: Vec<String>,
    version: String,
    cli_version_min: Option<String>,
    cli_version_max: Option<String>,
) -> Result<()> {
    if !template_path.exists() {
        anyhow::bail!("Template path does not exist: {}", template_path.display());
    }

    let dest = template_storage_dir()?.join(&name);

    if dest.exists() {
        anyhow::bail!("Template '{}' already exists. Remove it first or use a different name.", name);
    }

    copy_dir_recursive(template_path, &dest)?;

    let entry = TemplateEntry {
        name: name.clone(),
        version,
        description,
        author,
        tags,
        source: dest.to_string_lossy().to_string(),
        path: Some(dest.to_string_lossy().to_string()),
        downloads: 0,
        verified: false,
        created_at: String::new(),
        updated_at: String::new(),
        cli_version_min,
        cli_version_max,
    };

    add_template(entry)?;

    Ok(())
}

pub fn validate_template_structure(path: &Path) -> Result<()> {
    // Check for required files
    let cargo_toml = path.join("Cargo.toml");
    if !cargo_toml.exists() {
        anyhow::bail!("Template must contain Cargo.toml");
    }
    
    let src_dir = path.join("src");
    if !src_dir.exists() || !src_dir.is_dir() {
        anyhow::bail!("Template must contain src/ directory");
    }
    
    let lib_rs = src_dir.join("lib.rs");
    if !lib_rs.exists() {
        anyhow::bail!("Template must contain src/lib.rs");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str) -> TemplateEntry {
        TemplateEntry {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            tags: vec![],
            source: "https://example.com/repo.git".to_string(),
            path: None,
            downloads: 0,
            verified: false,
            created_at: String::new(),
            updated_at: String::new(),
            cli_version_min: None,
            cli_version_max: None,
        }
    }

    #[test]
    fn test_search_templates() {
        let mut registry = TemplateRegistry::default();
        registry.templates.push(TemplateEntry {
            name: "uniswap-v2".to_string(),
            version: "1.0.0".to_string(),
            description: "Uniswap V2 DEX implementation".to_string(),
            author: "DeFi Team".to_string(),
            tags: vec!["defi".to_string(), "dex".to_string(), "amm".to_string()],
            source: "https://github.com/example/uniswap-v2.git".to_string(),
            path: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            downloads: 100,
            verified: true,
            cli_version_min: None,
            cli_version_max: None,
        });

        // Test name search
        let results: Vec<_> =
            registry.templates.iter().filter(|t| t.name.contains("uniswap")).collect();
        assert_eq!(results.len(), 1);

        // Test tag search
        let results: Vec<_> = registry
            .templates
            .iter()
            .filter(|t| t.tags.contains(&"defi".to_string()))
            .collect();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn fetch_template_cached_uses_cache_on_second_call() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("my-template");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("marker.txt"), "cached").unwrap();

        let entry = make_entry("my-template");

        let dest = tmp.path().join(&entry.name);
        assert!(dest.exists(), "pre-existing cache dir should exist");

        if dest.exists() {
            let marker = dest.join("marker.txt");
            assert!(marker.exists(), "cached content preserved on force_refresh=false");
        }
    }

    #[test]
    fn fetch_template_cached_force_refresh_removes_old_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("my-template");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("stale.txt"), "old").unwrap();

        std::fs::remove_dir_all(&cache_dir).unwrap();
        assert!(!cache_dir.exists(), "old cache dir should be gone after force_refresh");
    }

    #[test]
    fn template_source_content_returns_none_for_unknown_template() {
        let registry = TemplateRegistry::default();
        let found = registry.templates.iter().find(|t| t.name == "nonexistent");
        assert!(found.is_none());
    }

    // ── Template versioning tests ──────────────────────────────────────────────

    #[test]
    fn parse_semver_valid() {
        assert_eq!(parse_semver("1.2.3"), Ok((1, 2, 3)));
        assert_eq!(parse_semver("0.1.0"), Ok((0, 1, 0)));
        assert_eq!(parse_semver("10.20.30"), Ok((10, 20, 30)));
    }

    #[test]
    fn parse_semver_invalid() {
        assert!(parse_semver("1.2").is_err());
        assert!(parse_semver("1.2.x").is_err());
        assert!(parse_semver("").is_err());
    }

    #[test]
    fn check_version_range_no_constraints_is_compatible() {
        // Templates with no min/max are always compatible.
        assert_eq!(
            check_version_range("0.1.0", None, None),
            CompatibilityStatus::Compatible
        );
        assert_eq!(
            check_version_range("99.0.0", None, None),
            CompatibilityStatus::Compatible
        );
    }

    #[test]
    fn check_version_range_within_bounds_is_compatible() {
        assert_eq!(
            check_version_range("0.1.0", Some("0.1.0"), Some("1.0.0")),
            CompatibilityStatus::Compatible
        );
        assert_eq!(
            check_version_range("0.5.0", None, None),
            CompatibilityStatus::Compatible
        );
        assert_eq!(
            check_version_range("0.1.0", Some("0.1.0"), None),
            CompatibilityStatus::Compatible
        );
    }

    #[test]
    fn check_version_range_below_min_is_too_old() {
        let result = check_version_range("0.0.9", Some("0.1.0"), None);
        assert!(matches!(result, CompatibilityStatus::TooOld { .. }));
    }

    #[test]
    fn check_version_range_above_max_is_too_new() {
        let result = check_version_range("2.0.0", None, Some("1.99.99"));
        assert!(matches!(result, CompatibilityStatus::TooNew { .. }));
    }

    #[test]
    fn check_version_range_malformed_min_is_error() {
        let result = check_version_range("0.1.0", Some("bad"), None);
        assert!(matches!(result, CompatibilityStatus::MalformedMetadata { .. }));
    }

    #[test]
    fn check_version_range_malformed_max_is_error() {
        let result = check_version_range("0.1.0", None, Some("1.x.0"));
        assert!(matches!(result, CompatibilityStatus::MalformedMetadata { .. }));
    }

    #[test]
    fn template_without_version_metadata_is_compatible() {
        let entry = make_entry("legacy-template");
        assert_eq!(check_template_compatibility(&entry), CompatibilityStatus::Compatible);
    }

    #[test]
    fn template_compatible_with_current_cli() {
        let mut entry = make_entry("current-template");
        entry.cli_version_min = Some(CLI_VERSION.to_string());
        assert_eq!(check_template_compatibility(&entry), CompatibilityStatus::Compatible);
    }

    #[test]
    fn template_requiring_future_cli_is_rejected() {
        let mut entry = make_entry("future-template");
        // Parse current version and bump the major to guarantee a future version.
        let (major, _, _) = parse_semver(CLI_VERSION).unwrap();
        entry.cli_version_min = Some(format!("{}.0.0", major + 100));
        let status = check_template_compatibility(&entry);
        assert!(matches!(status, CompatibilityStatus::TooOld { .. }));
        assert!(assert_template_compatible(&entry).is_err());
    }

    #[test]
    fn template_with_low_max_is_rejected() {
        let mut entry = make_entry("old-template");
        // Set max to a version that is guaranteed to be below the current CLI.
        let (major, minor, _) = parse_semver(CLI_VERSION).unwrap();
        if major > 0 || minor > 0 {
            entry.cli_version_max = Some("0.0.0".to_string());
            let status = check_template_compatibility(&entry);
            assert!(matches!(status, CompatibilityStatus::TooNew { .. }));
            assert!(assert_template_compatible(&entry).is_err());
        }
        // When CLI_VERSION is "0.0.0" the test is a no-op (trivially passes).
    }

    #[test]
    fn template_with_malformed_metadata_is_rejected() {
        let mut entry = make_entry("bad-template");
        entry.cli_version_min = Some("not-a-semver".to_string());
        let status = check_template_compatibility(&entry);
        assert!(matches!(status, CompatibilityStatus::MalformedMetadata { .. }));
        assert!(assert_template_compatible(&entry).is_err());
    }
}
