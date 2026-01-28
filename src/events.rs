use crate::page::content::CodeBlock;
use anyhow::{Error, Result};
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

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
        let recurrence = match event.frequency {
            Frequency::Daily => {
                if !event.weekdays.is_empty() {
                    anyhow::bail!("`weekdays` not allowed for daily recurrence");
                }
                if !event.monthdays.is_empty() {
                    anyhow::bail!("`monthdays` not allowed for daily recurrence");
                }
                if !event.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` not allowed for daily recurrence");
                }
                Recurrence::Daily
            }
            Frequency::Weekly => {
                if !event.monthdays.is_empty() {
                    anyhow::bail!("`monthdays` not allowed for weekly recurrence");
                }
                if !event.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` not allowed for weekly recurrence");
                }
                Recurrence::Weekly(event.weekdays)
            }
            Frequency::Monthly => {
                if !event.weekdays.is_empty() {
                    anyhow::bail!("`weekdays` not allowed for monthly recurrence");
                }
                if !event.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` not allowed for monthly recurrence");
                }
                Recurrence::Monthly(event.monthdays)
            }
            Frequency::Yearly => {
                if !event.weekdays.is_empty() {
                    anyhow::bail!("`weekdays` not allowed for yearly recurrence");
                }
                if !event.monthdays.is_empty() {
                    anyhow::bail!("`monthdays` not allowed for yearly recurrence");
                }
                Recurrence::Yearly(event.yeardays)
            }
        };

        Ok(Event {
            recurrence,
            content: event.content,
            validity: event.validity,
            exceptions: event.exceptions,
        })
    }
}

/// Describe a recurring event in a format that can easily be serialized and deserialized
#[derive(Debug, Serialize, Deserialize)]
pub struct SerdeEvent {
    frequency: Frequency,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    weekdays: Vec<Weekday>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    monthdays: Vec<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    yeardays: Vec<usize>,
    pub content: String,
    #[serde(flatten)]
    validity: DateRange,
    #[serde(default)]
    exceptions: Vec<DateRange>,
}

impl From<Event> for SerdeEvent {
    fn from(event: Event) -> SerdeEvent {
        let recurrence = event.recurrence;
        let serde_event = SerdeEvent {
            frequency: Frequency::Daily,
            weekdays: Default::default(),
            monthdays: Default::default(),
            yeardays: Default::default(),
            content: event.content,
            validity: event.validity,
            exceptions: event.exceptions,
        };

        match recurrence {
            Recurrence::Weekly(weekdays) => SerdeEvent {
                frequency: Frequency::Weekly,
                weekdays,
                ..serde_event
            },
            Recurrence::Monthly(monthdays) => SerdeEvent {
                frequency: Frequency::Monthly,
                monthdays,
                ..serde_event
            },
            Recurrence::Yearly(yeardays) => SerdeEvent {
                frequency: Frequency::Yearly,
                yeardays,
                ..serde_event
            },
            _ => serde_event,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Recurrence {
    Daily,
    /// Weekly every Weekday
    Weekly(Vec<Weekday>),
    /// Monthly each Nth day, starting from 1
    Monthly(Vec<usize>),
    /// Yearly each Nth day, starting from 1
    Yearly(Vec<usize>),
}

impl Recurrence {
    pub fn matches(&self, date: NaiveDate) -> bool {
        use Recurrence::*;
        match self {
            Daily => true,
            Weekly(weekdays) => weekdays.iter().any(|day| *day == date.weekday()),
            Monthly(monthdays) => monthdays.iter().any(|day| *day == date.day() as usize),
            Yearly(yeardays) => yeardays.iter().any(|day| *day == date.ordinal() as usize),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, derive_more::IsVariant)]
#[serde(rename_all = "snake_case")]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DateRange {
    /// lower bound, inclusive if present
    pub from: Option<NaiveDate>,
    /// higher bound, inclusive if present
    pub to: Option<NaiveDate>,
}

impl DateRange {
    pub fn contains(&self, date: NaiveDate) -> bool {
        (self.from.is_none() || self.from <= Some(date))
            && (self.to.is_none() || self.to >= Some(date))
    }
}

impl Event {
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

impl TryFrom<CodeBlock> for Event {
    type Error = Error;

    fn try_from(block: CodeBlock) -> Result<Event> {
        if block.kind != "toml" {
            anyhow::bail!("Not a toml block");
        }
        let event: SerdeEvent = toml::from_str(&block.code)?;
        event.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod event_from_code_block {
        use super::*;
        use indoc::indoc;

        fn block(content: &str) -> CodeBlock {
            CodeBlock {
                kind: "toml".to_string(),
                code: content.to_string(),
            }
        }

        #[test]
        fn not_a_toml_block() {
            let block = CodeBlock {
                kind: "foo".to_string(),
                code: "".to_string(),
            };
            assert!(Event::try_from(block).is_err());
        }

        #[test]
        fn no_frequency() {
            let block = block("");
            assert!(Event::try_from(block).is_err());
        }

        #[test]
        fn no_content() {
            let block = block(indoc! {r#"
                frequency = daily
            "#});
            assert!(Event::try_from(block).is_err());
        }

        #[test]
        fn simple() -> Result<()> {
            let block = block(indoc! {r#"
                frequency = "daily"
                content = "Foo"
            "#});
            let event = Event::try_from(block)?;
            assert!(matches!(event.recurrence, Recurrence::Daily));
            assert_eq!("Foo", event.content);
            Ok(())
        }

        #[test]
        fn dates() -> Result<()> {
            let block = block(indoc! {r#"
                frequency = "daily"
                content = "Foo"
                from = "2025-01-01"
                to = "2025-01-31"
            "#});
            let event = Event::try_from(block)?;
            assert_eq!("2025-01-01".parse().ok(), event.validity.from);
            assert_eq!("2025-01-31".parse().ok(), event.validity.to);
            Ok(())
        }
    }
}
