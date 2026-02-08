use crate::options::{GenericPage, GenericSettings};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum)]
pub enum Option {
    /// Add link to months
    Month,
    /// Add property links to previous and next year
    Nav,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Page {
    default: bool,
    settings: Settings,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub month: bool,
    #[serde(default)]
    pub nav_link: bool,
}

impl GenericSettings for Settings {
    type Option = Option;

    fn to_options(&self) -> Vec<Option> {
        let mut options = vec![];
        if self.month {
            options.push(Option::Month);
        }
        if self.nav_link {
            options.push(Option::Nav);
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
                Option::Month => settings.month = true,
                Option::Nav => settings.nav_link = true,
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
                month: true,
                nav_link: true,
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
        "Configure year pages"
    }
    fn disabling_help() -> &'static str {
        "Do not update year pages"
    }

    fn flag() -> &'static str {
        "year"
    }
    fn disabling_flag() -> &'static str {
        "no-year-page"
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
    fn flag_year_nav() {
        let Options {
            page_options: PageOptions { year: page, .. },
            ..
        } = parsed_cmd_ok!(["--year", "nav"]);

        assert!(!page.default);
        assert!(!page.settings().month);
        assert!(page.settings().nav_link);
    }

    #[test]
    fn flag_year_month() {
        let Options {
            page_options: PageOptions { year: page, .. },
            ..
        } = parsed_cmd_ok!(["--year", "month"]);

        assert!(!page.default);
        assert!(page.settings().month);
        assert!(!page.settings().nav_link);
    }

    #[test]
    fn all_flag_year() {
        let Options {
            page_options: PageOptions { year: page, .. },
            ..
        } = parsed_cmd_ok!(["--year", "nav", "--year", "month"]);

        assert!(!page.default);
        assert!(!page.is_default());
        assert!(page.settings().month);
        assert!(page.settings().nav_link);
    }

    #[test]
    fn all_flag_year_csv() {
        let Options {
            page_options: PageOptions { year: page, .. },
            ..
        } = parsed_cmd_ok!(["--year", "nav,month"]);

        assert!(!page.default);
        assert!(!page.is_default());
        assert!(page.settings().month);
        assert!(page.settings().nav_link);
    }

    #[test]
    fn flag_absence_produces_default_page() {
        let Options {
            page_options: PageOptions { year: page, .. },
            ..
        } = parsed_cmd_ok!(Vec::<&str>::new());
        assert!(page.is_default());
    }

    #[test]
    fn flag_requires_argument() {
        parsed_cmd_ok!(["--year", "nav"]);
        parsed_cmd_err!(["--year"]);
    }

    #[test]
    fn disabling_flag_produces_disabled_page() {
        let Options {
            page_options: PageOptions { year: page, .. },
            ..
        } = parsed_cmd_ok!(["--no-year-page"]);
        assert!(!page.is_default());
        assert!(page.settings().is_empty());
    }

    #[test]
    fn both_flags_are_exclusive() {
        parsed_cmd_ok!(["--year", "nav"]);
        parsed_cmd_ok!(["--no-year-page"]);
        parsed_cmd_err!(["--no-year-page", "--year", "nav"]);
    }
}
