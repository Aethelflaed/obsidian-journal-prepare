use anyhow::Result;
use chrono::{Datelike, Days, IsoWeek, Months, NaiveDate, Utc, Weekday};

mod options;

mod page;

mod date_utils;
use date_utils::{DateRange, Month, Navigation, Year};

mod metadata;
use metadata::ToMetadata;

mod utils;
use utils::{ToEmbedded, ToLink};

mod vault;
use vault::Vault;

fn main() -> Result<()> {
    use clap::error::ErrorKind::*;
    use clap::Parser;

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
        },
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
    pub vault: Vault,
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

        if to < from {
            anyhow::bail!("--from {} should be less than --to {}", from, to);
        }

        let vault = Vault::new(path)?;

        Ok(Preparer {
            from,
            to,
            vault,
            day_options: day.into(),
            week_options: week.into(),
            month_options: month.into(),
            year_options: year.into(),
        })
    }
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

impl Preparer {
    fn run(&self) -> Result<()> {
        log::debug!("Preparing journal {:?}", self.vault);
        log::debug!("from {} to {}", self.from, self.to);
        log::debug!("{}", self.day_options);
        log::debug!("{}", self.week_options);
        log::debug!("{}", self.month_options);
        log::debug!("{}", self.year_options);

        let mut date: NaiveDate = self.from;
        let mut year = Year::from(date.year());
        let mut month = Month::from(date);
        let mut week = date.iso_week();

        self.print_date(date)?;
        self.print_week(week)?;
        self.print_month(month)?;
        self.print_year(year)?;

        while date < self.to {
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
        }
        Ok(())
    }

    fn print_year(&self, year: Year) -> Result<()> {
        if self.year_options.none() {
            return Ok(());
        }

        self.vault.update(year, |mut page| {
            if self.year_options.nav {
                page.push_metadata(year.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(year.prev().to_link(&self.vault).to_metadata("prev"));
            }

            for month in year.iter() {
                page.push_content(month.to_link(&self.vault));
            }

            Ok(page)
        })
    }

    fn print_month(&self, month: Month) -> Result<()> {
        if self.month_options.none() {
            return Ok(());
        }

        self.vault.update(month, |mut page| {
            if self.month_options.nav {
                page.push_metadata(month.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(month.prev().to_link(&self.vault).to_metadata("prev"));
            }

            for (index, date) in month.iter().enumerate() {
                if index == 0 || date.weekday() == Weekday::Mon {
                    page.push_content(format!("#### {}", date.iso_week().to_link(&self.vault)));
                }
                page.push_content(format!(
                    "- {} {}",
                    weekday(date),
                    date.to_link(&self.vault).into_embedded()
                ));
            }

            Ok(page)
        })
    }

    fn print_week(&self, week: IsoWeek) -> Result<()> {
        if self.week_options.none() {
            return Ok(());
        }

        self.vault.update(week, |mut page| {
            if self.week_options.month {
                page.push_metadata(Month::from(week).to_link(&self.vault).to_metadata("month"));
            }
            if self.week_options.nav {
                page.push_metadata(week.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(week.prev().to_link(&self.vault).to_metadata("prev"));
            }

            for date in week.iter() {
                page.push_content(format!(
                    "- {} {}",
                    weekday(date),
                    date.to_link(&self.vault).into_embedded()
                ));
            }

            Ok(page)
        })
    }

    fn print_date(&self, date: NaiveDate) -> Result<()> {
        if self.day_options.none() {
            return Ok(());
        }

        self.vault.update(date, |mut page| {
            if self.day_options.day {
                page.push_metadata(weekday(date).to_metadata("day"));
            }
            if self.day_options.week {
                page.push_metadata(date.iso_week().to_link(&self.vault).to_metadata("week"));
            }
            if self.day_options.month {
                page.push_metadata(Month::from(date).to_link(&self.vault).to_metadata("month"));
            }
            if self.week_options.nav {
                page.push_metadata(date.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(date.prev().to_link(&self.vault).to_metadata("prev"));
            }

            Ok(page)
        })
    }
}
