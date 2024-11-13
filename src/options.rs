use chrono::NaiveDate;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Default, Clone, Debug, Parser)]
#[command(version, infer_subcommands = true)]
pub struct Cli {
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    /// Path to logseq graph
    #[arg(short, long)]
    pub path: PathBuf,

    /// Only prepare journal starting from given date
    #[arg(long, value_name = "DATE")]
    pub from: Option<NaiveDate>,

    /// Only prepare journal up to given date
    #[arg(long, value_name = "DATE")]
    pub to: Option<NaiveDate>,

    /// Configure day pages header
    #[arg(short, long, num_args = 1.., value_enum, value_delimiter = ',', default_values_t = [DayOption::Day, DayOption::Week])]
    pub day: Vec<DayOption>,

    /// Configure week pages header
    #[arg(short, long, num_args = 1.., value_enum, value_delimiter = ',', default_values_t = [WeekOption::Nav, WeekOption::Month])]
    pub week: Vec<WeekOption>,

    /// Configure month pages header
    #[arg(short, long, num_args = 1.., value_enum, value_delimiter = ',', default_values_t = [MonthOption::Nav])]
    pub month: Vec<MonthOption>,

    /// Configure year pages header
    #[arg(short, long, num_args = 1.., value_enum, value_delimiter = ',', default_values_t = [YearOption::Nav])]
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
}

#[derive(derive_more::Display)]
#[display("Day options: {{ day of week: {day}, week: {week}, month: {month} }}")]
pub struct DayOptions {
    pub day: bool,
    pub week: bool,
    pub month: bool,
}

impl<T> From<T> for DayOptions
where
    T: IntoIterator<Item = DayOption>,
{
    fn from(iter: T) -> Self {
        Self::from_iter(iter)
    }
}

impl FromIterator<DayOption> for DayOptions {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = DayOption>,
    {
        let mut iter = iter.into_iter();

        Self {
            day: iter.any(|o| matches!(o, DayOption::Day)),
            week: iter.any(|o| matches!(o, DayOption::Week)),
            month: iter.any(|o| matches!(o, DayOption::Month)),
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

impl<T> From<T> for WeekOptions
where
    T: IntoIterator<Item = WeekOption>,
{
    fn from(iter: T) -> Self {
        Self::from_iter(iter)
    }
}

impl FromIterator<WeekOption> for WeekOptions {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = WeekOption>,
    {
        let mut iter = iter.into_iter();

        Self {
            nav: iter.any(|o| matches!(o, WeekOption::Nav)),
            month: iter.any(|o| matches!(o, WeekOption::Month)),
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

impl<T> From<T> for MonthOptions
where
    T: IntoIterator<Item = MonthOption>,
{
    fn from(iter: T) -> Self {
        Self::from_iter(iter)
    }
}

impl FromIterator<MonthOption> for MonthOptions {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = MonthOption>,
    {
        let mut iter = iter.into_iter();

        Self {
            nav: iter.any(|o| matches!(o, MonthOption::Nav)),
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

impl<T> From<T> for YearOptions
where
    T: IntoIterator<Item = YearOption>,
{
    fn from(iter: T) -> Self {
        Self::from_iter(iter)
    }
}

impl FromIterator<YearOption> for YearOptions {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = YearOption>,
    {
        let mut iter = iter.into_iter();

        Self {
            nav: iter.any(|o| matches!(o, YearOption::Nav)),
        }
    }
}
