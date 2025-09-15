//! UUID type support for GaussDB
//!
//! This module provides support for PostgreSQL-compatible UUID types,
//! which are also supported by GaussDB.

use std::io::prelude::*;

use crate::backend::GaussDB;
use crate::value::GaussDBValue;

#[cfg(feature = "uuid")]
use diesel::deserialize::{self, FromSql, FromSqlRow};
#[cfg(feature = "uuid")]
use diesel::expression::AsExpression;
#[cfg(feature = "uuid")]
use diesel::serialize::{self, IsNull, Output, ToSql};
#[cfg(feature = "uuid")]
use diesel::sql_types::Uuid;



/// FromSql implementation for UUID
#[cfg(feature = "uuid")]
impl FromSql<Uuid, GaussDB> for uuid::Uuid {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("UUID value is null")?;
        uuid::Uuid::from_slice(bytes).map_err(Into::into)
    }
}

/// ToSql implementation for UUID
#[cfg(feature = "uuid")]
impl ToSql<Uuid, GaussDB> for uuid::Uuid {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(self.as_bytes())
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::GaussDBValue;

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_to_sql() {
        use diesel::query_builder::bind_collector::ByteWrapper;
        use diesel::serialize::{Output, ToSql};

        let mut buffer = Vec::new();
        let bytes = [
            0xFF_u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x31, 0x32,
        ];

        let test_uuid = uuid::Uuid::from_slice(&bytes).unwrap();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        ToSql::<Uuid, GaussDB>::to_sql(&test_uuid, &mut output).unwrap();
        assert_eq!(&buffer, test_uuid.as_bytes());
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_from_sql() {
        let bytes = [
            0xFF_u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x31, 0x32,
        ];
        let input_uuid = uuid::Uuid::from_slice(&bytes).unwrap();
        let value = GaussDBValue::new(Some(input_uuid.as_bytes()), 2950); // UUID OID
        let output_uuid: uuid::Uuid = FromSql::<Uuid, GaussDB>::from_sql(value).unwrap();
        assert_eq!(input_uuid, output_uuid);
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_from_sql_standard() {
        // Test with a standard UUID
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let input_uuid = uuid::Uuid::parse_str(uuid_str).unwrap();
        let value = GaussDBValue::new(Some(input_uuid.as_bytes()), 2950);
        let output_uuid: uuid::Uuid = FromSql::<Uuid, GaussDB>::from_sql(value).unwrap();
        assert_eq!(input_uuid, output_uuid);
        assert_eq!(output_uuid.to_string(), uuid_str);
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_bad_uuid_from_sql() {
        let value = GaussDBValue::new(Some(b"boom"), 2950);
        let result: Result<uuid::Uuid, _> = FromSql::<Uuid, GaussDB>::from_sql(value);
        assert!(result.is_err());
        
        // The error message changes slightly between different uuid versions
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("invalid"));
        assert!(error_message.contains("length"));
        assert!(error_message.contains("expected 16"));
        assert!(error_message.contains("found 4"));
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_null_uuid_from_sql() {
        let value = GaussDBValue::new(None, 2950);
        let result: Result<uuid::Uuid, _> = FromSql::<Uuid, GaussDB>::from_sql(value);
        assert_eq!(result.unwrap_err().to_string(), "UUID value is null");
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_roundtrip() {
        use diesel::query_builder::bind_collector::ByteWrapper;
        use diesel::serialize::{Output, ToSql};

        // Test roundtrip: UUID -> bytes -> UUID
        let original_uuid = uuid::Uuid::new_v4();
        
        // Serialize to bytes
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        ToSql::<Uuid, GaussDB>::to_sql(&original_uuid, &mut output).unwrap();
        
        // Deserialize from bytes
        let value = GaussDBValue::new(Some(&buffer), 2950);
        let deserialized_uuid: uuid::Uuid = FromSql::<Uuid, GaussDB>::from_sql(value).unwrap();
        
        assert_eq!(original_uuid, deserialized_uuid);
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_nil() {
        use diesel::query_builder::bind_collector::ByteWrapper;
        use diesel::serialize::{Output, ToSql};

        // Test nil UUID (all zeros)
        let nil_uuid = uuid::Uuid::nil();
        
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        ToSql::<Uuid, GaussDB>::to_sql(&nil_uuid, &mut output).unwrap();
        
        let value = GaussDBValue::new(Some(&buffer), 2950);
        let deserialized_uuid: uuid::Uuid = FromSql::<Uuid, GaussDB>::from_sql(value).unwrap();
        
        assert_eq!(nil_uuid, deserialized_uuid);
        assert!(deserialized_uuid.is_nil());
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_proxy_types() {
        // Test that proxy types are properly defined
        let _proxy = UuidProxy(uuid::Uuid::nil());
        // This test mainly ensures the types compile correctly
    }
}
