use crate::page::CodeBlock;
use anyhow::{Error, Result};
use chrono::{Datelike, NaiveDate, Weekday};
use std::str::FromStr;
use toml::Table;

/// Describe a recurring event
#[derive(Debug)]
pub struct Event {
    recurrence: Recurrence,
    pub content: String,
    validity: DateRange,
    exceptions: Vec<DateRange>,
}

#[derive(Debug)]
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

impl TryFrom<&Table> for Recurrence {
    type Error = Error;

    fn try_from(toml: &Table) -> Result<Self> {
        let Some(frequency) = toml.get("frequency").map(|frequency| {
            frequency
                .as_str()
                .ok_or(anyhow::anyhow!("Unknown frequency {:?}", frequency))
                .map(Frequency::from_str)
        }) else {
            anyhow::bail!("`frequency` is required");
        };
        let frequency = frequency??;

        match frequency {
            Frequency::Daily => {
                if toml.contains_key("weekdays") {
                    anyhow::bail!("`weekdays` not allowed for daily recurrence");
                }
                if toml.contains_key("monthdays") {
                    anyhow::bail!("`monthdays` not allowed for daily recurrence");
                }
                if toml.contains_key("yeardays") {
                    anyhow::bail!("`yeardays` not allowed for daily recurrence");
                }
                Ok(Recurrence::Daily)
            }
            Frequency::Weekly => {
                if toml.contains_key("monthdays") {
                    anyhow::bail!("`monthdays` not allowed for weekly recurrence");
                }
                if toml.contains_key("yeardays") {
                    anyhow::bail!("`yeardays` not allowed for weekly recurrence");
                }

                let Some(Some(array)) = toml.get("weekdays").map(|e| e.as_array()) else {
                    anyhow::bail!(
                        "`weekdays` required for weekly recurrence and should be an array"
                    );
                };

                array
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .ok_or(anyhow::anyhow!(
                                "`weekdays` values should be strings, not {:?}",
                                value
                            ))
                            .and_then(|string| {
                                Weekday::from_str(string).map_err(|err| {
                                    anyhow::anyhow!(
                                        "`weekdays` values should be parsable week days: {:?}",
                                        err
                                    )
                                })
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(Recurrence::Weekly)
            }
            Frequency::Monthly => {
                if toml.contains_key("weekdays") {
                    anyhow::bail!("`weekdays` not allowed for daily recurrence");
                }
                if toml.contains_key("yeardays") {
                    anyhow::bail!("`yeardays` not allowed for daily recurrence");
                }

                let Some(Some(array)) = toml.get("monthdays").map(|e| e.as_array()) else {
                    anyhow::bail!(
                        "`monthdays` required for monthly recurrence and should be an array"
                    );
                };

                array
                    .iter()
                    .map(|value| {
                        value
                            .as_integer()
                            .ok_or(anyhow::anyhow!(
                                "`monthdays` values should be integers, not {:?}",
                                value
                            ))
                            .map(|i| i as usize)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(Recurrence::Monthly)
            }
            Frequency::Yearly => {
                if toml.contains_key("weekdays") {
                    anyhow::bail!("`weekdays` not allowed for daily recurrence");
                }
                if toml.contains_key("monthdays") {
                    anyhow::bail!("`monthdays` not allowed for daily recurrence");
                }

                let Some(Some(array)) = toml.get("yeardays").map(|e| e.as_array()) else {
                    anyhow::bail!(
                        "`yeardays` required for yearly recurrence and should be an array"
                    );
                };

                array
                    .iter()
                    .map(|value| {
                        value
                            .as_integer()
                            .ok_or(anyhow::anyhow!(
                                "`yeardays` values should be integers, not {:?}",
                                value
                            ))
                            .map(|i| i as usize)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(Recurrence::Yearly)
            }
        }
    }
}

#[derive(Debug, derive_more::IsVariant)]
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

#[derive(Debug, Default)]
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

impl TryFrom<&Table> for DateRange {
    type Error = Error;

    fn try_from(toml: &Table) -> Result<DateRange> {
        let mut range = DateRange::default();

        if let Some(from) = toml.get("from") {
            range.from = from.as_str().map(|from| from.parse()).transpose()?;
        }

        if let Some(to) = toml.get("to") {
            range.to = to.as_str().map(|to| to.parse()).transpose()?;
        }

        if range.from.is_some() && range.to.is_some() && range.from >= range.to {
            anyhow::bail!(
                "Invalid range, {:?} should be strictly less than {:?}",
                range.from,
                range.to
            );
        }

        Ok(range)
    }
}

impl TryFrom<&toml::Value> for DateRange {
    type Error = Error;

    fn try_from(value: &toml::Value) -> Result<DateRange> {
        if let Some(table) = value.as_table() {
            Self::try_from(table)
        } else {
            anyhow::bail!("DateRange must be built from table not {:?}", value);
        }
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
        let toml = block.code.parse::<Table>()?;

        let Some(content) = toml.get("content").map(|content| {
            content
                .as_str()
                .ok_or(anyhow::anyhow!("Unknown content {:?}", content))
        }) else {
            anyhow::bail!("No content given in {:?}", block);
        };
        let content = content?.to_string();

        let recurrence = Recurrence::try_from(&toml)?;
        let validity = DateRange::try_from(&toml)?;

        let mut exceptions = vec![];

        if let Some(entry) = toml.get("exceptions") {
            if let Some(array) = entry.as_array() {
                for value in array {
                    exceptions.push(DateRange::try_from(value)?);
                }
            } else {
                anyhow::bail!("exceptions should be an array, not {:?}", entry);
            }
        }

        Ok(Event {
            recurrence,
            content,
            validity,
            exceptions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequency_from_str() -> Result<()> {
        assert!("DAILY".parse::<Frequency>()?.is_daily());
        assert!("WeekLy".parse::<Frequency>()?.is_weekly());
        assert!("MonthLy".parse::<Frequency>()?.is_monthly());
        assert!("YearLy".parse::<Frequency>()?.is_yearly());
        assert!("Other".parse::<Frequency>().is_err());

        Ok(())
    }

    mod date_range_from_toml {
        use super::*;

        #[test]
        fn table() -> Result<()> {
            let toml = r#"
                from = "2025-08-01"
                to = "2025-08-10"
            "#
            .parse::<Table>()?;
            let range = DateRange::try_from(&toml)?;

            assert_eq!(NaiveDate::from_ymd_opt(2025, 8, 1), range.from);
            assert_eq!(NaiveDate::from_ymd_opt(2025, 8, 10), range.to);
            Ok(())
        }

        #[test]
        fn invalid_range() -> Result<()> {
            let toml = r#"
                from = "2025-08-11"
                to = "2025-08-01"
            "#
            .parse::<Table>()?;
            let range = DateRange::try_from(&toml);

            assert!(range.is_err());
            Ok(())
        }

        #[test]
        fn empty_range() -> Result<()> {
            let toml = r#"
                from = "2025-08-01"
                to = "2025-08-01"
            "#
            .parse::<Table>()?;
            let range = DateRange::try_from(&toml);

            assert!(range.is_err());
            Ok(())
        }

        #[test]
        fn value() -> Result<()> {
            let toml = r#"
                from = "2025-08-01"
                to = "2025-08-10"
            "#
            .parse::<toml::Value>()?;
            let range = DateRange::try_from(&toml)?;

            assert_eq!(NaiveDate::from_ymd_opt(2025, 8, 1), range.from);
            assert_eq!(NaiveDate::from_ymd_opt(2025, 8, 10), range.to);
            Ok(())
        }

        #[test]
        fn from_only() -> Result<()> {
            let toml = r#"
                from = "2025-08-01"
            "#
            .parse::<Table>()?;
            let range = DateRange::try_from(&toml)?;

            assert_eq!(NaiveDate::from_ymd_opt(2025, 8, 1), range.from);
            assert_eq!(None, range.to);
            Ok(())
        }

        #[test]
        fn to_only() -> Result<()> {
            let toml = r#"
                to = "2025-08-01"
            "#
            .parse::<Table>()?;
            let range = DateRange::try_from(&toml)?;

            assert_eq!(None, range.from);
            assert_eq!(NaiveDate::from_ymd_opt(2025, 8, 1), range.to);
            Ok(())
        }
    }

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
            assert!(matches!(event.recurrence, Recurrence::Daily));
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
            assert_eq!("2025-01-01".parse().ok(), event.validity.from);
            assert_eq!("2025-01-31".parse().ok(), event.validity.to);
            Ok(())
        }
    }
}
