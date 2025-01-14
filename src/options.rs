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
    #[arg(
        short,
        long,
        num_args = 0..,
        value_enum,
        value_delimiter = ',',
        default_values_t = [DayOption::Day, DayOption::Week]
    )]
    pub day: Vec<DayOption>,

    /// Configure week pages header
    #[arg(
        short,
        long,
        num_args = 0..,
        value_enum,
        value_delimiter = ',',
        default_values_t = [WeekOption::Nav, WeekOption::Month, WeekOption::Week]
    )]
    pub week: Vec<WeekOption>,

    /// Configure month pages header
    #[arg(
        short,
        long,
        num_args = 0..,
        value_enum,
        value_delimiter = ',',
        default_values_t = [MonthOption::Nav, MonthOption::Month]
    )]
    pub month: Vec<MonthOption>,

    /// Configure year pages header
    #[arg(
        short,
        long,
        num_args = 0..,
        value_enum,
        value_delimiter = ',',
        default_values_t = [YearOption::Nav, YearOption::Month]
    )]
    pub year: Vec<YearOption>,
}

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
}

#[derive(Clone, Debug, ValueEnum)]
pub enum WeekOption {
    /// Add embedded week days
    Week,
    /// Add property link to month
    Month,
    /// Add property links to previous and next week
    Nav,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum MonthOption {
    /// Add embedded month days
    Month,
    /// Add property links to previous and next month
    Nav,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum YearOption {
    /// Add link to months
    Month,
    /// Add property links to previous and next year
    Nav,
}
