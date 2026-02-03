use crate::content::CodeBlock;
use anyhow::{Error, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub mod recurrence;
pub use recurrence::Recurrence;
use recurrence::SerdeRecurrence;

/// Describe a recurring event
#[derive(Debug, Clone)]
pub struct Event {
    recurrence: Recurrence,
    pub content: String,
    validity: DateRange,
    exceptions: Vec<DateRange>,
}

impl TryFrom<SerdeEvent> for Event {
    type Error = Error;

    fn try_from(event: SerdeEvent) -> Result<Self> {
        Ok(Self {
            recurrence: Recurrence::try_from(event.recurrence)?,
            content: event.content,
            validity: event.validity,
            exceptions: event.exceptions,
        })
    }
}

/// Describe a recurring event in a format that can easily be serialized and deserialized
#[derive(Debug, Serialize, Deserialize)]
pub struct SerdeEvent {
    #[serde(flatten)]
    recurrence: SerdeRecurrence,
    content: String,
    #[serde(flatten)]
    validity: DateRange,
    #[serde(default)]
    exceptions: Vec<DateRange>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DateRange {
    /// lower bound, inclusive if present
    pub from: Option<NaiveDate>,
    /// higher bound, inclusive if present
    pub to: Option<NaiveDate>,
}

impl DateRange {
    #[must_use]
    pub fn contains(&self, date: NaiveDate) -> bool {
        (self.from.is_none() || self.from <= Some(date))
            && (self.to.is_none() || self.to >= Some(date))
    }
}

impl Event {
    #[must_use]
    pub fn matches(&self, date: NaiveDate) -> bool {
        if !self.validity.contains(date) {
            return false;
        }

        for exception in &self.exceptions {
            if exception.contains(date) {
                return false;
            }
        }

        self.recurrence.matches(date)
    }
}

impl TryFrom<&CodeBlock> for Event {
    type Error = Error;

    fn try_from(block: &CodeBlock) -> Result<Self> {
        if !block.is_toml() {
            anyhow::bail!("Not a toml block");
        }
        let event: SerdeEvent = toml::from_str(block.code())?;
        event.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};

    #[test]
    fn try_from_not_a_toml_block() {
        assert_err!(Event::try_from(&CodeBlock::new("foo", "")));
    }

    #[test]
    fn no_frequency() {
        assert_err!(Event::try_from(&CodeBlock::toml(r#"content = "foo""#)));
    }

    #[test]
    fn no_content() {
        assert_err!(Event::try_from(&CodeBlock::toml(r#"frequency = "daily""#)));
    }

    #[test]
    fn simple() {
        let event = assert_ok!(Event::try_from(&CodeBlock::toml(
            r#"
                frequency = "daily"
                content = "Foo"
            "#,
        )));
        assert!(matches!(event.recurrence, Recurrence::Daily));
        assert_eq!("Foo", event.content);
    }

    #[test]
    fn dates() {
        let event = assert_ok!(Event::try_from(&CodeBlock::toml(
            r#"
                frequency = "daily"
                content = "Foo"
                from = "2025-01-01"
                to = "2025-01-31"
            "#,
        )));
        assert_eq!("2025-01-01".parse().ok(), event.validity.from);
        assert_eq!("2025-01-31".parse().ok(), event.validity.to);
    }
}
