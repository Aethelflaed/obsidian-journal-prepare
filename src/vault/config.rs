use crate::options::{day, month, week, year};
use crate::page::{CodeBlock, Entry, Page};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Default)]
pub struct Config {
    journals_folder: Option<String>,
    settings: Option<Settings>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub day: Option<day::Settings>,
    #[serde(default)]
    pub week: Option<week::Settings>,
    #[serde(default)]
    pub month: Option<month::Settings>,
    #[serde(default)]
    pub year: Option<year::Settings>,
}

impl Config {
    pub fn new(path: &Path) -> Result<Self> {
        let mut config = Config::default();

        config.read_daily_notes_config(path)?;
        config.read_config_path(&path.join("journal-automation.md"))?;

        Ok(config)
    }

    pub fn journals_folder(&self) -> Option<&str> {
        self.journals_folder.as_deref()
    }

    pub fn settings(&self) -> Option<&Settings> {
        self.settings.as_ref()
    }

    fn read_config_path(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let config = Page::try_from(path)?;

        for entry in config.content.content {
            if let Entry::CodeBlock(block) = entry {
                if block.kind.as_str() == "toml" {
                    self.read_config_block(block)?;
                }
            }
        }

        Ok(())
    }

    fn read_config_block(&mut self, block: CodeBlock) -> Result<()> {
        if block.kind != "toml" {
            anyhow::bail!("Not a toml block");
        }
        self.settings = Some(toml::from_str(&block.code)?);

        Ok(())
    }

    fn read_daily_notes_config(&mut self, path: &Path) -> Result<()> {
        let daily_notes_config = path.join(".obsidian").join("daily-notes.json");
        if !daily_notes_config.exists() {
            return Ok(());
        }

        let config = std::fs::read_to_string(&daily_notes_config)
            .with_context(|| format!("reading \"{}\"", daily_notes_config.display()))?;
        let config: Value = serde_json::from_str(&config)
            .with_context(|| format!("parsing \"{}\"", daily_notes_config.display()))?;

        if let Some(folder) = config["folder"].as_str() {
            log::info!("Using journals_folder {}", folder);
            self.journals_folder = Some(folder.to_owned());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn default() {
        let config = Config::default();
        assert_eq!(None, config.journals_folder());
    }

    #[test]
    fn build_with_non_existing_path() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let config = Config::new(temp_dir.path())?;
        assert_eq!(None, config.journals_folder());

        Ok(())
    }

    #[test]
    fn daily_notes_folder() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let obsidian = temp_dir.child(".obsidian");
        std::fs::create_dir_all(obsidian.path())?;

        let config = obsidian.child("daily-notes.json");
        config.write_str(
            r#"
            {
                "folder": "daily-notes/"
            }
            "#,
        )?;

        let config = Config::new(temp_dir.path())?;
        assert_eq!(Some("daily-notes/"), config.journals_folder());

        Ok(())
    }
}
