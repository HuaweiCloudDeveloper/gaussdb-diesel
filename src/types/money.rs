//! Support for Money values under GaussDB (PostgreSQL compatible).
//!
//! Money is represented in PostgreSQL/GaussDB as a 64 bit signed integer.
//! The fractional precision is determined by the `lc_monetary` setting of the database.

use std::ops::{Add, AddAssign, Sub, SubAssign};

use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::{BigInt, Money};

/// Money is represented in GaussDB as a 64 bit signed integer. This struct is a dumb wrapper
/// type, meant only to indicate the integer's meaning. The fractional precision of the value is
/// determined by the [`lc_monetary` setting of the database](https://www.postgresql.org/docs/9.6/static/datatype-money.html).
/// This struct is re-exported as `Cents` as a convenient and conventional expression of a typical
/// unit of 1/100th of currency. For other names or precisions, users might consider a differently
/// named `use` of the `GaussDBMoney` struct.
///
/// ```rust
/// use diesel_gaussdb::data_types::GaussDBMoney as Pence; // 1/100th unit of Pound
/// use diesel_gaussdb::data_types::GaussDBMoney as Fils; // 1/1000th unit of Dinar
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, AsExpression, FromSqlRow)]
#[diesel(sql_type = Money)]
pub struct GaussDBMoney(pub i64);

/// Convenient alias for GaussDBMoney representing cents (1/100th of currency)
pub type Cents = GaussDBMoney;

impl GaussDBMoney {
    /// Create a new money value from cents (assuming 2 decimal places)
    pub fn from_cents(cents: i64) -> Self {
        GaussDBMoney(cents)
    }

    /// Get the raw value in the smallest currency unit
    pub fn as_cents(&self) -> i64 {
        self.0
    }

    /// Create a money value from a floating point amount (in major currency units)
    /// This assumes 2 decimal places (e.g., dollars to cents)
    pub fn from_dollars(dollars: f64) -> Self {
        GaussDBMoney((dollars * 100.0).round() as i64)
    }

    /// Convert to floating point amount in major currency units
    /// This assumes 2 decimal places (e.g., cents to dollars)
    pub fn to_dollars(&self) -> f64 {
        self.0 as f64 / 100.0
    }

    /// Create a money value from a decimal string (e.g., "123.45")
    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        let dollars: f64 = s.parse()
            .map_err(|_| "Invalid money format")?;
        Ok(Self::from_dollars(dollars))
    }

    /// Convert to a decimal string representation
    pub fn to_string(&self) -> String {
        format!("{:.2}", self.to_dollars())
    }
}

#[cfg(feature = "gaussdb")]
impl FromSql<Money, GaussDB> for GaussDBMoney {
    fn from_sql(bytes: GaussDBValue<'_>) -> deserialize::Result<Self> {
        FromSql::<BigInt, GaussDB>::from_sql(bytes).map(GaussDBMoney)
    }
}

#[cfg(feature = "gaussdb")]
impl ToSql<Money, GaussDB> for GaussDBMoney {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        ToSql::<BigInt, GaussDB>::to_sql(&self.0, out)
    }
}

impl Add for GaussDBMoney {
    type Output = Self;
    /// # Panics
    ///
    /// Performs a checked addition, and will `panic!` on overflow in both `debug` and `release`.
    fn add(self, rhs: GaussDBMoney) -> Self::Output {
        self.0
            .checked_add(rhs.0)
            .map(GaussDBMoney)
            .expect("overflow adding money amounts")
    }
}

impl AddAssign for GaussDBMoney {
    /// # Panics
    ///
    /// Performs a checked addition, and will `panic!` on overflow in both `debug` and `release`.
    fn add_assign(&mut self, rhs: GaussDBMoney) {
        self.0 = self
            .0
            .checked_add(rhs.0)
            .expect("overflow adding money amounts")
    }
}

impl Sub for GaussDBMoney {
    type Output = Self;
    /// # Panics
    ///
    /// Performs a checked subtraction, and will `panic!` on underflow in both `debug` and `release`.
    fn sub(self, rhs: GaussDBMoney) -> Self::Output {
        self.0
            .checked_sub(rhs.0)
            .map(GaussDBMoney)
            .expect("underflow subtracting money amounts")
    }
}

impl SubAssign for GaussDBMoney {
    /// # Panics
    ///
    /// Performs a checked subtraction, and will `panic!` on underflow in both `debug` and `release`.
    fn sub_assign(&mut self, rhs: GaussDBMoney) {
        self.0 = self
            .0
            .checked_sub(rhs.0)
            .expect("underflow subtracting money amounts")
    }
}

impl std::fmt::Display for GaussDBMoney {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::str::FromStr for GaussDBMoney {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GaussDBMoney::from_string(s)
    }
}

impl From<i64> for GaussDBMoney {
    fn from(cents: i64) -> Self {
        GaussDBMoney::from_cents(cents)
    }
}

impl From<GaussDBMoney> for i64 {
    fn from(money: GaussDBMoney) -> Self {
        money.0
    }
}

#[cfg(feature = "quickcheck")]
mod quickcheck_impls {
    extern crate quickcheck;

    use self::quickcheck::{Arbitrary, Gen};
    use super::GaussDBMoney;

    impl Arbitrary for GaussDBMoney {
        fn arbitrary(g: &mut Gen) -> Self {
            GaussDBMoney(i64::arbitrary(g))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_money() {
        let c1 = GaussDBMoney(123);
        let c2 = GaussDBMoney(456);
        assert_eq!(GaussDBMoney(579), c1 + c2);
    }

    #[test]
    fn test_add_assign_money() {
        let mut c1 = GaussDBMoney(123);
        c1 += GaussDBMoney(456);
        assert_eq!(GaussDBMoney(579), c1);
    }

    #[test]
    #[should_panic(expected = "overflow adding money amounts")]
    fn test_add_money_overflow() {
        let c1 = GaussDBMoney(i64::MAX);
        let c2 = GaussDBMoney(1);
        let _overflow = c1 + c2;
    }

    #[test]
    #[should_panic(expected = "overflow adding money amounts")]
    fn test_add_assign_money_overflow() {
        let mut c1 = GaussDBMoney(i64::MAX);
        c1 += GaussDBMoney(1);
    }

    #[test]
    fn test_sub_money() {
        let c1 = GaussDBMoney(123);
        let c2 = GaussDBMoney(456);
        assert_eq!(GaussDBMoney(-333), c1 - c2);
    }

    #[test]
    fn test_sub_assign_money() {
        let mut c1 = GaussDBMoney(123);
        c1 -= GaussDBMoney(456);
        assert_eq!(GaussDBMoney(-333), c1);
    }

    #[test]
    #[should_panic(expected = "underflow subtracting money amounts")]
    fn test_sub_money_underflow() {
        let c1 = GaussDBMoney(i64::MIN);
        let c2 = GaussDBMoney(1);
        let _underflow = c1 - c2;
    }

    #[test]
    #[should_panic(expected = "underflow subtracting money amounts")]
    fn test_sub_assign_money_underflow() {
        let mut c1 = GaussDBMoney(i64::MIN);
        c1 -= GaussDBMoney(1);
    }

    #[test]
    fn test_from_cents() {
        let money = GaussDBMoney::from_cents(12345);
        assert_eq!(money.as_cents(), 12345);
    }

    #[test]
    fn test_from_dollars() {
        let money = GaussDBMoney::from_dollars(123.45);
        assert_eq!(money.as_cents(), 12345);
        assert_eq!(money.to_dollars(), 123.45);
    }

    #[test]
    fn test_from_string() {
        let money = GaussDBMoney::from_string("123.45").unwrap();
        assert_eq!(money.as_cents(), 12345);
        assert_eq!(money.to_string(), "123.45");
    }

    #[test]
    fn test_display() {
        let money = GaussDBMoney::from_cents(12345);
        assert_eq!(format!("{}", money), "123.45");
    }

    #[test]
    fn test_from_str_trait() {
        use std::str::FromStr;
        let money = GaussDBMoney::from_str("123.45").unwrap();
        assert_eq!(money.as_cents(), 12345);
    }

    #[test]
    fn test_conversions() {
        let cents = 12345i64;
        let money = GaussDBMoney::from(cents);
        let converted_cents: i64 = money.into();
        assert_eq!(cents, converted_cents);
    }

    #[test]
    fn test_cents_alias() {
        let cents: Cents = GaussDBMoney::from_cents(12345);
        assert_eq!(cents.as_cents(), 12345);
    }
}
