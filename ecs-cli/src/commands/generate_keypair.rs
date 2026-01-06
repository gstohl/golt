//! Generate a keypair for a program

use anyhow::Result;
use std::process::Command;

use crate::config::GoltConfig;

pub fn run(name: &str) -> Result<()> {
    let (config, project_root) = GoltConfig::find_config()?;

    let keypair_dir = project_root.join(&config.project.keypairs_dir);
    std::fs::create_dir_all(&keypair_dir)?;

    let keypair_path = keypair_dir.join(format!("{}-keypair.json", name));

    if keypair_path.exists() {
        anyhow::bail!("Keypair already exists: {:?}", keypair_path);
    }

    println!("Generating keypair for: {}", name);

    let output = Command::new("solana-keygen")
        .args([
            "new",
            "-o",
            keypair_path.to_str().unwrap(),
            "--no-bip39-passphrase",
            "--force",
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to generate keypair: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Extract pubkey from output
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("pubkey:") {
            let pubkey = line.trim_start_matches("pubkey:").trim();
            println!("Generated keypair: {}", keypair_path.display());
            println!("Program ID: {}", pubkey);
            println!();
            println!("Add this to your golt.toml or use it in your lib.rs:");
            println!("  pinocchio_pubkey::declare_id!(\"{}\");", pubkey);
            return Ok(());
        }
    }

    // Try to get pubkey separately
    let pubkey_output = Command::new("solana-keygen")
        .args(["pubkey", keypair_path.to_str().unwrap()])
        .output()?;

    if pubkey_output.status.success() {
        let pubkey = String::from_utf8_lossy(&pubkey_output.stdout)
            .trim()
            .to_string();
        println!("Generated keypair: {}", keypair_path.display());
        println!("Program ID: {}", pubkey);
        println!();
        println!("Add this to your lib.rs:");
        println!("  pinocchio_pubkey::declare_id!(\"{}\");", pubkey);
    }

    Ok(())
}
