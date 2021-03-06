//! Datetimes with a fixed UTC offset.

use std::error::Error as ErrorTrait;
use std::fmt;

use duration::Duration;
use cal::{DatePiece, TimePiece};
use cal::datetime::{LocalDateTime, Month, Weekday, Error as DateTimeError};
use util::RangeExt;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Offset {
    offset_seconds: Option<i32>,
}

impl Offset {
    fn adjust(&self, local: LocalDateTime) -> LocalDateTime {
        match self.offset_seconds {
            Some(s) => local + Duration::of(s as i64),
            None    => local,
        }
    }

    pub fn utc() -> Offset {
        Offset { offset_seconds: None }
    }

    pub fn of_seconds(seconds: i32) -> Result<Offset, Error> {
        if seconds.is_within(-86400..86401) {
            Ok(Offset { offset_seconds: Some(seconds) })
        }
        else {
            Err(Error::OutOfRange)
        }
    }

    pub fn of_hours_and_minutes(hours: i8, minutes: i8) -> Result<Offset, Error> {
        if (hours.is_positive() && minutes.is_negative())
        || (hours.is_negative() && minutes.is_positive()) {
            Err(Error::SignMismatch)
        }
        else if hours <= -24 || hours >= 24 {
            Err(Error::OutOfRange)
        }
        else if minutes <= -60 || minutes >= 60 {
            Err(Error::OutOfRange)
        }
        else {
            let hours = hours as i32;
            let minutes = minutes as i32;
            Offset::of_seconds(hours * 24 + minutes * 60)
        }
    }

    pub fn transform_date(&self, local: LocalDateTime) -> OffsetDateTime {
        OffsetDateTime {
            local: local,
            offset: self.clone(),
        }
    }
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Error {
    OutOfRange,
    SignMismatch,
    Date(DateTimeError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::OutOfRange    => "offset field out of range",
            Error::SignMismatch  => "sign mismatch",
            Error::Date(_)       => "datetime field out of range",
        }
    }

    fn cause(&self) -> Option<&ErrorTrait> {
        if let Error::Date(ref e) = *self {
            Some(e)
        }
        else {
            None
        }
    }
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct OffsetDateTime {
    local: LocalDateTime,
    offset: Offset,
}

impl DatePiece for OffsetDateTime {
    fn year(&self) -> i64 {
        self.offset.adjust(self.local).year()
    }

    fn month(&self) -> Month {
        self.offset.adjust(self.local).month()
    }

    fn day(&self) -> i8 {
        self.offset.adjust(self.local).day()
    }

    fn yearday(&self) -> i16 {
        self.offset.adjust(self.local).yearday()
    }

    fn weekday(&self) -> Weekday {
        self.offset.adjust(self.local).weekday()
    }
}

impl TimePiece for OffsetDateTime {
    fn hour(&self) -> i8 {
        self.offset.adjust(self.local).hour()
    }

    fn minute(&self) -> i8 {
        self.offset.adjust(self.local).minute()
    }

    fn second(&self) -> i8 {
        self.offset.adjust(self.local).second()
    }

    fn millisecond(&self) -> i16 {
        self.offset.adjust(self.local).millisecond()
    }
}


#[cfg(test)]
mod test {
    use super::Offset;

    #[test]
    fn fixed_seconds() {
        assert!(Offset::of_seconds(1234).is_ok());
    }

    #[test]
    fn fixed_seconds_panic() {
        assert!(Offset::of_seconds(100_000).is_err());
    }

    #[test]
    fn fixed_hm() {
        assert!(Offset::of_hours_and_minutes(5, 30).is_ok());
    }

    #[test]
    fn fixed_hm_negative() {
        assert!(Offset::of_hours_and_minutes(-3, -45).is_ok());
    }

    #[test]
    fn fixed_hm_err() {
        assert!(Offset::of_hours_and_minutes(8, 60).is_err());
    }

    #[test]
    fn fixed_hm_signs() {
        assert!(Offset::of_hours_and_minutes(-4, 30).is_err());
    }

    #[test]
    fn fixed_hm_signs_zero() {
        assert!(Offset::of_hours_and_minutes(4, 0).is_ok());
    }
}
