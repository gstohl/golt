//! Create a new component

use anyhow::Result;
use heck::{ToSnakeCase, ToUpperCamelCase};
use std::fs;
use std::path::Path;

use crate::config::{ComponentConfig, GoltConfig};
use crate::templates;

pub fn run(name: &str, seed: Option<&str>) -> Result<()> {
    let (mut config, project_root) = GoltConfig::find_config()?;

    let snake_name = name.to_snake_case();
    let pascal_name = name.to_upper_camel_case();
    let seed = seed.unwrap_or(&snake_name);

    println!("Creating component: {} (seed: {})", pascal_name, seed);

    // Check if component already exists
    if config.components.iter().any(|c| c.name == snake_name) {
        anyhow::bail!("Component '{}' already exists", snake_name);
    }

    // Create component directory
    let component_dir = project_root
        .join(&config.project.components_dir)
        .join(&snake_name);

    if component_dir.exists() {
        anyhow::bail!("Directory already exists: {:?}", component_dir);
    }

    fs::create_dir_all(component_dir.join("src"))?;

    // Generate Cargo.toml
    let cargo_toml = templates::component_cargo_toml(&snake_name);
    fs::write(component_dir.join("Cargo.toml"), cargo_toml)?;

    // Generate src/lib.rs
    let lib_rs = templates::component_lib_rs(&snake_name, &pascal_name);
    fs::write(component_dir.join("src/lib.rs"), lib_rs)?;

    // Generate src/state.rs
    let state_rs = templates::component_state_rs(&snake_name, &pascal_name, seed);
    fs::write(component_dir.join("src/state.rs"), state_rs)?;

    // Generate src/instruction.rs
    let instruction_rs = templates::component_instruction_rs(&pascal_name);
    fs::write(component_dir.join("src/instruction.rs"), instruction_rs)?;

    // Generate src/processor.rs
    let processor_rs = templates::component_processor_rs(&snake_name, &pascal_name);
    fs::write(component_dir.join("src/processor.rs"), processor_rs)?;

    // Generate src/entrypoint.rs
    let entrypoint_rs = templates::component_entrypoint_rs();
    fs::write(component_dir.join("src/entrypoint.rs"), entrypoint_rs)?;

    // Generate src/error.rs
    let error_rs = templates::component_error_rs(&pascal_name);
    fs::write(component_dir.join("src/error.rs"), error_rs)?;

    // Add to config
    config.components.push(ComponentConfig {
        name: snake_name.clone(),
        seed: seed.to_string(),
        program_id: None,
        fields: vec![],
    });
    config.save(&project_root.join("golt.toml"))?;

    // Update workspace Cargo.toml
    update_workspace_members(&project_root, &config)?;

    // Update core seeds and discriminators
    update_core_lib(&project_root, &config)?;

    println!("Created component at: {}", component_dir.display());
    println!();
    println!("Next steps:");
    println!("  1. Edit {}/src/state.rs to define your component fields", component_dir.display());
    println!("  2. Edit {}/src/instruction.rs to define instructions", component_dir.display());
    println!("  3. Run `golt generate keypair {}` to generate a keypair", snake_name);
    println!("  4. Run `golt build` to build");

    Ok(())
}

fn update_workspace_members(project_root: &Path, config: &GoltConfig) -> Result<()> {
    let cargo_path = project_root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)?;

    // Parse existing members
    let lines: Vec<&str> = content.lines().collect();

    // Find members array
    let mut in_members = false;
    let mut members_end = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("members") {
            in_members = true;
        }
        if in_members && line.trim() == "]" {
            members_end = i;
            break;
        }
    }

    // Build new members list
    let mut members = vec!["\"programs/core\"".to_string()];
    for comp in &config.components {
        members.push(format!("\"{}/{}\"", config.project.components_dir, comp.name));
    }
    for sys in &config.systems {
        members.push(format!("\"{}/{}\"", config.project.systems_dir, sys.name));
    }

    // Rebuild the file
    let mut new_content = String::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("members") {
            new_content.push_str("members = [\n");
            for member in &members {
                new_content.push_str(&format!("    {},\n", member));
            }
            new_content.push_str("]\n");
            // Skip until end of members
            continue;
        }
        if in_members && i <= members_end {
            if line.trim() == "]" {
                in_members = false;
            }
            continue;
        }
        new_content.push_str(line);
        new_content.push('\n');
    }

    fs::write(cargo_path, new_content)?;
    Ok(())
}

fn update_core_lib(project_root: &Path, config: &GoltConfig) -> Result<()> {
    let lib_path = project_root.join("programs/core/src/lib.rs");

    let mut seeds = String::new();
    let mut discriminators = String::new();

    for comp in &config.components {
        let upper = comp.name.to_uppercase();
        seeds.push_str(&format!(
            "    pub const {}: &[u8] = b\"{}\";\n",
            upper, comp.seed
        ));

        // Create 8-byte discriminator padded with zeros
        let mut disc_bytes = [0u8; 8];
        let seed_bytes = comp.seed.as_bytes();
        let len = seed_bytes.len().min(8);
        disc_bytes[..len].copy_from_slice(&seed_bytes[..len]);

        discriminators.push_str(&format!(
            "    pub const {}: [u8; 8] = {:?};\n",
            upper, disc_bytes
        ));
    }

    let content = format!(
        r#"//! ECS Core - Shared types and utilities
//!
//! This crate is auto-managed by Golt. Seeds and discriminators
//! are updated when you create new components/systems.

pub use pinocchio;
pub use pinocchio_pubkey;

/// PDA Seeds for all components and systems
pub mod seeds {{
{seeds}}}

/// Discriminators for all components
pub mod discriminators {{
{discriminators}}}
"#
    );

    fs::write(lib_path, content)?;
    Ok(())
}
