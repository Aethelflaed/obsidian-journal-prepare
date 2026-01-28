use super::Property;
use anyhow::Result;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct Content {
    pub(super) properties: Vec<Property>,
    pub(super) entries: VecDeque<Entry>,
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
                content.entries.push_back(Entry::CodeBlock(CodeBlock {
                    kind: kind.to_owned(),
                    code,
                }));
            } else {
                content.entries.push_back(Entry::Line(line.to_owned()));
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
                self.entries.push_back(line);
            }
        }
        self
    }
}
