use crate::events::Event;
use crate::options::PageOptions;
use crate::page::{Entry, Page};
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
    path: PathBuf,
    config: Config,
    events: Vec<Event>,
}

impl Vault {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            std::fs::create_dir_all(path.as_path())
                .with_context(|| format!("creating dir {:?}", path))?;
        }
        let mut vault = Vault {
            config: config::Config::new(&path)?,
            path,
            events: Default::default(),
        };
        vault.configure()?;

        Ok(vault)
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
        &self.path
    }

    fn configure(&mut self) -> Result<()> {
        self.configure_events()?;

        Ok(())
    }

    fn configure_events(&mut self) -> Result<()> {
        let event_page_path = self.path.join("events/recurring.md");
        if !event_page_path.exists() {
            return Ok(());
        }
        let event_page = Page::try_from(event_page_path.as_path())?;
        for entry in event_page.entries() {
            if let Entry::CodeBlock(block) = entry {
                if block.kind.as_str() == "toml" {
                    let event = block.try_into()?;
                    log::debug!("Event: {:?}", event);
                    self.events.push(event);
                }
            }
        }

        Ok(())
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
        self.path.join(format!("{}.md", self.page_path(page)))
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
            config: config::Config::new(temp_dir.path())?,
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
            config: config::Config::new(temp_dir.path())?,
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
        assert_eq!(temp_dir.path(), vault.path);

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
        vault.update(name.clone(), |mut page| {
            page.prepend_line("Hello");
            Ok(page)
        })?;

        let page: Page = vault.page_file_path(name).as_path().try_into()?;

        assert_eq!(
            page.entries()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join("\n"),
            "Hello\nWorld"
        );

        Ok(())
    }
}
