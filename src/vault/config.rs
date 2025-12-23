use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Default)]
pub struct Config {
    pub journals_folder: Option<String>,
}

impl Config {
    pub fn new(path: &Path) -> Result<Self> {
        Ok(Config {
            journals_folder: read_daily_notes_config(path)?,
        })
    }

    pub fn journals_folder(&self) -> Option<&str> {
        self.journals_folder.as_deref()
    }
}

fn read_daily_notes_config(path: &Path) -> Result<Option<String>> {
    let daily_notes_config = path.join(".obsidian").join("daily-notes.json");
    if !daily_notes_config.exists() {
        return Ok(None);
    }

    let config = std::fs::read_to_string(&daily_notes_config)
        .with_context(|| format!("reading \"{}\"", daily_notes_config.display()))?;
    let config: Value = serde_json::from_str(&config)
        .with_context(|| format!("parsing \"{}\"", daily_notes_config.display()))?;

    if let Some(folder) = config["folder"].as_str() {
        log::info!("Using journals_folder {}", folder);
        return Ok(Some(folder.to_owned()));
    }

    Ok(None)
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
