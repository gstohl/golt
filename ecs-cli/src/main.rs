//! Golt CLI - ECS Framework for Solana
//!
//! Commands:
//! - `golt init` - Initialize a new Golt project
//! - `golt new component <name>` - Create a new component
//! - `golt new system <name>` - Create a new system
//! - `golt generate ts` - Generate TypeScript bindings
//! - `golt build` - Build all programs

use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod config;
mod generators;
mod templates;

#[derive(Parser)]
#[command(name = "golt")]
#[command(author = "gstohl")]
#[command(version = "0.1.0")]
#[command(about = "Golt - ECS Framework for Solana", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Golt project
    Init {
        /// Project name
        name: String,
    },

    /// Create new components or systems
    New {
        #[command(subcommand)]
        entity_type: NewCommands,
    },

    /// Generate code (TypeScript bindings, etc.)
    Generate {
        #[command(subcommand)]
        gen_type: GenerateCommands,
    },

    /// Build all programs
    Build {
        /// Build for SBF target
        #[arg(long, default_value = "true")]
        sbf: bool,
    },

    /// List all components and systems
    List,
}

#[derive(Subcommand)]
enum NewCommands {
    /// Create a new component
    Component {
        /// Component name (e.g., "health", "inventory")
        name: String,
        /// PDA seed (defaults to name)
        #[arg(long)]
        seed: Option<String>,
    },

    /// Create a new system
    System {
        /// System name (e.g., "combat", "movement")
        name: String,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate TypeScript bindings
    Ts {
        /// Output directory
        #[arg(short, long, default_value = "generated")]
        output: String,
    },

    /// Generate a keypair for a program
    Keypair {
        /// Program name
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => commands::init::run(&name),
        Commands::New { entity_type } => match entity_type {
            NewCommands::Component { name, seed } => {
                commands::new_component::run(&name, seed.as_deref())
            }
            NewCommands::System { name } => commands::new_system::run(&name),
        },
        Commands::Generate { gen_type } => match gen_type {
            GenerateCommands::Ts { output } => commands::generate_ts::run(&output),
            GenerateCommands::Keypair { name } => commands::generate_keypair::run(&name),
        },
        Commands::Build { sbf } => commands::build::run(sbf),
        Commands::List => commands::list::run(),
    }
}
