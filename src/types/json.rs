//! Support for JSON and `jsonb` values under GaussDB.
//!
//! This module provides support for PostgreSQL-style JSON types,
//! which are also supported by GaussDB.



use std::io::prelude::*;

use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::{Json, Jsonb};

/// JSON type implementation for GaussDB
#[cfg(feature = "serde_json")]
impl FromSql<Json, GaussDB> for serde_json::Value {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("JSON value is null")?;
        serde_json::from_slice(bytes).map_err(|_| "Invalid Json".into())
    }
}

#[cfg(feature = "serde_json")]
impl ToSql<Json, GaussDB> for serde_json::Value {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        serde_json::to_writer(out, self)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

/// JSONB type implementation for GaussDB
#[cfg(feature = "serde_json")]
impl FromSql<Jsonb, GaussDB> for serde_json::Value {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("JSONB value is null")?;
        if bytes.is_empty() {
            return Err("Empty JSONB value".into());
        }
        if bytes[0] != 1 {
            return Err("Unsupported JSONB encoding version".into());
        }
        serde_json::from_slice(&bytes[1..]).map_err(|_| "Invalid Json".into())
    }
}

#[cfg(feature = "serde_json")]
impl ToSql<Jsonb, GaussDB> for serde_json::Value {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&[1])?;
        serde_json::to_writer(out, self)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::GaussDBValue;
    use diesel::query_builder::bind_collector::ByteWrapper;
    use diesel::serialize::{Output, ToSql};
    use diesel::deserialize::FromSql;

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_json_to_sql() {
        let mut buffer = Vec::new();
        let mut bytes = Output::test(ByteWrapper(&mut buffer));
        let test_json = serde_json::Value::Bool(true);
        ToSql::<sql_types::Json, GaussDB>::to_sql(&test_json, &mut bytes).unwrap();
        assert_eq!(buffer, b"true");
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_json_from_sql() {
        let input_json = b"true";
        let value = GaussDBValue::new(Some(input_json), 114); // JSON OID
        let output_json: serde_json::Value =
            FromSql::<sql_types::Json, GaussDB>::from_sql(value).unwrap();
        assert_eq!(output_json, serde_json::Value::Bool(true));
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_json_from_sql_complex() {
        let input_json = br#"{"name": "test", "value": 42}"#;
        let value = GaussDBValue::new(Some(input_json), 114);
        let output_json: serde_json::Value =
            FromSql::<sql_types::Json, GaussDB>::from_sql(value).unwrap();
        
        assert!(output_json.is_object());
        assert_eq!(output_json["name"], "test");
        assert_eq!(output_json["value"], 42);
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_bad_json_from_sql() {
        let value = GaussDBValue::new(Some(b"boom"), 114);
        let result: Result<serde_json::Value, _> =
            FromSql::<sql_types::Json, GaussDB>::from_sql(value);
        assert_eq!(result.unwrap_err().to_string(), "Invalid Json");
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_null_json_from_sql() {
        let value = GaussDBValue::new(None, 114);
        let result: Result<serde_json::Value, _> =
            FromSql::<sql_types::Json, GaussDB>::from_sql(value);
        assert_eq!(result.unwrap_err().to_string(), "JSON value is null");
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_jsonb_to_sql() {
        let mut buffer = Vec::new();
        let mut bytes = Output::test(ByteWrapper(&mut buffer));
        let test_json = serde_json::Value::Bool(true);
        ToSql::<sql_types::Jsonb, GaussDB>::to_sql(&test_json, &mut bytes).unwrap();
        assert_eq!(buffer, b"\x01true");
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_jsonb_from_sql() {
        let input_jsonb = b"\x01true";
        let value = GaussDBValue::new(Some(input_jsonb), 3802); // JSONB OID
        let output_json: serde_json::Value =
            FromSql::<sql_types::Jsonb, GaussDB>::from_sql(value).unwrap();
        assert_eq!(output_json, serde_json::Value::Bool(true));
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_jsonb_from_sql_complex() {
        let input_jsonb = b"\x01{\"name\": \"test\", \"value\": 42}";
        let value = GaussDBValue::new(Some(input_jsonb), 3802);
        let output_json: serde_json::Value =
            FromSql::<sql_types::Jsonb, GaussDB>::from_sql(value).unwrap();
        
        assert!(output_json.is_object());
        assert_eq!(output_json["name"], "test");
        assert_eq!(output_json["value"], 42);
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_bad_jsonb_from_sql() {
        let value = GaussDBValue::new(Some(b"\x01boom"), 3802);
        let result: Result<serde_json::Value, _> =
            FromSql::<sql_types::Jsonb, GaussDB>::from_sql(value);
        assert_eq!(result.unwrap_err().to_string(), "Invalid Json");
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_jsonb_wrong_version() {
        let value = GaussDBValue::new(Some(b"\x02true"), 3802); // Wrong version
        let result: Result<serde_json::Value, _> =
            FromSql::<sql_types::Jsonb, GaussDB>::from_sql(value);
        assert_eq!(result.unwrap_err().to_string(), "Unsupported JSONB encoding version");
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_jsonb_empty() {
        let value = GaussDBValue::new(Some(b""), 3802); // Empty bytes
        let result: Result<serde_json::Value, _> =
            FromSql::<sql_types::Jsonb, GaussDB>::from_sql(value);
        assert_eq!(result.unwrap_err().to_string(), "Empty JSONB value");
    }

    #[test]
    fn test_json_sql_types() {
        use super::sql_types::*;
        
        // Test that types implement required traits
        let json = Json;
        let jsonb = Jsonb;
        
        assert!(format!("{:?}", json).contains("Json"));
        assert!(format!("{:?}", jsonb).contains("Jsonb"));
        
        // Test QueryId implementation
        assert!(!Json::HAS_STATIC_QUERY_ID);
        assert!(!Jsonb::HAS_STATIC_QUERY_ID);
    }
}
