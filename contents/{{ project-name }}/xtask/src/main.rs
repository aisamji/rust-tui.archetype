use anyhow::{Context, Result};
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
        /// Don't push the tag to remote
        #[arg(long)]
        no_push: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version { packages, bump } => {
            handle_version(packages, bump)?;
        }
        Commands::Tag { packages, no_push } => {
            handle_tag(packages, no_push)?;
        }
    }
    
    Ok(())
}

fn handle_version(packages: Vec<String>, bump: Option<String>) -> Result<()> {
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

fn find_workspace_packages() -> Result<HashMap<String, (String, String)>> {
    let mut packages = HashMap::new();
    
    for entry in WalkDir::new(".").follow_links(true) {
        let entry = entry.context("Failed to read directory entry")?;
        if entry.file_name() == "Cargo.toml" {
            let path = entry.path();
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read Cargo.toml at {}", path.display()))?;
            let doc: Document = content.parse()
                .with_context(|| format!("Failed to parse Cargo.toml at {}", path.display()))?;
            
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

fn show_versions(packages: &HashMap<String, (String, String)>, targets: &[String]) -> Result<()> {
    for target in targets {
        if let Some((version, _)) = packages.get(target) {
            println!("{}: {}", target, version);
        } else {
            anyhow::bail!("Package '{}' not found", target);
        }
    }
    Ok(())
}

fn bump_versions(packages: &HashMap<String, (String, String)>, targets: &[String], bump_level: &str) -> Result<()> {
    for target in targets {
        if let Some((current_version, path)) = packages.get(target) {
            let mut version = Version::parse(current_version)
                .with_context(|| format!("Invalid version format for {}: {}", target, current_version))?;
            
            match bump_level {
                "major" => {
                    version.major += 1;
                    version.minor = 0;
                    version.patch = 0;
                    version.pre = semver::Prerelease::EMPTY;
                }
                "minor" => {
                    version.minor += 1;
                    version.patch = 0;
                    version.pre = semver::Prerelease::EMPTY;
                }
                "patch" => {
                    version.patch += 1;
                    version.pre = semver::Prerelease::EMPTY;
                }
                "alpha" => {
                    if version.pre.is_empty() {
                        version.patch += 1;
                        version.pre = semver::Prerelease::new("alpha.1")?;
                    } else if version.pre.as_str().starts_with("alpha.") {
                        let alpha_num: u64 = version.pre.as_str()[6..].parse()
                            .with_context(|| format!("Invalid alpha version: {}", version.pre))?;
                        version.pre = semver::Prerelease::new(&format!("alpha.{}", alpha_num + 1))?;
                    } else {
                        anyhow::bail!("Cannot bump to alpha from {} prerelease", version.pre);
                    }
                }
                "beta" => {
                    if version.pre.is_empty() {
                        version.patch += 1;
                        version.pre = semver::Prerelease::new("beta.1")?;
                    } else if version.pre.as_str().starts_with("beta.") {
                        let beta_num: u64 = version.pre.as_str()[5..].parse()
                            .with_context(|| format!("Invalid beta version: {}", version.pre))?;
                        version.pre = semver::Prerelease::new(&format!("beta.{}", beta_num + 1))?;
                    } else if version.pre.as_str().starts_with("alpha.") {
                        version.pre = semver::Prerelease::new("beta.1")?;
                    } else {
                        anyhow::bail!("Cannot bump to beta from {} prerelease", version.pre);
                    }
                }
                "release" => {
                    if !version.pre.is_empty() {
                        version.pre = semver::Prerelease::EMPTY;
                    } else {
                        anyhow::bail!("Version {} is already a release version", version);
                    }
                }
                _ => {
                    anyhow::bail!("Invalid bump level: {}. Use major, minor, patch, alpha, beta, or release", bump_level);
                }
            }
            
            update_cargo_toml(path, &version.to_string())?;
            update_workspace_dependencies(target, &version.to_string(), &workspace_packages)?;
            println!("Bumped {} from {} to {}", target, current_version, version);
        } else {
            anyhow::bail!("Package '{}' not found", target);
        }
    }
    Ok(())
}

fn update_cargo_toml(path: &str, new_version: &str) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read Cargo.toml at {}", path))?;
    let mut doc: Document = content.parse()
        .with_context(|| format!("Failed to parse Cargo.toml at {}", path))?;
    
    if let Some(package) = doc.get_mut("package") {
        if let Some(Item::Table(table)) = package.as_table_mut() {
            table["version"] = toml_edit::value(new_version);
            fs::write(path, doc.to_string())
                .with_context(|| format!("Failed to write Cargo.toml at {}", path))?;
        }
    }
    
    Ok(())
}

fn update_workspace_dependencies(package_name: &str, new_version: &str, workspace_packages: &HashMap<String, (String, String)>) -> Result<()> {
    for (_, (_, path)) in workspace_packages {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(mut doc) = content.parse::<Document>() {
                let mut updated = false;
                
                let dependency_sections = ["dependencies", "dev-dependencies", "build-dependencies"];
                
                for section in dependency_sections {
                    if let Some(dep_table) = doc.get_mut(section)
                        .and_then(|deps| deps.as_table_mut())
                        .and_then(|deps_table| deps_table.get_mut(package_name))
                        .and_then(|dep| dep.as_table_mut())
                    {
                        if dep_table.contains_key("path") {
                            dep_table["version"] = toml_edit::value(new_version);
                            updated = true;
                        }
                    }
                }
                
                if updated {
                    fs::write(path, doc.to_string())
                        .with_context(|| format!("Failed to write updated dependencies to {}", path))?;
                    println!("Updated {} dependency in {}", package_name, path);
                }
            }
        }
    }
    Ok(())
}

fn handle_tag(packages: Vec<String>, no_push: bool) -> Result<()> {
    let repo = Repository::open(".")
        .context("Failed to open git repository")?;
    let workspace_packages = find_workspace_packages()?;
    
    let target_packages = if packages.is_empty() {
        workspace_packages.keys().cloned().collect()
    } else {
        packages
    };
    
    // Get HEAD commit
    let head = repo.head()
        .context("Failed to get HEAD reference")?;
    let commit = head.peel_to_commit()
        .context("Failed to get HEAD commit")?;
    let signature = repo.signature()
        .context("Failed to get git signature")?;
    
    for target in target_packages {
        if let Some((version, _)) = workspace_packages.get(&target) {
            let tag_name = format!("{}-v{}", target, version);
            let tag_message = format!("Release {} {}", target, version);
            
            repo.tag(&tag_name, commit.as_object(), &signature, &tag_message, false)
                .with_context(|| format!("Failed to create tag {}", tag_name))?;
            println!("Created tag: {}", tag_name);
            
            if !no_push {
                let mut remote = repo.find_remote("origin")
                    .context("Failed to find 'origin' remote")?;
                let refspec = format!("refs/tags/{}:refs/tags/{}", tag_name, tag_name);
                remote.push(&[&refspec], None)
                    .with_context(|| format!("Failed to push tag {} to origin", tag_name))?;
                println!("Pushed tag {} to origin", tag_name);
            }
        } else {
            anyhow::bail!("Package '{}' not found", target);
        }
    }
    
    Ok(())
}
