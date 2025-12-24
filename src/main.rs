use anyhow::Result;
use chrono::{Datelike, Days, IsoWeek, Months, NaiveDate, Utc, Weekday};

mod options;
use options::{day::DayOption, week::WeekOption, month::MonthOption, year::YearOption};

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

fn main() -> Result<()> {
    use clap::{arg, command, value_parser};

    let mut command = command!()
        .arg(arg!(verbose: -v --verbose ... "Increase logging verbosity"))
        .arg(arg!(quiet: -q --quiet ... "Decrease logging verbosity").conflicts_with("verbose"))
        .arg(
            arg!(path: -p --path <PATH> "Path to notes")
                .required(true)
                .value_parser(value_parser!(std::path::PathBuf)),
        )
        .arg(
            arg!(from: --from <DATE> "Only prepare journal starting from given date")
                .required(false)
                .value_parser(value_parser!(NaiveDate)),
        )
        .arg(
            arg!(to: --to <DATE> "Only prepare journal up to given date")
                .required(false)
                .value_parser(value_parser!(NaiveDate)),
        )
        .arg(
            arg!(day_options: -d --day <DAY_OPTION> ...)
                .value_parser(value_parser!(DayOption))
                .value_delimiter(',')
                .help(options::day::Page::help())
                .long_help(options::day::Page::default().long_help()),
        )
        .arg(
            arg!(no_day_page: --"no-day-page" "Do not update day pages")
                .conflicts_with("day_options"),
        )
        .arg(
            arg!(week_options: -w --week <WEEK_OPTION> ...)
                .value_parser(value_parser!(WeekOption))
                .value_delimiter(',')
                .help(options::week::Page::help())
                .long_help(options::week::Page::default().long_help()),
        )
        .arg(
            arg!(no_week_page: --"no-week-page" "Do not update week pages")
                .conflicts_with("week_options"),
        )
        .arg(
            arg!(month_options: -m --month <MONTH_OPTION> ...)
                .value_parser(value_parser!(MonthOption))
                .value_delimiter(',')
                .help(options::month::Page::help())
                .long_help(options::month::Page::default().long_help()),
        )
        .arg(
            arg!(no_month_page: --"no-month-page" "Do not update month pages")
                .conflicts_with("month_options"),
        )
        .arg(
            arg!(year_options: -y --year <YEAR_OPTION> ...)
                .value_parser(value_parser!(YearOption))
                .value_delimiter(',')
                .help(options::year::Page::help())
                .long_help(options::year::Page::default().long_help()),
        )
        .arg(
            arg!(no_year_page: --"no-year-page" "Do not update year pages")
                .conflicts_with("year_options"),
        );

    let matches = command.get_matches_mut();

    let from = matches
        .get_one::<NaiveDate>("from")
        .cloned()
        .unwrap_or(Utc::now().date_naive());
    let to = matches
        .get_one::<NaiveDate>("to")
        .cloned()
        .unwrap_or(from + Months::new(1));

    if to < from {
        command
            .error(
                clap::error::ErrorKind::ArgumentConflict,
                format!("--from {} should be less than --to {}", from, to),
            )
            .exit();
    }

    let day_options = options::day::Page::from(&matches);
    let week_options = options::week::Page::from(&matches);
    let month_options = options::month::Page::from(&matches);
    let year_options = options::year::Page::from(&matches);

    setup_log(
        clap_verbosity_flag::Verbosity::<clap_verbosity_flag::ErrorLevel>::new(
            matches.get_one::<u8>("verbose").cloned().unwrap_or(0u8),
            matches.get_one::<u8>("quiet").cloned().unwrap_or(0u8),
        )
        .log_level_filter(),
    )?;

    let path = matches
        .get_one::<std::path::PathBuf>("path")
        .expect("'PATH' is required and parsing will fail if its missing")
        .clone();
    let vault = Vault::new(path)?;

    Preparer {
        from,
        to,
        vault,
        day_options,
        week_options,
        month_options,
        year_options,
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
    pub day_options: options::day::Page,
    pub week_options: options::week::Page,
    pub month_options: options::month::Page,
    pub year_options: options::year::Page,
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
        log::debug!("Day options {:?}", self.day_options);
        log::debug!("Week options {:?}", self.week_options);
        log::debug!("Month options {:?}", self.month_options);
        log::debug!("Year options {:?}", self.year_options);

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
        if self.year_options.is_empty() {
            return Ok(());
        }

        self.vault.update(year, |mut page| {
            if self.year_options.nav_link() {
                page.push_metadata(year.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(year.prev().to_link(&self.vault).to_metadata("prev"));
            }
            if self.year_options.month() {
                for month in year.iter() {
                    page.push_content(month.to_link(&self.vault));
                }
            }

            Ok(page)
        })
    }

    fn print_month(&self, month: Month) -> Result<()> {
        if self.month_options.is_empty() {
            return Ok(());
        }

        self.vault.update(month, |mut page| {
            if self.month_options.nav_link() {
                page.push_metadata(month.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(month.prev().to_link(&self.vault).to_metadata("prev"));
            }
            if self.month_options.month() {
                for (index, date) in month.iter().enumerate() {
                    if index == 0 || date.weekday() == Weekday::Mon {
                        page.push_content(format!(
                                "#### {}",
                                date.iso_week().to_link(&self.vault)
                        ));
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
        if self.week_options.is_empty() {
            return Ok(());
        }

        self.vault.update(week, |mut page| {
            if self.week_options.link_to_month() {
                page.push_metadata(Month::from(week).to_link(&self.vault).to_metadata("month"));
            }
            if self.week_options.nav_link() {
                page.push_metadata(week.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(week.prev().to_link(&self.vault).to_metadata("prev"));
            }
            if self.week_options.week() {
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
        if self.day_options.is_empty() {
            return Ok(());
        }

        self.vault.update(date, |mut page| {
            if self.day_options.day_of_week() {
                page.push_metadata(weekday(date).to_metadata("day"));
            }
            if self.day_options.link_to_week() {
                page.push_metadata(date.iso_week().to_link(&self.vault).to_metadata("week"));
            }
            if self.day_options.link_to_month() {
                page.push_metadata(Month::from(date).to_link(&self.vault).to_metadata("month"));
            }
            if self.day_options.nav_link() {
                page.push_metadata(date.next().to_link(&self.vault).to_metadata("next"));
                page.push_metadata(date.prev().to_link(&self.vault).to_metadata("prev"));
            }
            if self.day_options.events() {
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
