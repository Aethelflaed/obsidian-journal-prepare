use anyhow::{Context, Result};
use std::fmt::Display;
use std::io::Write;
use std::ops::Add;
use std::path::{Path, PathBuf};

pub mod property;
use property::Property;

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

    pub fn push_line<L: Display>(&mut self, line: L) {
        self.modified = true;
        self.content.entries.push(Entry::Line(format!("{}", line)))
    }

    pub fn push_property<P: Into<Property>>(&mut self, property: P) {
        self.modified = true;
        self.content.properties.push(property.into());
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

impl Add for Page {
    type Output = Page;

    fn add(mut self, rhs: Page) -> Self::Output {
        self.content = self.content + rhs.content;
        self
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

        let mut page: Page = file.path().try_into()?;
        page.write()?;
        file.assert(formatdoc! {"
            ---
            {properties}
            ---
            {entries}"});

        let second_file = temp_dir.child("another page.md");
        second_file.write_str(indoc! {r#"
            ---
            week: "yes"
            ---

            - DONE Something
            - TODO Something
            - One other thing
        "#})?;

        let second_page: Page = second_file.path().try_into()?;
        let mut final_page = second_page + page;
        final_page.write()?;

        second_file.assert(indoc! {r#"
            ---
            week: "yes"
            month: "[[2024/September]]"
            ---
            - DONE Something
            - TODO Something
            - One other thing
            - DONE Something else
        "#});

        Ok(())
    }

    #[test]
    fn codeblocks() -> anyhow::Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let file = temp_dir.child("page.md");

        let raw_content = indoc! {r#"
            ---
            foo: "bar"
            ---
            ```toml
            value = "test"
            ```
            Hello World
        "#};

        file.write_str(raw_content)?;
        let page: Page = file.path().try_into()?;

        assert!(matches!(
            page.content.entries.first(),
            Some(Entry::CodeBlock(_))
        ));
        if let Some(Entry::CodeBlock(code_block)) = page.content.entries.first() {
            assert_eq!("toml", code_block.kind);
            assert_eq!("value = \"test\"\n", code_block.code);
        }

        assert_eq!(raw_content, format!("{}", page.content).as_str());

        Ok(())
    }
}
