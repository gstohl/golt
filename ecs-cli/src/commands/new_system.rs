//! Create a new system

use anyhow::Result;
use heck::{ToSnakeCase, ToUpperCamelCase};
use std::fs;

use crate::config::{GoltConfig, SystemConfig};
use crate::templates;

pub fn run(name: &str) -> Result<()> {
    let (mut config, project_root) = GoltConfig::find_config()?;

    let snake_name = name.to_snake_case();
    let pascal_name = name.to_upper_camel_case();

    println!("Creating system: {}", pascal_name);

    // Check if system already exists
    if config.systems.iter().any(|s| s.name == snake_name) {
        anyhow::bail!("System '{}' already exists", snake_name);
    }

    // Create system directory
    let system_dir = project_root
        .join(&config.project.systems_dir)
        .join(&snake_name);

    if system_dir.exists() {
        anyhow::bail!("Directory already exists: {:?}", system_dir);
    }

    fs::create_dir_all(system_dir.join("src"))?;

    // Generate Cargo.toml
    let cargo_toml = templates::system_cargo_toml(&snake_name);
    fs::write(system_dir.join("Cargo.toml"), cargo_toml)?;

    // Generate src/lib.rs
    let lib_rs = templates::system_lib_rs(&snake_name, &pascal_name);
    fs::write(system_dir.join("src/lib.rs"), lib_rs)?;

    // Generate src/instruction.rs
    let instruction_rs = templates::system_instruction_rs(&pascal_name);
    fs::write(system_dir.join("src/instruction.rs"), instruction_rs)?;

    // Generate src/processor.rs
    let processor_rs = templates::system_processor_rs(&pascal_name);
    fs::write(system_dir.join("src/processor.rs"), processor_rs)?;

    // Generate src/entrypoint.rs
    let entrypoint_rs = templates::component_entrypoint_rs(); // Same as component
    fs::write(system_dir.join("src/entrypoint.rs"), entrypoint_rs)?;

    // Generate src/error.rs
    let error_rs = templates::system_error_rs(&pascal_name);
    fs::write(system_dir.join("src/error.rs"), error_rs)?;

    // Add to config
    config.systems.push(SystemConfig {
        name: snake_name.clone(),
        program_id: None,
    });
    config.save(&project_root.join("golt.toml"))?;

    println!("Created system at: {}", system_dir.display());
    println!();
    println!("Next steps:");
    println!("  1. Edit {}/src/instruction.rs to define instructions", system_dir.display());
    println!("  2. Edit {}/src/processor.rs to implement logic", system_dir.display());
    println!("  3. Run `golt generate keypair {}` to generate a keypair", snake_name);
    println!("  4. Run `golt build` to build");

    Ok(())
}
