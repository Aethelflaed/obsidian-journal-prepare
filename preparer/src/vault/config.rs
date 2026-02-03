use crate::options::PageSettings;
use crate::page::{Entry, Page};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use utils::events::Event;

#[derive(Debug)]
pub struct Config {
    path: PathBuf,
    journals_folder: Option<String>,
    settings: PageSettings,
    event_files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerdeConfig {
    #[serde(default)]
    journals_folder: Option<String>,
    #[serde(flatten)]
    settings: PageSettings,
    #[serde(default)]
    event_files: Vec<String>,
}

impl Default for SerdeConfig {
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
    ReadingFile(anyhow::Error),
    Toml(toml::de::Error),
}

impl TryFrom<PathBuf> for Config {
    type Error = ConfigError;

    fn try_from(path: PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok((path, SerdeConfig::default()).into());
        }

        let page = Page::try_from(path.join("journal-preparation-config.md").as_path())?;
        let mut configs = Vec::<SerdeConfig>::new();

        for entry in page.entries() {
            if let Entry::CodeBlock(block) = entry {
                if block.is_toml() {
                    configs.push(toml::from_str(block.code())?);
                }
            }
        }

        let merged_configs = configs
            .into_iter()
            .fold(SerdeConfig::default(), |config_a, config_b| {
                config_a.merge(config_b)
            });

        Ok((path, merged_configs).into())
    }
}

impl From<(PathBuf, SerdeConfig)> for Config {
    fn from(tuple: (PathBuf, SerdeConfig)) -> Self {
        Self {
            path: tuple.0,
            journals_folder: tuple.1.journals_folder,
            event_files: tuple.1.event_files,
            settings: tuple.1.settings,
        }
    }
}

impl Config {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut config = match Self::try_from(path) {
            Ok(config) => config,
            Err(e) => Err(e)?,
        };

        config.read_daily_notes_config()?;

        Ok(config)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn journals_folder(&self) -> Option<&str> {
        self.journals_folder.as_deref()
    }

    pub const fn settings(&self) -> &PageSettings {
        &self.settings
    }

    fn read_daily_notes_config(&mut self) -> Result<()> {
        let daily_notes_config = self.path.join(".obsidian").join("daily-notes.json");
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

    pub fn read_events(&self) -> Result<Vec<Event>> {
        let mut events = vec![];
        for event_file in &self.event_files {
            let event_page_path = self.path.join(event_file);
            if !event_page_path.exists() {
                log::info!("Event file not found: {event_file:?}");
                continue;
            }

            let event_page = Page::try_from(event_page_path.as_path())?;
            for entry in event_page.entries() {
                if let Entry::CodeBlock(block) = entry {
                    if block.is_toml() {
                        let event = block.try_into()?;
                        log::debug!("Event: {:?}", event);
                        events.push(event);
                    }
                }
            }
        }

        Ok(events)
    }
}

impl SerdeConfig {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use indoc::indoc;

    #[test]
    fn default() {
        let config = Config::from((PathBuf::new(), SerdeConfig::default()));
        assert!(config.journals_folder.is_none());
        assert!(config.settings.day.is_none());
        assert!(config.settings.week.is_none());
        assert!(config.settings.month.is_none());
        assert!(config.settings.year.is_none());
    }

    #[test]
    fn build_with_non_existing_path() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let config = Config::new(temp_dir.path().to_path_buf())?;

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

        let config = Config::new(temp_dir.path().to_path_buf())?;

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

        let config = Config::new(temp_dir.path().to_path_buf())?;

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

        let config = Config::new(temp_dir.path().to_path_buf())?;
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

        let config = Config::new(temp_dir.path().to_path_buf())?;
        assert_eq!(Some("daily-notes/"), config.journals_folder());

        Ok(())
    }
}
