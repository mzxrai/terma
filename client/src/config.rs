use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
}

impl Config {
    fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".terma"))
    }

    fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    fn load() -> Result<Self> {
        let path = Self::config_file()?;
        if !path.exists() {
            return Err(anyhow::anyhow!("Config file does not exist"));
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;

        let path = Self::config_file()?;
        let contents = serde_json::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }
}

pub fn get_or_prompt_username() -> Result<String> {
    match Config::load() {
        Ok(config) => Ok(config.username),
        Err(_) => {
            // Prompt for username
            print!("Enter your username: ");
            io::stdout().flush()?;

            let mut username = String::new();
            io::stdin().read_line(&mut username)?;
            let username = username.trim().to_string();

            if username.is_empty() {
                return Err(anyhow::anyhow!("Username cannot be empty"));
            }

            // Save config
            let config = Config {
                username: username.clone(),
            };
            config.save()?;

            Ok(username)
        }
    }
}
