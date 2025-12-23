use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum)]
pub enum WeekOption {
    /// Add embedded week days
    Week,
    /// Add property link to month
    Month,
    /// Add property links to previous and next week
    Nav,
}

#[derive(Debug)]
pub struct Page {
    enabled: bool,
    week: bool,
    link_to_month: bool,
    nav_link: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub week: bool,
    pub link_to_month: bool,
    pub nav_link: bool,
}

impl From<Page> for Settings {
    fn from(page: Page) -> Self {
        Settings {
            week: page.week,
            link_to_month: page.link_to_month,
            nav_link: page.nav_link,
        }
    }
}

impl From<&clap::ArgMatches> for Page {
    fn from(matches: &clap::ArgMatches) -> Page {
        if matches.get_flag("no_week_page") {
            Page::disabled()
        } else {
            matches
                .get_many::<WeekOption>("week_options")
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
            week: true,
            link_to_month: true,
            nav_link: true,
        }
    }
}

impl Page {
    pub fn disabled() -> Self {
        Page {
            enabled: false,
            week: false,
            link_to_month: false,
            nav_link: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_empty(&self) -> bool {
        !(self.week || self.link_to_month || self.nav_link)
    }

    pub fn week(&self) -> bool {
        self.week
    }

    pub fn link_to_month(&self) -> bool {
        self.link_to_month
    }

    pub fn nav_link(&self) -> bool {
        self.nav_link
    }

    pub fn to_options(&self) -> Vec<WeekOption> {
        let mut options = vec![];
        if self.week {
            options.push(WeekOption::Week);
        }
        if self.link_to_month {
            options.push(WeekOption::Month);
        }
        if self.nav_link {
            options.push(WeekOption::Nav);
        }
        options
    }

    pub fn help() -> &'static str {
        "Configure week pages"
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
        self.week = settings.week;
        self.link_to_month = settings.link_to_month;
        self.nav_link = settings.nav_link;
    }

    fn add_option(&mut self, option: &WeekOption) {
        match option {
            WeekOption::Week => self.week = true,
            WeekOption::Month => self.link_to_month = true,
            WeekOption::Nav => self.nav_link = true,
        }
    }
}
