use crate::date_utils::{Month, Year};
use chrono::{Datelike, IsoWeek, NaiveDate};

#[derive(Debug, Clone, derive_more::Display)]
#[display("[[{name}]]")]
pub struct Link {
    pub name: String,
}

pub trait ToLink {
    fn to_link(&self) -> Link;
}
impl<T: JournalName> ToLink for T {
    fn to_link(&self) -> Link {
        Link {
            name: self.to_journal_name(),
        }
    }
}

#[derive(Debug, Clone, derive_more::Display)]
#[display("{{{{embed {link}}}}}")]
pub struct Embedded {
    pub link: Link,
}

pub trait ToEmbedded {
    fn into_embedded(self) -> Embedded;
}
impl ToEmbedded for Link {
    fn into_embedded(self) -> Embedded {
        Embedded { link: self }
    }
}

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
        format!("{}/{}", self.year(), self.name())
    }
    fn to_journal_path_name(&self) -> String {
        format!("{}___{}.md", self.year(), self.name())
    }
}

impl JournalName for Year {
    fn to_journal_name(&self) -> String {
        self.to_string()
    }

    fn to_journal_path_name(&self) -> String {
        format!("{}.md", self)
    }
}
