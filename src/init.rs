use clap::builder::Str;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// instead of [cup] -> [your_string]
    pub cup_pattern: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Target {
    pub name: String,
    pub tag: Tag,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tag {
    /// The repository location where releases can be found (e.g., "owner/repo" for GitHub)
    pub remote_tag: String,
    pub remote_type: Remote,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Remote {
    GitHub,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cup_pattern: "cup".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from TOML file
    pub fn create() -> Result<(), String> {
        let current_dir = env::current_dir().map_err(|e| e.to_string())?;
        let file_path = current_dir.join("cup.toml");
        if file_path.exists() {
            println!("Configuration exists already {}", file_path.display());
            Ok(())
        } else {
            let default_config = Config::default();
            let toml_string = toml::to_string_pretty(&default_config).map_err(|e| e.to_string())?;

            fs::write(&file_path, toml_string).map_err(|e| e.to_string())?;
            println!("Configuration saved {}", file_path.display());
            Ok(())
        }
    }

    pub fn load() -> Result<Self, String> {
        let current_dir = env::current_dir().map_err(|e| e.to_string())?;
        let file_path = current_dir.join("cup.toml");
        if !file_path.exists() {
            Err("Configuration does not exist".to_string())
        } else {
            let raw = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
            let c = toml::from_str(&raw).map_err(|e| e.to_string())?;
            Ok(c)
        }
    }
}

pub fn init() -> Result<(), String> {
    Config::create()?;
    Ok(())
}

pub fn load_config() -> Result<Config, String> {
    Config::load()
}
