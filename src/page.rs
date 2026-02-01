use anyhow::{Context, Result};
use std::fmt::Display;
use std::io::Write;
use std::path::{Path, PathBuf};

pub mod content;
pub use content::{Content, Entry};

#[derive(Debug)]
pub struct Page {
    path: PathBuf,
    exists: bool,
    modified: bool,
    content: Content,
}

impl Page {
    pub fn write(&mut self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating dir {:?}", parent))?;
            }
        }

        let mut file = std::fs::File::create(&self.path)
            .with_context(|| format!("creating file {:?}", self.path))?;
        write!(file, "{}", self.content)
            .with_context(|| format!("writing file {:?}", self.path))?;

        self.exists = true;

        Ok(())
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.content.entries.iter()
    }

    pub fn prepend_lines<I, L>(&mut self, lines: I)
    where
        I: IntoIterator<Item = L>,
        L: Display,
        <I as IntoIterator>::IntoIter: DoubleEndedIterator,
    {
        for line in lines.into_iter().rev() {
            self.prepend_line(line);
        }
    }

    pub fn prepend_line<L: Display>(&mut self, line: L) {
        let entry = Entry::Line(format!("{}", line));

        if self.content.prepend_unique_entry(entry) {
            self.modified = true;
        }
    }

    pub fn insert_property<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Display,
    {
        self.modified = true;
        self.content
            .insert_property(key.into(), format!("{}", value));
    }

    pub fn modified(&self) -> bool {
        self.modified
    }
}

impl TryFrom<&Path> for Page {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Page> {
        Page::try_from(path.to_path_buf())
    }
}

impl TryFrom<PathBuf> for Page {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Page> {
        let page = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("reading file {:?}", path))?
                .parse()
                .with_context(|| format!("reading file {:?}", path))?;
            Page {
                path,
                exists: true,
                modified: false,
                content,
            }
        } else {
            Page {
                path,
                exists: false,
                modified: false,
                content: Content::default(),
            }
        };

        Ok(page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use indoc::{formatdoc, indoc};

    #[test]
    fn page() -> anyhow::Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let file = temp_dir.child("page.md");

        let properties = r#"month: "[[2024/September]]""#;
        let entries = indoc! {"
            - TODO Something
            - DONE Something else
            - One other thing
        "};

        file.write_str(
            formatdoc!(
                "
                ---
                {properties}
                ---
                {entries}"
            )
            .as_str(),
        )?;

        let mut page = Page::try_from(file.path())?;
        page.write()?;
        file.assert(formatdoc! {"
            ---
            {properties}
            ---
            {entries}"});

        Ok(())
    }
}
