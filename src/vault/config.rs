use crate::options::PageSettings;
use crate::page::{Entry, Page};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    journals_folder: Option<String>,
    #[serde(flatten)]
    settings: PageSettings,
    #[serde(default)]
    event_files: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            journals_folder: None,
            settings: PageSettings::default(),
            event_files: vec!["events/recurring.md".to_owned()],
        }
    }
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum ConfigError {
    Unexisting,
    ReadingFile(anyhow::Error),
    Toml(toml::de::Error),
}

impl TryFrom<&Path> for Config {
    type Error = ConfigError;

    fn try_from(path: &Path) -> Result<Config, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::Unexisting);
        }

        let page = Page::try_from(path)?;
        let mut configs = Vec::<Config>::new();

        for entry in page.entries() {
            if let Entry::CodeBlock(block) = entry {
                if block.kind == "toml" {
                    configs.push(toml::from_str(&block.code)?);
                }
            }
        }

        Ok(configs
            .into_iter()
            .fold(Config::default(), |config_a, config_b| {
                config_a.merge(config_b)
            }))
    }
}

impl Config {
    pub fn new(path: &Path) -> Result<Self> {
        let mut config = match Self::try_from(path.join("journal-preparation-config.md").as_path())
        {
            Ok(config) => config,
            Err(ConfigError::Unexisting) => Self::default(),
            Err(e) => Err(e)?,
        };

        config.read_daily_notes_config(path)?;

        Ok(config)
    }

    fn merge(mut self, other: Self) -> Self {
        let journals_folder = self.journals_folder.or(other.journals_folder);
        let settings = PageSettings {
            day: self.settings.day.or(other.settings.day),
            week: self.settings.week.or(other.settings.week),
            month: self.settings.month.or(other.settings.month),
            year: self.settings.year.or(other.settings.year),
        };

        for file in other.event_files {
            if self.event_files.iter().all(|f| f != &file) {
                self.event_files.push(file);
            }
        }

        Self {
            journals_folder,
            settings,
            event_files: self.event_files,
        }
    }

    pub fn journals_folder(&self) -> Option<&str> {
        self.journals_folder.as_deref()
    }

    pub fn settings(&self) -> &PageSettings {
        &self.settings
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
            log::info!("Using journals folder {}", folder);
            self.journals_folder = Some(folder.to_owned());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use indoc::indoc;

    #[test]
    fn default() {
        let config = Config::default();
        assert!(config.journals_folder().is_none());
        assert!(config.settings.day.is_none());
        assert!(config.settings.week.is_none());
        assert!(config.settings.month.is_none());
        assert!(config.settings.year.is_none());
    }

    #[test]
    fn build_with_non_existing_path() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let config = Config::new(temp_dir.path())?;

        assert!(config.journals_folder().is_none());
        assert!(config.settings.day.is_none());
        assert!(config.settings.week.is_none());
        assert!(config.settings.month.is_none());
        assert!(config.settings.year.is_none());

        Ok(())
    }

    #[test]
    fn build_with_empty_preparation_config() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        std::fs::create_dir_all(temp_dir.path())?;

        let config = temp_dir.child("journal-preparation-config.md");
        config.write_str("")?;

        let config = Config::new(temp_dir.path())?;

        assert!(config.journals_folder().is_none());
        assert!(config.settings.day.is_none());
        assert!(config.settings.week.is_none());
        assert!(config.settings.month.is_none());
        assert!(config.settings.year.is_none());

        Ok(())
    }

    #[test]
    fn build_with_preparation_config() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        std::fs::create_dir_all(temp_dir.path())?;

        let config = temp_dir.child("journal-preparation-config.md");
        config.write_str(indoc! {r#"
            ```toml
            journals_folder = "Foo"
            event_files = ["Hello"]
            [day]
            day_of_week = true
            ```
        "#})?;

        let config = Config::new(temp_dir.path())?;

        assert_eq!(Some("Foo"), config.journals_folder());
        assert_eq!(
            vec!["events/recurring.md".to_owned(), "Hello".to_owned()],
            config.event_files
        );
        assert!(config.settings.day.is_some());
        assert!(config.settings.week.is_none());
        assert!(config.settings.month.is_none());
        assert!(config.settings.year.is_none());

        Ok(())
    }

    #[test]
    fn build_with_multiple_preparation_config() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        std::fs::create_dir_all(temp_dir.path())?;

        let config = temp_dir.child("journal-preparation-config.md");
        config.write_str(indoc! {r#"
            ```toml
            journals_folder = "Foo"
            event_files = ["Hello"]
            [day]
            day_of_week = true
            ```

            ```toml
            journals_folder = "Bar"
            event_files = [
                "World"
            ]
            [week]
            nav_link = true
            ```
        "#})?;

        let config = Config::new(temp_dir.path())?;
        println!("{config:?}");

        assert_eq!(Some("Foo"), config.journals_folder());
        assert_eq!(
            vec![
                "events/recurring.md".to_owned(),
                "Hello".to_owned(),
                "World".to_owned()
            ],
            config.event_files
        );
        assert!(config.settings.day.is_some());
        assert!(config.settings.week.is_some());
        assert!(config.settings.month.is_none());
        assert!(config.settings.year.is_none());

        Ok(())
    }

    #[test]
    fn daily_notes_folder() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let obsidian = temp_dir.child(".obsidian");
        std::fs::create_dir_all(obsidian.path())?;

        let config = obsidian.child("daily-notes.json");
        config.write_str(indoc! {r#"
            {
                "folder": "daily-notes/"
            }
        "#})?;

        let config = Config::new(temp_dir.path())?;
        assert_eq!(Some("daily-notes/"), config.journals_folder());

        Ok(())
    }
}
