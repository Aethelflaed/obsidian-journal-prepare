use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;

lazy_static! {
    static ref REGEX: Regex =
        Regex::new(r#"^\s*"(?<word>[^"]+)"\s*(?<boolean>true|false)\s*$"#).unwrap();
}

#[derive(Debug, Clone, PartialEq, derive_more::Display)]
#[display("{_0}")]
pub enum Value {
    Text(String),
    Filters(Filters),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Filters(HashMap<String, bool>);

impl Filters {
    pub fn push<S: Into<String>>(mut self, key: S, value: bool) -> Self {
        self.0.insert(key.into(), value);
        self
    }
}

impl FromStr for Filters {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut filters = HashMap::<String, bool>::new();

        let s = &s[1..s.len() - 1];

        for part in s.split(",") {
            let Some((_, [word, boolean])) = REGEX.captures(part).map(|caps| caps.extract()) else {
                anyhow::bail!("Cannot parse {:?} with regex", part);
            };

            filters.insert(word.to_owned(), boolean.parse::<bool>()?);
        }

        Ok(Self(filters))
    }
}

impl std::fmt::Display for Filters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{{{}}}",
            self.0
                .iter()
                .map(|(key, b)| format!(r#""{key}" {b}"#))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Add for Filters {
    type Output = Filters;

    fn add(mut self, rhs: Filters) -> Self::Output {
        for (word, boolean) in rhs.0.into_iter() {
            self.0.insert(word, boolean);
        }

        self
    }
}

#[derive(Debug, Clone, PartialEq, derive_more::Display)]
#[display("{key}:: {value}")]
pub struct Metadata {
    pub key: String,
    pub value: Value,
}

impl Metadata {
    pub fn update(&mut self, rhs: Metadata) {
        if self.key == rhs.key {
            self.value = match (self.value.clone(), rhs.value) {
                (Value::Filters(f), Value::Filters(g)) => Value::Filters(f + g),
                (_, v) => v,
            }
        }
    }
}

impl FromStr for Metadata {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let Some((key, value)) = s.split_once("::") else {
            anyhow::bail!("Can't find :: in metadata {:?}", s);
        };

        let key = key.trim();
        match key {
            "filters" => Ok(Self {
                key: key.trim().to_owned(),
                value: Value::Filters(
                    value
                        .trim()
                        .parse()
                        .with_context(|| format!("Parsing metadata {:?}", s))?,
                ),
            }),
            _ => Ok(Self {
                key: key.trim().to_owned(),
                value: Value::Text(value.trim().to_owned()),
            }),
        }
    }
}

impl From<Filters> for Metadata {
    fn from(f: Filters) -> Self {
        Metadata {
            key: "filters".to_owned(),
            value: Value::Filters(f),
        }
    }
}

pub trait ToMetadata {
    fn to_metadata<K: Into<String>>(&self, key: K) -> Metadata;
}
impl<V: ToString> ToMetadata for V {
    fn to_metadata<K: Into<String>>(&self, key: K) -> Metadata {
        Metadata {
            key: key.into(),
            value: Value::Text(self.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_display() {
        assert_eq!("foo".to_owned(), Value::Text("foo".to_owned()).to_string());
        assert_eq!(
            "{}".to_owned(),
            Value::Filters(Filters::default()).to_string()
        );
    }

    #[test]
    fn filters_add() {
        let mut f1 = Filters::default();
        f1.0.insert("hello".to_owned(), false);
        f1.0.insert("world".to_owned(), false);

        let mut f2 = Filters::default();
        f2.0.insert("hello".to_owned(), true);
        f2.0.insert("World".to_owned(), true);

        let f3 = f1 + f2;
        assert_eq!(f3.0.len(), 3);
        assert_eq!(f3.0["hello"], true);
        assert_eq!(f3.0["world"], false);
        assert_eq!(f3.0["World"], true);
    }

    #[test]
    fn filters_display_parse() {
        let s = r#"{"hel lo" true, "world" false}"#;
        let f = s.parse::<Filters>().unwrap();
        assert_eq!(f.0.len(), 2);
        assert_eq!(f.0["hel lo"], true);
        assert_eq!(f.0["world"], false);

        let alt = r#"{"world" false, "hel lo" true}"#;
        let result = f.to_string();
        assert!(result.as_str() == s || result.as_str() == alt);
    }

    #[test]
    fn metadata_display_parse() {
        let s = "month:: January";
        let m = s.parse::<Metadata>().unwrap();

        assert_eq!("month", m.key.as_str());
        assert_eq!(s, m.to_string().as_str());
        assert_eq!(Value::Text("January".to_owned()), m.value);

        let s = r#"filters:: {"month" false}"#;
        let m = s.parse::<Metadata>().unwrap();

        assert_eq!("filters", m.key.as_str());
        assert_eq!(s, m.to_string().as_str());
        let mut f = Filters::default();
        f.0.insert("month".to_owned(), false);
        assert_eq!(Value::Filters(f), m.value);
    }

    #[test]
    fn metadata_update() -> anyhow::Result<()> {
        let f1 = r#"filters:: {"hello" true}"#.parse::<Metadata>()?;
        let f2 = r#"filters:: {"world" false}"#.parse::<Metadata>()?;
        let f3 = Metadata {
            key: "filters".to_owned(),
            value: Value::Text("foo".to_owned()),
        };
        let v1 = r#"month:: true"#.parse::<Metadata>()?;
        let v2 = r#"month:: false"#.parse::<Metadata>()?;
        let v3 = r#"week:: false"#.parse::<Metadata>()?;

        // different keys
        let mut v4 = v3.clone();
        v4.update(v2.clone());
        assert_eq!(v4, v3);

        let mut v4 = v1.clone();
        v4.update(v2.clone());
        assert_eq!(v2, v4);

        let mut f4 = f3.clone();
        f4.update(f2.clone());
        assert_eq!(f2, f4);

        let mut f4 = f2.clone();
        f4.update(f3.clone());
        assert_eq!(f3, f4);

        let mut f4 = f2.clone();
        f4.update(f1.clone());
        let Value::Filters(f) = f4.value else {
            panic!("f4.value is not a Filters");
        };
        assert_eq!(f.0.len(), 2);
        assert_eq!(f.0["hello"], true);
        assert_eq!(f.0["world"], false);

        Ok(())
    }
}
