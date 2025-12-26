use crate::options::{GenericPage, GenericSettings};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum)]
pub enum Option {
    /// Add embedded week days
    Week,
    /// Add property link to month
    Month,
    /// Add property links to previous and next week
    Nav,
}

#[derive(Debug, PartialEq)]
pub struct Page {
    default: bool,
    settings: Settings,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub week: bool,
    pub link_to_month: bool,
    pub nav_link: bool,
}

impl GenericSettings for Settings {
    type Option = Option;

    fn to_options(&self) -> Vec<Option> {
        let mut options = vec![];
        if self.week {
            options.push(Option::Week);
        }
        if self.link_to_month {
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
                Option::Week => settings.week = true,
                Option::Month => settings.link_to_month = true,
                Option::Nav => settings.nav_link = true,
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
                week: true,
                link_to_month: true,
                nav_link: true,
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
        "Configure week pages"
    }
    fn disabling_help() -> &'static str {
        "Do not update week pages"
    }

    fn flag() -> &'static str {
        "week"
    }
    fn disabling_flag() -> &'static str {
        "no-week-page"
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
    fn flag_week_nav() -> anyhow::Result<()> {
        let matches = cmd(["--week", "nav"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.settings().week);
        assert!(!page.settings().link_to_month);
        assert!(page.settings().nav_link);

        Ok(())
    }

    #[test]
    fn flag_week_month() -> anyhow::Result<()> {
        let matches = cmd(["--week", "month"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.settings().week);
        assert!(page.settings().link_to_month);
        assert!(!page.settings().nav_link);

        Ok(())
    }

    #[test]
    fn flag_week_week() -> anyhow::Result<()> {
        let matches = cmd(["--week", "week"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(page.settings().week);
        assert!(!page.settings().link_to_month);
        assert!(!page.settings().nav_link);

        Ok(())
    }

    #[test]
    fn all_flag_week() -> anyhow::Result<()> {
        let matches = cmd(["--week", "nav", "--week", "month", "--week", "week"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.is_default());
        assert!(page.settings().week);
        assert!(page.settings().link_to_month);
        assert!(page.settings().nav_link);

        Ok(())
    }

    #[test]
    fn all_flag_week_csv() -> anyhow::Result<()> {
        let matches = cmd(["--week", "nav,month,week"])?;
        let page = Page::from(&matches);

        assert!(!page.default);
        assert!(!page.is_default());
        assert!(page.settings().week);
        assert!(page.settings().link_to_month);
        assert!(page.settings().nav_link);

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
        assert!(cmd(["--week", "nav"]).is_ok());
        assert!(cmd(["--week"]).is_err());
    }

    #[test]
    fn disabling_flag_produces_disabled_page() -> anyhow::Result<()> {
        let matches = cmd(["--no-week-page"])?;
        let page = Page::from(&matches);
        assert!(!page.is_default());
        assert!(page.settings().is_empty());

        Ok(())
    }

    #[test]
    fn both_flags_are_exclusive() {
        assert!(cmd(["--week", "nav"]).is_ok());
        assert!(cmd(["--no-week-page"]).is_ok());
        assert!(cmd(["--no-week-page", "--week", "nav"]).is_err());
    }
}
