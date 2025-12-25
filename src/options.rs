use anyhow::Result;
use chrono::NaiveDate;
use clap::Arg;
use std::path::PathBuf;

pub mod day;
pub mod month;
pub mod week;
pub mod year;

pub trait GenericSettings: Default + PartialEq {
    type Option: clap::ValueEnum + Clone + Send + Sync + 'static;

    fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    fn to_options(&self) -> Vec<Self::Option>;
}

pub trait GenericPage: Default {
    type Settings: GenericSettings;

    fn help() -> &'static str;
    fn disabling_help() -> &'static str;
    fn default_long_help() -> String {
        <Self as Default>::default().long_help()
    }
    fn long_help(&self) -> String {
        use clap::ValueEnum;

        let default_values = self
            .settings()
            .to_options()
            .into_iter()
            .map(|opt| {
                opt.to_possible_value()
                    .expect("option to have possible value")
                    .get_name()
                    .to_owned()
            })
            .collect::<Vec<String>>()
            .join(" ");

        format!(
            "{}\n\nUse --{} instead to disable.\n\n[default: {}]",
            Self::help(),
            Self::disabling_flag(),
            default_values
        )
    }

    fn disabled() -> Self;
    fn is_enabled(&self) -> bool;

    fn settings(&self) -> &Self::Settings;
    fn update(&mut self, settings: &Self::Settings);

    fn flag() -> &'static str;
    fn disabling_flag() -> &'static str;

    fn flag_short() -> Option<char> {
        Self::flag().chars().next()
    }

    fn arg() -> Arg {
        use clap::builder::EnumValueParser;

        Arg::new(Self::flag())
            .short(Self::flag_short())
            .long(Self::flag())
            .help(Self::help())
            .long_help(Self::default_long_help())
            .value_parser(EnumValueParser::<<Self::Settings as GenericSettings>::Option>::new())
            .value_delimiter(',')
            .action(clap::ArgAction::Append)
    }

    fn disabling_arg() -> Arg {
        Arg::new(Self::disabling_flag())
            .long(Self::disabling_flag())
            .help(Self::disabling_help())
            .action(clap::ArgAction::SetTrue)
            .conflicts_with(Self::flag())
    }
}

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
        if self.day.is_enabled() && self.day.settings().is_empty() {
            if let Some(ref day_settings) = settings.day {
                self.day.update(day_settings);
            }
        }

        if self.week.is_enabled() && self.week.settings().is_empty() {
            if let Some(ref week_settings) = settings.week {
                self.week.update(week_settings);
            }
        }

        if self.month.is_enabled() && self.month.settings().is_empty() {
            if let Some(ref month_settings) = settings.month {
                self.month.update(month_settings);
            }
        }

        if self.year.is_enabled() && self.year.settings().is_empty() {
            if let Some(ref year_settings) = settings.year {
                self.year.update(year_settings);
            }
        }
    }
}

pub fn parse() -> Result<Options> {
    use clap::{arg, command, value_parser};

    let from_help = "Only prepare journal start from given date";
    let from_default = chrono::Utc::now().date_naive();
    let from_long_help = format!("{}\n\n[default: {}]", from_help, from_default);

    let to_help = "Only prepare journal start from given date";
    let to_long_help = format!("{}\n\n[default: 1 month after --from]", to_help);

    let mut command = command!()
        .arg(arg!(verbose: -v --verbose ... "Increase logging verbosity"))
        .arg(arg!(quiet: -q --quiet ... "Decrease logging verbosity").conflicts_with("verbose"))
        .arg(
            arg!(path: -p --path <PATH> "Path to notes")
                .required(true)
                .value_parser(value_parser!(std::path::PathBuf)),
        )
        .arg(
            arg!(from: --from <DATE>)
                .help(from_help)
                .long_help(from_long_help)
                .required(false)
                .value_parser(value_parser!(NaiveDate)),
        )
        .arg(
            arg!(to: --to <DATE> "Only prepare journal up to given date")
                .help(to_help)
                .long_help(to_long_help)
                .required(false)
                .value_parser(value_parser!(NaiveDate)),
        )
        .arg(day::Page::arg())
        .arg(day::Page::disabling_arg())
        .arg(week::Page::arg())
        .arg(week::Page::disabling_arg())
        .arg(month::Page::arg())
        .arg(month::Page::disabling_arg())
        .arg(year::Page::arg())
        .arg(year::Page::disabling_arg());

    let matches = command.get_matches_mut();

    let from = matches
        .get_one::<NaiveDate>("from")
        .cloned()
        .unwrap_or(from_default);
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
