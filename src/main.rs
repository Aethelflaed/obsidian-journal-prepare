use anyhow::Result;
use chrono::{Datelike, Days, IsoWeek, Months, NaiveDate, Utc, Weekday};
use clap::Parser;
use std::path::PathBuf;

mod options;

mod page;
use page::Page;

mod journal_name;
use journal_name::JournalName;

mod navigation;
use navigation::Navigation;

mod date_range;
use date_range::DateRange;

mod metadata;
use metadata::{Filters, ToMetadata};

mod utils;
use utils::{ToEmbedded, ToLink};

fn main() -> Result<()> {
    let cli = options::Cli::try_parse_from(std::env::args_os())?;

    Preparer::try_from(cli)?.run()?;

    Ok(())
}

#[derive(Debug, Default, Clone, Copy, PartialEq, derive_more::From)]
struct Year(i32);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Month {
    year: i32,
    month: u32,
}

impl Month {
    pub fn name(&self) -> &str {
        chrono::Month::try_from(self.month as u8).unwrap().name()
    }
}

impl From<NaiveDate> for Month {
    fn from(date: NaiveDate) -> Self {
        Month {
            year: date.year(),
            month: date.month(),
        }
    }
}

struct Preparer {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub path: PathBuf,
    pub day_options: options::DayOptions,
}

impl TryFrom<options::Cli> for Preparer {
    type Error = anyhow::Error;

    fn try_from(
        options::Cli {
            to,
            from,
            path,
            day,
        }: options::Cli,
    ) -> Result<Self> {
        let from = from.unwrap_or(Utc::now().date_naive());
        let to = to.unwrap_or(from + Months::new(1));

        if to <= from {
            anyhow::bail!("--from {} should be less than --to {}", from, to);
        }

        Ok(Preparer {
            from,
            to,
            path,
            day_options: day.into(),
        })
    }
}

impl Preparer {
    fn run(&self) -> Result<()> {
        let mut date = self.from.clone();
        let mut year = Year::from(date.year());
        let mut month = Month::from(date);
        let mut week = date.iso_week();

        self.print_date(date)?;
        self.print_week(week)?;
        self.print_month(month)?;
        self.print_year(year)?;

        loop {
            date = date + Days::new(1);
            self.print_date(date)?;

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

            if date >= self.to {
                break;
            }
        }
        Ok(())
    }

    fn print_year(&self, year: Year) -> Result<()> {
        let path = self.page_path(year.to_journal_path_name());

        println!("{}", path.display());
        Ok(())
    }

    fn print_month(&self, month: Month) -> Result<()> {
        let path = self.page_path(month.to_journal_path_name());
        let mut page = Page::new(&path);

        let first = month.first();
        let last = month.last();

        let next = month.next().to_link().to_metadata("next");
        let prev = month.prev().to_link().to_metadata("prev");

        page.push_metadata(Filters::default().push("month", false));
        page.push_metadata(next);
        page.push_metadata(prev);

        let mut date = first;
        loop {
            page.push_content(date.to_link().into_embedded());

            date = date + Days::new(1);
            if date > last {
                break;
            }
        }

        if path.exists() {
            page = Page::try_from(path.as_path())? + page;
        }

        page.write()?;

        println!("{}", path.display());
        Ok(())
    }

    fn print_week(&self, week: IsoWeek) -> Result<()> {
        let path = self.page_path(week.to_journal_path_name());
        let mut page = Page::new(&path);

        let first = week.first();
        let last = week.last();

        let month = Month::from(first).to_link().to_metadata("month");
        let next = week.next().to_link().to_metadata("next");
        let prev = week.prev().to_link().to_metadata("prev");

        page.push_metadata(Filters::default().push("week", false).push("month", false));
        page.push_metadata(month);
        page.push_metadata(next);
        page.push_metadata(prev);

        let mut date = first;
        loop {
            page.push_content(date.to_link().into_embedded());

            date = date + Days::new(1);
            if date > last {
                break;
            }
        }

        if path.exists() {
            page = Page::try_from(path.as_path())? + page;
        }

        page.write()?;

        println!("{}", path.display());
        Ok(())
    }

    fn print_date(&self, date: NaiveDate) -> Result<()> {
        let path = self.journal_path(date.to_journal_path_name());
        let mut page = Page::new(&path);

        page.push_metadata(
            Filters::default()
                .push(date.iso_week().to_journal_name(), false)
                .push(Month::from(date).to_journal_name(), false),
        );

        if self.day_options.day {
            let day = match date.weekday() {
                Weekday::Mon => "Monday",
                Weekday::Tue => "Tuesday",
                Weekday::Wed => "Wednesday",
                Weekday::Thu => "Thursday",
                Weekday::Fri => "Friday",
                Weekday::Sat => "Saturday",
                Weekday::Sun => "Sunday",
            };

            page.push_metadata(day.to_metadata("day"));
        }

        if self.day_options.week {
            page.push_metadata(date.iso_week().to_link().to_metadata("week"));
        }
        if self.day_options.month {
            page.push_metadata(Month::from(date).to_link().to_metadata("month"));
        }

        if path.exists() {
            page = Page::try_from(path.as_path())? + page;
        }

        page.write()?;

        println!("{}", path.display());
        Ok(())
    }

    fn page_path(&self, name: String) -> PathBuf {
        self.path.join("pages").join(name)
    }

    fn journal_path(&self, name: String) -> PathBuf {
        self.path.join("journals").join(name)
    }
}
