use anyhow::Result;
use saphyr::{ScalarOwned, YamlOwned};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug)]
pub struct Content {
    pub(super) properties: YamlOwned,
    pub(super) entries: VecDeque<Entry>,
}

impl Default for Content {
    fn default() -> Self {
        Self {
            properties: YamlOwned::Mapping(saphyr::MappingOwned::default()),
            entries: Default::default(),
        }
    }
}

impl Content {
    pub(super) fn insert_property(&mut self, key: String, value: String) {
        let Some(mapping) = self.properties.as_mapping_mut() else {
            unreachable!()
        };
        mapping.insert(
            YamlOwned::Value(ScalarOwned::String(key)),
            YamlOwned::Value(ScalarOwned::String(value)),
        );
    }
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
        use saphyr::{Yaml, YamlEmitter};

        if self.properties.is_empty_collection() {
            writeln!(f, "---\n---")?;
        } else {
            let mut emitter = YamlEmitter::new(f);
            emitter
                .dump(&Yaml::from(&self.properties))
                .map_err(|_| std::fmt::Error)?;
            writeln!(f, "\n---")?;
        }

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

        let mut properties = String::new();

        if lines.next_if_eq(&"---").is_some() {
            for line in lines.by_ref() {
                if line == "---" {
                    break;
                } else {
                    properties.push_str(line);
                    properties.push('\n');
                }
            }
        }

        use saphyr::LoadableYamlNode;
        let mut yaml_documents = YamlOwned::load_from_str(properties.as_str())?;
        if yaml_documents.len() > 1 {
            anyhow::bail!(
                "Multiple YAML documents parsed from the page properties: {:?}",
                properties
            );
        }

        if let Some(yaml) = yaml_documents.pop() {
            if yaml.is_mapping() {
                content.properties = yaml;
            } else {
                anyhow::bail!("Properties is not a YAML Mapping: {:?}", properties);
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
