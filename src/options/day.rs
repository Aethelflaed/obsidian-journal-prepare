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

#[derive(Debug)]
pub struct Page {
    enabled: bool,
    settings: Settings,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub day_of_week: bool,
    pub link_to_week: bool,
    pub link_to_month: bool,
    pub nav_link: bool,
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
        let mut settings = Settings::default();
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
    fn from(matches: &clap::ArgMatches) -> Page {
        if matches.get_flag("no_day_page") {
            Page::disabled()
        } else {
            matches
                .get_many::<Option>("day_options")
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
        Page {
            enabled: false,
            settings: Settings::default(),
        }
    }

    fn help() -> &'static str {
        "Configure day pages"
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
