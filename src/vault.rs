use crate::events::Event;
use crate::options::PageOptions;
use crate::page::Page;
use crate::utils::{PageKind, PageName, ToPageName};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::path::{Path, PathBuf};

pub mod config;
pub use config::Config;

pub mod preparer;

/// A vault represents the whole folder with all the documents, e.g. the obsidian folder (which
/// they name a vault)
#[derive(Debug)]
pub struct Vault {
    config: Config,
    events: Vec<Event>,
}

impl Vault {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            std::fs::create_dir_all(path.as_path())
                .with_context(|| format!("creating dir {}", path.display()))?;
        }
        let config = Config::new(path)?;
        let events = config.read_events()?;

        Ok(Self { config, events })
    }

    pub fn prepare(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        mut page_options: PageOptions,
    ) -> Result<()> {
        page_options.update(self.config.settings());

        preparer::Preparer {
            from,
            to,
            page_options,
            vault: self,
        }
        .run()
    }

    pub fn path(&self) -> &Path {
        self.config.path()
    }

    pub fn events(&self) -> std::slice::Iter<'_, Event> {
        self.events.iter()
    }

    pub fn page_path<T: ToPageName>(&self, object: T) -> String {
        let PageName { name, kind } = object.to_page_name();
        match kind {
            PageKind::Journal => {
                if let Some(journals_folder) = self.config.journals_folder() {
                    journals_folder.to_owned() + name.as_str()
                } else {
                    name
                }
            }
            PageKind::Default => name,
        }
    }

    pub fn page_file_path<T: ToPageName>(&self, page: T) -> PathBuf {
        self.path().join(format!("{}.md", self.page_path(page)))
    }

    pub fn update<F, T>(&self, page: T, f: F) -> Result<()>
    where
        T: ToPageName,
        F: FnOnce(Page) -> Result<Page>,
    {
        let path = self.page_file_path(page);
        log::info!("Updating page {}", path.display());

        let mut page = f(Page::try_from(path)?)?;

        if page.modified() {
            page.write()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    fn create_daily_notes_config(temp_dir: &assert_fs::TempDir) -> Result<()> {
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

        Ok(())
    }

    #[test]
    fn page_file_path() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let vault = Vault::new(temp_dir.path().to_path_buf())?;

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

        create_daily_notes_config(&temp_dir)?;

        let vault = Vault {
            config: config::Config::new(temp_dir.path().to_path_buf())?,
            ..vault
        };

        assert_eq!(
            temp_dir.child("page.md").path(),
            vault.page_file_path(PageName {
                name: "page".to_owned(),
                kind: PageKind::Default
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

    #[test]
    fn page_path() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let vault = Vault::new(temp_dir.path().to_path_buf())?;

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

        create_daily_notes_config(&temp_dir)?;

        let vault = Vault {
            config: config::Config::new(temp_dir.path().to_path_buf())?,
            ..vault
        };

        assert_eq!(
            "page",
            vault.page_path(PageName {
                name: "page".to_owned(),
                kind: PageKind::Default
            })
        );
        assert_eq!(
            "daily-notes/page",
            vault.page_path(PageName {
                name: "page".to_owned(),
                kind: PageKind::Journal
            })
        );

        Ok(())
    }

    #[test]
    fn creates_vault() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?.child("dir");
        let vault = Vault::new(temp_dir.path().to_path_buf())?;

        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
        assert_eq!(temp_dir.path(), vault.path());

        Ok(())
    }

    #[test]
    fn update() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let vault = Vault::new(temp_dir.path().to_path_buf())?;
        let name: PageName = "foo".to_string().into();

        vault.update(name.clone(), |mut page| {
            page.prepend_line("World");
            Ok(page)
        })?;

        let path = vault.page_file_path(name.clone());
        let content = std::fs::read_to_string(&path)?;
        assert_eq!(content, "World\n");

        vault.update(name.clone(), |mut page| {
            page.prepend_line("Hello");
            Ok(page)
        })?;

        let path = vault.page_file_path(name.clone());
        let content = std::fs::read_to_string(&path)?;
        assert_eq!(content, "Hello\nWorld\n");

        Ok(())
    }
}
