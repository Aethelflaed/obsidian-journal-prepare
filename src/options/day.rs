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

#[derive(Debug, PartialEq)]
pub struct Page {
    default: bool,
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
        if matches.get_flag(Self::disabling_flag()) {
            Page::disabled()
        } else {
            matches
                .get_many::<Option>(Self::flag())
                .map(|options| Page {
                    default: false,
                    settings: Settings::from_iter(options),
                })
                .unwrap_or_default()
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page {
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
        Page {
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
    use clap::{ArgMatches, Command};
    use std::ffi::OsString;

    fn cmd<I, T>(args_iter: I) -> Result<ArgMatches, clap::error::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Command::new("test")
            .no_binary_name(true)
            .arg(Page::arg())
            .arg(Page::disabling_arg())
            .try_get_matches_from(args_iter)
    }

    #[test]
    fn flag_day_day() -> anyhow::Result<()> {
        let matches = cmd(["--day", "day"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(page.settings().day_of_week);
        assert!(!page.settings().link_to_week);
        assert!(!page.settings().link_to_month);
        assert!(!page.settings().nav_link);
        assert!(!page.settings().events);

        Ok(())
    }

    #[test]
    fn flag_day_nav() -> anyhow::Result<()> {
        let matches = cmd(["--day", "nav"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.settings().day_of_week);
        assert!(!page.settings().link_to_week);
        assert!(!page.settings().link_to_month);
        assert!(page.settings().nav_link);
        assert!(!page.settings().events);

        Ok(())
    }

    #[test]
    fn flag_day_month() -> anyhow::Result<()> {
        let matches = cmd(["--day", "month"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.settings().day_of_week);
        assert!(!page.settings().link_to_week);
        assert!(page.settings().link_to_month);
        assert!(!page.settings().nav_link);
        assert!(!page.settings().events);

        Ok(())
    }

    #[test]
    fn flag_day_week() -> anyhow::Result<()> {
        let matches = cmd(["--day", "week"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.settings().day_of_week);
        assert!(page.settings().link_to_week);
        assert!(!page.settings().link_to_month);
        assert!(!page.settings().nav_link);
        assert!(!page.settings().events);

        Ok(())
    }

    #[test]
    fn flag_day_events() -> anyhow::Result<()> {
        let matches = cmd(["--day", "events"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.settings().day_of_week);
        assert!(!page.settings().link_to_week);
        assert!(!page.settings().link_to_month);
        assert!(!page.settings().nav_link);
        assert!(page.settings().events);

        Ok(())
    }

    #[test]
    fn all_flag_day() -> anyhow::Result<()> {
        let matches = cmd([
            "--day", "nav", "--day", "month", "--day", "week", "--day", "day", "--day", "events",
        ])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.is_default());
        assert!(page.settings().day_of_week);
        assert!(page.settings().link_to_week);
        assert!(page.settings().link_to_month);
        assert!(page.settings().nav_link);
        assert!(page.settings().events);

        Ok(())
    }

    #[test]
    fn all_flag_day_csv() -> anyhow::Result<()> {
        let matches = cmd(["--day", "day,events,nav,month,week"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.is_default());
        assert!(page.settings().day_of_week);
        assert!(page.settings().link_to_week);
        assert!(page.settings().link_to_month);
        assert!(page.settings().nav_link);
        assert!(page.settings().events);

        Ok(())
    }

    #[test]
    fn flag_absence_produces_default_page() -> anyhow::Result<()> {
        let matches = cmd(Vec::<&str>::new())?;
        let page = Page::from(&matches);
        assert!(page.is_default());

        Ok(())
    }

    #[test]
    fn flag_requires_argument() {
        assert!(cmd(["--day", "nav"]).is_ok());
        assert!(cmd(["--day"]).is_err());
    }

    #[test]
    fn disabling_flag_produces_disabled_page() -> anyhow::Result<()> {
        let matches = cmd(["--no-day-page"])?;
        let page = Page::from(&matches);
        assert!(!page.is_default());
        assert!(page.settings().is_empty());

        Ok(())
    }

    #[test]
    fn both_flags_are_exclusive() {
        assert!(cmd(["--day", "nav"]).is_ok());
        assert!(cmd(["--no-day-page"]).is_ok());
        assert!(cmd(["--no-day-page", "--day", "nav"]).is_err());
    }
}
