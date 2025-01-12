use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;

#[derive(derive_more::Debug)]
#[debug("Vault({path:?}, {journal_path:?})")]
pub struct Vault {
    path: PathBuf,
    journal_path: Option<PathBuf>,
}

impl Vault {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            std::fs::create_dir_all(path.as_path())
                .with_context(|| format!("creating dir {:?}", path))?;
        }
        let mut vault = Vault {
            path,
            journal_path: None,
        };

        let daily_notes_config = vault.path.join(".obsidian").join("daily-notes.json");
        if !daily_notes_config.exists() {
            return Ok(vault);
        }

        let config = std::fs::read_to_string(daily_notes_config).with_context(|| {
            format!(
                "reading \"{}/.obsidian/daily-notes.json\"",
                vault.path.display()
            )
        })?;
        let config: Value = serde_json::from_str(&config).with_context(|| {
            format!(
                "parsing \"{}/.obsidian/daily-notes.json\"",
                vault.path.display()
            )
        })?;
        if let Some(folder) = config["folder"].as_str() {
            vault.journal_path = Some(vault.path.join(folder));
        }

        Ok(vault)
    }

    pub fn page_path<T: std::fmt::Display>(&self, name: T) -> PathBuf {
        self.path.join(format!("{}.md", name))
    }

    pub fn journal_path<T: std::fmt::Display>(&self, name: T) -> PathBuf {
        self.journal_path
            .as_deref()
            .unwrap_or(self.path.as_path())
            .join(format!("{}.md", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn default() -> anyhow::Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let vault = Vault::new(temp_dir.path().to_path_buf())?;

        assert_eq!(temp_dir.path(), vault.path);
        assert_eq!(temp_dir.child("page.md").path(), vault.page_path("page"));
        assert_eq!(temp_dir.child("page.md").path(), vault.journal_path("page"));

        Ok(())
    }

    #[test]
    fn create_vault() -> anyhow::Result<()> {
        let temp_dir = assert_fs::TempDir::new()?.child("dir");
        let vault = Vault::new(temp_dir.path().to_path_buf())?;

        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
        assert_eq!(temp_dir.path(), vault.path);

        Ok(())
    }

    #[test]
    fn daily_notes_folder() -> anyhow::Result<()> {
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

        let vault = Vault::new(temp_dir.path().to_path_buf())?;
        assert_eq!(
            temp_dir.child("daily-notes/page.md").path(),
            vault.journal_path("page")
        );

        Ok(())
    }
}
