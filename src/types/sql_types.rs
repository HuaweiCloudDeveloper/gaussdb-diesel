//! GaussDB specific SQL types
//!
//! This module defines SQL types that are specific to GaussDB or extensions
//! to PostgreSQL-compatible types.

use diesel::query_builder::QueryId;
use diesel::sql_types::SqlType;

/// GaussDB specific SQL types
///
/// Note: All types in this module can be accessed through `diesel_gaussdb::sql_types`
pub mod sql_types {
    use super::*;

    /// The [`OID`] SQL type. This is a PostgreSQL/GaussDB specific type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`u32`]
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`u32`]
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`u32`]: https://doc.rust-lang.org/nightly/std/primitive.u32.html
    /// [`OID`]: https://www.postgresql.org/docs/current/datatype-oid.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 26, array_oid = 1028))]
    pub struct Oid;

    /// The ["timestamp with time zone" SQL type][tz], which GaussDB abbreviates
    /// to `timestamptz`.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`chrono::NaiveDateTime`] with `feature = "chrono"`
    /// - [`chrono::DateTime`] with `feature = "chrono"`
    /// - [`time::PrimitiveDateTime`] with `feature = "time"`
    /// - [`time::OffsetDateTime`] with `feature = "time"`
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`chrono::NaiveDateTime`] with `feature = "chrono"`
    /// - [`chrono::DateTime`] with `feature = "chrono"`
    /// - [`time::PrimitiveDateTime`] with `feature = "time"`
    /// - [`time::OffsetDateTime`] with `feature = "time"`
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [tz]: https://www.postgresql.org/docs/current/datatype-datetime.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 1184, array_oid = 1185))]
    pub struct Timestamptz;

    /// The [`Array`] SQL type.
    ///
    /// This wraps another type to represent a SQL array of that type.
    /// Multidimensional arrays are not supported.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`Vec<T>`][Vec] for any `T` which implements `ToSql<ST>`
    /// - [`&[T]`][slice] for any `T` which implements `ToSql<ST>`
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`Vec<T>`][Vec] for any `T` which implements `ToSql<ST>`
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [Vec]: std::vec::Vec
    /// [slice]: https://doc.rust-lang.org/nightly/std/primitive.slice.html
    /// [`Array`]: https://www.postgresql.org/docs/current/arrays.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    pub struct Array<ST: 'static>(pub ST);

    /// The [`UUID`] SQL type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`uuid::Uuid`] with `feature = "uuid"`
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`uuid::Uuid`] with `feature = "uuid"`
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`UUID`]: https://www.postgresql.org/docs/current/datatype-uuid.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 2950, array_oid = 2951))]
    pub struct Uuid;

    /// The [`JSON`] SQL type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`serde_json::Value`] with `feature = "serde_json"`
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`serde_json::Value`] with `feature = "serde_json"`
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`JSON`]: https://www.postgresql.org/docs/current/datatype-json.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 114, array_oid = 199))]
    pub struct Json;

    /// The [`JSONB`] SQL type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`serde_json::Value`] with `feature = "serde_json"`
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`serde_json::Value`] with `feature = "serde_json"`
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`JSONB`]: https://www.postgresql.org/docs/current/datatype-json.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 3802, array_oid = 3807))]
    pub struct Jsonb;

    /// The [`BYTEA`] SQL type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`Vec<u8>`]
    /// - [`&[u8]`]
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`Vec<u8>`]
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`Vec<u8>`]: std::vec::Vec
    /// [`&[u8]`]: https://doc.rust-lang.org/std/primitive.slice.html
    /// [`BYTEA`]: https://www.postgresql.org/docs/current/datatype-binary.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 17, array_oid = 1001))]
    pub struct Bytea;

    /// The [`INET`] SQL type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`std::net::IpAddr`]
    /// - [`std::net::Ipv4Addr`]
    /// - [`std::net::Ipv6Addr`]
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`std::net::IpAddr`]
    /// - [`std::net::Ipv4Addr`]
    /// - [`std::net::Ipv6Addr`]
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`INET`]: https://www.postgresql.org/docs/current/datatype-net-types.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 869, array_oid = 1041))]
    pub struct Inet;

    /// The [`CIDR`] SQL type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`ipnetwork::IpNetwork`] with `feature = "ipnet-address"`
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`ipnetwork::IpNetwork`] with `feature = "ipnet-address"`
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`CIDR`]: https://www.postgresql.org/docs/current/datatype-net-types.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 650, array_oid = 651))]
    pub struct Cidr;

    /// The [`MACADDR`] SQL type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - `[u8; 6]`
    ///
    /// ### [`FromSql`] impls
    ///
    /// - `[u8; 6]`
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`MACADDR`]: https://www.postgresql.org/docs/current/datatype-net-types.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 829, array_oid = 1040))]
    pub struct MacAddr;

    /// The [`MACADDR8`] SQL type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - `[u8; 8]`
    ///
    /// ### [`FromSql`] impls
    ///
    /// - `[u8; 8]`
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`MACADDR8`]: https://www.postgresql.org/docs/current/datatype-net-types.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 774, array_oid = 775))]
    pub struct MacAddr8;

    /// The [`MONEY`] SQL type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - [`i64`] (representing cents)
    ///
    /// ### [`FromSql`] impls
    ///
    /// - [`i64`] (representing cents)
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`MONEY`]: https://www.postgresql.org/docs/current/datatype-money.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 790, array_oid = 791))]
    pub struct Money;

    // Multirange types

    /// The [`INT4MULTIRANGE`] SQL type.
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 4451, array_oid = 6150))]
    pub struct Int4multirange;

    /// The [`INT8MULTIRANGE`] SQL type.
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 4536, array_oid = 6157))]
    pub struct Int8multirange;

    /// The [`NUMMULTIRANGE`] SQL type.
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 4532, array_oid = 6151))]
    pub struct Nummultirange;

    /// The [`TSMULTIRANGE`] SQL type.
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 4533, array_oid = 6152))]
    pub struct Tsmultirange;

    /// The [`TSTZMULTIRANGE`] SQL type.
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 4534, array_oid = 6153))]
    pub struct Tstzmultirange;

    /// The [`DATEMULTIRANGE`] SQL type.
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    #[diesel(postgres_type(oid = 4535, array_oid = 6155))]
    pub struct Datemultirange;



    /// The [`Multirange`] SQL type.
    ///
    /// This wraps another type to represent a SQL multirange of that type.
    ///
    /// ### [`ToSql`] impls
    ///
    /// - `Vec<(Bound<T>, Bound<T>)>`
    /// - `&[(Bound<T>, Bound<T>)]`
    ///
    /// ### [`FromSql`] impls
    ///
    /// - `Vec<(Bound<T>, Bound<T>)>`
    ///
    /// [`ToSql`]: diesel::serialize::ToSql
    /// [`FromSql`]: diesel::deserialize::FromSql
    /// [`Multirange`]: https://www.postgresql.org/docs/current/rangetypes.html
    #[derive(Debug, Clone, Copy, Default, QueryId, SqlType)]
    pub struct Multirange<ST: 'static>(ST);
}

// Re-export for convenience
pub use sql_types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_types_exist() {
        // Just test that the types can be instantiated
        let _oid = Oid;
        let _timestamptz = Timestamptz;
        let _uuid = Uuid;
        let _json = Json;
        let _jsonb = Jsonb;
        let _inet = Inet;
        let _cidr = Cidr;
        let _macaddr = MacAddr;
        let _money = Money;
    }

    #[test]
    fn test_array_type() {
        use diesel::sql_types::Integer;
        let _int_array: Array<Integer> = Array(Integer);
    }

    #[test]
    fn test_type_definitions() {
        // Test that our types can be instantiated and have the expected properties
        let _oid = Oid;
        let _timestamptz = Timestamptz;
        let _uuid = Uuid;
        let _json = Json;
        let _jsonb = Jsonb;
        let _inet = Inet;
        let _cidr = Cidr;
        let _macaddr = MacAddr;
        let _money = Money;

        // Test that they implement the required traits
        use std::fmt::Debug;
        let _: &dyn Debug = &_oid;
        let _: &dyn Debug = &_timestamptz;
        let _: &dyn Debug = &_uuid;
    }

    #[test]
    fn test_array_with_different_types() {
        use diesel::sql_types::{Integer, Text, Bool};

        let _int_array: Array<Integer> = Array(Integer);
        let _text_array: Array<Text> = Array(Text);
        let _bool_array: Array<Bool> = Array(Bool);

        // Test nested arrays
        let _nested_int_array: Array<Array<Integer>> = Array(Array(Integer));
    }
}
