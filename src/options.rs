use anyhow::Result;
use chrono::NaiveDate;
use clap::Arg;
use serde::{Deserialize, Serialize};
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

pub trait GenericPage: Default + PartialEq {
    type Settings: GenericSettings;

    fn is_default(&self) -> bool {
        self == &Self::default()
    }

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

#[derive(Debug, Default)]
pub struct PageOptions {
    pub day: day::Page,
    pub week: week::Page,
    pub month: month::Page,
    pub year: year::Page,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PageSettings {
    #[serde(default)]
    pub day: Option<day::Settings>,
    #[serde(default)]
    pub week: Option<week::Settings>,
    #[serde(default)]
    pub month: Option<month::Settings>,
    #[serde(default)]
    pub year: Option<year::Settings>,
}

impl PageOptions {
    pub fn update(&mut self, settings: &PageSettings) {
        if self.day.is_default() {
            if let Some(day_settings) = settings.day.as_ref() {
                self.day.update(day_settings);
            }
        }

        if self.week.is_default() {
            if let Some(week_settings) = settings.week.as_ref() {
                self.week.update(week_settings);
            }
        }

        if self.month.is_default() {
            if let Some(month_settings) = settings.month.as_ref() {
                self.month.update(month_settings);
            }
        }

        if self.year.is_default() {
            if let Some(year_settings) = settings.year.as_ref() {
                self.year.update(year_settings);
            }
        }
    }
}

impl From<&clap::ArgMatches> for PageOptions {
    fn from(matches: &clap::ArgMatches) -> PageOptions {
        PageOptions {
            day: day::Page::from(matches),
            week: week::Page::from(matches),
            month: month::Page::from(matches),
            year: year::Page::from(matches),
        }
    }
}

pub fn command() -> clap::Command {
    use clap::{arg, command, value_parser};

    let from_help = "Only prepare journal start from given date";
    let from_default = chrono::Utc::now().date_naive();
    let from_long_help = format!("{}\n\n[default: {}]", from_help, from_default);

    let to_help = "Only prepare journal start from given date";
    let to_long_help = format!("{}\n\n[default: 1 month after --from]", to_help);

    command!()
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
        .arg(year::Page::disabling_arg())
}

pub fn parse() -> Result<Options> {
    let mut command = command();
    let matches = command.get_matches_mut();

    let from_default = chrono::Utc::now().date_naive();
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

    let page_options = PageOptions::from(&matches);

    let path = matches
        .get_one::<std::path::PathBuf>("path")
        .expect("'PATH' is required and parsing will fail if its missing")
        .clone();

    use clap_verbosity_flag::{ErrorLevel, Verbosity};
    let log_level_filter = Verbosity::<ErrorLevel>::new(
        matches.get_one::<u8>("verbose").cloned().unwrap_or(0u8),
        matches.get_one::<u8>("quiet").cloned().unwrap_or(0u8),
    )
    .log_level_filter();

    Ok(Options {
        from,
        to,
        path,
        log_level_filter,
        page_options,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::ArgMatches;
    use std::ffi::OsString;

    fn cmd<I, T>(args_iter: I) -> Result<ArgMatches, clap::error::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        command()
            .no_binary_name(true)
            .try_get_matches_from(args_iter)
    }

    #[test]
    fn update_page_options_day_does_not_override_flags() -> anyhow::Result<()> {
        let matches = cmd(["--path", ".", "--day", "day,week"])?;
        let mut page_options = PageOptions::from(&matches);

        let page_settings = PageSettings {
            day: Some(day::Settings::default()),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.day.is_default());
        assert!(page_options.day.settings().day_of_week);
        Ok(())
    }

    #[test]
    fn update_page_options_day_does_not_override_disabling_flag() -> anyhow::Result<()> {
        let matches = cmd(["--path", ".", "--no-day-page"])?;
        let mut page_options = PageOptions::from(&matches);

        let page_settings = PageSettings {
            day: Some(day::Settings {
                day_of_week: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.day.is_default());
        assert!(!page_options.day.settings().day_of_week);
        Ok(())
    }

    #[test]
    fn update_page_options_day_with_empty_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings::default();

        page_options.update(&page_settings);
        assert!(page_options.day.is_default());
        assert!(page_options.day.settings().day_of_week);
    }

    #[test]
    fn update_page_options_day_with_some_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings {
            day: Some(day::Settings::default()),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.day.is_default());
        assert!(!page_options.day.settings().day_of_week);
    }

    #[test]
    fn update_page_options_day_with_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings {
            day: Some(day::Settings {
                day_of_week: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.day.is_default());
        assert!(page_options.day.settings().day_of_week);
    }

    #[test]
    fn update_page_options_week_does_not_override_flags() -> anyhow::Result<()> {
        let matches = cmd(["--path", ".", "--week", "week,month"])?;
        let mut page_options = PageOptions::from(&matches);

        let page_settings = PageSettings {
            week: Some(week::Settings::default()),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.week.is_default());
        assert!(page_options.week.settings().link_to_month);
        Ok(())
    }

    #[test]
    fn update_page_options_week_does_not_override_disabling_flag() -> anyhow::Result<()> {
        let matches = cmd(["--path", ".", "--no-week-page"])?;
        let mut page_options = PageOptions::from(&matches);

        let page_settings = PageSettings {
            week: Some(week::Settings {
                link_to_month: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.week.is_default());
        assert!(!page_options.week.settings().link_to_month);
        Ok(())
    }

    #[test]
    fn update_page_options_week_with_empty_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings::default();

        page_options.update(&page_settings);
        assert!(page_options.week.is_default());
        assert!(page_options.week.settings().link_to_month);
    }

    #[test]
    fn update_page_options_week_with_some_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings {
            week: Some(week::Settings::default()),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.week.is_default());
        assert!(!page_options.week.settings().link_to_month);
    }

    #[test]
    fn update_page_options_week_with_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings {
            week: Some(week::Settings {
                link_to_month: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.week.is_default());
        assert!(page_options.week.settings().link_to_month);
    }

    #[test]
    fn update_page_options_month_does_not_override_flags() -> anyhow::Result<()> {
        let matches = cmd(["--path", ".", "--month", "nav"])?;
        let mut page_options = PageOptions::from(&matches);

        let page_settings = PageSettings {
            month: Some(month::Settings::default()),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.month.is_default());
        assert!(page_options.month.settings().nav_link);
        Ok(())
    }

    #[test]
    fn update_page_options_month_does_not_override_disabling_flag() -> anyhow::Result<()> {
        let matches = cmd(["--path", ".", "--no-month-page"])?;
        let mut page_options = PageOptions::from(&matches);

        let page_settings = PageSettings {
            month: Some(month::Settings {
                nav_link: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.month.is_default());
        assert!(!page_options.month.settings().nav_link);
        Ok(())
    }

    #[test]
    fn update_page_options_month_with_empty_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings::default();

        page_options.update(&page_settings);
        assert!(page_options.month.is_default());
        assert!(page_options.month.settings().nav_link);
    }

    #[test]
    fn update_page_options_month_with_some_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings {
            month: Some(month::Settings::default()),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.month.is_default());
        assert!(!page_options.month.settings().nav_link);
    }

    #[test]
    fn update_page_options_month_with_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings {
            month: Some(month::Settings {
                nav_link: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.month.is_default());
        assert!(page_options.month.settings().nav_link);
    }

    #[test]
    fn update_page_options_year_does_not_override_flags() -> anyhow::Result<()> {
        let matches = cmd(["--path", ".", "--year", "nav"])?;
        let mut page_options = PageOptions::from(&matches);

        let page_settings = PageSettings {
            year: Some(year::Settings::default()),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.year.is_default());
        assert!(page_options.year.settings().nav_link);
        Ok(())
    }

    #[test]
    fn update_page_options_year_does_not_override_disabling_flag() -> anyhow::Result<()> {
        let matches = cmd(["--path", ".", "--no-year-page"])?;
        let mut page_options = PageOptions::from(&matches);

        let page_settings = PageSettings {
            year: Some(year::Settings {
                nav_link: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.year.is_default());
        assert!(!page_options.year.settings().nav_link);
        Ok(())
    }

    #[test]
    fn update_page_options_year_with_empty_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings::default();

        page_options.update(&page_settings);
        assert!(page_options.year.is_default());
        assert!(page_options.year.settings().nav_link);
    }

    #[test]
    fn update_page_options_year_with_some_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings {
            year: Some(year::Settings::default()),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.year.is_default());
        assert!(!page_options.year.settings().nav_link);
    }

    #[test]
    fn update_page_options_year_with_settings() {
        let mut page_options = PageOptions::default();
        let page_settings = PageSettings {
            year: Some(year::Settings {
                nav_link: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        page_options.update(&page_settings);
        assert!(!page_options.year.is_default());
        assert!(page_options.year.settings().nav_link);
    }
}
