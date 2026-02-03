use anyhow::Result;
use chrono::{Datelike, Days, IsoWeek, NaiveDate, Weekday};

use super::Vault;
use crate::date_utils::{Month, Navigation, ToDateIterator, Year};
use crate::options::{GenericPage, GenericSettings, PageOptions};
use crate::utils::{ToEmbedded, ToLink};

pub struct Preparer<'a> {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub page_options: PageOptions,
    pub vault: &'a Vault,
}

fn weekday(date: NaiveDate) -> &'static str {
    match date.weekday() {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

impl Preparer<'_> {
    pub fn run(&self) -> Result<()> {
        log::info!(
            "Preparing journal {:?} from {} to {}",
            self.vault.path(),
            self.from,
            self.to
        );
        log::debug!("day options: {:?}", self.page_options.day);
        log::debug!("week options: {:?}", self.page_options.week);
        log::debug!("month options: {:?}", self.page_options.month);
        log::debug!("year options: {:?}", self.page_options.year);

        let mut date: NaiveDate = self.from;
        let mut year = Year::from(date.year());
        let mut month = Month::from(date);
        let mut week = date.iso_week();

        self.day(date)?;
        self.week(week)?;
        self.month(month)?;
        self.year(year)?;

        while date < self.to {
            date = date + Days::new(1);
            self.day(date)?;

            let new_week = date.iso_week();
            if week != new_week {
                self.week(new_week)?;
                week = new_week;
            }

            let new_year = Year::from(date.year());
            if year != new_year {
                self.year(new_year)?;
                year = new_year;
            }

            let new_month = Month::from(date);
            if month != new_month {
                self.month(new_month)?;
                month = new_month;
            }
        }
        Ok(())
    }

    fn year(&self, year: Year) -> Result<()> {
        let settings = self.page_options.year.settings();
        if settings.is_empty() {
            return Ok(());
        }

        self.vault.update(&year, |mut page| {
            if settings.nav_link {
                page.insert_property("next", year.next().to_link(self.vault));
                page.insert_property("prev", year.prev().to_link(self.vault));
            }
            if settings.month {
                page.prepend_lines(year.iter().map(|month| month.to_link(self.vault)));
            }

            Ok(page)
        })
    }

    fn month(&self, month: Month) -> Result<()> {
        let settings = self.page_options.month.settings();
        if settings.is_empty() {
            return Ok(());
        }

        self.vault.update(&month, |mut page| {
            if settings.nav_link {
                page.insert_property("next", month.next().to_link(self.vault));
                page.insert_property("prev", month.prev().to_link(self.vault));
            }
            if settings.month {
                // 31 days max plus 5 weeks headers
                let mut lines = Vec::with_capacity(36);
                for (index, date) in month.iter().enumerate() {
                    if index == 0 || date.weekday() == Weekday::Mon {
                        lines.push(format!("#### {}", date.iso_week().to_link(self.vault)));
                    }
                    lines.push(format!(
                        "- {} {}",
                        weekday(date),
                        date.to_link(self.vault).into_embedded()
                    ));
                }

                page.prepend_lines(lines);
            }

            Ok(page)
        })
    }

    fn week(&self, week: IsoWeek) -> Result<()> {
        let settings = self.page_options.week.settings();
        if settings.is_empty() {
            return Ok(());
        }

        self.vault.update(&week, |mut page| {
            if settings.link_to_month {
                page.insert_property("month", Month::from(week).to_link(self.vault));
            }
            if settings.nav_link {
                page.insert_property("next", week.next().to_link(self.vault));
                page.insert_property("prev", week.prev().to_link(self.vault));
            }
            if settings.week {
                page.prepend_lines(week.iter().map(|date| {
                    format!(
                        "- {} {}",
                        weekday(date),
                        date.to_link(self.vault).into_embedded()
                    )
                }));
            }

            Ok(page)
        })
    }

    fn day(&self, date: NaiveDate) -> Result<()> {
        let settings = self.page_options.day.settings();
        if settings.is_empty() {
            return Ok(());
        }

        self.vault.update(&date, |mut page| {
            if settings.day_of_week {
                page.insert_property("day", weekday(date));
            }
            if settings.link_to_week {
                page.insert_property("week", date.iso_week().to_link(self.vault));
            }
            if settings.link_to_month {
                page.insert_property("month", Month::from(date).to_link(self.vault));
            }
            if settings.nav_link {
                page.insert_property("next", date.next().to_link(self.vault));
                page.insert_property("prev", date.prev().to_link(self.vault));
            }
            if settings.events {
                page.prepend_lines(
                    self.vault
                        .events()
                        .filter(|ev| ev.matches(date))
                        .map(|ev| &ev.content),
                );
            }

            Ok(page)
        })
    }
}
