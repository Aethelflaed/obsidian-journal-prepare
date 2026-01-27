use anyhow::Result;
use chrono::{Datelike, Days, IsoWeek, NaiveDate, Weekday};

use super::Vault;
use crate::date_utils::{Month, Navigation, ToDateIterator, Year};
use crate::options::{GenericPage, GenericSettings, PageOptions};
use crate::page::property::ToProperty;
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

        self.print_day(date)?;
        self.print_week(week)?;
        self.print_month(month)?;
        self.print_year(year)?;

        while date < self.to {
            date = date + Days::new(1);
            self.print_day(date)?;

            let new_week = date.iso_week();
            if week != new_week {
                self.print_week(new_week)?;
                week = new_week;
            }

            let new_year = Year::from(date.year());
            if year != new_year {
                self.print_year(new_year)?;
                year = new_year;
            }

            let new_month = Month::from(date);
            if month != new_month {
                self.print_month(new_month)?;
                month = new_month;
            }
        }
        Ok(())
    }

    fn print_year(&self, year: Year) -> Result<()> {
        let settings = self.page_options.year.settings();
        if settings.is_empty() {
            return Ok(());
        }

        self.vault.update(year, |mut page| {
            if settings.nav_link {
                page.push_property(year.next().to_link(self.vault).to_property("next"));
                page.push_property(year.prev().to_link(self.vault).to_property("prev"));
            }
            if settings.month {
                for month in year.iter() {
                    page.push_content(month.to_link(self.vault));
                }
            }

            Ok(page)
        })
    }

    fn print_month(&self, month: Month) -> Result<()> {
        let settings = self.page_options.month.settings();
        if settings.is_empty() {
            return Ok(());
        }

        self.vault.update(month, |mut page| {
            if settings.nav_link {
                page.push_property(month.next().to_link(self.vault).to_property("next"));
                page.push_property(month.prev().to_link(self.vault).to_property("prev"));
            }
            if settings.month {
                for (index, date) in month.iter().enumerate() {
                    if index == 0 || date.weekday() == Weekday::Mon {
                        page.push_content(format!("#### {}", date.iso_week().to_link(self.vault)));
                    }
                    page.push_content(format!(
                        "- {} {}",
                        weekday(date),
                        date.to_link(self.vault).into_embedded()
                    ));
                }
            }

            Ok(page)
        })
    }

    fn print_week(&self, week: IsoWeek) -> Result<()> {
        let settings = self.page_options.week.settings();
        if settings.is_empty() {
            return Ok(());
        }

        self.vault.update(week, |mut page| {
            if settings.link_to_month {
                page.push_property(Month::from(week).to_link(self.vault).to_property("month"));
            }
            if settings.nav_link {
                page.push_property(week.next().to_link(self.vault).to_property("next"));
                page.push_property(week.prev().to_link(self.vault).to_property("prev"));
            }
            if settings.week {
                for date in week.iter() {
                    page.push_content(format!(
                        "- {} {}",
                        weekday(date),
                        date.to_link(self.vault).into_embedded()
                    ));
                }
            }

            Ok(page)
        })
    }

    fn print_day(&self, date: NaiveDate) -> Result<()> {
        let settings = self.page_options.day.settings();
        if settings.is_empty() {
            return Ok(());
        }

        self.vault.update(date, |mut page| {
            if settings.day_of_week {
                page.push_property(weekday(date).to_property("day"));
            }
            if settings.link_to_week {
                page.push_property(date.iso_week().to_link(self.vault).to_property("week"));
            }
            if settings.link_to_month {
                page.push_property(Month::from(date).to_link(self.vault).to_property("month"));
            }
            if settings.nav_link {
                page.push_property(date.next().to_link(self.vault).to_property("next"));
                page.push_property(date.prev().to_link(self.vault).to_property("prev"));
            }
            if settings.events {
                for event in self.vault.events() {
                    if event.matches(date) {
                        page.push_content(&event.content);
                    }
                }
            }

            Ok(page)
        })
    }
}
