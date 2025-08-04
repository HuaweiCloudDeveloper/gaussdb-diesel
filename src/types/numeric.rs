//! Numeric type support for GaussDB
//!
//! This module provides PostgreSQL-compatible NUMERIC type implementation
//! for GaussDB, closely mirroring the PostgreSQL wire protocol representation.

use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Numeric;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
// Note: std::error::Error import removed as it's unused

/// Represents a NUMERIC value, closely mirroring the PostgreSQL wire protocol
/// representation for GaussDB compatibility.
#[derive(Debug, Default, Clone, PartialEq, Eq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Numeric)]
pub enum GaussDBNumeric {
    /// A positive number
    Positive {
        /// How many digits come before the decimal point?
        weight: i16,
        /// How many significant digits are there?
        scale: u16,
        /// The digits in this number, stored in base 10000
        digits: Vec<i16>,
    },
    /// A negative number
    Negative {
        /// How many digits come before the decimal point?
        weight: i16,
        /// How many significant digits are there?
        scale: u16,
        /// The digits in this number, stored in base 10000
        digits: Vec<i16>,
    },
    /// Not a number
    #[default]
    NaN,
}

impl GaussDBNumeric {
    /// Create a new positive numeric value
    pub fn positive(weight: i16, scale: u16, digits: Vec<i16>) -> Self {
        GaussDBNumeric::Positive { weight, scale, digits }
    }

    /// Create a new negative numeric value
    pub fn negative(weight: i16, scale: u16, digits: Vec<i16>) -> Self {
        GaussDBNumeric::Negative { weight, scale, digits }
    }

    /// Create a NaN value
    pub fn nan() -> Self {
        GaussDBNumeric::NaN
    }

    /// Check if this numeric is NaN
    pub fn is_nan(&self) -> bool {
        matches!(self, GaussDBNumeric::NaN)
    }

    /// Check if this numeric is positive
    pub fn is_positive(&self) -> bool {
        matches!(self, GaussDBNumeric::Positive { .. })
    }

    /// Check if this numeric is negative
    pub fn is_negative(&self) -> bool {
        matches!(self, GaussDBNumeric::Negative { .. })
    }
}

impl FromSql<Numeric, GaussDB> for GaussDBNumeric {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Numeric value is null")?;
        if bytes.len() < 8 {
            return Err("Invalid Numeric: too few bytes".into());
        }

        let mut cursor = std::io::Cursor::new(bytes);
        
        // Read header
        let ndigits = cursor.read_u16::<NetworkEndian>()?;
        let weight = cursor.read_i16::<NetworkEndian>()?;
        let sign = cursor.read_u16::<NetworkEndian>()?;
        let scale = cursor.read_u16::<NetworkEndian>()?;

        // Handle special cases
        match sign {
            0xC000 => return Ok(GaussDBNumeric::NaN),
            0x0000 | 0x4000 => {
                // Positive or negative number
                let mut digits = Vec::with_capacity(ndigits as usize);
                for _ in 0..ndigits {
                    digits.push(cursor.read_i16::<NetworkEndian>()?);
                }

                if sign == 0x0000 {
                    Ok(GaussDBNumeric::Positive { weight, scale, digits })
                } else {
                    Ok(GaussDBNumeric::Negative { weight, scale, digits })
                }
            }
            _ => Err(format!("Invalid Numeric sign: {:#x}", sign).into()),
        }
    }
}

impl ToSql<Numeric, GaussDB> for GaussDBNumeric {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        let sign = match *self {
            GaussDBNumeric::Positive { .. } => 0x0000u16,
            GaussDBNumeric::Negative { .. } => 0x4000u16,
            GaussDBNumeric::NaN => 0xC000u16,
        };

        let empty_vec = Vec::new();
        let digits = match *self {
            GaussDBNumeric::Positive { ref digits, .. } | GaussDBNumeric::Negative { ref digits, .. } => {
                digits
            }
            GaussDBNumeric::NaN => &empty_vec,
        };

        let weight = match *self {
            GaussDBNumeric::Positive { weight, .. } | GaussDBNumeric::Negative { weight, .. } => weight,
            GaussDBNumeric::NaN => 0,
        };

        let scale = match *self {
            GaussDBNumeric::Positive { scale, .. } | GaussDBNumeric::Negative { scale, .. } => scale,
            GaussDBNumeric::NaN => 0,
        };

        // Write header
        out.write_u16::<NetworkEndian>(digits.len() as u16)?;
        out.write_i16::<NetworkEndian>(weight)?;
        out.write_u16::<NetworkEndian>(sign)?;
        out.write_u16::<NetworkEndian>(scale)?;

        // Write digits
        for &digit in digits {
            out.write_i16::<NetworkEndian>(digit)?;
        }

        Ok(IsNull::No)
    }
}

// Note: Defaultable implementations are not needed as they are internal to Diesel

// Convenience implementations for common Rust numeric types
impl From<i32> for GaussDBNumeric {
    fn from(value: i32) -> Self {
        if value == 0 {
            return GaussDBNumeric::Positive { weight: 0, scale: 0, digits: vec![] };
        }

        let abs_value = value.abs() as u32;
        let mut digits = Vec::new();
        let mut remaining = abs_value;

        // Convert to base 10000 digits
        while remaining > 0 {
            digits.insert(0, (remaining % 10000) as i16);
            remaining /= 10000;
        }

        let weight = (digits.len() as i16) - 1;

        if value >= 0 {
            GaussDBNumeric::Positive { weight, scale: 0, digits }
        } else {
            GaussDBNumeric::Negative { weight, scale: 0, digits }
        }
    }
}

impl From<i64> for GaussDBNumeric {
    fn from(value: i64) -> Self {
        if value == 0 {
            return GaussDBNumeric::Positive { weight: 0, scale: 0, digits: vec![] };
        }

        let abs_value = value.abs() as u64;
        let mut digits = Vec::new();
        let mut remaining = abs_value;

        // Convert to base 10000 digits
        while remaining > 0 {
            digits.insert(0, (remaining % 10000) as i16);
            remaining /= 10000;
        }

        let weight = (digits.len() as i16) - 1;

        if value >= 0 {
            GaussDBNumeric::Positive { weight, scale: 0, digits }
        } else {
            GaussDBNumeric::Negative { weight, scale: 0, digits }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussdb_numeric_creation() {
        let positive = GaussDBNumeric::positive(1, 2, vec![1, 2345]);
        assert!(positive.is_positive());
        assert!(!positive.is_negative());
        assert!(!positive.is_nan());

        let negative = GaussDBNumeric::negative(1, 2, vec![1, 2345]);
        assert!(!negative.is_positive());
        assert!(negative.is_negative());
        assert!(!negative.is_nan());

        let nan = GaussDBNumeric::nan();
        assert!(!nan.is_positive());
        assert!(!nan.is_negative());
        assert!(nan.is_nan());
    }

    #[test]
    fn test_from_i32() {
        let numeric = GaussDBNumeric::from(12345);
        match numeric {
            GaussDBNumeric::Positive { weight, scale, digits } => {
                assert_eq!(weight, 1);
                assert_eq!(scale, 0);
                assert_eq!(digits, vec![1, 2345]);
            }
            _ => panic!("Expected positive numeric"),
        }

        let negative = GaussDBNumeric::from(-12345);
        match negative {
            GaussDBNumeric::Negative { weight, scale, digits } => {
                assert_eq!(weight, 1);
                assert_eq!(scale, 0);
                assert_eq!(digits, vec![1, 2345]);
            }
            _ => panic!("Expected negative numeric"),
        }
    }

    #[test]
    fn test_from_zero() {
        let zero = GaussDBNumeric::from(0i32);
        match zero {
            GaussDBNumeric::Positive { weight, scale, digits } => {
                assert_eq!(weight, 0);
                assert_eq!(scale, 0);
                assert_eq!(digits, Vec::<i16>::new());
            }
            _ => panic!("Expected positive numeric for zero"),
        }
    }

    #[test]
    fn test_default() {
        let default = GaussDBNumeric::default();
        assert!(default.is_nan());
    }
}
