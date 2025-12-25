use crate::options::{GenericPage, GenericSettings};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum)]
pub enum Option {
    /// Add embedded month days
    Month,
    /// Add property links to previous and next month
    Nav,
}

#[derive(Debug)]
pub struct Page {
    enabled: bool,
    settings: Settings,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub month: bool,
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
        let mut settings = Settings::default();
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
    fn from(matches: &clap::ArgMatches) -> Page {
        if matches.get_flag("no_month_page") {
            Page::disabled()
        } else {
            matches
                .get_many::<Option>("month_options")
                .map(|options| Page {
                    enabled: true,
                    settings: Settings::from_iter(options),
                })
                .unwrap_or_default()
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page {
            enabled: true,
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
        Page {
            enabled: false,
            settings: Settings::default(),
        }
    }

    fn help() -> &'static str {
        "Configure month pages"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn settings(&self) -> &Settings {
        &self.settings
    }

    fn update(&mut self, settings: &Settings) {
        self.settings = settings.clone();
    }
}
