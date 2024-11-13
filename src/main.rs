use anyhow::Result;
use chrono::{Datelike, Days, IsoWeek, Months, NaiveDate, Utc, Weekday};
use std::path::PathBuf;

mod options;

mod page;
use page::Page;

mod date_utils;
use date_utils::{DateRange, Month, Navigation, Year};

mod metadata;
use metadata::{Filters, ToMetadata};

mod utils;
use utils::{JournalName, ToEmbedded, ToLink};

fn main() -> Result<()> {
    use clap::Parser;
    use clap::error::ErrorKind::*;

    let cli = match options::Cli::try_parse_from(std::env::args_os()) {
        Ok(cli) => cli,
        Err(e) => match e.kind() {
            DisplayHelp | DisplayVersion => {
                println!("{}", e);
                return Ok(());
            }
            _ => {
                return Err(e.into());
            }
        }
    };

    setup_log(cli.verbose.log_level_filter())?;

    Preparer::try_from(cli)?.run()?;

    Ok(())
}

fn setup_log(level: log::LevelFilter) -> Result<()> {
    use env_logger::{Builder, Env};
    use systemd_journal_logger::{connected_to_journal, JournalLog};

    // If the output streams of this process are directly connected to the
    // systemd journal log directly to the journal to preserve structured
    // log entries (e.g. proper multiline messages, metadata fields, etc.)
    if connected_to_journal() {
        JournalLog::new()
            .unwrap()
            .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
            .install()?;
    } else {
        let name = String::from(env!("CARGO_PKG_NAME"))
            .replace('-', "_")
            .to_uppercase();
        let env = Env::new()
            .filter(format!("{}_LOG", name))
            .write_style(format!("{}_LOG_STYLE", name));

        Builder::new()
            .filter_level(log::LevelFilter::Trace)
            .parse_env(env)
            .try_init()?;
    }

    log::set_max_level(level);

    Ok(())
}

struct Preparer {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub path: PathBuf,
    pub day_options: options::DayOptions,
    pub week_options: options::WeekOptions,
    pub month_options: options::MonthOptions,
    pub year_options: options::YearOptions,
}

impl TryFrom<options::Cli> for Preparer {
    type Error = anyhow::Error;

    fn try_from(
        options::Cli {
            to,
            from,
            path,
            day,
            week,
            month,
            year,
            ..
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
            week_options: week.into(),
            month_options: month.into(),
            year_options: year.into(),
        })
    }
}

impl Preparer {
    fn run(&self) -> Result<()> {
        log::debug!("Preparing journal {:?}", self.path.display());
        log::debug!("from {} to {}", self.from, self.to);
        log::debug!("{}", self.day_options);
        log::debug!("{}", self.week_options);
        log::debug!("{}", self.month_options);

        let mut date: NaiveDate = self.from;
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
        let mut page = Page::new(&path);

        if self.year_options.nav {
            page.push_metadata(year.next().to_link().to_metadata("next"));
            page.push_metadata(year.prev().to_link().to_metadata("prev"));
        }

        let first = year.first();
        let last = year.last();

        let mut month = first;
        loop {
            page.push_content(month.to_link());

            month = month + Months::new(1);
            if month > last {
                break;
            }
        }

        if path.exists() {
            page = Page::try_from(path.as_path())? + page;
        }

        page.write()?;

        log::info!("year: {}", path.display());
        Ok(())
    }

    fn print_month(&self, month: Month) -> Result<()> {
        let path = self.page_path(month.to_journal_path_name());
        let mut page = Page::new(&path);

        let first = month.first();
        let last = month.last();

        page.push_metadata(Filters::default().push("month", false));

        if self.month_options.nav {
            page.push_metadata(month.next().to_link().to_metadata("next"));
            page.push_metadata(month.prev().to_link().to_metadata("prev"));
        }

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

        log::info!("month: {}", path.display());
        Ok(())
    }

    fn print_week(&self, week: IsoWeek) -> Result<()> {
        let path = self.page_path(week.to_journal_path_name());
        let mut page = Page::new(&path);

        let first = week.first();
        let last = week.last();

        page.push_metadata(Filters::default().push("week", false).push("month", false));

        if self.week_options.month {
            page.push_metadata(Month::from(first).to_link().to_metadata("month"));
        }
        if self.week_options.nav {
            page.push_metadata(week.next().to_link().to_metadata("next"));
            page.push_metadata(week.prev().to_link().to_metadata("prev"));
        }

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

        log::info!("week: {}", path.display());
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

        log::info!("day: {}", path.display());
        Ok(())
    }

    fn page_path(&self, name: String) -> PathBuf {
        self.path.join("pages").join(name)
    }

    fn journal_path(&self, name: String) -> PathBuf {
        self.path.join("journals").join(name)
    }
}
