use anyhow::Result;
use chrono::{Datelike, Days, IsoWeek, NaiveDate, Weekday};

mod options;
use options::{GenericPage, GenericSettings};

mod page;

mod date_utils;
use date_utils::{Month, Navigation, ToDateIterator, Year};

mod metadata;
use metadata::ToMetadata;

mod utils;
use utils::{ToEmbedded, ToLink};

mod vault;
use vault::Vault;

mod events;

fn parse() -> options::Options {
    match options::parse(std::env::args_os()) {
        Ok(options) => options,
        Err(err) => err.exit(),
    }
}

fn main() -> Result<()> {
    let options::Options {
        from,
        to,
        path,
        log_level_filter,
        mut page_options,
    } = parse();

    setup_log(log_level_filter)?;

    let vault = Vault::new(path)?;

    if let Some(settings) = vault.config().settings() {
        page_options.update(settings);
    }

    Preparer {
        from,
        to,
        vault,
        page_options,
    }
    .run()?;

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
    pub page_options: options::PageOptions,
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
                page.push_metadata(year.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(year.prev().to_link(&self.vault).to_metadata("prev"));
            }
            if settings.month {
                for month in year.iter() {
                    page.push_content(month.to_link(&self.vault));
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
                page.push_metadata(month.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(month.prev().to_link(&self.vault).to_metadata("prev"));
            }
            if settings.month {
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
                page.push_metadata(Month::from(week).to_link(&self.vault).to_metadata("month"));
            }
            if settings.nav_link {
                page.push_metadata(week.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(week.prev().to_link(&self.vault).to_metadata("prev"));
            }
            if settings.week {
                for date in week.iter() {
                    page.push_content(format!(
                        "- {} {}",
                        weekday(date),
                        date.to_link(&self.vault).into_embedded()
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
                page.push_metadata(weekday(date).to_metadata("day"));
            }
            if settings.link_to_week {
                page.push_metadata(date.iso_week().to_link(&self.vault).to_metadata("week"));
            }
            if settings.link_to_month {
                page.push_metadata(Month::from(date).to_link(&self.vault).to_metadata("month"));
            }
            if settings.nav_link {
                page.push_metadata(date.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(date.prev().to_link(&self.vault).to_metadata("prev"));
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
