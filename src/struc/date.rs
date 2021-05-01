use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use super::{
    common::{DATE_CHAR, TIME_CHAR},
    err::{ParseTomlError, TomlErrorKind, TomlResult},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum TomlDate {
    DateTime(NaiveDateTime),
    Date(NaiveDate),
    Time(NaiveTime),
}

impl TomlDate {
    pub(crate) fn from_str(input: &str) -> TomlResult<TomlDate> {
        if input.contains(DATE_CHAR) && input.contains(TIME_CHAR) && input.contains('T') {
            let dt = input.split('T').collect::<Vec<_>>();

            assert_eq!(dt.len(), 2);

            let date = dt[0].split(DATE_CHAR).collect::<Vec<_>>();
            let time = dt[1].split(TIME_CHAR).collect::<Vec<_>>();

            let ndt = if time.len() > 3 {
                if input.contains('+') {
                    // TODO dont include offset for now
                    NaiveDate::from_ymd(
                        date[0].parse()?,
                        date[1].parse()?,
                        date[2].parse()?,
                    )
                    .and_hms(
                        time[0].parse()?,
                        time[1].parse()?,
                        time[2].parse()?,
                    )
                } else {
                    NaiveDate::from_ymd(
                        date[0].parse()?,
                        date[1].parse()?,
                        date[2].parse()?,
                    )
                    .and_hms_milli(
                        time[0].parse()?,
                        time[1].parse()?,
                        time[2].parse()?,
                        time[3].parse()?,
                    )
                }
            } else {
                NaiveDate::from_ymd(date[0].parse()?, date[1].parse()?, date[2].parse()?)
                    .and_hms(time[0].parse()?, time[1].parse()?, time[2].parse()?)
            };
            Ok(TomlDate::DateTime(ndt))
        } else if input.contains(TIME_CHAR) {
            let time = input.split(TIME_CHAR).collect::<Vec<_>>();

            assert!(time.len() >= 3);

            let ndt = if time[2].contains('.') {
                let (sec, milli) = {
                    let fractional = time[2].split('.').collect::<Vec<_>>();
                    (fractional[0].parse()?, fractional[1].parse()?)
                };
                NaiveTime::from_hms_milli(time[0].parse()?, time[1].parse()?, sec, milli)
            } else {
                NaiveTime::from_hms(time[0].parse()?, time[1].parse()?, time[2].parse()?)
            };
            Ok(TomlDate::Time(ndt))
        } else if input.contains(DATE_CHAR) {
            let date = input.split(DATE_CHAR).collect::<Vec<_>>();

            assert_eq!(date.len(), 3);

            let ndt =
                NaiveDate::from_ymd(date[0].parse()?, date[1].parse()?, date[2].parse()?);
            Ok(TomlDate::Date(ndt))
        } else {
            Err(ParseTomlError::new(String::default(), TomlErrorKind::DateError))
        }
    }
}
impl PartialEq<NaiveDateTime> for TomlDate {
    fn eq(&self, other: &NaiveDateTime) -> bool {
        match self {
            Self::DateTime(dt) => dt == other,
            _ => false,
        }
    }
}
impl PartialEq<NaiveDate> for TomlDate {
    fn eq(&self, other: &NaiveDate) -> bool {
        match self {
            Self::Date(d) => d == other,
            _ => false,
        }
    }
}
impl PartialEq<NaiveTime> for TomlDate {
    fn eq(&self, other: &NaiveTime) -> bool {
        match self {
            Self::Time(t) => t == other,
            _ => false,
        }
    }
}
