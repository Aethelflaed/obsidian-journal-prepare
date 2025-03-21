use crate::page::CodeBlock;
use anyhow::{Error, Result};
use chrono::{NaiveDate, Weekday};
use std::str::FromStr;
use toml::Table;

#[derive(Debug)]
pub struct Event {
    frequency: Frequency,
    content: String,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    weekdays: Vec<WeekdayCriterion>,
    monthdays: Vec<usize>,
    yeardays: Vec<usize>,
    interval: usize,
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

#[derive(Debug)]
pub struct WeekdayCriterion {
    day: Weekday,
    index: isize,
}

impl TryFrom<CodeBlock> for Event {
    type Error = Error;

    fn try_from(block: CodeBlock) -> Result<Event> {
        if block.kind != "toml" {
            anyhow::bail!("Not a toml block");
        }
        let toml = block.code.parse::<Table>()?;
        let frequency = toml.get("frequency").map(|frequency| {
            frequency
                .as_str()
                .ok_or(anyhow::anyhow!("Unknown frequency {:?}", frequency))
                .map(|str_freq| str_freq.parse())
        });
        if frequency.is_none() {
            anyhow::bail!("No frequency given in {:?}", block);
        }
        let frequency = frequency.unwrap()??;
        let content = toml.get("content").map(|content| {
            content
                .as_str()
                .ok_or(anyhow::anyhow!("Unknown content {:?}", content))
        });
        if content.is_none() {
            anyhow::bail!("No content given in {:?}", block);
        }
        let content = content.unwrap()?.to_string();

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

        let interval = toml
            .get("interval")
            .map(|i| i.as_integer())
            .flatten()
            .unwrap_or(1) as usize;

        let monthdays = Into::<Result<Vec<_>>>::into(
            toml.get("monthdays")
                .map(|monthdays| {
                    monthdays.as_array().map(|monthdays| {
                        monthdays.iter().map(|value| {
                            value.as_integer().ok_or(anyhow::anyhow!(
                                "monthdays values should be integers, not {:?}",
                                value
                            ))
                        })
                    })
                })
                .flatten(),
        )?;

        let mut event = Event {
            frequency,
            content,
            from,
            to,
            weekdays: Default::default(),
            monthdays: Default::default(),
            yeardays: Default::default(),
            interval,
        };

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequency_from_str() -> Result<()> {
        use Frequency::*;

        assert!(matches!("DAILY".parse::<Frequency>()?, Daily));
        assert!(matches!("WeekLy".parse::<Frequency>()?, Weekly));
        assert!(matches!("MonthLy".parse::<Frequency>()?, Monthly));
        assert!(matches!("YearLy".parse::<Frequency>()?, Yearly));
        assert!(matches!("Other".parse::<Frequency>(), Err(_)));

        Ok(())
    }
}
