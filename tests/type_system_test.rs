//! Tests for the GaussDB type system implementation
//!
//! This module tests the type conversion functionality between Rust types
//! and GaussDB types, ensuring compatibility with PostgreSQL wire protocol.

use diesel_gaussdb::backend::GaussDB;
use diesel_gaussdb::value::GaussDBValue;
use diesel_gaussdb::types::numeric::GaussDBNumeric;
use diesel_gaussdb::types::date_and_time::{GaussDBTimestamp, GaussDBDate, GaussDBTime, GaussDBInterval};
use diesel::deserialize::FromSql;
use diesel::sql_types::*;

#[test]
fn test_integer_types_from_sql() {
    // Test SmallInt (i16)
    let bytes = 1i16.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 21); // SmallInt OID
    let result: i16 = FromSql::<SmallInt, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, 1i16);

    // Test Integer (i32)
    let bytes = 12345i32.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 23); // Integer OID
    let result: i32 = FromSql::<Integer, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, 12345i32);

    // Test BigInt (i64)
    let bytes = 1234567890123456789i64.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 20); // BigInt OID
    let result: i64 = FromSql::<BigInt, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, 1234567890123456789i64);
}

#[test]
fn test_float_types_from_sql() {
    // Test Float (f32)
    let bytes = 3.14159f32.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 700); // Float OID
    let result: f32 = FromSql::<Float, GaussDB>::from_sql(value).unwrap();
    assert!((result - 3.14159f32).abs() < f32::EPSILON);

    // Test Double (f64)
    let bytes = 3.141592653589793f64.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 701); // Double OID
    let result: f64 = FromSql::<Double, GaussDB>::from_sql(value).unwrap();
    assert!((result - 3.141592653589793f64).abs() < f64::EPSILON);
}

#[test]
fn test_boolean_from_sql() {
    // Test true
    let bytes = [1u8];
    let value = GaussDBValue::new(Some(&bytes), 16); // Bool OID
    let result: bool = FromSql::<Bool, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, true);

    // Test false
    let bytes = [0u8];
    let value = GaussDBValue::new(Some(&bytes), 16); // Bool OID
    let result: bool = FromSql::<Bool, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, false);
}

#[test]
fn test_text_from_sql() {
    // Test Text as *const str
    let text = "Hello, GaussDB!";
    let bytes = text.as_bytes();
    let value = GaussDBValue::new(Some(bytes), 25); // Text OID
    let result: *const str = FromSql::<Text, GaussDB>::from_sql(value).unwrap();
    let result_str = unsafe { &*result };
    assert_eq!(result_str, "Hello, GaussDB!");
}

#[test]
fn test_binary_from_sql() {
    // Test Binary data
    let data = vec![0x01, 0x02, 0x03, 0x04, 0xFF];
    let value = GaussDBValue::new(Some(&data), 17); // Binary OID
    let result: Vec<u8> = FromSql::<Binary, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_oid_from_sql() {
    // Test OID
    let bytes = 12345u32.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 26); // OID OID
    let result: u32 = FromSql::<Oid, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, 12345u32);
}

#[test]
fn test_numeric_type() {
    // Test GaussDBNumeric creation and properties
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
fn test_numeric_from_integers() {
    // Test conversion from i32
    let numeric = GaussDBNumeric::from(12345);
    match numeric {
        GaussDBNumeric::Positive { weight, scale, digits } => {
            assert_eq!(weight, 1);
            assert_eq!(scale, 0);
            assert_eq!(digits, vec![1, 2345]);
        }
        _ => panic!("Expected positive numeric"),
    }

    // Test conversion from negative i32
    let negative = GaussDBNumeric::from(-12345);
    match negative {
        GaussDBNumeric::Negative { weight, scale, digits } => {
            assert_eq!(weight, 1);
            assert_eq!(scale, 0);
            assert_eq!(digits, vec![1, 2345]);
        }
        _ => panic!("Expected negative numeric"),
    }

    // Test conversion from zero
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
fn test_error_handling() {
    // Test null value handling
    let value = GaussDBValue::new(None, 23);
    let result: Result<i32, _> = FromSql::<Integer, GaussDB>::from_sql(value);
    assert!(result.is_err());

    // Test wrong size for i32
    let bytes = vec![0, 1]; // Only 2 bytes
    let value = GaussDBValue::new(Some(&bytes), 23);
    let result: Result<i32, _> = FromSql::<Integer, GaussDB>::from_sql(value);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("less than 4 bytes"));

    // Test wrong size for i16
    let bytes = vec![0, 1, 2, 3]; // 4 bytes instead of 2
    let value = GaussDBValue::new(Some(&bytes), 21);
    let result: Result<i16, _> = FromSql::<SmallInt, GaussDB>::from_sql(value);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("more than 2 bytes"));
}

#[test]
fn test_special_float_values() {
    // Test NaN
    let nan_bytes = f32::NAN.to_be_bytes();
    let value = GaussDBValue::new(Some(&nan_bytes), 700);
    let result: f32 = FromSql::<Float, GaussDB>::from_sql(value).unwrap();
    assert!(result.is_nan());

    // Test infinity
    let inf_bytes = f32::INFINITY.to_be_bytes();
    let value = GaussDBValue::new(Some(&inf_bytes), 700);
    let result: f32 = FromSql::<Float, GaussDB>::from_sql(value).unwrap();
    assert!(result.is_infinite() && result.is_sign_positive());

    // Test negative infinity
    let neg_inf_bytes = f32::NEG_INFINITY.to_be_bytes();
    let value = GaussDBValue::new(Some(&neg_inf_bytes), 700);
    let result: f32 = FromSql::<Float, GaussDB>::from_sql(value).unwrap();
    assert!(result.is_infinite() && result.is_sign_negative());
}

#[test]
fn test_utf8_validation() {
    // Test invalid UTF-8 handling for text
    let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
    let value = GaussDBValue::new(Some(invalid_utf8), 25);
    let result: Result<*const str, _> = FromSql::<Text, GaussDB>::from_sql(value);
    assert!(result.is_err());
}

#[test]
fn test_large_numbers() {
    // Test maximum values
    let max_i64 = i64::MAX;
    let bytes = max_i64.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 20);
    let result: i64 = FromSql::<BigInt, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, max_i64);

    // Test minimum values
    let min_i64 = i64::MIN;
    let bytes = min_i64.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 20);
    let result: i64 = FromSql::<BigInt, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, min_i64);
}

#[test]
fn test_empty_binary_data() {
    // Test empty binary data
    let empty_data: Vec<u8> = vec![];
    let value = GaussDBValue::new(Some(&empty_data), 17);
    let result: Vec<u8> = FromSql::<Binary, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result, empty_data);
}

#[test]
fn test_empty_text() {
    // Test empty text
    let empty_text = "";
    let bytes = empty_text.as_bytes();
    let value = GaussDBValue::new(Some(bytes), 25);
    let result: *const str = FromSql::<Text, GaussDB>::from_sql(value).unwrap();
    let result_str = unsafe { &*result };
    assert_eq!(result_str, "");
}

#[test]
fn test_numeric_from_i64() {
    // Test conversion from large i64
    let large_number = 1234567890123456789i64;
    let numeric = GaussDBNumeric::from(large_number);

    match numeric {
        GaussDBNumeric::Positive { weight, scale, digits } => {
            assert_eq!(scale, 0);
            assert!(weight >= 0);
            assert!(!digits.is_empty());
        }
        _ => panic!("Expected positive numeric"),
    }
}

#[test]
fn test_timestamp_from_sql() {
    // Test Timestamp
    let microseconds = 1234567890123456i64;
    let bytes = microseconds.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 1114); // Timestamp OID
    let result: GaussDBTimestamp = FromSql::<Timestamp, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result.microseconds(), microseconds);

    // Test Timestamptz
    let value = GaussDBValue::new(Some(&bytes), 1184); // Timestamptz OID
    let result: GaussDBTimestamp = FromSql::<Timestamptz, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result.microseconds(), microseconds);
}

#[test]
fn test_date_from_sql() {
    // Test Date
    let julian_days = 12345i32;
    let bytes = julian_days.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 1082); // Date OID
    let result: GaussDBDate = FromSql::<Date, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result.julian_days(), julian_days);
}

#[test]
fn test_time_from_sql() {
    // Test Time
    let microseconds = 86400000000i64; // 24 hours in microseconds
    let bytes = microseconds.to_be_bytes();
    let value = GaussDBValue::new(Some(&bytes), 1083); // Time OID
    let result: GaussDBTime = FromSql::<Time, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result.microseconds(), microseconds);
}

#[test]
fn test_interval_from_sql() {
    // Test Interval - PostgreSQL format: microseconds (8 bytes), days (4 bytes), months (4 bytes)
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&3600000000i64.to_be_bytes()); // 1 hour in microseconds
    bytes.extend_from_slice(&30i32.to_be_bytes()); // 30 days
    bytes.extend_from_slice(&12i32.to_be_bytes()); // 12 months

    let value = GaussDBValue::new(Some(&bytes), 1186); // Interval OID
    let result: GaussDBInterval = FromSql::<Interval, GaussDB>::from_sql(value).unwrap();
    assert_eq!(result.microseconds, 3600000000);
    assert_eq!(result.days, 30);
    assert_eq!(result.months, 12);
}

#[test]
fn test_date_time_error_handling() {
    // Test null timestamp
    let value = GaussDBValue::new(None, 1114);
    let result: Result<GaussDBTimestamp, _> = FromSql::<Timestamp, GaussDB>::from_sql(value);
    assert!(result.is_err());

    // Test wrong size for timestamp
    let bytes = vec![0, 1, 2, 3]; // Only 4 bytes instead of 8
    let value = GaussDBValue::new(Some(&bytes), 1114);
    let result: Result<GaussDBTimestamp, _> = FromSql::<Timestamp, GaussDB>::from_sql(value);
    assert!(result.is_err());

    // Test wrong size for date
    let bytes = vec![0, 1]; // Only 2 bytes instead of 4
    let value = GaussDBValue::new(Some(&bytes), 1082);
    let result: Result<GaussDBDate, _> = FromSql::<Date, GaussDB>::from_sql(value);
    assert!(result.is_err());

    // Test wrong size for interval
    let bytes = vec![0; 8]; // Only 8 bytes instead of 16
    let value = GaussDBValue::new(Some(&bytes), 1186);
    let result: Result<GaussDBInterval, _> = FromSql::<Interval, GaussDB>::from_sql(value);
    assert!(result.is_err());
}

#[test]
fn test_date_time_creation() {
    // Test creation functions
    let timestamp = GaussDBTimestamp::new(1234567890);
    assert_eq!(timestamp.microseconds(), 1234567890);

    let date = GaussDBDate::new(12345);
    assert_eq!(date.julian_days(), 12345);

    let time = GaussDBTime::new(86400000000);
    assert_eq!(time.microseconds(), 86400000000);

    let interval = GaussDBInterval::new(12, 30, 3600000000);
    assert_eq!(interval.months, 12);
    assert_eq!(interval.days, 30);
    assert_eq!(interval.microseconds, 3600000000);
}

#[test]
fn test_date_time_defaults() {
    // Test default values
    assert_eq!(GaussDBTimestamp::default().microseconds(), 0);
    assert_eq!(GaussDBDate::default().julian_days(), 0);
    assert_eq!(GaussDBTime::default().microseconds(), 0);

    let default_interval = GaussDBInterval::default();
    assert_eq!(default_interval.months, 0);
    assert_eq!(default_interval.days, 0);
    assert_eq!(default_interval.microseconds, 0);
}
