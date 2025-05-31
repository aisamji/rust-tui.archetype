use clap::{Parser, Subcommand};
use git2::{Repository, Signature};
use semver::Version;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml_edit::{Document, Item};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development tasks for {{ project-name }}")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get version information for packages in the workspace
    Version {
        /// Specific packages to get versions for (defaults to all)
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Bump version level (major, minor, patch)
        #[arg(long)]
        bump: Option<String>,
    },
    /// Tag the current HEAD with the version
    Tag {
        /// Specific packages to tag (defaults to all)
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Also push the tag to remote
        #[arg(long)]
        push: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version { packages, bump } => {
            handle_version(packages, bump)?;
        }
        Commands::Tag { packages, push } => {
            handle_tag(packages, push)?;
        }
    }
    
    Ok(())
}

fn handle_version(packages: Vec<String>, bump: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_packages = find_workspace_packages()?;
    
    let target_packages = if packages.is_empty() {
        workspace_packages.keys().cloned().collect()
    } else {
        packages
    };

    if let Some(bump_level) = bump {
        bump_versions(&workspace_packages, &target_packages, &bump_level)?;
    } else {
        show_versions(&workspace_packages, &target_packages)?;
    }
    
    Ok(())
}

fn find_workspace_packages() -> Result<HashMap<String, (String, String)>, Box<dyn std::error::Error>> {
    let mut packages = HashMap::new();
    
    for entry in WalkDir::new(".").follow_links(true) {
        let entry = entry?;
        if entry.file_name() == "Cargo.toml" {
            let path = entry.path();
            let content = fs::read_to_string(path)?;
            let doc: Document = content.parse()?;
            
            if let Some(package) = doc.get("package") {
                if let (Some(name), Some(version)) = (
                    package.get("name").and_then(|n| n.as_str()),
                    package.get("version").and_then(|v| v.as_str())
                ) {
                    packages.insert(
                        name.to_string(),
                        (version.to_string(), path.to_string_lossy().to_string())
                    );
                }
            }
        }
    }
    
    Ok(packages)
}

fn show_versions(packages: &HashMap<String, (String, String)>, targets: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    for target in targets {
        if let Some((version, _)) = packages.get(target) {
            println!("{}: {}", target, version);
        } else {
            return Err(format!("Package '{}' not found", target).into());
        }
    }
    Ok(())
}

fn bump_versions(packages: &HashMap<String, (String, String)>, targets: &[String], bump_level: &str) -> Result<(), Box<dyn std::error::Error>> {
    for target in targets {
        if let Some((current_version, path)) = packages.get(target) {
            let mut version = Version::parse(current_version)?;
            
            match bump_level {
                "major" => {
                    version.major += 1;
                    version.minor = 0;
                    version.patch = 0;
                }
                "minor" => {
                    version.minor += 1;
                    version.patch = 0;
                }
                "patch" => {
                    version.patch += 1;
                }
                _ => {
                    return Err(format!("Invalid bump level: {}. Use major, minor, or patch", bump_level).into());
                }
            }
            
            update_cargo_toml(path, &version.to_string())?;
            println!("Bumped {} from {} to {}", target, current_version, version);
        } else {
            return Err(format!("Package '{}' not found", target).into());
        }
    }
    Ok(())
}

fn update_cargo_toml(path: &str, new_version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut doc: Document = content.parse()?;
    
    if let Some(package) = doc.get_mut("package") {
        if let Some(Item::Table(table)) = package.as_table_mut() {
            table["version"] = toml_edit::value(new_version);
            fs::write(path, doc.to_string())?;
        }
    }
    
    Ok(())
}

fn handle_tag(packages: Vec<String>, push: bool) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open(".")?;
    let workspace_packages = find_workspace_packages()?;
    
    let target_packages = if packages.is_empty() {
        workspace_packages.keys().cloned().collect()
    } else {
        packages
    };
    
    // Get HEAD commit
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    let signature = repo.signature()?;
    
    for target in target_packages {
        if let Some((version, _)) = workspace_packages.get(&target) {
            let tag_name = format!("{}-v{}", target, version);
            let tag_message = format!("Release {} {}", target, version);
            
            repo.tag(&tag_name, commit.as_object(), &signature, &tag_message, false)?;
            println!("Created tag: {}", tag_name);
            
            if push {
                let mut remote = repo.find_remote("origin")?;
                let refspec = format!("refs/tags/{}:refs/tags/{}", tag_name, tag_name);
                remote.push(&[&refspec], None)?;
                println!("Pushed tag {} to origin", tag_name);
            }
        } else {
            return Err(format!("Package '{}' not found", target).into());
        }
    }
    
    Ok(())
}
