//! Network address types support for GaussDB
//!
//! This module provides support for PostgreSQL-compatible network address types,
//! which are also supported by GaussDB.

#[cfg(feature = "ipnetwork")]
extern crate ipnetwork;

#[cfg(feature = "ipnetwork")]
use self::ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use std::io::prelude::*;
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::{Cidr, Inet};

// Platform-specific address family constants
#[cfg(windows)]
const AF_INET: u8 = 2;
#[cfg(target_os = "redox")]
const AF_INET: u8 = 1;

#[allow(clippy::cast_possible_truncation)]
#[cfg(not(any(windows, target_os = "redox")))]
const AF_INET: u8 = {
    #[cfg(feature = "ipnetwork")]
    {
        extern crate libc;
        libc::AF_INET as u8
    }
    #[cfg(not(feature = "ipnetwork"))]
    2
};

const GAUSSDB_AF_INET: u8 = AF_INET;
const GAUSSDB_AF_INET6: u8 = AF_INET + 1;

#[allow(dead_code)]
mod foreign_derives {
    use super::*;

    #[cfg(feature = "ipnetwork")]
    #[derive(AsExpression, FromSqlRow)]
    #[diesel(foreign_derive)]
    #[diesel(sql_type = Inet)]
    #[diesel(sql_type = Cidr)]
    struct IpNetworkProxy(IpNetwork);
}

macro_rules! err {
    () => {
        Err("invalid network address format".into())
    };
    ($msg:expr) => {
        Err(format!("invalid network address format. {}", $msg).into())
    };
}

macro_rules! assert_or_error {
    ($cond:expr) => {
        if !$cond {
            return err!();
        }
    };

    ($cond:expr, $msg:expr) => {
        if !$cond {
            return err!($msg);
        }
    };
}

macro_rules! impl_network_sql {
    ($ty: ty, $net_type: expr) => {
        #[cfg(feature = "ipnetwork")]
        impl FromSql<$ty, GaussDB> for IpNetwork {
            fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
                // Based on PostgreSQL inet.h structure
                let bytes = value.as_bytes().ok_or("Network address value is null")?;
                assert_or_error!(4 <= bytes.len(), "input is too short.");
                let af = bytes[0];
                let prefix = bytes[1];
                let net_type = bytes[2];
                let len = bytes[3];
                assert_or_error!(
                    net_type == $net_type,
                    format!("returned type isn't a {}", stringify!($ty))
                );
                if af == GAUSSDB_AF_INET {
                    assert_or_error!(bytes.len() == 8);
                    assert_or_error!(len == 4, "the data isn't the size of ipv4");
                    let b = &bytes[4..];
                    let addr = Ipv4Addr::new(b[0], b[1], b[2], b[3]);
                    let inet = Ipv4Network::new(addr, prefix)?;
                    Ok(IpNetwork::V4(inet))
                } else if af == GAUSSDB_AF_INET6 {
                    assert_or_error!(bytes.len() == 20);
                    assert_or_error!(len == 16, "the data isn't the size of ipv6");
                    let b = &bytes[4..];
                    let addr = Ipv6Addr::from([
                        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], 
                        b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15],
                    ]);
                    let inet = Ipv6Network::new(addr, prefix)?;
                    Ok(IpNetwork::V6(inet))
                } else {
                    err!()
                }
            }
        }

        #[cfg(feature = "ipnetwork")]
        impl ToSql<$ty, GaussDB> for IpNetwork {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
                match *self {
                    IpNetwork::V4(ref net) => {
                        out.write_all(&[GAUSSDB_AF_INET])?; // family
                        out.write_all(&[net.prefix()])?; // prefix
                        out.write_all(&[$net_type])?; // type (inet vs cidr)
                        out.write_all(&[4])?; // length
                        out.write_all(&net.ip().octets())?; // address
                    }
                    IpNetwork::V6(ref net) => {
                        out.write_all(&[GAUSSDB_AF_INET6])?; // family
                        out.write_all(&[net.prefix()])?; // prefix
                        out.write_all(&[$net_type])?; // type (inet vs cidr)
                        out.write_all(&[16])?; // length
                        out.write_all(&net.ip().octets())?; // address
                    }
                }
                Ok(IsNull::No)
            }
        }
    };
}

// Implement for both Inet and Cidr types
impl_network_sql!(Inet, 0);
impl_network_sql!(Cidr, 1);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::GaussDBValue;

    #[cfg(feature = "ipnetwork")]
    #[test]
    fn test_ipv4_inet_from_sql() {
        // Test IPv4 INET deserialization
        let bytes = [
            GAUSSDB_AF_INET, // family
            24,               // prefix
            0,                // type (inet)
            4,                // length
            192, 168, 1, 1,   // address
        ];
        
        let value = GaussDBValue::new(Some(&bytes), 869); // INET OID
        let result: Result<IpNetwork, _> = FromSql::<Inet, GaussDB>::from_sql(value);
        
        assert!(result.is_ok());
        let network = result.unwrap();
        assert!(network.is_ipv4());
        
        if let IpNetwork::V4(ipv4_net) = network {
            assert_eq!(ipv4_net.ip(), Ipv4Addr::new(192, 168, 1, 1));
            assert_eq!(ipv4_net.prefix(), 24);
        } else {
            panic!("Expected IPv4 network");
        }
    }

    #[cfg(feature = "ipnetwork")]
    #[test]
    fn test_ipv6_inet_from_sql() {
        // Test IPv6 INET deserialization
        let bytes = [
            GAUSSDB_AF_INET6, // family
            64,               // prefix
            0,                // type (inet)
            16,               // length
            // IPv6 address: 2001:db8::1
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        
        let value = GaussDBValue::new(Some(&bytes), 869);
        let result: Result<IpNetwork, _> = FromSql::<Inet, GaussDB>::from_sql(value);
        
        assert!(result.is_ok());
        let network = result.unwrap();
        assert!(network.is_ipv6());
        
        if let IpNetwork::V6(ipv6_net) = network {
            assert_eq!(ipv6_net.prefix(), 64);
        } else {
            panic!("Expected IPv6 network");
        }
    }

    #[cfg(feature = "ipnetwork")]
    #[test]
    fn test_cidr_from_sql() {
        // Test CIDR type deserialization
        let bytes = [
            GAUSSDB_AF_INET, // family
            16,               // prefix
            1,                // type (cidr)
            4,                // length
            10, 0, 0, 0,      // address
        ];
        
        let value = GaussDBValue::new(Some(&bytes), 650); // CIDR OID
        let result: Result<IpNetwork, _> = FromSql::<Cidr, GaussDB>::from_sql(value);
        
        assert!(result.is_ok());
        let network = result.unwrap();
        
        if let IpNetwork::V4(ipv4_net) = network {
            assert_eq!(ipv4_net.ip(), Ipv4Addr::new(10, 0, 0, 0));
            assert_eq!(ipv4_net.prefix(), 16);
        } else {
            panic!("Expected IPv4 network");
        }
    }

    #[cfg(feature = "ipnetwork")]
    #[test]
    fn test_inet_to_sql() {
        use diesel::query_builder::bind_collector::ByteWrapper;
        use diesel::serialize::{Output, ToSql};

        let network = IpNetwork::V4(
            Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap()
        );
        
        let mut buffer = Vec::new();
        let mut output = Output::test(ByteWrapper(&mut buffer));
        ToSql::<Inet, GaussDB>::to_sql(&network, &mut output).unwrap();
        
        let expected = [
            GAUSSDB_AF_INET, // family
            24,               // prefix
            0,                // type (inet)
            4,                // length
            192, 168, 1, 0,   // address
        ];
        
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_invalid_network_address() {
        // Test error handling for invalid data
        let bytes = [1, 2]; // Too short
        let value = GaussDBValue::new(Some(&bytes), 869);
        
        #[cfg(feature = "ipnetwork")]
        {
            let result: Result<IpNetwork, _> = FromSql::<Inet, GaussDB>::from_sql(value);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("input is too short"));
        }
    }

    #[test]
    fn test_null_network_address() {
        let value = GaussDBValue::new(None, 869);
        
        #[cfg(feature = "ipnetwork")]
        {
            let result: Result<IpNetwork, _> = FromSql::<Inet, GaussDB>::from_sql(value);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Network address value is null");
        }
    }

    #[test]
    fn test_address_family_constants() {
        // Test that constants are properly defined
        assert!(GAUSSDB_AF_INET > 0);
        assert_eq!(GAUSSDB_AF_INET6, GAUSSDB_AF_INET + 1);
    }
}
