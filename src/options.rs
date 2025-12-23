use anyhow::Result;
use chrono::NaiveDate;
use std::path::PathBuf;

pub mod day;
pub mod month;
pub mod week;
pub mod year;
use day::DayOption;
use month::MonthOption;
use week::WeekOption;
use year::YearOption;

pub struct Options {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub path: PathBuf,
    pub log_level_filter: log::LevelFilter,
    pub page_options: PageOptions,
}

pub struct PageOptions {
    pub day: day::Page,
    pub week: week::Page,
    pub month: month::Page,
    pub year: year::Page,
}

impl PageOptions {
    pub fn update(&mut self, settings: &crate::vault::config::Settings) {
        if self.day.is_enabled() && self.day.is_empty() {
            if let Some(ref day_settings) = settings.day {
                self.day.update(day_settings);
            }
        }

        if self.week.is_enabled() && self.week.is_empty() {
            if let Some(ref week_settings) = settings.week {
                self.week.update(week_settings);
            }
        }

        if self.month.is_enabled() && self.month.is_empty() {
            if let Some(ref month_settings) = settings.month {
                self.month.update(month_settings);
            }
        }

        if self.year.is_enabled() && self.year.is_empty() {
            if let Some(ref year_settings) = settings.year {
                self.year.update(year_settings);
            }
        }
    }
}

pub fn parse() -> Result<Options> {
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
                .help(day::Page::help())
                .long_help(day::Page::default().long_help()),
        )
        .arg(
            arg!(no_day_page: --"no-day-page" "Do not update day pages")
                .conflicts_with("day_options"),
        )
        .arg(
            arg!(week_options: -w --week <WEEK_OPTION> ...)
                .value_parser(value_parser!(WeekOption))
                .value_delimiter(',')
                .help(week::Page::help())
                .long_help(week::Page::default().long_help()),
        )
        .arg(
            arg!(no_week_page: --"no-week-page" "Do not update week pages")
                .conflicts_with("week_options"),
        )
        .arg(
            arg!(month_options: -m --month <MONTH_OPTION> ...)
                .value_parser(value_parser!(MonthOption))
                .value_delimiter(',')
                .help(month::Page::help())
                .long_help(month::Page::default().long_help()),
        )
        .arg(
            arg!(no_month_page: --"no-month-page" "Do not update month pages")
                .conflicts_with("month_options"),
        )
        .arg(
            arg!(year_options: -y --year <YEAR_OPTION> ...)
                .value_parser(value_parser!(YearOption))
                .value_delimiter(',')
                .help(year::Page::help())
                .long_help(year::Page::default().long_help()),
        )
        .arg(
            arg!(no_year_page: --"no-year-page" "Do not update year pages")
                .conflicts_with("year_options"),
        );

    let matches = command.get_matches_mut();

    let from = matches
        .get_one::<NaiveDate>("from")
        .cloned()
        .unwrap_or(chrono::Utc::now().date_naive());
    let to = matches
        .get_one::<NaiveDate>("to")
        .cloned()
        .unwrap_or(from + chrono::Months::new(1));

    if to < from {
        command
            .error(
                clap::error::ErrorKind::ArgumentConflict,
                format!("--from {} should be less than --to {}", from, to),
            )
            .exit();
    }

    let day_options = day::Page::from(&matches);
    let week_options = week::Page::from(&matches);
    let month_options = month::Page::from(&matches);
    let year_options = year::Page::from(&matches);

    let path = matches
        .get_one::<std::path::PathBuf>("path")
        .expect("'PATH' is required and parsing will fail if its missing")
        .clone();

    let log_level_filter = clap_verbosity_flag::Verbosity::<clap_verbosity_flag::ErrorLevel>::new(
        matches.get_one::<u8>("verbose").cloned().unwrap_or(0u8),
        matches.get_one::<u8>("quiet").cloned().unwrap_or(0u8),
    )
    .log_level_filter();

    Ok(Options {
        from,
        to,
        path,
        log_level_filter,
        page_options: PageOptions {
            day: day_options,
            week: week_options,
            month: month_options,
            year: year_options,
        },
    })
}
