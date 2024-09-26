use crate::{Month, Year};
use chrono::{Days, IsoWeek, Months, NaiveDate, Weekday};

pub trait DateRange {
    type Element;

    fn first(&self) -> Self::Element;
    fn last(&self) -> Self::Element;
}
impl DateRange for IsoWeek {
    type Element = NaiveDate;

    fn first(&self) -> NaiveDate {
        NaiveDate::from_isoywd_opt(self.year(), self.week(), Weekday::Mon).unwrap()
    }
    fn last(&self) -> NaiveDate {
        NaiveDate::from_isoywd_opt(self.year(), self.week(), Weekday::Sun).unwrap()
    }
}
impl DateRange for Month {
    type Element = NaiveDate;

    fn first(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, 1).unwrap()
    }
    fn last(&self) -> NaiveDate {
        self.first() + Months::new(1) - Days::new(1)
    }
}
impl DateRange for Year {
    type Element = Month;

    fn first(&self) -> Month {
        Month {
            year: self.0,
            month: 1,
        }
    }
    fn last(&self) -> Month {
        Month {
            year: self.0,
            month: 12,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn week() {
        let week = NaiveDate::from_ymd_opt(2024, 9, 24).unwrap().iso_week();
        assert_eq!(week.first(), NaiveDate::from_ymd_opt(2024, 9, 23).unwrap());
        assert_eq!(week.last(), NaiveDate::from_ymd_opt(2024, 9, 29).unwrap());
    }

    #[test]
    fn month() {
        let month = Month::from(NaiveDate::from_ymd_opt(2024, 2, 5).unwrap());
        assert_eq!(month.first(), NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(month.last(), NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    #[test]
    fn year() {
        let year = Year::from(2024);
        assert_eq!(
            year.first(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().into()
        );
        assert_eq!(
            year.last(),
            NaiveDate::from_ymd_opt(2024, 12, 1).unwrap().into()
        );
    }
}
