use anyhow::{Context, Result};
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub mod property;
use property::Property;

#[derive(Debug)]
pub struct Page {
    path: PathBuf,
    pub content: Content,
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

    pub fn push_line<L: Display>(&mut self, line: L) {
        self.content.entries.push(Entry::Line(format!("{}", line)))
    }

    pub fn push_property<P: Into<Property>>(&mut self, property: P) {
        self.content.properties.push(property.into());
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
    pub properties: Vec<Property>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, derive_more::Display, PartialEq)]
#[display("{_variant}")]
pub enum Entry {
    Line(String),
    CodeBlock(CodeBlock),
}

impl Entry {
    pub fn is_empty(&self) -> bool {
        match &self {
            Entry::Line(s) => s.is_empty(),
            Entry::CodeBlock(block) => block.is_empty(),
        }
    }
}

#[derive(Debug, derive_more::Display, PartialEq)]
#[display("```{kind}\n{code}```")]
pub struct CodeBlock {
    pub kind: String,
    pub code: String,
}

impl CodeBlock {
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
}

impl Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "---")?;
        for line in &self.properties {
            writeln!(f, "{}", line)?;
        }
        writeln!(f, "---")?;

        let mut entries_started = false;

        for line in &self.entries {
            if !entries_started {
                if line.is_empty() {
                    continue;
                } else {
                    entries_started = true;
                }
            }
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl FromStr for Content {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self> {
        let mut content = Content::default();
        let mut lines = string.lines().peekable();

        if lines.next_if_eq(&"---").is_some() {
            for line in lines.by_ref() {
                if line == "---" {
                    break;
                } else {
                    content.properties.push(line.parse()?);
                }
            }
        }

        while let Some(line) = lines.next() {
            if let Some(kind) = line.strip_prefix("```") {
                let mut code = String::new();
                for line in lines.by_ref() {
                    if line == "```" {
                        break;
                    } else {
                        code += line;
                        code += "\n";
                    }
                }
                content.entries.push(Entry::CodeBlock(CodeBlock {
                    kind: kind.to_owned(),
                    code,
                }));
            } else {
                content.entries.push(Entry::Line(line.to_owned()));
            }
        }

        Ok(content)
    }
}

impl Add for Content {
    type Output = Content;

    fn add(mut self, rhs: Content) -> Self::Output {
        for line in rhs.properties {
            if let Some(property) = self.properties.iter_mut().find(|l| l.key == line.key) {
                property.update(line);
            } else {
                self.properties.push(line);
            }
        }
        for line in rhs.entries {
            if self.entries.iter().all(|l| *l != line) {
                self.entries.push(line);
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
