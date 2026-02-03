use crate::vault::Vault;
use chrono::{Datelike, IsoWeek, NaiveDate};
use utils::date::{Month, Year};

#[derive(Debug, Clone, derive_more::Display)]
#[display("[[/{path}|{title}]]")]
pub struct Link {
    pub path: String,
    pub title: String,
}

pub trait ToLink {
    fn to_link(self, vault: &Vault) -> Link;
}
impl<T: ToPageName> ToLink for T {
    fn to_link(self, vault: &Vault) -> Link {
        let path = vault.page_path(&self);
        let title = if let Some((_, title)) = path.rsplit_once('/') {
            title.to_owned()
        } else {
            path.clone()
        };
        Link { path, title }
    }
}

#[derive(Debug, Clone, derive_more::Display)]
#[display("!{link}")]
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

#[derive(Default, Debug, Clone, Copy)]
pub enum PageKind {
    #[default]
    Default,
    Journal,
}

#[derive(Clone, Debug)]
pub struct PageName {
    pub name: String,
    pub kind: PageKind,
}

impl From<String> for PageName {
    fn from(name: String) -> Self {
        Self {
            name,
            kind: PageKind::default(),
        }
    }
}

pub trait ToPageName {
    fn to_page_name(&self) -> PageName;
}

impl ToPageName for PageName {
    fn to_page_name(&self) -> PageName {
        self.clone()
    }
}

impl ToPageName for IsoWeek {
    fn to_page_name(&self) -> PageName {
        format!("{:04}/Week {:02}", self.year(), self.week()).into()
    }
}

impl ToPageName for NaiveDate {
    fn to_page_name(&self) -> PageName {
        PageName {
            name: format!("{:04}-{:02}-{:02}", self.year(), self.month(), self.day()),
            kind: PageKind::Journal,
        }
    }
}

impl ToPageName for Month {
    fn to_page_name(&self) -> PageName {
        format!("{}/{}", self.year(), self.name()).into()
    }
}

impl ToPageName for Year {
    fn to_page_name(&self) -> PageName {
        self.to_string().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::date::{Month, Year};

    mod page_name {
        use super::*;

        #[test]
        fn date() {
            let date = NaiveDate::from_ymd_opt(2025, 1, 12).unwrap().to_page_name();
            assert_eq!("2025-01-12", date.name);
            assert!(matches!(date.kind, PageKind::Journal));
        }

        #[test]
        fn week() {
            let week = NaiveDate::from_ymd_opt(2025, 1, 12)
                .unwrap()
                .iso_week()
                .to_page_name();
            assert_eq!("2025/Week 02", week.name);
            assert!(matches!(week.kind, PageKind::Default));
        }

        #[test]
        fn month() {
            let month = Month::from(NaiveDate::from_ymd_opt(2025, 1, 12).unwrap()).to_page_name();
            assert_eq!("2025/January", month.name);
            assert!(matches!(month.kind, PageKind::Default));
        }

        #[test]
        fn year() {
            let year = Year::from(2025).to_page_name();
            assert_eq!("2025", year.name);
            assert!(matches!(year.kind, PageKind::Default));
        }
    }
}
