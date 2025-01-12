use crate::metadata::Metadata;
use anyhow::{Context, Result};
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
pub struct Page {
    path: PathBuf,
    content: Content,
}

impl Page {
    pub fn new(path: &Path) -> Page {
        Self {
            path: path.to_path_buf(),
            content: Default::default(),
        }
    }

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
        Ok(())
    }

    pub fn push_content<C: Display>(&mut self, content: C) {
        self.content.content.push(format!("{}", content))
    }

    pub fn push_metadata<M: Into<Metadata>>(&mut self, metadata: M) {
        self.content.metadata.push(metadata.into());
    }
}

impl TryFrom<&Path> for Page {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Page> {
        let mut page = Page::new(path);
        page.content = std::fs::read_to_string(path)
            .with_context(|| format!("reading file {:?}", path))?
            .parse()
            .with_context(|| format!("reading file {:?}", path))?;

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

#[derive(Debug, Default)]
pub struct Content {
    metadata: Vec<Metadata>,
    content: Vec<String>,
}

impl Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "---")?;
        for line in &self.metadata {
            writeln!(f, "{}", line)?;
        }
        writeln!(f, "---")?;

        if let Some(line) = self.content.first() {
            if !line.is_empty() {
                writeln!(f)?;
            }
        }

        for line in &self.content {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl FromStr for Content {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self> {
        let mut page = Content::default();
        let mut read_content = false;
        let mut read_metadata = false;

        for line in string.lines() {
            if !read_metadata && !read_content {
                if line == "---" {
                    read_metadata = true;
                }
            } else if read_metadata {
                if line == "---" {
                    read_content = true;
                    read_metadata = false;
                } else {
                    page.metadata.push(line.parse()?);
                }
            } else {
                page.content.push(line.to_owned())
            }
        }

        Ok(page)
    }
}

impl Add for Content {
    type Output = Content;

    fn add(mut self, rhs: Content) -> Self::Output {
        for line in rhs.metadata {
            if let Some(metadata) = self.metadata.iter_mut().find(|l| l.key == line.key) {
                metadata.update(line);
            } else {
                self.metadata.push(line);
            }
        }
        for line in rhs.content {
            if self.content.iter().all(|l| *l != line) {
                self.content.push(line);
            }
        }
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

        let metadata = r#"month: "[[2024/September]]""#;
        let content = indoc! {"
            - TODO Something
            - DONE Something else
            - One other thing
        "};

        file.write_str(
            formatdoc!(
                "
            ---
            {metadata}
            ---
            {content}"
            )
            .as_str(),
        )?;

        let mut page: Page = file.path().try_into()?;
        page.write()?;
        file.assert(formatdoc! {"
            ---
            {metadata}
            ---

            {content}"});

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
}
