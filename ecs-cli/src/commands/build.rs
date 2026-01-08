//! Build all programs

use anyhow::Result;
use std::process::{Command, Stdio};

use crate::config::GoltConfig;

pub fn run(sbf: bool) -> Result<()> {
    let (config, project_root) = GoltConfig::find_config()?;

    println!("Building Golt project: {}", config.project.name);

    let status = if sbf {
        println!("Running cargo build-sbf...");
        println!();

        // Run through shell to ensure proper environment setup
        Command::new("sh")
            .arg("-c")
            .arg("cargo build-sbf")
            .current_dir(&project_root)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?
    } else {
        Command::new("cargo")
            .arg("build")
            .current_dir(&project_root)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
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
