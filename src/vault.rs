use crate::events::Event;
use crate::page::{Entry, Page};
use crate::utils::{PageKind, PageName, ToPageName};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Vault {
    path: PathBuf,
    journals_folder: Option<String>,
    events: Vec<Event>,
}

impl Vault {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            std::fs::create_dir_all(path.as_path())
                .with_context(|| format!("creating dir {:?}", path))?;
        }
        let mut vault = Vault {
            path,
            journals_folder: None,
            events: Default::default(),
        };
        vault.configure()?;

        Ok(vault)
    }

    fn configure(&mut self) -> Result<()> {
        self.configure_journal()?;
        self.configure_events()?;

        Ok(())
    }

    fn configure_journal(&mut self) -> Result<()> {
        let daily_notes_config = self.path.join(".obsidian").join("daily-notes.json");
        if !daily_notes_config.exists() {
            return Ok(());
        }

        let config = std::fs::read_to_string(daily_notes_config).with_context(|| {
            format!(
                "reading \"{}/.obsidian/daily-notes.json\"",
                self.path.display()
            )
        })?;
        let config: Value = serde_json::from_str(&config).with_context(|| {
            format!(
                "parsing \"{}/.obsidian/daily-notes.json\"",
                self.path.display()
            )
        })?;
        if let Some(folder) = config["folder"].as_str() {
            log::info!("Using journals_folder {}", folder);
            self.journals_folder = Some(folder.to_owned());
        }

        Ok(())
    }

    fn configure_events(&mut self) -> Result<()> {
        let event_page_path = self.path.join("events/recurring.md");
        if !event_page_path.exists() {
            return Ok(());
        }
        let event_page = Page::try_from(event_page_path.as_path())?;
        for entry in &event_page.content.content {
            if let Entry::CodeBlock(block) = entry {
                log::info!("Block: {:?}", block);
            }
        }

        Ok(())
    }

    pub fn events(&self) {}

    pub fn page_path<T: ToPageName>(&self, object: T) -> String {
        let PageName { name, kind } = object.to_page_name();
        match kind {
            PageKind::Journal => {
                if let Some(journals_folder) = self.journals_folder.clone() {
                    journals_folder + name.as_str()
                } else {
                    name
                }
            }
            PageKind::Default => name,
        }
    }

    pub fn page_file_path<T: ToPageName>(&self, page: T) -> PathBuf {
        self.path.join(format!("{}.md", self.page_path(page)))
    }

    pub fn update<F, T>(&self, page: T, f: F) -> Result<()>
    where
        T: ToPageName,
        F: FnOnce(Page) -> Result<Page>,
    {
        let path = self.page_file_path(page);
        log::info!("Updating page {}", path.display());

        let mut page = f(Page::new(&path))?;

        if path.exists() {
            page = Page::try_from(path.as_path())? + page;
        }

        page.write()?;

        Ok(())
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

        assert_eq!(
            "page",
            vault.page_path(PageName {
                name: "page".to_owned(),
                kind: PageKind::Default
            })
        );
        assert_eq!(
            "page",
            vault.page_path(PageName {
                name: "page".to_owned(),
                kind: PageKind::Journal
            })
        );

        assert_eq!(
            temp_dir.child("page.md").path(),
            vault.page_file_path(PageName {
                name: "page".to_owned(),
                kind: PageKind::Default
            })
        );
        assert_eq!(
            temp_dir.child("page.md").path(),
            vault.page_file_path(PageName {
                name: "page".to_owned(),
                kind: PageKind::Journal
            })
        );

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
            "daily-notes/page",
            vault.page_path(PageName {
                name: "page".to_owned(),
                kind: PageKind::Journal
            })
        );
        assert_eq!(
            temp_dir.child("daily-notes/page.md").path(),
            vault.page_file_path(PageName {
                name: "page".to_owned(),
                kind: PageKind::Journal
            })
        );
        Ok(())
    }
}
