use crate::{DateRange, Month, Year};
use chrono::{Datelike, Days, IsoWeek, Months, NaiveDate};

pub trait Navigation {
    fn next(&self) -> Self;
    fn prev(&self) -> Self;
}

impl Navigation for NaiveDate {
    fn next(&self) -> Self {
        *self + Days::new(1)
    }
    fn prev(&self) -> Self {
        *self - Days::new(1)
    }
}

impl Navigation for Month {
    fn next(&self) -> Self {
        (self.first() + Months::new(1)).into()
    }
    fn prev(&self) -> Self {
        (self.first() - Months::new(1)).into()
    }
}

impl Navigation for Year {
    fn next(&self) -> Self {
        Year(self.0 + 1)
    }
    fn prev(&self) -> Self {
        Year(self.0 - 1)
    }
}

impl Navigation for IsoWeek {
    fn next(&self) -> Self {
        (self.last() + Days::new(1)).iso_week()
    }
    fn prev(&self) -> Self {
        (self.first() - Days::new(1)).iso_week()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date() {
        let date = NaiveDate::from_ymd_opt(2024, 9, 1).unwrap();
        assert_eq!(date.next(), NaiveDate::from_ymd_opt(2024, 9, 2).unwrap());
        assert_eq!(date.prev(), NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    }

    #[test]
    fn week() {
        let week = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap().iso_week();
        let prev = week.prev();
        assert_eq!(52, prev.week());
        assert_eq!(2024, prev.year());

        let next = week.next();
        assert_eq!(2, next.week());
        assert_eq!(2025, next.year());
    }

    #[test]
    fn month() {
        let month = Month::from(NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());

        assert_eq!(
            Month {
                year: 2024,
                month: 11
            },
            month.prev()
        );
        assert_eq!(
            Month {
                year: 2025,
                month: 1
            },
            month.next()
        );
    }

    #[test]
    fn year() {
        let year = Year::from(2024);
        assert_eq!(Year::from(2023), year.prev());
        assert_eq!(Year::from(2025), year.next());
    }
}
