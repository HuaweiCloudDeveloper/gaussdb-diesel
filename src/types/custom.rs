//! Custom type support for GaussDB
//!
//! This module provides support for user-defined types and enums,
//! which are compatible with PostgreSQL custom types.

use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::SqlType;
use std::io::Write;

/// A trait for custom types that can be represented as strings
pub trait CustomType: Sized {
    /// The name of the custom type in the database
    const TYPE_NAME: &'static str;
    
    /// Convert from a string representation
    fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Convert to a string representation
    fn to_string(&self) -> String;
}

/// A wrapper type for custom enum types
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomEnum<T> {
    value: T,
}

impl<T> CustomEnum<T> {
    /// Create a new custom enum value
    pub fn new(value: T) -> Self {
        Self { value }
    }
    
    /// Get the inner value
    pub fn into_inner(self) -> T {
        self.value
    }
    
    /// Get a reference to the inner value
    pub fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> From<T> for CustomEnum<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// Implement FromSql for custom enum types
impl<T, ST> FromSql<ST, GaussDB> for CustomEnum<T>
where
    T: CustomType,
    ST: SqlType,
{
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Custom enum value is null")?;
        let s = std::str::from_utf8(bytes)?;
        let inner = T::from_str(s)?;
        Ok(CustomEnum::new(inner))
    }
}

/// Implement ToSql for custom enum types
impl<T, ST> ToSql<ST, GaussDB> for CustomEnum<T>
where
    T: CustomType + std::fmt::Debug,
    ST: SqlType,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        let s = self.value.to_string();
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

/// A macro to help define custom enum types
#[macro_export]
macro_rules! define_custom_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl $crate::types::custom::CustomType for $name {
            const TYPE_NAME: &'static str = stringify!($name);

            fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                match s {
                    $(
                        $value => Ok($name::$variant),
                    )*
                    _ => Err(format!("Unknown {} variant: {}", Self::TYPE_NAME, s).into()),
                }
            }

            fn to_string(&self) -> String {
                match self {
                    $(
                        $name::$variant => $value.to_string(),
                    )*
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", <Self as CustomType>::to_string(self))
            }
        }

        impl std::str::FromStr for $name {
            type Err = Box<dyn std::error::Error + Send + Sync>;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <Self as CustomType>::from_str(s)
            }
        }
    };
}



/// Network address type support
pub mod network_support {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    
    /// Custom SQL type for INET
    #[derive(Debug, Clone, Copy, Default, diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    // Note: Custom type attributes are not supported in this version
    pub struct Inet;
    
    impl FromSql<Inet, GaussDB> for IpAddr {
        fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
            let bytes = value.as_bytes().ok_or("INET value is null")?;
            let s = std::str::from_utf8(bytes)?;
            s.parse().map_err(|e| format!("Invalid IP address: {}", e).into())
        }
    }
    
    impl ToSql<Inet, GaussDB> for IpAddr {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
            let s = self.to_string();
            out.write_all(s.as_bytes())?;
            Ok(IsNull::No)
        }
    }
    
    impl FromSql<Inet, GaussDB> for Ipv4Addr {
        fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
            let bytes = value.as_bytes().ok_or("INET value is null")?;
            let s = std::str::from_utf8(bytes)?;
            s.parse().map_err(|e| format!("Invalid IPv4 address: {}", e).into())
        }
    }
    
    impl ToSql<Inet, GaussDB> for Ipv4Addr {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
            let s = self.to_string();
            out.write_all(s.as_bytes())?;
            Ok(IsNull::No)
        }
    }
    
    impl FromSql<Inet, GaussDB> for Ipv6Addr {
        fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
            let bytes = value.as_bytes().ok_or("INET value is null")?;
            let s = std::str::from_utf8(bytes)?;
            s.parse().map_err(|e| format!("Invalid IPv6 address: {}", e).into())
        }
    }
    
    impl ToSql<Inet, GaussDB> for Ipv6Addr {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
            let s = self.to_string();
            out.write_all(s.as_bytes())?;
            Ok(IsNull::No)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::GaussDBValue;

    use diesel::deserialize::FromSql;
    use diesel::sql_types::Text;

    // Define a test enum
    define_custom_enum! {
        pub enum TestStatus {
            Active = "active",
            Inactive = "inactive",
            Pending = "pending",
        }
    }

    #[test]
    fn test_custom_enum_from_str() {
        assert_eq!(TestStatus::from_str("active").unwrap(), TestStatus::Active);
        assert_eq!(TestStatus::from_str("inactive").unwrap(), TestStatus::Inactive);
        assert_eq!(TestStatus::from_str("pending").unwrap(), TestStatus::Pending);
        
        assert!(TestStatus::from_str("unknown").is_err());
    }

    #[test]
    fn test_custom_enum_to_string() {
        assert_eq!(CustomType::to_string(&TestStatus::Active), "active");
        assert_eq!(CustomType::to_string(&TestStatus::Inactive), "inactive");
        assert_eq!(CustomType::to_string(&TestStatus::Pending), "pending");
    }

    #[test]
    fn test_custom_enum_display() {
        assert_eq!(format!("{}", TestStatus::Active), "active");
        assert_eq!(format!("{}", TestStatus::Inactive), "inactive");
        assert_eq!(format!("{}", TestStatus::Pending), "pending");
    }

    #[test]
    fn test_custom_enum_wrapper() {
        let wrapped = CustomEnum::new(TestStatus::Active);
        assert_eq!(wrapped.as_ref(), &TestStatus::Active);
        assert_eq!(wrapped.into_inner(), TestStatus::Active);
        
        let wrapped2 = CustomEnum::from(TestStatus::Pending);
        assert_eq!(wrapped2.as_ref(), &TestStatus::Pending);
    }

    // Note: ToSql tests require a proper Output setup which is complex to mock.
    // The ToSql implementations are tested through integration tests with real connections.

    #[test]
    fn test_custom_enum_sql_deserialization() {
        let value = GaussDBValue::new(Some(b"inactive"), 0);
        let result: Result<CustomEnum<TestStatus>, _> = 
            FromSql::<Text, GaussDB>::from_sql(value);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().into_inner(), TestStatus::Inactive);
    }

    #[test]
    fn test_custom_enum_sql_deserialization_error() {
        let value = GaussDBValue::new(Some(b"unknown"), 0);
        let result: Result<CustomEnum<TestStatus>, _> = 
            FromSql::<Text, GaussDB>::from_sql(value);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown TestStatus variant"));
    }

    #[test]
    fn test_custom_enum_sql_null() {
        let value = GaussDBValue::new(None, 0);
        let result: Result<CustomEnum<TestStatus>, _> = 
            FromSql::<Text, GaussDB>::from_sql(value);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Custom enum value is null");
    }
}
