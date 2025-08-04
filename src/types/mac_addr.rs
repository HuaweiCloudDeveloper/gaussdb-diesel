//! Support for PostgreSQL MAC address types in GaussDB
//!
//! This module provides support for MAC address (macaddr) types,
//! which are 6-byte hardware addresses used in Ethernet networks.

use std::io::prelude::*;

use diesel::deserialize::{self, FromSql};
use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::MacAddr;

/// FromSql implementation for MAC address (6 bytes)
#[cfg(feature = "gaussdb")]
impl FromSql<MacAddr, GaussDB> for [u8; 6] {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        match value.as_bytes() {
            Some(bytes) if bytes.len() == 6 => {
                let mut result = [0u8; 6];
                result.copy_from_slice(bytes);
                Ok(result)
            }
            _ => Err("invalid MAC address format: input isn't 6 bytes.".into())
        }
    }
}

/// ToSql implementation for MAC address (6 bytes)
#[cfg(feature = "gaussdb")]
impl ToSql<MacAddr, GaussDB> for [u8; 6] {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&self[..])
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

/// A wrapper type for MAC addresses that provides convenient methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Create a new MAC address from 6 bytes
    pub fn new(bytes: [u8; 6]) -> Self {
        MacAddress(bytes)
    }

    /// Get the raw bytes of the MAC address
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }

    /// Convert to a standard string representation (XX:XX:XX:XX:XX:XX)
    pub fn to_string(&self) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }

    /// Parse a MAC address from string format
    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 6 {
            return Err("MAC address must have 6 parts separated by colons");
        }

        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            bytes[i] = u8::from_str_radix(part, 16)
                .map_err(|_| "Invalid hexadecimal digit in MAC address")?;
        }

        Ok(MacAddress(bytes))
    }
}

impl From<[u8; 6]> for MacAddress {
    fn from(bytes: [u8; 6]) -> Self {
        MacAddress(bytes)
    }
}

impl From<MacAddress> for [u8; 6] {
    fn from(mac: MacAddress) -> Self {
        mac.0
    }
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::str::FromStr for MacAddress {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        MacAddress::from_string(s)
    }
}

#[cfg(feature = "gaussdb")]
impl FromSql<MacAddr, GaussDB> for MacAddress {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes: [u8; 6] = <[u8; 6] as FromSql<MacAddr, GaussDB>>::from_sql(value)?;
        Ok(MacAddress(bytes))
    }
}

#[cfg(feature = "gaussdb")]
impl ToSql<MacAddr, GaussDB> for MacAddress {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        ToSql::<MacAddr, GaussDB>::to_sql(&self.0, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macaddr_basic() {
        let input_address = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16];
        let mac_address = MacAddress::new(input_address);
        let bytes: [u8; 6] = mac_address.into();
        assert_eq!(input_address, bytes);
    }

    #[test]
    fn test_mac_address_wrapper() {
        let bytes = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16];
        let mac = MacAddress::new(bytes);
        
        assert_eq!(mac.as_bytes(), &bytes);
        assert_eq!(mac.to_string(), "52:54:00:fb:c6:16");
    }

    #[test]
    fn test_mac_address_from_string() {
        let mac_str = "52:54:00:fb:c6:16";
        let mac = MacAddress::from_string(mac_str).unwrap();
        let expected = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16];
        
        assert_eq!(mac.as_bytes(), &expected);
        assert_eq!(mac.to_string(), mac_str);
    }

    #[test]
    fn test_mac_address_invalid_format() {
        assert!(MacAddress::from_string("invalid").is_err());
        assert!(MacAddress::from_string("52:54:00:fb:c6").is_err()); // Too few parts
        assert!(MacAddress::from_string("52:54:00:fb:c6:16:17").is_err()); // Too many parts
        assert!(MacAddress::from_string("52:54:00:fb:c6:gg").is_err()); // Invalid hex
    }

    #[test]
    fn test_mac_address_display() {
        let mac = MacAddress::new([0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16]);
        assert_eq!(format!("{}", mac), "52:54:00:fb:c6:16");
    }

    #[test]
    fn test_mac_address_from_str_trait() {
        use std::str::FromStr;
        let mac = MacAddress::from_str("52:54:00:fb:c6:16").unwrap();
        let expected = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16];
        assert_eq!(mac.as_bytes(), &expected);
    }

    #[test]
    fn test_mac_address_conversions() {
        let bytes = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16];
        let mac = MacAddress::from(bytes);
        let converted_bytes: [u8; 6] = mac.into();
        assert_eq!(bytes, converted_bytes);
    }

    #[cfg(feature = "gaussdb")]
    #[test]
    fn test_mac_address_wrapper_basic() {
        let input_mac = MacAddress::new([0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16]);
        let bytes: [u8; 6] = input_mac.into();
        let output_mac = MacAddress::from(bytes);
        assert_eq!(input_mac, output_mac);
    }
}
