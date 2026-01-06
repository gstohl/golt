//! Project configuration

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};

/// Golt project configuration (golt.toml)
#[derive(Debug, Serialize, Deserialize)]
pub struct GoltConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub components: Vec<ComponentConfig>,
    #[serde(default)]
    pub systems: Vec<SystemConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub components_dir: String,
    #[serde(default)]
    pub systems_dir: String,
    #[serde(default)]
    pub keypairs_dir: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentConfig {
    pub name: String,
    pub seed: String,
    #[serde(default)]
    pub program_id: Option<String>,
    #[serde(default)]
    pub fields: Vec<FieldConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldConfig {
    pub name: String,
    pub field_type: String,
    #[serde(default)]
    pub is_bump: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemConfig {
    pub name: String,
    #[serde(default)]
    pub program_id: Option<String>,
}

impl GoltConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read golt.toml")?;
        toml::from_str(&content).context("Failed to parse golt.toml")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn find_config() -> Result<(Self, std::path::PathBuf)> {
        let mut current = std::env::current_dir()?;
        loop {
            let config_path = current.join("golt.toml");
            if config_path.exists() {
                let config = Self::load(&config_path)?;
                return Ok((config, current));
            }
            if !current.pop() {
                anyhow::bail!("No golt.toml found in current directory or parents");
            }
        }
    }
}

impl Default for GoltConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                name: "my-ecs-project".to_string(),
                version: "0.1.0".to_string(),
                components_dir: "programs/components".to_string(),
                systems_dir: "programs/systems".to_string(),
                keypairs_dir: "keypairs".to_string(),
            },
            components: vec![],
            systems: vec![],
        }
    }
}
