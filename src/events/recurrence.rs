use crate::date_utils::{InvalidMonthday, InvalidYearday, Month, Monthday, Yearday};
use anyhow::{Error, Result};
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, derive_more::IsVariant)]
#[serde(rename_all = "snake_case")]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Once,
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
    /// Once on specific dates
    Once(Vec<NaiveDate>),
}

impl Recurrence {
    pub fn matches(&self, date: NaiveDate) -> bool {
        use Recurrence::*;
        match self {
            Daily => true,
            Weekly(weekdays) => weekdays.contains(&date.weekday()),
            Monthly(monthdays) => monthdays.contains(&Monthday::try_from(date.day()).unwrap()),
            Yearly(yeardays) => yeardays.contains(&Yearday::try_from(date.ordinal()).unwrap()),
            Once(dates) => dates.contains(&date),

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

#[derive(Debug, Serialize, Deserialize)]
pub struct SerdeRecurrence {
    frequency: Frequency,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    weekdays: Vec<Weekday>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    monthdays: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    yeardays: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dates: Vec<NaiveDate>,
    index: Option<WeekIndex>,
}

impl TryFrom<SerdeRecurrence> for Recurrence {
    type Error = Error;

    fn try_from(serde: SerdeRecurrence) -> Result<Self> {
        Ok(match serde.frequency {
            Frequency::Daily => {
                if !serde.weekdays.is_empty() {
                    anyhow::bail!("`weekdays` not allowed for daily recurrence");
                }
                if !serde.monthdays.is_empty() {
                    anyhow::bail!("`monthdays` not allowed for daily recurrence");
                }
                if !serde.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` not allowed for daily recurrence");
                }
                if !serde.dates.is_empty() {
                    anyhow::bail!("`dates` not allowed for daily recurrence");
                }
                Recurrence::Daily
            }
            Frequency::Weekly => {
                if !serde.monthdays.is_empty() {
                    anyhow::bail!("`monthdays` not allowed for weekly recurrence");
                }
                if !serde.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` not allowed for weekly recurrence");
                }
                if !serde.dates.is_empty() {
                    anyhow::bail!("`dates` not allowed for weekly recurrence");
                }
                if serde.weekdays.is_empty() {
                    anyhow::bail!("`weekdays` must be specified");
                }
                Recurrence::Weekly(serde.weekdays)
            }
            Frequency::Monthly => {
                if !serde.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` not allowed for monthly recurrence");
                }
                if !serde.dates.is_empty() {
                    anyhow::bail!("`dates` not allowed for monthly recurrence");
                }
                if serde.weekdays.is_empty() {
                    if serde.monthdays.is_empty() {
                        anyhow::bail!("Either `monthdays` or `weekdays` must be specified");
                    }
                    Recurrence::Monthly(
                        serde
                            .monthdays
                            .into_iter()
                            .map(Monthday::try_from)
                            .collect::<Result<Vec<_>, InvalidMonthday>>()?,
                    )
                } else {
                    Recurrence::RelativeMonthly(
                        serde.weekdays,
                        serde.index.unwrap_or(WeekIndex::First),
                    )
                }
            }
            Frequency::Yearly => {
                if !serde.weekdays.is_empty() {
                    anyhow::bail!("`weekdays` not allowed for yearly recurrence");
                }
                if !serde.monthdays.is_empty() {
                    anyhow::bail!("`monthdays` not allowed for yearly recurrence");
                }
                if !serde.dates.is_empty() {
                    anyhow::bail!("`dates` not allowed for yearly recurrence");
                }
                if serde.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` must be specified");
                }
                Recurrence::Yearly(
                    serde
                        .yeardays
                        .into_iter()
                        .map(Yearday::try_from)
                        .collect::<Result<Vec<_>, InvalidYearday>>()?,
                )
            }
            Frequency::Once => {
                if !serde.weekdays.is_empty() {
                    anyhow::bail!("`weekdays` not allowed for once recurrence");
                }
                if !serde.monthdays.is_empty() {
                    anyhow::bail!("`monthdays` not allowed for once recurrence");
                }
                if !serde.yeardays.is_empty() {
                    anyhow::bail!("`yeardays` not allowed for once recurrence");
                }
                if serde.dates.is_empty() {
                    anyhow::bail!("`dates` must be specified");
                }
                Recurrence::Once(serde.dates)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Event;
    use crate::page::content::CodeBlock;
    use claim::{assert_err, assert_ok};

    fn block(content: &str) -> CodeBlock {
        CodeBlock {
            kind: "toml".to_owned(),
            code: content.to_owned(),
        }
    }

    fn monthday(index: u32) -> Monthday {
        Monthday::try_from(index).unwrap()
    }

    fn yearday(index: u32) -> Yearday {
        Yearday::try_from(index).unwrap()
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
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

        assert!(Monthly(vec![monthday(1)]).matches(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()));
        assert!(!Monthly(vec![monthday(1)]).matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
        assert!(Monthly(vec![monthday(1), monthday(2)])
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

        assert!(Yearly(vec![yearday(32)]).matches(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()));
        assert!(!Yearly(vec![yearday(32)]).matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
        assert!(Yearly(vec![yearday(32), yearday(33)])
            .matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));

        assert!(Once(vec![NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()])
            .matches(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()));
        assert!(!Once(vec![NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()])
            .matches(NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()));
        assert!(Once(vec![
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 2).unwrap()
        ])
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

        #[test]
        fn daily_dates() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "daily"
                dates = ["2026-02-03"]
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

        #[test]
        fn weekly_dates() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "weekly"
                dates = ["2026-02-03"]
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

            assert_eq!(Recurrence::Monthly(vec![monthday(1)]), event.recurrence);
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

        #[test]
        fn monthly_dates() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "monthly"
                dates = ["2026-02-03"]
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

            assert_eq!(Recurrence::Yearly(vec![yearday(1)]), event.recurrence);
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

        #[test]
        fn yearly_dates() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "yearly"
                dates = ["2026-02-03"]
                content = "Happy new year"
            "#,
            )));
        }
    }

    mod once {
        use super::*;

        #[test]
        fn once_dates() {
            let event = assert_ok!(Event::try_from(&block(
                r#"
                frequency = "once"
                dates = ["2026-02-03"]
                content = "Special date"
            "#,
            )));

            assert_eq!(Recurrence::Once(vec![date(2026, 2, 3)]), event.recurrence);
        }

        #[test]
        fn once_empty_dates() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "once"
                content = "Special date"
            "#,
            )));
        }

        #[test]
        fn once_weekdays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "once"
                weekdays = ["Monday"]
                content = "Special date"
            "#,
            )));
        }

        #[test]
        fn once_monthdays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "once"
                monthdays = [1]
                content = "Special date"
            "#,
            )));
        }

        #[test]
        fn once_yeardays() {
            assert_err!(Event::try_from(&block(
                r#"
                frequency = "once"
                yeardays = [1]
                content = "Special date"
            "#,
            )));
        }
    }
}
