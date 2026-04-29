use crate::utils::{print as p, templates};
use anyhow::Result;
use clap::Subcommand;
use colored::*;
use dialoguer::{Confirm, Input};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// Search for templates in the marketplace
    Search {
        /// Search query (matches name, description, or tags)
        query: String,
        /// Filter by tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// List all available templates
    List,
    /// Show details of a specific template
    Show {
        /// Template name
        name: String,
    },
    /// Publish a template to the local marketplace
    Publish {
        /// Path to the template directory
        path: PathBuf,
        /// Template name
        #[arg(long)]
        name: Option<String>,
        /// Template description
        #[arg(long)]
        description: Option<String>,
        /// Author name
        #[arg(long)]
        author: Option<String>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        /// Version
        #[arg(long, default_value = "1.0.0")]
        version: String,
    },
    /// Remove a template from the local marketplace
    Remove {
        /// Template name
        name: String,
    },
    /// Initialize the template registry with example templates
    Init,
}

pub fn handle(cmd: TemplateCommands) -> Result<()> {
    match cmd {
        TemplateCommands::Search { query, tags } => search(query, tags),
        TemplateCommands::List => list(),
        TemplateCommands::Show { name } => show(name),
        TemplateCommands::Publish { path, name, description, author, tags, version } => {
            publish(path, name, description, author, tags, version)
        }
        TemplateCommands::Remove { name } => remove(name),
        TemplateCommands::Init => init(),
    }
}

fn search(query: String, tags: Option<String>) -> Result<()> {
    p::header("Template Marketplace — Search");
    p::kv("Query", &query);
    
    let tag_list = tags.as_ref().map(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });
    
    if let Some(ref tags) = tag_list {
        p::kv("Tags", &tags.join(", "));
    }
    
    println!();
    
    let results = templates::search_templates(&query, tag_list.as_deref())?;
    
    if results.is_empty() {
        p::info("No templates found matching your search.");
        p::info("Try: starforge template publish ./my-template");
        return Ok(());
    }
    
    p::separator();
    println!("  Found {} template(s):\n", results.len());
    
    for (i, tmpl) in results.iter().enumerate() {
        let verified = if tmpl.verified { " ✓".green() } else { "".normal() };
        println!("  {}. {}{}", i + 1, tmpl.name.cyan().bold(), verified);
        println!("     {}", tmpl.description.dimmed());
        println!("     {} • {} • {} downloads", 
            tmpl.version.yellow(),
            tmpl.author.dimmed(),
            tmpl.downloads
        );
        
        if !tmpl.tags.is_empty() {
            println!("     Tags: {}", tmpl.tags.join(", ").bright_black());
        }
        
        if i < results.len() - 1 {
            println!();
        }
    }
    
    p::separator();
    println!();
    p::info("Use a template:");
    println!("  {}", format!("starforge new contract my-project --template {} --from marketplace", 
        results[0].name).cyan());
    
    Ok(())
}

fn list() -> Result<()> {
    p::header("Template Marketplace — All Templates");
    
    let registry = templates::load_registry()?;
    
    if registry.templates.is_empty() {
        p::info("No templates available yet.");
        p::info("Initialize with example templates: starforge template init");
        p::info("Or publish your own: starforge template publish ./my-template");
        return Ok(());
    }
    
    p::separator();
    println!("  {} template(s) available:\n", registry.templates.len());
    
    for (i, tmpl) in registry.templates.iter().enumerate() {
        let verified = if tmpl.verified { " ✓".green() } else { "".normal() };
        println!("  {}. {}{}", i + 1, tmpl.name.cyan().bold(), verified);
        println!("     {}", tmpl.description.dimmed());
        println!("     {} • {} • {} downloads", 
            tmpl.version.yellow(),
            tmpl.author.dimmed(),
            tmpl.downloads
        );
        
        if !tmpl.tags.is_empty() {
            println!("     Tags: {}", tmpl.tags.join(", ").bright_black());
        }
        
        if i < registry.templates.len() - 1 {
            println!();
        }
    }
    
    p::separator();
    
    Ok(())
}

fn show(name: String) -> Result<()> {
    let template = templates::get_template(&name)?;
    
    p::header(&format!("Template: {}", template.name));
    p::separator();
    
    let verified = if template.verified { " ✓".green() } else { "".normal() };
    println!("  {}{}", template.name.cyan().bold(), verified);
    println!();
    
    p::kv("Description", &template.description);
    p::kv("Version", &template.version);
    p::kv("Author", &template.author);
    p::kv("Downloads", &template.downloads.to_string());
    
    if !template.tags.is_empty() {
        p::kv("Tags", &template.tags.join(", "));
    }
    
    println!();
    p::kv("Created", &template.created_at);
    p::kv("Updated", &template.updated_at);
    
    println!();
    match &template.source {
        templates::TemplateSource::Git { url, branch } => {
            p::kv("Source", "Git Repository");
            p::kv("URL", url);
            if let Some(b) = branch {
                p::kv("Branch", b);
            }
        }
        templates::TemplateSource::Local { path } => {
            p::kv("Source", "Local");
            p::kv("Path", path);
        }
        templates::TemplateSource::Builtin { id } => {
            p::kv("Source", "Built-in");
            p::kv("ID", id);
        }
    }
    
    p::separator();
    println!();
    p::info("Use this template:");
    println!("  {}", format!("starforge new contract my-project --template {} --from marketplace", 
        template.name).cyan());
    println!();
    
    Ok(())
}

fn publish(
    path: PathBuf,
    name: Option<String>,
    description: Option<String>,
    author: Option<String>,
    tags: Option<String>,
    version: String,
) -> Result<()> {
    p::header("Publish Template");
    
    if !path.exists() {
        anyhow::bail!("Template path does not exist: {}", path.display());
    }
    
    // Validate template structure
    p::step(1, 3, "Validating template structure...");
    templates::validate_template_structure(&path)?;
    
    // Get template name from path if not provided
    let template_name = name.unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-template")
            .to_string()
    });
    
    // Use interactive prompts if values not provided
    let description = description.unwrap_or_else(|| {
        dialoguer::Input::<String>::new()
            .with_prompt("Template description")
            .default("A Soroban smart contract template".to_string())
            .interact_text()
            .unwrap_or_else(|_| "A Soroban smart contract template".to_string())
    });
    
    let author = author.unwrap_or_else(|| {
        dialoguer::Input::<String>::new()
            .with_prompt("Author name")
            .default("Anonymous".to_string())
            .interact_text()
            .unwrap_or_else(|_| "Anonymous".to_string())
    });
    
    let tag_list = tags
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["soroban".to_string(), "contract".to_string()]);
    
    p::step(2, 3, "Copying template to local registry...");
    
    templates::publish_template(
        &path,
        template_name.clone(),
        description.clone(),
        author.clone(),
        tag_list.clone(),
        version.clone(),
    )?;
    
    p::step(3, 3, "Updating registry...");
    
    println!();
    p::success(&format!("Template '{}' published successfully!", template_name));
    p::separator();
    p::kv_accent("Name", &template_name);
    p::kv("Version", &version);
    p::kv("Author", &author);
    p::kv("Tags", &tag_list.join(", "));
    p::separator();
    println!();
    p::info("Others can now use your template:");
    println!("  {}", format!("starforge new contract my-project --template {} --from marketplace", 
        template_name).cyan());
    println!();
    
    Ok(())
}

fn remove(name: String) -> Result<()> {
    p::header("Remove Template");
    
    let template = templates::get_template(&name)?;
    
    p::kv("Template", &template.name);
    p::kv("Version", &template.version);
    println!();
    
    let confirm = dialoguer::Confirm::new()
        .with_prompt("Are you sure you want to remove this template?")
        .default(false)
        .interact()?;
    
    if !confirm {
        p::info("Cancelled.");
        return Ok(());
    }
    
    templates::remove_template(&name)?;
    
    // Also remove the local template directory if it's a local template
    if let templates::TemplateSource::Local { path } = &template.source {
        let template_path = std::path::Path::new(path);
        if template_path.exists() {
            std::fs::remove_dir_all(template_path).ok();
        }
    }
    
    println!();
    p::success(&format!("Template '{}' removed from registry", name));
    
    Ok(())
}

fn init() -> Result<()> {
    p::header("Initialize Template Registry");
    
    p::info("Adding example templates to the marketplace...");
    println!();
    
    let examples = vec![
        templates::TemplateEntry {
            name: "uniswap-v2".to_string(),
            version: "1.0.0".to_string(),
            description: "Uniswap V2 style automated market maker (AMM) DEX implementation".to_string(),
            author: "Stellar Community".to_string(),
            tags: vec!["defi".to_string(), "dex".to_string(), "amm".to_string(), "swap".to_string()],
            source: templates::TemplateSource::Git {
                url: "https://github.com/stellar/soroban-examples".to_string(),
                branch: Some("main".to_string()),
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            downloads: 0,
            verified: true,
        },
        templates::TemplateEntry {
            name: "lending-pool".to_string(),
            version: "1.0.0".to_string(),
            description: "Decentralized lending and borrowing protocol with collateralization".to_string(),
            author: "Stellar Community".to_string(),
            tags: vec!["defi".to_string(), "lending".to_string(), "borrowing".to_string()],
            source: templates::TemplateSource::Git {
                url: "https://github.com/stellar/soroban-examples".to_string(),
                branch: Some("main".to_string()),
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            downloads: 0,
            verified: true,
        },
        templates::TemplateEntry {
            name: "governance".to_string(),
            version: "1.0.0".to_string(),
            description: "DAO governance contract with proposal creation and voting mechanisms".to_string(),
            author: "Stellar Community".to_string(),
            tags: vec!["dao".to_string(), "governance".to_string(), "voting".to_string()],
            source: templates::TemplateSource::Git {
                url: "https://github.com/stellar/soroban-examples".to_string(),
                branch: Some("main".to_string()),
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            downloads: 0,
            verified: true,
        },
        templates::TemplateEntry {
            name: "multisig-wallet".to_string(),
            version: "1.0.0".to_string(),
            description: "Multi-signature wallet with threshold-based transaction approval".to_string(),
            author: "Stellar Community".to_string(),
            tags: vec!["wallet".to_string(), "multisig".to_string(), "security".to_string()],
            source: templates::TemplateSource::Git {
                url: "https://github.com/stellar/soroban-examples".to_string(),
                branch: Some("main".to_string()),
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            downloads: 0,
            verified: true,
        },
    ];
    
    for template in examples {
        templates::add_template(template.clone())?;
        println!("  ✓ Added: {}", template.name.cyan());
    }
    
    println!();
    p::success("Template registry initialized with {} example templates", examples.len());
    println!();
    p::info("Browse templates:");
    println!("  {}", "starforge template list".cyan());
    println!("  {}", "starforge template search defi".cyan());
    println!();
    
    Ok(())
}
