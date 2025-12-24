use clap::ValueEnum;

#[derive(Clone, Debug, ValueEnum)]
pub enum MonthOption {
    /// Add embedded month days
    Month,
    /// Add property links to previous and next month
    Nav,
}

#[derive(Debug)]
pub struct Page {
    enabled: bool,
    month: bool,
    nav_link: bool,
}

impl From<&clap::ArgMatches> for Page {
    fn from(matches: &clap::ArgMatches) -> Page {
        if matches.get_flag("no_month_page") {
            Page::disabled()
        } else {
            matches
                .get_many::<MonthOption>("month_options")
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
                .unwrap_or_else(Page::default)
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page {
            enabled: true,
            month: true,
            nav_link: true,
        }
    }
}

impl Page {
    pub fn disabled() -> Self {
        Page {
            enabled: false,
            month: false,
            nav_link: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_empty(&self) -> bool {
        !(self.month || self.nav_link)
    }

    pub fn month(&self) -> bool {
        self.month
    }

    pub fn nav_link(&self) -> bool {
        self.nav_link
    }

    pub fn to_options(&self) -> Vec<MonthOption> {
        let mut options = vec![];
        if self.month {
            options.push(MonthOption::Month);
        }
        if self.nav_link {
            options.push(MonthOption::Nav);
        }
        options
    }

    pub fn help() -> &'static str {
        "Configure month pages"
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

    fn add_option(&mut self, option: &MonthOption) {
        match option {
            MonthOption::Month => self.month = true,
            MonthOption::Nav => self.nav_link = true,
        }
    }
}
