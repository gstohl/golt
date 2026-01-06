//! Build all programs

use anyhow::Result;
use std::process::Command;

use crate::config::GoltConfig;

pub fn run(sbf: bool) -> Result<()> {
    let (config, project_root) = GoltConfig::find_config()?;

    println!("Building Golt project: {}", config.project.name);

    let status = if sbf {
        // Use cargo-build-sbf directly for reliable SBF builds
        println!("Running cargo-build-sbf...");
        Command::new("cargo-build-sbf")
            .current_dir(&project_root)
            .status()?
    } else {
        Command::new("cargo")
            .arg("build")
            .current_dir(&project_root)
            .status()?
    };

    if !status.success() {
        anyhow::bail!("Build failed");
    }

    println!();
    println!("Build successful!");

    if sbf {
        println!();
        println!("Deploy with:");
        for comp in &config.components {
            let keypair = format!("{}/{}-keypair.json", config.project.keypairs_dir, comp.name);
            let so_file = format!("target/deploy/{}.so", comp.name.replace("-", "_"));
            println!(
                "  solana program deploy {} --program-id {} --url localhost",
                so_file, keypair
            );
        }
        for sys in &config.systems {
            let keypair = format!("{}/{}-keypair.json", config.project.keypairs_dir, sys.name);
            let so_file = format!("target/deploy/{}.so", sys.name.replace("-", "_"));
            println!(
                "  solana program deploy {} --program-id {} --url localhost",
                so_file, keypair
            );
        }
    }

    Ok(())
}
