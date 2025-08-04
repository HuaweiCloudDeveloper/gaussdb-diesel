//! Date and time type support for GaussDB
//!
//! This module provides PostgreSQL-compatible date and time type implementations
//! for GaussDB, following the same wire protocol and representation.

use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::{Date, Interval, Time, Timestamp, Timestamptz};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

/// Timestamps are represented in GaussDB as a 64 bit signed integer representing the number of
/// microseconds since January 1st 2000. This struct is a dumb wrapper type, meant only to indicate
/// the integer's meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, AsExpression, FromSqlRow)]
#[diesel(sql_type = Timestamp)]
#[diesel(sql_type = Timestamptz)]
pub struct GaussDBTimestamp(pub i64);

impl GaussDBTimestamp {
    /// Create a new timestamp from microseconds since January 1st 2000
    pub fn new(microseconds: i64) -> Self {
        GaussDBTimestamp(microseconds)
    }

    /// Get the microseconds since January 1st 2000
    pub fn microseconds(&self) -> i64 {
        self.0
    }
}

impl Default for GaussDBTimestamp {
    fn default() -> Self {
        GaussDBTimestamp(0)
    }
}

/// Dates are represented in GaussDB as a 32 bit signed integer representing the number of julian
/// days since January 1st 2000. This struct is a dumb wrapper type, meant only to indicate the
/// integer's meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, AsExpression, FromSqlRow)]
#[diesel(sql_type = Date)]
pub struct GaussDBDate(pub i32);

impl GaussDBDate {
    /// Create a new date from julian days since January 1st 2000
    pub fn new(julian_days: i32) -> Self {
        GaussDBDate(julian_days)
    }

    /// Get the julian days since January 1st 2000
    pub fn julian_days(&self) -> i32 {
        self.0
    }
}

impl Default for GaussDBDate {
    fn default() -> Self {
        GaussDBDate(0)
    }
}

/// Time is represented in GaussDB as a 64 bit signed integer representing the number of
/// microseconds since midnight. This struct is a dumb wrapper type, meant only to indicate the
/// integer's meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, AsExpression, FromSqlRow)]
#[diesel(sql_type = Time)]
pub struct GaussDBTime(pub i64);

impl GaussDBTime {
    /// Create a new time from microseconds since midnight
    pub fn new(microseconds: i64) -> Self {
        GaussDBTime(microseconds)
    }

    /// Get the microseconds since midnight
    pub fn microseconds(&self) -> i64 {
        self.0
    }
}

impl Default for GaussDBTime {
    fn default() -> Self {
        GaussDBTime(0)
    }
}

/// Intervals are represented in GaussDB as a struct containing months, days, and microseconds.
/// This follows the PostgreSQL wire protocol format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Interval)]
pub struct GaussDBInterval {
    /// Number of months
    pub months: i32,
    /// Number of days
    pub days: i32,
    /// Number of microseconds
    pub microseconds: i64,
}

impl GaussDBInterval {
    /// Create a new interval
    pub fn new(months: i32, days: i32, microseconds: i64) -> Self {
        GaussDBInterval {
            months,
            days,
            microseconds,
        }
    }
}

impl Default for GaussDBInterval {
    fn default() -> Self {
        GaussDBInterval {
            months: 0,
            days: 0,
            microseconds: 0,
        }
    }
}

// FromSql implementations
impl FromSql<Timestamp, GaussDB> for GaussDBTimestamp {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Timestamp value is null")?;
        if bytes.len() != 8 {
            return Err("Invalid Timestamp length".into());
        }
        let mut cursor = std::io::Cursor::new(bytes);
        let microseconds = cursor.read_i64::<NetworkEndian>()?;
        Ok(GaussDBTimestamp(microseconds))
    }
}

impl FromSql<Timestamptz, GaussDB> for GaussDBTimestamp {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Timestamptz value is null")?;
        if bytes.len() != 8 {
            return Err("Invalid Timestamptz length".into());
        }
        let mut cursor = std::io::Cursor::new(bytes);
        let microseconds = cursor.read_i64::<NetworkEndian>()?;
        Ok(GaussDBTimestamp(microseconds))
    }
}

impl FromSql<Date, GaussDB> for GaussDBDate {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Date value is null")?;
        if bytes.len() != 4 {
            return Err("Invalid Date length".into());
        }
        let mut cursor = std::io::Cursor::new(bytes);
        let julian_days = cursor.read_i32::<NetworkEndian>()?;
        Ok(GaussDBDate(julian_days))
    }
}

impl FromSql<Time, GaussDB> for GaussDBTime {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Time value is null")?;
        if bytes.len() != 8 {
            return Err("Invalid Time length".into());
        }
        let mut cursor = std::io::Cursor::new(bytes);
        let microseconds = cursor.read_i64::<NetworkEndian>()?;
        Ok(GaussDBTime(microseconds))
    }
}

impl FromSql<Interval, GaussDB> for GaussDBInterval {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Interval value is null")?;
        if bytes.len() != 16 {
            return Err("Invalid Interval length".into());
        }
        let mut cursor = std::io::Cursor::new(bytes);
        let microseconds = cursor.read_i64::<NetworkEndian>()?;
        let days = cursor.read_i32::<NetworkEndian>()?;
        let months = cursor.read_i32::<NetworkEndian>()?;
        Ok(GaussDBInterval {
            months,
            days,
            microseconds,
        })
    }
}

// ToSql implementations
impl ToSql<Timestamp, GaussDB> for GaussDBTimestamp {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_i64::<NetworkEndian>(self.0)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl ToSql<Timestamptz, GaussDB> for GaussDBTimestamp {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_i64::<NetworkEndian>(self.0)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl ToSql<Date, GaussDB> for GaussDBDate {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_i32::<NetworkEndian>(self.0)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl ToSql<Time, GaussDB> for GaussDBTime {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_i64::<NetworkEndian>(self.0)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl ToSql<Interval, GaussDB> for GaussDBInterval {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_i64::<NetworkEndian>(self.microseconds)?;
        out.write_i32::<NetworkEndian>(self.days)?;
        out.write_i32::<NetworkEndian>(self.months)?;
        Ok(IsNull::No)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussdb_timestamp_creation() {
        let timestamp = GaussDBTimestamp::new(1234567890);
        assert_eq!(timestamp.microseconds(), 1234567890);
    }

    #[test]
    fn test_gaussdb_date_creation() {
        let date = GaussDBDate::new(12345);
        assert_eq!(date.julian_days(), 12345);
    }

    #[test]
    fn test_gaussdb_time_creation() {
        let time = GaussDBTime::new(86400000000); // 24 hours in microseconds
        assert_eq!(time.microseconds(), 86400000000);
    }

    #[test]
    fn test_gaussdb_interval_creation() {
        let interval = GaussDBInterval::new(12, 30, 3600000000); // 12 months, 30 days, 1 hour
        assert_eq!(interval.months, 12);
        assert_eq!(interval.days, 30);
        assert_eq!(interval.microseconds, 3600000000);
    }

    #[test]
    fn test_default_values() {
        assert_eq!(GaussDBTimestamp::default().microseconds(), 0);
        assert_eq!(GaussDBDate::default().julian_days(), 0);
        assert_eq!(GaussDBTime::default().microseconds(), 0);
        
        let default_interval = GaussDBInterval::default();
        assert_eq!(default_interval.months, 0);
        assert_eq!(default_interval.days, 0);
        assert_eq!(default_interval.microseconds, 0);
    }
}
