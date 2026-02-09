use crate::content::{Content, ContentError, Entry};
use std::fmt::Display;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Page {
    path: PathBuf,
    exists: bool,
    modified: bool,
    content: Content,
}

#[derive(Debug, derive_more::Error, derive_more::Display)]
pub enum PageError {
    #[display("Error creating dir {}: {_0}", _1.display())]
    CreatingDir(std::io::Error, PathBuf),
    #[display("Error creating file {}: {_0}", _1.display())]
    CreatingFile(std::io::Error, PathBuf),
    #[display("Error writing file {}: {_0}", _1.display())]
    WritingFile(std::io::Error, PathBuf),
    #[display("Error reading file {}: {_0}", _1.display())]
    ReadingFile(std::io::Error, PathBuf),
    ParsingContent(ContentError),
}

impl Page {
    /// Write the page to disk
    ///
    /// # Errors
    /// - `CreatingDir`
    /// - `CreatingFile`
    /// - `WritingFile`
    pub fn write(&mut self) -> Result<(), PageError> {
        if let Some(parent) = self.path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)
                .map_err(|e| PageError::CreatingDir(e, parent.to_path_buf()))?;
        }

        let mut file = std::fs::File::create(&self.path)
            .map_err(|e| PageError::CreatingFile(e, self.path.clone()))?;
        write!(file, "{}", self.content)
            .map_err(|e| PageError::WritingFile(e, self.path.clone()))?;

        self.exists = true;
        self.modified = false;

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
        let entry = Entry::Line(format!("{line}"));

        if self.content.prepend_unique_entry(entry) {
            self.modified = true;
        }
    }

    pub fn insert_property<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Display,
    {
        if self.content.insert_property(key.into(), format!("{value}")) {
            self.modified = true;
        }
    }

    #[must_use]
    pub const fn modified(&self) -> bool {
        self.modified
    }

    #[must_use]
    pub const fn exists(&self) -> bool {
        self.exists
    }
}

impl TryFrom<&Path> for Page {
    type Error = PageError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Self::try_from(path.to_path_buf())
    }
}

impl TryFrom<PathBuf> for Page {
    type Error = PageError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let page = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| PageError::ReadingFile(e, path.clone()))?
                .parse()
                .map_err(PageError::ParsingContent)?;
            Self {
                path,
                exists: true,
                modified: false,
                content,
            }
        } else {
            Self {
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
    use claim::assert_ok;
    use indoc::{formatdoc, indoc};

    #[test]
    fn track_existence_and_modification() {
        let temp_dir = assert_ok!(assert_fs::TempDir::new());
        let file = temp_dir.child("dir/page.md");
        let mut page = assert_ok!(Page::try_from(file.path()));

        assert!(!page.exists());
        assert!(!page.modified());

        page.insert_property("foo", "bar");
        page.prepend_line("Hello, World");

        assert!(!page.exists());
        assert!(page.modified());

        assert_ok!(page.write());
        file.assert(indoc! {"
            ---
            foo: bar
            ---
            Hello, World
        "});

        assert!(page.exists());
        assert!(!page.modified());

        page.insert_property("foo", "bar");
        assert!(!page.modified());

        page.prepend_line("Hello, World");
        assert!(!page.modified());

        page.prepend_line("Hello World");
        assert!(page.modified());
    }

    #[test]
    fn parse_page_from_path_and_write_it_again() {
        let temp_dir = assert_ok!(assert_fs::TempDir::new());
        let file = temp_dir.child("page.md");

        let properties = r#"month: "[[2024/September]]""#;
        let entries = indoc! {"
            - TODO Something
            - DONE Something else
            - One other thing
        "};

        assert_ok!(
            file.write_str(
                formatdoc!(
                    "
                ---
                {properties}
                ---
                {entries}"
                )
                .as_str(),
            )
        );

        let mut page = assert_ok!(Page::try_from(file.path()));
        assert_ok!(page.write());
        file.assert(formatdoc! {"
            ---
            {properties}
            ---
            {entries}"});
    }
}
