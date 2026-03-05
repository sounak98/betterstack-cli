pub mod schema;

use std::path::PathBuf;

use anyhow::{Context, Result};

use schema::ConfigFile;

pub struct FileConfigStore {
    path: PathBuf,
}

impl FileConfigStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("bs")
            .join("config.toml")
    }

    pub fn load(&self) -> Result<ConfigFile> {
        let contents = std::fs::read_to_string(&self.path).context("Failed to read config file")?;
        toml::from_str(&contents).context("Failed to parse config file")
    }

    pub fn save(&self, config: &ConfigFile) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        let contents = toml::to_string_pretty(config).context("Failed to serialize config")?;
        std::fs::write(&self.path, contents).context("Failed to write config file")?;
        Ok(())
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn path_display(&self) -> String {
        self.path.display().to_string()
    }
}
