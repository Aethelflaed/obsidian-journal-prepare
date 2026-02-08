use crate::options::{GenericPage, GenericSettings};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum)]
pub enum Option {
    /// Add property day of week
    Day,
    /// Add property link to week
    Week,
    /// Add property link to month
    Month,
    /// Add property links to previous and next day
    Nav,
    /// Add recurring events content, from events/recurring.md
    Events,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Page {
    default: bool,
    settings: Settings,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
// The flags are non-exclusive so we really need a bool
#[allow(clippy::struct_excessive_bools)]
pub struct Settings {
    #[serde(default)]
    pub day_of_week: bool,
    #[serde(default)]
    pub link_to_week: bool,
    #[serde(default)]
    pub link_to_month: bool,
    #[serde(default)]
    pub nav_link: bool,
    #[serde(default)]
    pub events: bool,
}

impl GenericSettings for Settings {
    type Option = Option;

    fn to_options(&self) -> Vec<Option> {
        let mut options = vec![];
        if self.day_of_week {
            options.push(Option::Day);
        }
        if self.link_to_week {
            options.push(Option::Week);
        }
        if self.link_to_month {
            options.push(Option::Month);
        }
        if self.nav_link {
            options.push(Option::Nav);
        }
        if self.events {
            options.push(Option::Events);
        }
        options
    }
}

impl<'a> FromIterator<&'a Option> for Settings {
    fn from_iter<T>(options: T) -> Self
    where
        T: IntoIterator<Item = &'a Option>,
    {
        let mut settings = Self::default();
        for option in options {
            match option {
                Option::Day => settings.day_of_week = true,
                Option::Week => settings.link_to_week = true,
                Option::Month => settings.link_to_month = true,
                Option::Nav => settings.nav_link = true,
                Option::Events => settings.events = true,
            }
        }
        settings
    }
}

impl From<&clap::ArgMatches> for Page {
    fn from(matches: &clap::ArgMatches) -> Self {
        if matches.get_flag(Self::disabling_flag()) {
            Self::disabled()
        } else {
            matches
                .get_many::<Option>(Self::flag())
                .map(|options| Self {
                    default: false,
                    settings: options.collect(),
                })
                .unwrap_or_default()
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self {
            default: true,
            settings: Settings {
                day_of_week: true,
                link_to_week: true,
                link_to_month: true,
                nav_link: true,
                events: true,
            },
        }
    }
}

impl GenericPage for Page {
    type Settings = Settings;

    fn disabled() -> Self {
        Self {
            default: false,
            settings: Settings::default(),
        }
    }

    fn help() -> &'static str {
        "Configure day pages"
    }
    fn disabling_help() -> &'static str {
        "Do not update day pages"
    }

    fn flag() -> &'static str {
        "day"
    }
    fn disabling_flag() -> &'static str {
        "no-day-page"
    }

    fn settings(&self) -> &Settings {
        &self.settings
    }

    fn update(&mut self, settings: &Settings) {
        self.default = false;
        self.settings = settings.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::tests::{parsed_cmd_err, parsed_cmd_ok};
    use crate::options::{Options, PageOptions};

    #[test]
    fn flag_day_day() {
        let Options {
            page_options: PageOptions { day: page, .. },
            ..
        } = parsed_cmd_ok!(["--day", "day"]);

        assert!(!page.default);
        assert!(page.settings().day_of_week);
        assert!(!page.settings().link_to_week);
        assert!(!page.settings().link_to_month);
        assert!(!page.settings().nav_link);
        assert!(!page.settings().events);
    }

    #[test]
    fn flag_day_nav() {
        let Options {
            page_options: PageOptions { day: page, .. },
            ..
        } = parsed_cmd_ok!(["--day", "nav"]);

        assert!(!page.default);
        assert!(!page.settings().day_of_week);
        assert!(!page.settings().link_to_week);
        assert!(!page.settings().link_to_month);
        assert!(page.settings().nav_link);
        assert!(!page.settings().events);
    }

    #[test]
    fn flag_day_month() {
        let Options {
            page_options: PageOptions { day: page, .. },
            ..
        } = parsed_cmd_ok!(["--day", "month"]);

        assert!(!page.default);
        assert!(!page.settings().day_of_week);
        assert!(!page.settings().link_to_week);
        assert!(page.settings().link_to_month);
        assert!(!page.settings().nav_link);
        assert!(!page.settings().events);
    }

    #[test]
    fn flag_day_week() {
        let Options {
            page_options: PageOptions { day: page, .. },
            ..
        } = parsed_cmd_ok!(["--day", "week"]);

        assert!(!page.default);
        assert!(!page.settings().day_of_week);
        assert!(page.settings().link_to_week);
        assert!(!page.settings().link_to_month);
        assert!(!page.settings().nav_link);
        assert!(!page.settings().events);
    }

    #[test]
    fn flag_day_events() {
        let Options {
            page_options: PageOptions { day: page, .. },
            ..
        } = parsed_cmd_ok!(["--day", "events"]);

        assert!(!page.default);
        assert!(!page.settings().day_of_week);
        assert!(!page.settings().link_to_week);
        assert!(!page.settings().link_to_month);
        assert!(!page.settings().nav_link);
        assert!(page.settings().events);
    }

    #[test]
    fn all_flag_day() {
        let Options {
            page_options: PageOptions { day: page, .. },
            ..
        } = parsed_cmd_ok!([
            "--day", "nav", "--day", "month", "--day", "week", "--day", "day", "--day", "events",
        ]);

        assert!(!page.default);
        assert!(!page.is_default());
        assert!(page.settings().day_of_week);
        assert!(page.settings().link_to_week);
        assert!(page.settings().link_to_month);
        assert!(page.settings().nav_link);
        assert!(page.settings().events);
    }

    #[test]
    fn all_flag_day_csv() {
        let Options {
            page_options: PageOptions { day: page, .. },
            ..
        } = parsed_cmd_ok!(["--day", "day,events,nav,month,week"]);

        assert!(!page.default);
        assert!(!page.is_default());
        assert!(page.settings().day_of_week);
        assert!(page.settings().link_to_week);
        assert!(page.settings().link_to_month);
        assert!(page.settings().nav_link);
        assert!(page.settings().events);
    }

    #[test]
    fn flag_absence_produces_default_page() {
        let Options {
            page_options: PageOptions { day: page, .. },
            ..
        } = parsed_cmd_ok!(Vec::<&str>::new());
        assert!(page.is_default());
    }

    #[test]
    fn flag_requires_argument() {
        parsed_cmd_ok!(["--day", "nav"]);
        parsed_cmd_err!(["--day"]);
    }

    #[test]
    fn disabling_flag_produces_disabled_page() {
        let Options {
            page_options: PageOptions { day: page, .. },
            ..
        } = parsed_cmd_ok!(["--no-day-page"]);
        assert!(!page.is_default());
        assert!(page.settings().is_empty());
    }

    #[test]
    fn both_flags_are_exclusive() {
        parsed_cmd_ok!(["--day", "nav"]);
        parsed_cmd_ok!(["--no-day-page"]);
        parsed_cmd_err!(["--no-day-page", "--day", "nav"]);
    }
}
