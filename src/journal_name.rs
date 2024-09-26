use crate::{Month, Year};
use chrono::{Datelike, IsoWeek, NaiveDate};

pub trait JournalName {
    fn to_journal_name(&self) -> String;
    fn to_journal_path_name(&self) -> String;
}

impl JournalName for IsoWeek {
    fn to_journal_name(&self) -> String {
        format!("{:04}/Week {:02}", self.year(), self.week())
    }
    fn to_journal_path_name(&self) -> String {
        format!("{:04}___Week {:02}.md", self.year(), self.week())
    }
}

impl JournalName for NaiveDate {
    fn to_journal_name(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year(), self.month(), self.day())
    }
    fn to_journal_path_name(&self) -> String {
        format!(
            "{:04}_{:02}_{:02}.md",
            self.year(),
            self.month(),
            self.day()
        )
    }
}

impl JournalName for Month {
    fn to_journal_name(&self) -> String {
        format!("{:04}/{}", self.year, self.name())
    }
    fn to_journal_path_name(&self) -> String {
        format!("{:04}___{}.md", self.year, self.name())
    }
}

impl JournalName for Year {
    fn to_journal_name(&self) -> String {
        format!("{:04}", self.0)
    }

    fn to_journal_path_name(&self) -> String {
        format!("{:04}.md", self.0)
    }
}
