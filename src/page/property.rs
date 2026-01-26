use anyhow::Result;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, derive_more::Display)]
#[display("{key}: \"{value}\"")]
pub struct Property {
    pub key: String,
    pub value: String,
}

impl Property {
    pub fn update(&mut self, rhs: Property) {
        if self.key == rhs.key {
            self.value = rhs.value
        }
    }
}

impl FromStr for Property {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let Some((key, value)) = s.split_once(":") else {
            anyhow::bail!("Can't find : in property {:?}", s);
        };

        let key = key.trim().to_owned();
        let mut value = value.trim();

        if let Some(dequoted) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
            value = dequoted;
        }
        let value = value.to_owned();

        Ok(Self { key, value })
    }
}

pub trait ToProperty {
    fn to_property<K: Into<String>>(&self, key: K) -> Property;
}
impl<V: ToString> ToProperty for V {
    fn to_property<K: Into<String>>(&self, key: K) -> Property {
        Property {
            key: key.into(),
            value: self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_display_parse() {
        let s = r#"month: "January""#;
        let m = s.parse::<Property>().unwrap();

        assert_eq!("month", m.key.as_str());
        assert_eq!(s, m.to_string().as_str());
        assert_eq!("January".to_owned(), m.value);

        let s = r#"filters: "{"month" false}""#;
        let m = s.parse::<Property>().unwrap();

        assert_eq!("filters", m.key.as_str());
        assert_eq!(s, m.to_string().as_str());
        assert_eq!(r#"{"month" false}"#, m.value);
    }

    #[test]
    fn property_update() -> anyhow::Result<()> {
        let v1 = r#"month: true"#.parse::<Property>()?;
        let v2 = r#"month: false"#.parse::<Property>()?;
        let v3 = r#"week: false"#.parse::<Property>()?;

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
