use chrono::NaiveDate;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Default, Clone, Debug, Parser)]
#[command(version, infer_subcommands = true)]
pub struct Cli {
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    /// Path to notes
    #[arg(short, long)]
    pub path: PathBuf,

    /// Only prepare journal starting from given date
    #[arg(long, value_name = "DATE")]
    pub from: Option<NaiveDate>,

    /// Only prepare journal up to given date
    #[arg(long, value_name = "DATE")]
    pub to: Option<NaiveDate>,

    /// Configure day pages header
    #[arg(short, long, num_args = 0.., value_enum, value_delimiter = ',', default_values_t = [DayOption::Day, DayOption::Week])]
    pub day: Vec<DayOption>,

    /// Configure week pages header
    #[arg(short, long, num_args = 0.., value_enum, value_delimiter = ',', default_values_t = [WeekOption::Nav, WeekOption::Month])]
    pub week: Vec<WeekOption>,

    /// Configure month pages header
    #[arg(short, long, num_args = 0.., value_enum, value_delimiter = ',', default_values_t = [MonthOption::Nav])]
    pub month: Vec<MonthOption>,

    /// Configure year pages header
    #[arg(short, long, num_args = 0.., value_enum, value_delimiter = ',', default_values_t = [YearOption::Nav])]
    pub year: Vec<YearOption>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DayOption {
    /// Display day of week
    Day,
    /// Display link to week
    Week,
    /// Display link to month
    Month,
    /// Display links to previous and next day
    Nav,
}

#[derive(derive_more::Display)]
#[display("Day options: {{ day of week: {day}, week: {week}, month: {month}, navigation links: {nav} }}")]
pub struct DayOptions {
    pub day: bool,
    pub week: bool,
    pub month: bool,
    pub nav: bool,
}

impl DayOptions {
    pub fn none(&self) -> bool {
        !(self.day || self.week || self.month || self.nav)
    }
}

impl From<Vec<DayOption>> for DayOptions {
    fn from(vec: Vec<DayOption>) -> Self {
        Self {
            day: vec.iter().any(|o| matches!(o, DayOption::Day)),
            week: vec.iter().any(|o| matches!(o, DayOption::Week)),
            month: vec.iter().any(|o| matches!(o, DayOption::Month)),
            nav: vec.iter().any(|o| matches!(o, DayOption::Nav)),
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum WeekOption {
    /// Display link to month
    Month,
    /// Display links to previous and next week
    Nav,
}

#[derive(derive_more::Display)]
#[display("Week options: {{ navigation links: {nav}, month: {month} }}")]
pub struct WeekOptions {
    pub nav: bool,
    pub month: bool,
}

impl WeekOptions {
    pub fn none(&self) -> bool {
        !(self.nav || self.month)
    }
}

impl From<Vec<WeekOption>> for WeekOptions {
    fn from(vec: Vec<WeekOption>) -> Self {
        Self {
            nav: vec.iter().any(|o| matches!(o, WeekOption::Nav)),
            month: vec.iter().any(|o| matches!(o, WeekOption::Month)),
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum MonthOption {
    /// Display links to previous and next month
    Nav,
}

#[derive(derive_more::Display)]
#[display("Month options: {{ navigation links: {nav} }}")]
pub struct MonthOptions {
    pub nav: bool,
}

impl MonthOptions {
    pub fn none(&self) -> bool {
        !self.nav
    }
}

impl From<Vec<MonthOption>> for MonthOptions {
    fn from(vec: Vec<MonthOption>) -> Self {
        Self {
            nav: vec.iter().any(|o| matches!(o, MonthOption::Nav)),
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum YearOption {
    /// Display links to previous and next year
    Nav,
}

#[derive(derive_more::Display)]
#[display("Year options: {{ navigation links: {nav} }}")]
pub struct YearOptions {
    pub nav: bool,
}

impl YearOptions {
    pub fn none(&self) -> bool {
        !self.nav
    }
}

impl From<Vec<YearOption>> for YearOptions {
    fn from(vec: Vec<YearOption>) -> Self {
        Self {
            nav: vec.iter().any(|o| matches!(o, YearOption::Nav)),
        }
    }
}
