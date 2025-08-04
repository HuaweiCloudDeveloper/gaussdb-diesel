//! Type system for GaussDB
//!
//! This module provides type mappings and value handling for GaussDB,
//! which is largely PostgreSQL-compatible.

// Basic type support modules
pub mod primitives;
pub mod numeric;
pub mod date_and_time;
pub mod array;
pub mod ranges;
pub mod sql_types;

// JSON support (feature-gated)
#[cfg(feature = "serde_json")]
pub mod json;

// Advanced type support (feature-gated)
#[cfg(feature = "uuid")]
pub mod uuid;
pub mod multirange;
pub mod mac_addr;
pub mod mac_addr_8;
pub mod money;

#[cfg(feature = "ipnetwork")]
pub mod network_address;

// Custom type support
pub mod custom;

// Re-export GaussDBValue from the value module
pub use crate::value::GaussDBValue;

// Re-export commonly used types from diesel
pub use diesel::sql_types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussdb_value_creation() {
        let bytes = b"test";
        let value = GaussDBValue::new(Some(bytes), 25);
        
        assert_eq!(value.as_bytes(), Some(bytes.as_slice()));
        assert_eq!(value.type_oid(), 25);
        assert!(!value.is_null());
    }

    #[test]
    fn test_gaussdb_value_null() {
        let value = GaussDBValue::new(None, 25);
        
        assert_eq!(value.as_bytes(), None);
        assert_eq!(value.type_oid(), 25);
        assert!(value.is_null());
    }

    #[test]
    fn test_gaussdb_value_debug() {
        let bytes = b"test";
        let value = GaussDBValue::new(Some(bytes), 25);
        let debug = format!("{:?}", value);

        assert!(debug.contains("type_oid: 25"));
        assert!(debug.contains("4 bytes"));
    }

    #[test]
    fn test_gaussdb_value_debug_null() {
        let value = GaussDBValue::new(None, 25);
        let debug = format!("{:?}", value);

        assert!(debug.contains("type_oid: 25"));
        assert!(debug.contains("None"));
    }
}
