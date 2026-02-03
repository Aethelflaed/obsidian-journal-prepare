use anyhow::Result;
use saphyr::{ScalarOwned, YamlOwned};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utils::content::CodeBlock;

#[derive(Debug)]
pub struct Content {
    pub(super) properties: YamlOwned,
    pub(super) entries: VecDeque<Entry>,
}

impl Default for Content {
    fn default() -> Self {
        Self {
            properties: YamlOwned::Mapping(saphyr::MappingOwned::default()),
            entries: VecDeque::default(),
        }
    }
}

const fn to_yaml_str(string: String) -> YamlOwned {
    YamlOwned::Value(ScalarOwned::String(string))
}

impl Content {
    /// Insert the given property (key, value)
    ///
    /// Return value indicates if the content has been modified or not
    pub(super) fn insert_property(&mut self, key: String, value: String) -> bool {
        let Some(mapping) = self.properties.as_mapping_mut() else {
            unreachable!()
        };
        mapping
            .insert(to_yaml_str(key), to_yaml_str(value.clone()))
            .is_none_or(|previous_value| previous_value != to_yaml_str(value))
    }

    /// Prepend the given entry if it is not already present
    ///
    /// Return value indicates if the content has been modified or not
    pub(super) fn prepend_unique_entry(&mut self, entry: Entry) -> bool {
        if self.entries.iter().all(|e| *e != entry) {
            self.entries.push_front(entry);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, derive_more::From, derive_more::Display, Eq, PartialEq)]
#[display("{_variant}")]
pub enum Entry {
    Line(String),
    CodeBlock(CodeBlock),
}

impl Entry {
    pub const fn is_empty(&self) -> bool {
        match &self {
            Self::Line(s) => s.is_empty(),
            Self::CodeBlock(block) => block.is_empty(),
        }
    }
}

impl Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use saphyr::{Yaml, YamlEmitter};

        if !self.properties.is_empty_collection() {
            YamlEmitter::new(f)
                .dump(&Yaml::from(&self.properties))
                .map_err(|_| std::fmt::Error)?;
            writeln!(f, "\n---")?;
        }

        let mut entries_started = false;

        for line in &self.entries {
            if !entries_started {
                if line.is_empty() {
                    continue;
                }
                entries_started = true;
            }
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl FromStr for Content {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self> {
        use saphyr::LoadableYamlNode;

        let mut content = Self::default();
        let mut lines = string.lines().peekable();

        // If it starts with a document separator, it means there is properties to read
        if lines.next_if_eq(&"---").is_some() {
            let mut properties = String::new();
            for line in lines.by_ref() {
                if line == "---" {
                    break;
                }
                properties = properties + line + "\n";
            }

            let mut yaml_documents = YamlOwned::load_from_str(properties.as_str())?;
            if yaml_documents.len() > 1 {
                // This shouldn't be possible as we read the content until the second document
                // separator (---)
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
        }

        while let Some(line) = lines.next() {
            let entry = line.strip_prefix("```").map_or_else(
                || line.to_owned().into(),
                |kind| {
                    let mut code = String::new();
                    for line in lines.by_ref() {
                        if line == "```" {
                            break;
                        }
                        code = code + line + "\n";
                    }

                    CodeBlock {
                        kind: kind.to_owned(),
                        code,
                    }
                    .into()
                },
            );

            content.entries.push_back(entry);
        }

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use saphyr::{Scalar, Yaml, Yaml::Value};

    #[test]
    fn basic_document() -> Result<()> {
        let string = "Hello World\n";
        let content = Content::from_str(string)?;
        assert!(content.properties.is_empty_collection());
        assert_eq!(content.entries.len(), 1);

        assert_eq!(string, format!("{content}").as_str());

        Ok(())
    }

    #[test]
    fn adds_final_new_line() -> Result<()> {
        let string = "Hello World";
        let content = Content::from_str(string)?;
        assert!(content.properties.is_empty_collection());
        assert_eq!(content.entries.len(), 1);

        assert_eq!("Hello World\n", format!("{content}").as_str());

        Ok(())
    }

    #[test]
    fn parse_block_code_as_single_entry() -> Result<()> {
        let string = "```\nfoo\n```\n";
        let content = Content::from_str(string)?;
        assert!(content.properties.is_empty_collection());
        assert_eq!(content.entries.len(), 1);

        let Entry::CodeBlock(ref code_block) = content.entries[0] else {
            panic!("Code block not parsed as code block");
        };

        assert_eq!("", code_block.kind.as_str());
        assert_eq!("foo\n", code_block.code.as_str());

        assert_eq!(string, format!("{content}").as_str());

        Ok(())
    }

    #[test]
    fn parse_toml_block_code_as_single_entry() -> Result<()> {
        let string = "```toml\nfoo\n```\n";
        let content = Content::from_str(string)?;
        assert!(content.properties.is_empty_collection());
        assert_eq!(content.entries.len(), 1);

        let Entry::CodeBlock(ref code_block) = content.entries[0] else {
            panic!("Code block not parsed as code block");
        };

        assert_eq!("toml", code_block.kind.as_str());
        assert_eq!("foo\n", code_block.code.as_str());

        assert_eq!(string, format!("{content}").as_str());

        Ok(())
    }

    #[test]
    fn parse_multiple_entries_and_remove_initial_empty_lines() -> Result<()> {
        let string = indoc! {"


            Hello World
            ```toml
            Block
            ```

            - test
            - foo
            ```sh
            fkjdlsk
            ```
        "};
        let content = Content::from_str(string)?;
        assert!(content.properties.is_empty_collection());
        assert_eq!(content.entries.len(), 8);

        assert_eq!(string, format!("\n\n{content}").as_str());

        Ok(())
    }

    #[test]
    fn parse_simple_metadata() -> Result<()> {
        let string = indoc! {"
            ---
            foo: bar
            ---
        "};
        let content = Content::from_str(string)?;
        assert!(!content.properties.is_empty_collection());
        assert_eq!(content.entries.len(), 0);
        assert_eq!(string, format!("{content}").as_str());

        let properties = Yaml::from(&content.properties);
        assert_eq!(
            properties.as_mapping_get("foo").unwrap(),
            &Value(Scalar::String("bar".into()))
        );

        Ok(())
    }

    #[test]
    fn parse_multiple_metadata() -> Result<()> {
        let string = indoc! {"
            ---
            foo: bar
            baz: 1
            date: 2026-01-29
            ---
        "};
        let content = Content::from_str(string)?;
        assert!(!content.properties.is_empty_collection());
        assert_eq!(content.entries.len(), 0);
        assert_eq!(string, format!("{content}").as_str());

        let properties = Yaml::from(&content.properties);
        assert_eq!(
            properties.as_mapping_get("foo").unwrap(),
            &Value(Scalar::String("bar".into()))
        );
        assert_eq!(
            properties.as_mapping_get("baz").unwrap(),
            &Value(Scalar::Integer(1))
        );
        assert_eq!(
            properties.as_mapping_get("date").unwrap(),
            &Value(Scalar::String("2026-01-29".into()))
        );

        Ok(())
    }

    #[test]
    fn parse_sequence_metadata_with_content() -> Result<()> {
        let string = indoc! {"
            ---
            aliases:
              - Note
            ---
            # This is a page

            Blabla
        "};

        let content = Content::from_str(string)?;
        assert!(!content.properties.is_empty_collection());
        assert_eq!(content.entries.len(), 3);
        assert_eq!(string, format!("{content}").as_str());

        let properties = Yaml::from(&content.properties);
        assert_eq!(
            properties
                .as_mapping_get("aliases")
                .unwrap()
                .as_sequence()
                .unwrap(),
            &vec![Value(Scalar::String("Note".into()))]
        );

        Ok(())
    }

    #[test]
    fn insert_property_on_default_content() {
        let mut content = Content::default();
        assert!(content.insert_property("foo".to_owned(), "bar".to_owned()));

        let string = indoc! {"
            ---
            foo: bar
            ---
        "};
        assert_eq!(string, format!("{content}").as_str());
    }

    #[test]
    fn insert_property_update_existing() -> Result<()> {
        let string = indoc! {"
            ---
            foo: bar
            ---
        "};
        let mut content = Content::from_str(string)?;
        assert!(!content.insert_property("foo".to_owned(), "bar".to_owned()));
        assert!(content.insert_property("foo".to_owned(), "baz".to_owned()));

        assert_eq!(
            indoc! {"
            ---
            foo: baz
            ---
        "},
            format!("{content}").as_str()
        );

        Ok(())
    }

    #[test]
    fn prepend_unique_entry_on_default_content() {
        let mut content = Content::default();

        let entry = Entry::Line("Hello, World".to_owned());
        assert!(content.prepend_unique_entry(entry.clone()));
        assert!(!content.prepend_unique_entry(entry));
    }

    #[test]
    fn prepend_unique_entry_update_existing() -> Result<()> {
        let string = indoc! {"
            Hello, World
        "};
        let mut content = Content::from_str(string)?;
        let entry = Entry::Line("Hello, World".to_owned());
        assert!(!content.prepend_unique_entry(entry));

        Ok(())
    }
}
