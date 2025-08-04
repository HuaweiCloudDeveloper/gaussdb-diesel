//! Value handling for GaussDB
//!
//! This module provides the value type used for handling raw database values
//! in GaussDB, similar to PostgreSQL's PgValue.

use std::fmt;

/// A raw value from a GaussDB query result
///
/// This type is similar to PostgreSQL's PgValue and provides access to
/// the raw bytes and type information for a database value.
#[derive(Clone, Copy)]
pub struct GaussDBValue<'a> {
    raw_bytes: Option<&'a [u8]>,
    type_oid: u32,
}

impl<'a> GaussDBValue<'a> {
    /// Create a new GaussDBValue from raw bytes and type OID
    pub fn new(raw_bytes: Option<&'a [u8]>, type_oid: u32) -> Self {
        Self {
            raw_bytes,
            type_oid,
        }
    }

    /// Get the raw bytes of this value
    pub fn as_bytes(&self) -> Option<&'a [u8]> {
        self.raw_bytes
    }

    /// Get the type OID of this value
    pub fn type_oid(&self) -> u32 {
        self.type_oid
    }

    /// Check if this value is NULL
    pub fn is_null(&self) -> bool {
        self.raw_bytes.is_none()
    }

    /// Get the length of the raw bytes
    pub fn len(&self) -> Option<usize> {
        self.raw_bytes.map(|bytes| bytes.len())
    }

    /// Check if the value is empty (has zero-length bytes)
    pub fn is_empty(&self) -> bool {
        self.raw_bytes.map_or(false, |bytes| bytes.is_empty())
    }

    /// Create a test value for testing purposes
    #[cfg(test)]
    pub fn for_test(bytes: &'a [u8]) -> Self {
        Self {
            raw_bytes: Some(bytes),
            type_oid: 0,
        }
    }
}

impl<'a> fmt::Debug for GaussDBValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GaussDBValue")
            .field("type_oid", &self.type_oid)
            .field("raw_bytes", &self.raw_bytes.map(|b| format!("{} bytes", b.len())))
            .finish()
    }
}

/// Trait for looking up type OIDs in GaussDB
///
/// This trait is used to resolve type names to OIDs at runtime,
/// similar to PostgreSQL's type lookup mechanism.
pub trait TypeOidLookup {
    /// Look up the OID for a given type name
    fn lookup_type_oid(&mut self, type_name: &str) -> Option<u32>;
    
    /// Look up the array OID for a given type name
    fn lookup_array_type_oid(&mut self, type_name: &str) -> Option<u32>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussdb_value_creation() {
        let data = b"test data";
        let value = GaussDBValue::new(Some(data), 25); // text type
        
        assert_eq!(value.type_oid(), 25);
        assert_eq!(value.as_bytes(), Some(data.as_slice()));
        assert!(!value.is_null());
        assert_eq!(value.len(), Some(9));
        assert!(!value.is_empty());
    }

    #[test]
    fn test_gaussdb_value_null() {
        let value = GaussDBValue::new(None, 25);
        
        assert_eq!(value.type_oid(), 25);
        assert_eq!(value.as_bytes(), None);
        assert!(value.is_null());
        assert_eq!(value.len(), None);
        assert!(!value.is_empty());
    }

    #[test]
    fn test_gaussdb_value_empty() {
        let data = b"";
        let value = GaussDBValue::new(Some(data), 25);
        
        assert_eq!(value.type_oid(), 25);
        assert_eq!(value.as_bytes(), Some(data.as_slice()));
        assert!(!value.is_null());
        assert_eq!(value.len(), Some(0));
        assert!(value.is_empty());
    }

    #[test]
    fn test_gaussdb_value_debug() {
        let data = b"test";
        let value = GaussDBValue::new(Some(data), 25);
        let debug_str = format!("{:?}", value);
        
        assert!(debug_str.contains("GaussDBValue"));
        assert!(debug_str.contains("type_oid: 25"));
        assert!(debug_str.contains("4 bytes"));
    }
}
