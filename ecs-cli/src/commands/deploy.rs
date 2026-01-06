//! Deploy a program to Solana

use anyhow::{Context, Result};
use std::process::Command;

use crate::config::GoltConfig;

pub fn run(name: &str, url: &str, keypair: Option<&str>) -> Result<()> {
    let (mut config, project_root) = GoltConfig::find_config()?;

    // Find the program in components or systems
    let is_component = config.components.iter().any(|c| c.name == name);
    let is_system = config.systems.iter().any(|s| s.name == name);

    if !is_component && !is_system {
        anyhow::bail!(
            "Program '{}' not found in golt.toml. Available programs:\n  Components: {}\n  Systems: {}",
            name,
            config.components.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "),
            config.systems.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", ")
        );
    }

    // Construct paths
    let keypair_path = project_root
        .join(&config.project.keypairs_dir)
        .join(format!("{}-keypair.json", name));

    let so_file = project_root
        .join("target")
        .join("deploy")
        .join(format!("{}.so", name.replace("-", "_")));

    // Verify files exist
    if !keypair_path.exists() {
        anyhow::bail!(
            "Keypair not found: {:?}\nRun: golt generate keypair {}",
            keypair_path,
            name
        );
    }

    if !so_file.exists() {
        anyhow::bail!(
            "Program binary not found: {:?}\nRun: golt build --sbf",
            so_file
        );
    }

    println!("Deploying {} to {}", name, url);
    println!("  Program binary: {}", so_file.display());
    println!("  Program keypair: {}", keypair_path.display());

    // Build the deploy command
    let mut cmd = Command::new("solana");
    cmd.args([
        "program",
        "deploy",
        so_file.to_str().unwrap(),
        "--program-id",
        keypair_path.to_str().unwrap(),
        "--url",
        url,
    ]);

    // Add payer keypair if provided
    if let Some(kp) = keypair {
        cmd.args(["--keypair", kp]);
    }

    cmd.current_dir(&project_root);

    println!();
    println!("Running: solana program deploy ...");

    let output = cmd.output().context("Failed to run solana program deploy")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Deploy failed:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);

    // Extract program ID from keypair
    let pubkey_output = Command::new("solana-keygen")
        .args(["pubkey", keypair_path.to_str().unwrap()])
        .output()
        .context("Failed to get program ID from keypair")?;

    let program_id = if pubkey_output.status.success() {
        String::from_utf8_lossy(&pubkey_output.stdout)
            .trim()
            .to_string()
    } else {
        // Try to extract from deploy output
        stdout
            .lines()
            .find(|line| line.contains("Program Id:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };

    if program_id.is_empty() {
        println!("Warning: Could not extract program ID");
    } else {
        println!("Program ID: {}", program_id);

        // Update golt.toml with the program ID
        if is_component {
            if let Some(comp) = config.components.iter_mut().find(|c| c.name == name) {
                comp.program_id = Some(program_id.clone());
            }
        } else if is_system {
            if let Some(sys) = config.systems.iter_mut().find(|s| s.name == name) {
                sys.program_id = Some(program_id.clone());
            }
        }

        let config_path = project_root.join("golt.toml");
        config.save(&config_path).context("Failed to update golt.toml")?;
        println!("Updated golt.toml with program ID");
    }

    println!();
    println!("Deploy successful!");

    Ok(())
}
