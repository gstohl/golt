//! Run tests for all programs

use anyhow::Result;
use std::process::Command;

use crate::config::GoltConfig;

pub fn run(target: Option<&str>) -> Result<()> {
    let (config, project_root) = GoltConfig::find_config()?;

    println!("Testing Golt project: {}", config.project.name);
    println!();

    // Build the cargo test command
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.current_dir(&project_root);

    // If a specific target is provided, filter to that package
    if let Some(name) = target {
        // Check if it matches a component or system
        let is_component = config.components.iter().any(|c| c.name == name);
        let is_system = config.systems.iter().any(|s| s.name == name);

        if is_component {
            println!("Running tests for component: {}", name);
        } else if is_system {
            println!("Running tests for system: {}", name);
        } else {
            println!("Running tests for: {}", name);
        }

        cmd.arg("-p").arg(name);
    } else {
        println!("Running tests for all programs...");
        println!();
        println!("Components:");
        for comp in &config.components {
            println!("  - {}", comp.name);
        }
        println!();
        println!("Systems:");
        for sys in &config.systems {
            println!("  - {}", sys.name);
        }
    }

    println!();

    let status = cmd.status()?;

    println!();

    if status.success() {
        println!("All tests passed!");
    } else {
        anyhow::bail!("Some tests failed");
    }

    Ok(())
}
