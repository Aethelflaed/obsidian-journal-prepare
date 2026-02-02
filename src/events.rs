use crate::date_utils::Month;
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
                if event.weekdays.is_empty() {
                    anyhow::bail!("`weekdays` must be specified");
                }
                Recurrence::Weekly(event.weekdays)
            }
            Frequency::Monthly => {
                if !event.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` not allowed for monthly recurrence");
                }
                if event.weekdays.is_empty() {
                    if event.monthdays.is_empty() {
                        anyhow::bail!("Either `monthdays` or `weekdays` must be specified");
                    }
                    Recurrence::Monthly(
                        event
                            .monthdays
                            .into_iter()
                            .map(Monthday::try_from)
                            .collect::<Result<Vec<_>>>()?,
                    )
                } else {
                    Recurrence::RelativeMonthly(
                        event.weekdays,
                        event.index.unwrap_or(WeekIndex::First),
                    )
                }
            }
            Frequency::Yearly => {
                if !event.weekdays.is_empty() {
                    anyhow::bail!("`weekdays` not allowed for yearly recurrence");
                }
                if !event.monthdays.is_empty() {
                    anyhow::bail!("`monthdays` not allowed for yearly recurrence");
                }
                if event.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` must be specified");
                }
                Recurrence::Yearly(
                    event
                        .yeardays
                        .into_iter()
                        .map(Yearday::try_from)
                        .collect::<Result<Vec<_>>>()?,
                )
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
    monthdays: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    yeardays: Vec<u32>,
    index: Option<WeekIndex>,
    content: String,
    #[serde(flatten)]
    validity: DateRange,
    #[serde(default)]
    exceptions: Vec<DateRange>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Recurrence {
    Daily,
    /// Weekly every Weekday
    Weekly(Vec<Weekday>),
    /// Monthly each Nth day, starting from 1
    Monthly(Vec<Monthday>),
    /// Relative monthly, e.g. each First Monday
    RelativeMonthly(Vec<Weekday>, WeekIndex),
    /// Yearly each Nth day, starting from 1
    Yearly(Vec<Yearday>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Monthday(u32);

impl TryFrom<u32> for Monthday {
    type Error = Error;

    fn try_from(index: u32) -> Result<Monthday> {
        if index > 0 && index < 32 {
            Ok(Self(index))
        } else {
            anyhow::bail!("Monthday {index} is invalid")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Yearday(u32);

impl TryFrom<u32> for Yearday {
    type Error = Error;

    fn try_from(index: u32) -> Result<Yearday> {
        if index > 0 && index < 367 {
            Ok(Self(index))
        } else {
            anyhow::bail!("Yearday {index} is invalid")
        }
    }
}

impl Recurrence {
    pub fn matches(&self, date: NaiveDate) -> bool {
        use Recurrence::*;
        match self {
            Daily => true,
            Weekly(weekdays) => weekdays.contains(&date.weekday()),
            Monthly(monthdays) => monthdays.contains(&Monthday(date.day())),
            Yearly(yeardays) => yeardays.contains(&Yearday(date.ordinal())),

            RelativeMonthly(weekdays, index) => {
                if weekdays.contains(&date.weekday()) {
                    let monthday0 = date.day0();
                    let week_index = monthday0 / 7;
                    let month = Month::from(date);
                    let from_last_index = (month.num_days() - date.day()) / 7;

                    match index {
                        WeekIndex::First => week_index == 0,
                        WeekIndex::Second => week_index == 1,
                        WeekIndex::Third => week_index == 2,
                        WeekIndex::Fourth => week_index == 3,
                        WeekIndex::Last => from_last_index == 0,
                    }
                } else {
                    false
                }
            }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::IsVariant)]
#[serde(rename_all = "snake_case")]
pub enum WeekIndex {
    First,
    Second,
    Third,
    Fourth,
    Last,
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

impl TryFrom<&CodeBlock> for Event {
    type Error = Error;

    fn try_from(block: &CodeBlock) -> Result<Event> {
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
    use claim::{assert_err, assert_ok};

    fn block(content: &str) -> CodeBlock {
        CodeBlock {
            kind: "toml".to_owned(),
            code: content.to_owned(),
        }
    }

    #[test]
    fn try_from_not_a_toml_block() {
        assert_err!(Event::try_from(&CodeBlock {
            kind: "foo".to_owned(),
            code: "".to_owned(),
        }));
    }

    #[test]
    fn no_frequency() {
        assert_err!(Event::try_from(&block(r#"content = "foo""#)));
    }

    #[test]
    fn no_content() {
        assert_err!(Event::try_from(&block(r#"frequency = "daily""#)));
    }

    #[test]
    fn simple() -> Result<()> {
        let block = block(
            r#"
                frequency = "daily"
                content = "Foo"
            "#,
        );
        let event = Event::try_from(&block)?;
        assert!(matches!(event.recurrence, Recurrence::Daily));
        assert_eq!("Foo", event.content);
        Ok(())
    }

    #[test]
    fn dates() {
        let event = assert_ok!(Event::try_from(&block(
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

    #[test]
    fn recurrence_matches() {
        use Recurrence::*;
        use WeekIndex::*;
        use Weekday::*;

        assert!(Daily.matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
        assert!(Daily.matches(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()));

        assert!(Weekly(vec![Mon]).matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
        assert!(!Weekly(vec![Mon]).matches(NaiveDate::from_ymd_opt(2026, 2, 3).unwrap()));
        assert!(Weekly(vec![Mon, Tue]).matches(NaiveDate::from_ymd_opt(2026, 2, 3).unwrap()));

        assert!(Monthly(vec![Monthday(1)]).matches(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()));
        assert!(!Monthly(vec![Monthday(1)]).matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
        assert!(Monthly(vec![Monthday(1), Monthday(2)])
            .matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));

        assert!(!RelativeMonthly(vec![Mon], First)
            .matches(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()));
        assert!(
            RelativeMonthly(vec![Sun], First).matches(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap())
        );
        assert!(RelativeMonthly(vec![Sun, Mon], First)
            .matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
        assert!(!RelativeMonthly(vec![Sun, Mon], First)
            .matches(NaiveDate::from_ymd_opt(2026, 2, 8).unwrap()));
        assert!(RelativeMonthly(vec![Sun, Mon], Second)
            .matches(NaiveDate::from_ymd_opt(2026, 2, 8).unwrap()));
        assert!(!RelativeMonthly(vec![Sun, Mon], Third)
            .matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
        assert!(RelativeMonthly(vec![Sun], Fourth)
            .matches(NaiveDate::from_ymd_opt(2026, 2, 22).unwrap()));
        assert!(
            RelativeMonthly(vec![Sun], Last).matches(NaiveDate::from_ymd_opt(2026, 2, 22).unwrap())
        );

        assert!(Yearly(vec![Yearday(32)]).matches(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()));
        assert!(!Yearly(vec![Yearday(32)]).matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
        assert!(Yearly(vec![Yearday(32), Yearday(33)])
            .matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
    }

    mod daily {
        use super::*;

        #[test]
        fn daily() {
            assert_ok!(Event::try_from(&block(
                r#"
                frequency = "daily"
                content = "Daily"
            "#,
            )));
        }

        #[test]
        fn daily_weekdays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "daily"
                weekdays = ["Monday"]
                content = "Daily"
            "#,
            )));
        }

        #[test]
        fn daily_monthdays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "daily"
                monthdays = [1]
                content = "Daily"
            "#,
            )));
        }

        #[test]
        fn daily_yeardays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "daily"
                yeardays = [1]
                content = "Daily"
            "#,
            )));
        }
    }

    mod weekly {
        use super::*;

        #[test]
        fn weekly_weekdays() {
            let event = assert_ok!(Event::try_from(&block(
                r#"
                frequency = "weekly"
                weekdays = ["Monday"]
                content = "Weekly"
            "#,
            )));

            assert_eq!(Recurrence::Weekly(vec![Weekday::Mon]), event.recurrence);
        }

        #[test]
        fn weekly_empty_weekdays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "weekly"
                content = "Weekly"
            "#,
            )));
        }

        #[test]
        fn weekly_monthdays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "weekly"
                monthdays = [1]
                content = "Weekly"
            "#,
            )));
        }

        #[test]
        fn weekly_yeardays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "weekly"
                yeardays = [1]
                content = "Weekly"
            "#,
            )));
        }
    }

    mod monthly {
        use super::*;

        #[test]
        fn monthly_unspecified() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "monthly"
                content = "Weekly"
            "#,
            )));
        }

        #[test]
        fn monthly_weekdays() {
            let event = assert_ok!(Event::try_from(&block(
                r#"
                frequency = "monthly"
                weekdays = ["Monday"]
                content = "Weekly"
            "#,
            )));

            assert_eq!(
                Recurrence::RelativeMonthly(vec![Weekday::Mon], WeekIndex::First),
                event.recurrence
            );
        }

        #[test]
        fn monthly_weekdays_index() {
            let event = assert_ok!(Event::try_from(&block(
                r#"
                frequency = "monthly"
                weekdays = ["Sunday", "Friday"]
                index = "last"
                content = "Weekly"
            "#,
            )));

            assert_eq!(
                Recurrence::RelativeMonthly(vec![Weekday::Sun, Weekday::Fri], WeekIndex::Last),
                event.recurrence
            );
        }

        #[test]
        fn monthly_monthdays() {
            let event = assert_ok!(Event::try_from(&block(
                r#"
                frequency = "monthly"
                monthdays = [1]
                content = "Weekly"
            "#,
            )));

            assert_eq!(Recurrence::Monthly(vec![Monthday(1)]), event.recurrence);
        }

        #[test]
        fn monthly_yeardays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "monthly"
                yeardays = [1]
                content = "Weekly"
            "#,
            )));
        }
    }

    mod yearly {
        use super::*;

        #[test]
        fn yearly_yeardays() {
            let event = assert_ok!(Event::try_from(&block(
                r#"
                frequency = "yearly"
                yeardays = [1]
                content = "Happy new year"
            "#,
            )));

            assert_eq!(Recurrence::Yearly(vec![Yearday(1)]), event.recurrence);
        }

        #[test]
        fn yearly_empty_yeardays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "yearly"
                content = "Happy new year"
            "#,
            )));
        }

        #[test]
        fn yearly_weekdays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "yearly"
                weekdays = ["Monday"]
                content = "Happy new year"
            "#,
            )));
        }

        #[test]
        fn yearly_monthdays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "yearly"
                monthdays = [1]
                content = "Happy new year"
            "#,
            )));
        }
    }
}
