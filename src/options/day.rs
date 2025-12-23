use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum)]
pub enum DayOption {
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
    day_of_week: bool,
    link_to_week: bool,
    link_to_month: bool,
    nav_link: bool,
    events: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub day_of_week: bool,
    pub link_to_week: bool,
    pub link_to_month: bool,
    pub nav_link: bool,
    pub events: bool,
}

impl From<Page> for Settings {
    fn from(page: Page) -> Self {
        Settings {
            day_of_week: page.day_of_week,
            link_to_week: page.link_to_week,
            link_to_month: page.link_to_month,
            nav_link: page.nav_link,
            events: page.events,
        }
    }
}

impl From<&clap::ArgMatches> for Page {
    fn from(matches: &clap::ArgMatches) -> Page {
        if matches.get_flag("no_day_page") {
            Page::disabled()
        } else {
            matches
                .get_many::<DayOption>("day_options")
                .map(|options| {
                    let mut page = Page {
                        enabled: true,
                        ..Page::disabled()
                    };
                    for option in options {
                        page.add_option(option);
                    }
                    page
                })
                .unwrap_or_default()
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page {
            enabled: true,
            day_of_week: true,
            link_to_week: true,
            link_to_month: true,
            nav_link: true,
            events: true,
        }
    }
}

impl Page {
    pub fn disabled() -> Self {
        Page {
            enabled: false,
            day_of_week: false,
            link_to_week: false,
            link_to_month: false,
            nav_link: false,
            events: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_empty(&self) -> bool {
        !(self.day_of_week
            || self.link_to_week
            || self.link_to_month
            || self.nav_link
            || self.events)
    }

    pub fn day_of_week(&self) -> bool {
        self.day_of_week
    }

    pub fn link_to_week(&self) -> bool {
        self.link_to_week
    }

    pub fn link_to_month(&self) -> bool {
        self.link_to_month
    }

    pub fn nav_link(&self) -> bool {
        self.nav_link
    }

    pub fn events(&self) -> bool {
        self.events
    }

    pub fn to_options(&self) -> Vec<DayOption> {
        let mut options = vec![];
        if self.day_of_week {
            options.push(DayOption::Day);
        }
        if self.link_to_week {
            options.push(DayOption::Week);
        }
        if self.link_to_month {
            options.push(DayOption::Month);
        }
        if self.nav_link {
            options.push(DayOption::Nav);
        }
        if self.events {
            options.push(DayOption::Events);
        }
        options
    }

    pub fn help() -> &'static str {
        "Configure day pages"
    }

    pub fn long_help(&self) -> String {
        let default_values = self
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

        format!("{}\n\n[default: {}]", Self::help(), default_values)
    }

    pub fn update(&mut self, settings: &Settings) {
        self.day_of_week = settings.day_of_week;
        self.link_to_week = settings.link_to_week;
        self.link_to_month = settings.link_to_month;
        self.nav_link = settings.nav_link;
        self.events = settings.events;
    }

    fn add_option(&mut self, option: &DayOption) {
        match option {
            DayOption::Day => self.day_of_week = true,
            DayOption::Week => self.link_to_week = true,
            DayOption::Month => self.link_to_month = true,
            DayOption::Nav => self.nav_link = true,
            DayOption::Events => self.events = true,
        }
    }
}
