use chrono::{Datelike, Days, IsoWeek, Months, NaiveDate, Weekday};

#[derive(Debug, Default, Clone, Copy, PartialEq, derive_more::From, derive_more::Display)]
#[display("{:04}", _0)]
pub struct Year(i32);

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Month {
    year: i32,
    month: u32,
}

impl Month {
    pub fn name(&self) -> &str {
        chrono::Month::try_from(self.month as u8).unwrap().name()
    }

    pub fn year(&self) -> Year {
        self.year.into()
    }

    pub fn num_days(&self) -> u32 {
        match self.month {
            2 => {
                if NaiveDate::from_ymd_opt(self.year, self.month, 29).is_some() {
                    29
                } else {
                    28
                }
            }
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => 30,
        }
    }
}

impl From<NaiveDate> for Month {
    fn from(date: NaiveDate) -> Self {
        Month {
            year: date.year(),
            month: date.month(),
        }
    }
}
impl From<IsoWeek> for Month {
    fn from(week: IsoWeek) -> Self {
        Self::from(week.first())
    }
}
impl std::ops::Add<Months> for Month {
    type Output = Self;

    fn add(self, rhs: Months) -> Self {
        let month = self.month - 1 + rhs.as_u32();

        Month {
            year: self.year + month.div_euclid(12) as i32,
            month: month.rem_euclid(12) + 1,
        }
    }
}
impl std::ops::Sub<Months> for Month {
    type Output = Self;

    fn sub(self, rhs: Months) -> Self {
        let month = self.month as i32 - 1 - rhs.as_u32() as i32;

        Month {
            year: self.year + month.div_euclid(12),
            month: month.rem_euclid(12) as u32 + 1,
        }
    }
}

pub trait ToDateIterator {
    type Element;

    fn first(&self) -> Self::Element;
    fn last(&self) -> Self::Element;

    fn iter(&self) -> DateIterator<'_, Self, Self::Element>
    where
        Self::Element: Navigation + std::cmp::PartialOrd + Clone,
    {
        DateIterator {
            range: self,
            current: None,
        }
    }
}

impl ToDateIterator for IsoWeek {
    type Element = NaiveDate;

    fn first(&self) -> NaiveDate {
        NaiveDate::from_isoywd_opt(self.year(), self.week(), Weekday::Mon).unwrap()
    }
    fn last(&self) -> NaiveDate {
        NaiveDate::from_isoywd_opt(self.year(), self.week(), Weekday::Sun).unwrap()
    }
}
impl ToDateIterator for Month {
    type Element = NaiveDate;

    fn first(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, 1).unwrap()
    }
    fn last(&self) -> NaiveDate {
        self.first() + Months::new(1) - Days::new(1)
    }
}
impl ToDateIterator for Year {
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
        *self + Months::new(1)
    }
    fn prev(&self) -> Self {
        *self - Months::new(1)
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

pub struct DateIterator<'a, T, U>
where
    T: ToDateIterator<Element = U> + ?Sized,
    U: Navigation + std::cmp::PartialOrd + Clone,
{
    range: &'a T,
    current: Option<U>,
}

impl<T, U> std::iter::FusedIterator for DateIterator<'_, T, U>
where
    T: ToDateIterator<Element = U>,
    U: Navigation + std::cmp::PartialOrd + Clone,
{
}

impl<T, U> Iterator for DateIterator<'_, T, U>
where
    T: ToDateIterator<Element = U>,
    U: Navigation + std::cmp::PartialOrd + Clone,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.current {
            None => {
                self.current = Some(self.range.first());
                self.current.clone()
            }
            Some(value) if *value < self.range.last() => {
                self.current = Some(value.next());
                self.current.clone()
            }
            _ => None,
        }
    }
}

impl<T, U> DoubleEndedIterator for DateIterator<'_, T, U>
where
    T: ToDateIterator<Element = U>,
    U: Navigation + std::cmp::PartialOrd + Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match &self.current {
            None => {
                self.current = Some(self.range.last());
                self.current.clone()
            }
            Some(value) if *value > self.range.first() => {
                self.current = Some(value.prev());
                self.current.clone()
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_num_days() {
        assert_eq!(31, Month { year: 2025, month: 1 }.num_days());
        assert_eq!(28, Month { year: 2025, month: 2 }.num_days());
        assert_eq!(29, Month { year: 2024, month: 2 }.num_days());
        assert_eq!(31, Month { year: 2025, month: 3 }.num_days());
        assert_eq!(30, Month { year: 2025, month: 4 }.num_days());
        assert_eq!(31, Month { year: 2025, month: 5 }.num_days());
        assert_eq!(30, Month { year: 2025, month: 6 }.num_days());
        assert_eq!(31, Month { year: 2025, month: 7 }.num_days());
        assert_eq!(31, Month { year: 2025, month: 8 }.num_days());
        assert_eq!(30, Month { year: 2025, month: 9 }.num_days());
        assert_eq!(31, Month { year: 2025, month: 10 }.num_days());
        assert_eq!(30, Month { year: 2025, month: 11 }.num_days());
        assert_eq!(31, Month { year: 2025, month: 12 }.num_days());
    }

    #[test]
    fn month_arithmetic() {
        let month = Month::from(NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());

        assert_eq!(
            Month {
                year: 2025,
                month: 1
            },
            month + Months::new(1)
        );

        assert_eq!(
            Month {
                year: 2023,
                month: 12
            },
            month - Months::new(12)
        );
    }

    mod to_date_iterator {
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

    mod navigation {
        use super::*;

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

    mod date_iterator {
        use super::*;

        #[test]
        fn week() {
            let week = NaiveDate::from_ymd_opt(2024, 9, 24).unwrap().iso_week();
            assert_eq!(7, week.iter().count());
            assert_eq!(week.iter().next(), NaiveDate::from_ymd_opt(2024, 9, 23));
            assert_eq!(
                week.iter().next_back(),
                NaiveDate::from_ymd_opt(2024, 9, 29)
            );
        }

        #[test]
        fn month() {
            let month = Month::from(NaiveDate::from_ymd_opt(2024, 2, 5).unwrap());
            assert_eq!(29, month.iter().count());
            assert_eq!(month.iter().next(), NaiveDate::from_ymd_opt(2024, 2, 1));
            assert_eq!(
                month.iter().next_back(),
                NaiveDate::from_ymd_opt(2024, 2, 29)
            );
        }

        #[test]
        fn year() {
            let year = Year::from(2024);
            assert_eq!(12, year.iter().count());
            assert_eq!(
                year.iter().next(),
                Some(Month {
                    year: 2024,
                    month: 1
                })
            );
            assert_eq!(
                year.iter().next_back(),
                Some(Month {
                    year: 2024,
                    month: 12
                })
            );
        }
    }
}
