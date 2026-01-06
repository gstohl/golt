//! Initialize a new Golt project

use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::{GoltConfig, ProjectConfig};

pub fn run(name: &str) -> Result<()> {
    println!("Initializing new Golt project: {}", name);

    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create directory structure
    fs::create_dir_all(project_dir.join("programs/components"))?;
    fs::create_dir_all(project_dir.join("programs/systems"))?;
    fs::create_dir_all(project_dir.join("programs/core"))?;
    fs::create_dir_all(project_dir.join("keypairs"))?;
    fs::create_dir_all(project_dir.join("generated"))?;

    // Create golt.toml
    let config = GoltConfig {
        project: ProjectConfig {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            components_dir: "programs/components".to_string(),
            systems_dir: "programs/systems".to_string(),
            keypairs_dir: "keypairs".to_string(),
        },
        components: vec![],
        systems: vec![],
    };
    config.save(&project_dir.join("golt.toml"))?;

    // Create workspace Cargo.toml
    let cargo_toml = format!(
        r#"[workspace]
members = [
    "programs/core",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
pinocchio = "0.8"
pinocchio-pubkey = "0.2"
pinocchio-system = "0.2"
golt-runtime = {{ git = "https://github.com/gstohl/golt" }}
golt-macros = {{ git = "https://github.com/gstohl/golt" }}

# Dev dependencies
mollusk-svm = "0.0.12"
solana-sdk = "2.1"
"#
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create core crate
    create_core_crate(project_dir)?;

    // Create .gitignore
    let gitignore = r#"target/
*.so
keypairs/*.json
"#;
    fs::write(project_dir.join(".gitignore"), gitignore)?;

    println!("Created project structure:");
    println!("  {}/", name);
    println!("  ├── golt.toml");
    println!("  ├── Cargo.toml");
    println!("  ├── programs/");
    println!("  │   ├── core/");
    println!("  │   ├── components/");
    println!("  │   └── systems/");
    println!("  ├── keypairs/");
    println!("  └── generated/");
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  golt new component health");
    println!("  golt build");

    Ok(())
}

fn create_core_crate(project_dir: &Path) -> Result<()> {
    let core_dir = project_dir.join("programs/core");
    fs::create_dir_all(&core_dir.join("src"))?;

    // Cargo.toml
    let cargo_toml = r#"[package]
name = "ecs-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
pinocchio.workspace = true
pinocchio-pubkey.workspace = true

[lib]
crate-type = ["lib"]
"#;
    fs::write(core_dir.join("Cargo.toml"), cargo_toml)?;

    // src/lib.rs
    let lib_rs = r#"//! ECS Core - Shared types and utilities
//!
//! This crate is auto-managed by Golt. Seeds and discriminators
//! are updated when you create new components/systems.

pub use pinocchio;
pub use pinocchio_pubkey;

/// PDA Seeds for all components and systems
pub mod seeds {
    // Seeds are added here by `golt new component`
}

/// Discriminators for all components
pub mod discriminators {
    // Discriminators are added here by `golt new component`
}
"#;
    fs::write(core_dir.join("src/lib.rs"), lib_rs)?;

    Ok(())
}
