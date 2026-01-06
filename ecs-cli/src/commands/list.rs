//! List all components and systems

use anyhow::Result;

use crate::config::GoltConfig;

pub fn run() -> Result<()> {
    let (config, _) = GoltConfig::find_config()?;

    println!("Project: {}", config.project.name);
    println!();

    println!("Components ({}):", config.components.len());
    if config.components.is_empty() {
        println!("  (none)");
    } else {
        for comp in &config.components {
            let id = comp.program_id.as_deref().unwrap_or("(no keypair)");
            println!("  - {} (seed: {}) -> {}", comp.name, comp.seed, id);
        }
    }

    println!();
    println!("Systems ({}):", config.systems.len());
    if config.systems.is_empty() {
        println!("  (none)");
    } else {
        for sys in &config.systems {
            let id = sys.program_id.as_deref().unwrap_or("(no keypair)");
            println!("  - {} -> {}", sys.name, id);
        }
    }

    Ok(())
}
