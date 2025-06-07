use crate::page::CodeBlock;
use anyhow::{Error, Result};
use chrono::{NaiveDate, Weekday};
use std::str::FromStr;
use toml::Table;

/// Describe a recurring event
#[derive(Debug)]
pub struct Event {
    frequency: Frequency,
    content: String,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    weekdays: Vec<Weekday>,
    monthdays: Vec<usize>,
    yeardays: Vec<usize>,
}

#[derive(Debug)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl FromStr for Frequency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(Frequency::Daily),
            "weekly" => Ok(Frequency::Weekly),
            "monthly" => Ok(Frequency::Monthly),
            "yearly" => Ok(Frequency::Yearly),
            _ => anyhow::bail!("Unknown frequency {s}"),
        }
    }
}

impl Event {
    fn valid(self) -> Result<Self> {
        use Frequency::*;

        match self.frequency {
            Daily => Ok(self),
            Weekly => {
                if self.weekdays.is_empty() {
                    anyhow::bail!("No weekdays configured for monthly event {:?}", self)
                } else {
                    Ok(self)
                }
            },
            Monthly => {
                if self.monthdays.is_empty() {
                    anyhow::bail!("No monthdays configured for monthly event {:?}", self)
                } else {
                    Ok(self)
                }
            },
            Yearly => {
                if self.yeardays.is_empty() {
                    anyhow::bail!("No yeardays configured for yearly event {:?}", self)
                } else {
                    Ok(self)
                }
            },
        }
    }
}

impl TryFrom<CodeBlock> for Event {
    type Error = Error;

    fn try_from(block: CodeBlock) -> Result<Event> {
        if block.kind != "toml" {
            anyhow::bail!("Not a toml block");
        }
        let toml = block.code.parse::<Table>()?;

        let Some(frequency) = toml.get("frequency").map(|frequency| {
            frequency
                .as_str()
                .ok_or(anyhow::anyhow!("Unknown frequency {:?}", frequency))
                .map(|str_freq| str_freq.parse())
        }) else {
            anyhow::bail!("No frequency given in {:?}", block);
        };
        let frequency = frequency??;

        let Some(content) = toml.get("content").map(|content| {
            content
                .as_str()
                .ok_or(anyhow::anyhow!("Unknown content {:?}", content))
        }) else {
            anyhow::bail!("No content given in {:?}", block);
        };
        let content = content?.to_string();

        let from = if let Some(from) = toml.get("from") {
            from.as_str().map(|from| from.parse()).transpose()?
        } else {
            None
        };

        let to = if let Some(to) = toml.get("to") {
            to.as_str().map(|to| to.parse()).transpose()?
        } else {
            None
        };

        let mut weekdays : Vec<Weekday> = vec![];

        if let Some(entry) = toml.get("weekdays") {
            if let Some(array) = entry.as_array() {
                for value in array {
                    if let Some(string) = value.as_str() {
                        weekdays.push(string.parse()?);
                    } else {
                        anyhow::bail!("weekdays values should be strings, not {:?}", value);
                    }
                }
            } else {
                anyhow::bail!("weekdays should be an array, not {:?}", entry);
            }
        }

        let mut monthdays = vec![];

        if let Some(entry) = toml.get("monthdays") {
            if let Some(array) = entry.as_array() {
                for value in array {
                    if let Some(integer) = value.as_integer() {
                        monthdays.push(integer as usize);
                    } else {
                        anyhow::bail!("monthdays values should be integers, not {:?}", value);
                    }
                }
            } else {
                anyhow::bail!("monthdays should be an array, not {:?}", entry);
            }
        }

        let mut yeardays = vec![];

        if let Some(entry) = toml.get("yeardays") {
            if let Some(array) = entry.as_array() {
                for value in array {
                    if let Some(integer) = value.as_integer() {
                        yeardays.push(integer as usize);
                    } else {
                        anyhow::bail!("yeardays values should be integers, not {:?}", value);
                    }
                }
            } else {
                anyhow::bail!("yeardays should be an array, not {:?}", entry);
            }
        }

        let event = Event {
            frequency,
            content,
            from,
            to,
            weekdays,
            monthdays,
            yeardays,
        };

        event.valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Frequency::*;

    #[test]
    fn frequency_from_str() -> Result<()> {
        assert!(matches!("DAILY".parse::<Frequency>()?, Daily));
        assert!(matches!("WeekLy".parse::<Frequency>()?, Weekly));
        assert!(matches!("MonthLy".parse::<Frequency>()?, Monthly));
        assert!(matches!("YearLy".parse::<Frequency>()?, Yearly));
        assert!(matches!("Other".parse::<Frequency>(), Err(_)));

        Ok(())
    }

    mod event_from_code_block {
        use super::*;
        use indoc::indoc;

        fn block(content: &str) -> CodeBlock {
            CodeBlock { kind: "toml".to_string(), code: content.to_string() }
        }

        #[test]
        fn not_a_toml_block() {
            let block = CodeBlock { kind: "foo".to_string(), code: "".to_string() };
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
                frequency = Daily
            "#});
            assert!(Event::try_from(block).is_err());
        }

        #[test]
        fn simple() -> Result<()> {
            let block = block(indoc! {r#"
                frequency = "Daily"
                content = "Foo"
            "#});
            let event = Event::try_from(block)?;
            assert!(matches!(event.frequency, Daily));
            assert_eq!("Foo", event.content);
            Ok(())
        }

        #[test]
        fn dates() -> Result<()> {
            let block = block(indoc! {r#"
                frequency = "Daily"
                content = "Foo"
                from = "2025-01-01"
                to = "2025-01-31"
            "#});
            let event = Event::try_from(block)?;
            assert_eq!("2025-01-01".parse().ok(), event.from);
            assert_eq!("2025-01-31".parse().ok(), event.to);
            Ok(())
        }
    }
}
