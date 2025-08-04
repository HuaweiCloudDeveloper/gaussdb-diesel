//! Support for PostgreSQL MAC address 8 types in GaussDB
//!
//! This module provides support for MAC address 8 (macaddr8) types,
//! which are 8-byte extended hardware addresses used in modern Ethernet networks.

use std::io::prelude::*;

use diesel::deserialize::{self, FromSql};
use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::serialize::{self, IsNull, Output, ToSql};
use crate::types::sql_types::MacAddr8;



/// FromSql implementation for MAC address 8 (8 bytes)
#[cfg(feature = "gaussdb")]
impl FromSql<MacAddr8, GaussDB> for [u8; 8] {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        match value.as_bytes() {
            Some(bytes) if bytes.len() == 8 => {
                let mut result = [0u8; 8];
                result.copy_from_slice(bytes);
                Ok(result)
            }
            _ => Err("invalid MAC address 8 format: input isn't 8 bytes.".into())
        }
    }
}

/// ToSql implementation for MAC address 8 (8 bytes)
#[cfg(feature = "gaussdb")]
impl ToSql<MacAddr8, GaussDB> for [u8; 8] {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_all(&self[..])
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

/// A wrapper type for MAC address 8 that provides convenient methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress8([u8; 8]);

impl MacAddress8 {
    /// Create a new MAC address 8 from 8 bytes
    pub fn new(bytes: [u8; 8]) -> Self {
        MacAddress8(bytes)
    }

    /// Get the raw bytes of the MAC address 8
    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    /// Convert to a standard string representation (XX:XX:XX:XX:XX:XX:XX:XX)
    pub fn to_string(&self) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], 
            self.0[4], self.0[5], self.0[6], self.0[7]
        )
    }

    /// Parse a MAC address 8 from string format
    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 8 {
            return Err("MAC address 8 must have 8 parts separated by colons");
        }

        let mut bytes = [0u8; 8];
        for (i, part) in parts.iter().enumerate() {
            bytes[i] = u8::from_str_radix(part, 16)
                .map_err(|_| "Invalid hexadecimal digit in MAC address 8")?;
        }

        Ok(MacAddress8(bytes))
    }

    /// Convert a 6-byte MAC address to 8-byte format by inserting FF:FE
    pub fn from_mac6(mac6: [u8; 6]) -> Self {
        let mut bytes = [0u8; 8];
        bytes[0] = mac6[0];
        bytes[1] = mac6[1];
        bytes[2] = mac6[2];
        bytes[3] = 0xFF;  // Standard insertion
        bytes[4] = 0xFE;  // Standard insertion
        bytes[5] = mac6[3];
        bytes[6] = mac6[4];
        bytes[7] = mac6[5];
        MacAddress8(bytes)
    }

    /// Try to convert to 6-byte MAC address if it follows the standard format
    pub fn to_mac6(&self) -> Option<[u8; 6]> {
        if self.0[3] == 0xFF && self.0[4] == 0xFE {
            Some([
                self.0[0], self.0[1], self.0[2],
                self.0[5], self.0[6], self.0[7]
            ])
        } else {
            None
        }
    }
}

impl From<[u8; 8]> for MacAddress8 {
    fn from(bytes: [u8; 8]) -> Self {
        MacAddress8(bytes)
    }
}

impl From<MacAddress8> for [u8; 8] {
    fn from(mac: MacAddress8) -> Self {
        mac.0
    }
}

impl std::fmt::Display for MacAddress8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::str::FromStr for MacAddress8 {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        MacAddress8::from_string(s)
    }
}

#[cfg(feature = "gaussdb")]
impl FromSql<MacAddr8, GaussDB> for MacAddress8 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes: [u8; 8] = <[u8; 8] as FromSql<MacAddr8, GaussDB>>::from_sql(value)?;
        Ok(MacAddress8(bytes))
    }
}

#[cfg(feature = "gaussdb")]
impl ToSql<MacAddr8, GaussDB> for MacAddress8 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        ToSql::<MacAddr8, GaussDB>::to_sql(&self.0, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_macaddr8_basic() {
        let input_address = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16, 0x17, 0xFF];
        let mac_address = MacAddress8::new(input_address);
        let bytes: [u8; 8] = mac_address.into();
        assert_eq!(input_address, bytes);
    }

    #[test]
    fn test_mac_address8_wrapper() {
        let bytes = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16, 0x17, 0xFF];
        let mac = MacAddress8::new(bytes);
        
        assert_eq!(mac.as_bytes(), &bytes);
        assert_eq!(mac.to_string(), "52:54:00:fb:c6:16:17:ff");
    }

    #[test]
    fn test_mac_address8_from_string() {
        let mac_str = "52:54:00:fb:c6:16:17:ff";
        let mac = MacAddress8::from_string(mac_str).unwrap();
        let expected = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16, 0x17, 0xFF];
        
        assert_eq!(mac.as_bytes(), &expected);
        assert_eq!(mac.to_string(), mac_str);
    }

    #[test]
    fn test_mac_address8_from_mac6() {
        let mac6 = [0x52, 0x54, 0x00, 0xc6, 0x16, 0x17];
        let mac8 = MacAddress8::from_mac6(mac6);
        let expected = [0x52, 0x54, 0x00, 0xFF, 0xFE, 0xc6, 0x16, 0x17];
        
        assert_eq!(mac8.as_bytes(), &expected);
        assert_eq!(mac8.to_mac6(), Some(mac6));
    }

    #[test]
    fn test_mac_address8_to_mac6_invalid() {
        let mac8 = MacAddress8::new([0x52, 0x54, 0x00, 0x12, 0x34, 0xc6, 0x16, 0x17]);
        assert_eq!(mac8.to_mac6(), None); // Not standard FF:FE format
    }

    #[test]
    fn test_mac_address8_invalid_format() {
        assert!(MacAddress8::from_string("invalid").is_err());
        assert!(MacAddress8::from_string("52:54:00:fb:c6:16:17").is_err()); // Too few parts
        assert!(MacAddress8::from_string("52:54:00:fb:c6:16:17:ff:aa").is_err()); // Too many parts
        assert!(MacAddress8::from_string("52:54:00:fb:c6:16:17:gg").is_err()); // Invalid hex
    }

    #[test]
    fn test_mac_address8_display() {
        let mac = MacAddress8::new([0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16, 0x17, 0xFF]);
        assert_eq!(format!("{}", mac), "52:54:00:fb:c6:16:17:ff");
    }

    #[test]
    fn test_mac_address8_from_str_trait() {
        use std::str::FromStr;
        let mac = MacAddress8::from_str("52:54:00:fb:c6:16:17:ff").unwrap();
        let expected = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16, 0x17, 0xFF];
        assert_eq!(mac.as_bytes(), &expected);
    }

    #[test]
    fn test_mac_address8_conversions() {
        let bytes = [0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16, 0x17, 0xFF];
        let mac = MacAddress8::from(bytes);
        let converted_bytes: [u8; 8] = mac.into();
        assert_eq!(bytes, converted_bytes);
    }

    #[cfg(feature = "gaussdb")]
    #[test]
    fn test_mac_address8_wrapper_basic() {
        let input_mac = MacAddress8::new([0x52, 0x54, 0x00, 0xfb, 0xc6, 0x16, 0x17, 0xFF]);
        let bytes: [u8; 8] = input_mac.into();
        let output_mac = MacAddress8::from(bytes);
        assert_eq!(input_mac, output_mac);
    }
}
