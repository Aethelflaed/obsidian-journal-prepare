use anyhow::Result;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, derive_more::Display)]
#[display("{key}: \"{value}\"")]
pub struct Metadata {
    pub key: String,
    pub value: String,
}

impl Metadata {
    pub fn update(&mut self, rhs: Metadata) {
        if self.key == rhs.key {
            self.value = rhs.value
        }
    }
}

impl FromStr for Metadata {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let Some((key, value)) = s.split_once(":") else {
            anyhow::bail!("Can't find : in metadata {:?}", s);
        };

        let key = key.trim().to_owned();
        let mut value = value.trim();

        if let Some(dequoted) = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
        {
            value = dequoted;
        }
        let value = value.to_owned();

        Ok(Self { key, value })
    }
}

pub trait ToMetadata {
    fn to_metadata<K: Into<String>>(&self, key: K) -> Metadata;
}
impl<V: ToString> ToMetadata for V {
    fn to_metadata<K: Into<String>>(&self, key: K) -> Metadata {
        Metadata {
            key: key.into(),
            value: self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_display_parse() {
        let s = r#"month: "January""#;
        let m = s.parse::<Metadata>().unwrap();

        assert_eq!("month", m.key.as_str());
        assert_eq!(s, m.to_string().as_str());
        assert_eq!("January".to_owned(), m.value);

        let s = r#"filters: "{"month" false}""#;
        let m = s.parse::<Metadata>().unwrap();

        assert_eq!("filters", m.key.as_str());
        assert_eq!(s, m.to_string().as_str());
        assert_eq!(r#"{"month" false}"#, m.value);
    }

    #[test]
    fn metadata_update() -> anyhow::Result<()> {
        let v1 = r#"month: true"#.parse::<Metadata>()?;
        let v2 = r#"month: false"#.parse::<Metadata>()?;
        let v3 = r#"week: false"#.parse::<Metadata>()?;

        // different keys
        let mut v4 = v3.clone();
        v4.update(v2.clone());
        assert_eq!(v4, v3);

        let mut v4 = v1.clone();
        v4.update(v2.clone());
        assert_eq!(v2, v4);

        Ok(())
    }
}
