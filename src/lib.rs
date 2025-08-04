//! # Diesel-GaussDB
//!
//! A GaussDB backend for the Diesel ORM framework.
//!
//! This crate provides GaussDB database support for Diesel, enabling Rust applications
//! to work with GaussDB and OpenGauss databases using Diesel's type-safe query builder.
//!
//! ## Features
//!
//! - **GaussDB Compatibility**: Full support for GaussDB and OpenGauss databases
//! - **PostgreSQL Protocol**: Leverages PostgreSQL compatibility for maximum feature support
//! - **Type Safety**: Compile-time verified queries with Diesel's type system
//! - **Authentication**: Support for GaussDB's SHA256 and MD5_SHA256 authentication methods
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use diesel::prelude::*;
//! use diesel_gaussdb::GaussDBConnection;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let database_url = "gaussdb://username:password@localhost:5432/database_name";
//!     let mut connection = GaussDBConnection::establish(&database_url)?;
//!
//!     // Use Diesel as normal
//!     // ...
//!
//!     Ok(())
//! }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod backend;
pub mod connection;
pub mod metadata_lookup;
pub mod query_builder;
pub mod types;
pub mod gaussdb_extensions;
pub mod expression;
pub mod serialize;
pub mod transaction;
pub mod value;

// Re-export core types
pub use backend::GaussDB;
pub use connection::GaussDBConnection;
pub use query_builder::GaussDBQueryBuilder;

/// Data types for GaussDB
pub mod data_types {
    pub use crate::types::money::{GaussDBMoney, Cents};
    pub use crate::types::mac_addr::MacAddress;
    pub use crate::types::mac_addr_8::MacAddress8;
}

// Re-export commonly used types from diesel
pub use diesel::prelude::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::backend::GaussDB;
    pub use crate::connection::GaussDBConnection;
    pub use crate::query_builder::GaussDBQueryBuilder;
    pub use diesel::prelude::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_advanced_features() {
        use crate::query_builder::{on_constraint, ConflictTarget};
        use crate::query_builder::distinct_on::DistinctOnClause;
        use crate::query_builder::limit_offset::LimitOffsetClause;

        // Test ON CONSTRAINT functionality
        let constraint = on_constraint("users_email_unique");
        assert_eq!(constraint.constraint_name(), "users_email_unique");

        let target = ConflictTarget::new(constraint);
        assert_eq!(target.0.constraint_name(), "users_email_unique");

        // Test DISTINCT ON clause
        let distinct_clause = DistinctOnClause::new("user_id");
        // Test that the clause was created successfully
        assert!(format!("{:?}", distinct_clause).contains("user_id"));

        // Test LIMIT OFFSET clause
        let limit_offset = LimitOffsetClause::limit_and_offset(10i64, 20i64);
        // Test that the clause was created successfully
        assert!(format!("{:?}", limit_offset).contains("10"));
        assert!(format!("{:?}", limit_offset).contains("20"));
    }

    #[test]
    fn test_sql_generation_advanced() {
        use crate::query_builder::GaussDBQueryBuilder;
        use diesel::query_builder::QueryBuilder;

        let mut builder = GaussDBQueryBuilder::new();

        // Test complex query building
        builder.push_sql("SELECT DISTINCT ON (");
        builder.push_identifier("user_id").unwrap();
        builder.push_sql(") ");
        builder.push_identifier("user_id").unwrap();
        builder.push_sql(", ");
        builder.push_identifier("name").unwrap();
        builder.push_sql(" FROM ");
        builder.push_identifier("users").unwrap();
        builder.push_sql(" WHERE ");
        builder.push_identifier("active").unwrap();
        builder.push_sql(" = ");
        builder.push_bind_param();
        builder.push_sql(" ORDER BY ");
        builder.push_identifier("user_id").unwrap();
        builder.push_sql(", ");
        builder.push_identifier("created_at").unwrap();
        builder.push_sql(" DESC LIMIT ");
        builder.push_bind_param();
        builder.push_sql(" OFFSET ");
        builder.push_bind_param();

        let expected = "SELECT DISTINCT ON (\"user_id\") \"user_id\", \"name\" FROM \"users\" WHERE \"active\" = $1 ORDER BY \"user_id\", \"created_at\" DESC LIMIT $2 OFFSET $3";
        assert_eq!(builder.sql(), expected);
        assert_eq!(builder.bind_idx(), 3);
    }

    #[test]
    fn test_type_metadata() {
        use crate::metadata_lookup::{GaussDBMetadataCache, GaussDBMetadataCacheKey};
        use std::borrow::Cow;

        let mut cache = GaussDBMetadataCache::new();
        let key = GaussDBMetadataCacheKey::new(None, Cow::Borrowed("text"));

        // Test that we can store and retrieve metadata
        assert!(cache.lookup_type(&key).is_none());

        // Store some metadata
        cache.store_type(key.clone(), (25, 1009));

        // Verify we can retrieve it
        let metadata = cache.lookup_type(&key);
        assert!(metadata.is_some());
    }

    #[test]
    fn test_copy_operations() {
        use crate::query_builder::copy::{CopyFormat, CopyFromQuery, CopyToQuery};

        // Test COPY FROM functionality
        let copy_from: CopyFromQuery<(), ()> = CopyFromQuery::new(())
            .with_format(CopyFormat::Csv)
            .with_delimiter(',')
            .with_null("NULL".to_string())
            .with_quote('"')
            .with_freeze(true);

        // Test that the query was configured correctly
        assert!(format!("{:?}", copy_from).contains("Csv"));

        // Test COPY TO functionality
        let copy_to = CopyToQuery::<()>::new()
            .with_format(CopyFormat::Binary)
            .with_header(true);

        // Test that the query was configured correctly
        assert!(format!("{:?}", copy_to).contains("Binary"));

        // Test format enum variants exist
        let _text = CopyFormat::Text;
        let _csv = CopyFormat::Csv;
        let _binary = CopyFormat::Binary;
    }

    #[test]
    fn test_extended_type_system() {
        use crate::types::sql_types::*;

        // Test that all extended types can be instantiated
        let _oid = Oid;
        let _timestamptz = Timestamptz;
        let _uuid = Uuid;
        let _json = Json;
        let _jsonb = Jsonb;
        let _bytea = Bytea;
        let _inet = Inet;
        let _cidr = Cidr;
        let _macaddr = MacAddr;
        let _macaddr8 = MacAddr8;
        let _money = Money;

        // Test array types
        use diesel::sql_types::{Integer, Text, Bool};
        let _int_array: Array<Integer> = Array(Integer);
        let _text_array: Array<Text> = Array(Text);
        let _bool_array: Array<Bool> = Array(Bool);

        // Test nested arrays
        let _nested_int_array: Array<Array<Integer>> = Array(Array(Integer));

        // Test that types implement Debug
        use std::fmt::Debug;
        let _: &dyn Debug = &_oid;
        let _: &dyn Debug = &_timestamptz;
        let _: &dyn Debug = &_uuid;
        let _: &dyn Debug = &_json;
        let _: &dyn Debug = &_jsonb;
        let _: &dyn Debug = &_bytea;
        let _: &dyn Debug = &_inet;
        let _: &dyn Debug = &_cidr;
        let _: &dyn Debug = &_macaddr;
        let _: &dyn Debug = &_macaddr8;
        let _: &dyn Debug = &_money;
    }

    #[test]
    fn test_expression_system_foundation() {
        use crate::expression::*;

        // Test that expression modules are accessible
        array::array_placeholder();
        array_comparison::any_placeholder();
        array_comparison::all_placeholder();
        expression_methods::expression_methods_placeholder();
        functions::functions_placeholder();
        operators::operators_placeholder();
        dsl::dsl_placeholder();

        // This test verifies that the expression system foundation is in place
        // for future array expression implementations
    }

    #[test]
    fn test_date_time_expressions() {
        use crate::expression::dsl::*;
        use crate::types::sql_types::Timestamptz;
        use diesel::sql_types::{Date, Time, Timestamp, Double};

        // Test that date/time functions are accessible and have correct types
        let now_expr = now;
        let current_timestamp_expr = current_timestamp;
        let current_date_expr = current_date;
        let current_time_expr = current_time;

        // Test type assertions
        fn assert_timestamptz_expr<T: diesel::expression::Expression<SqlType = Timestamptz>>(_: T) {}
        fn assert_date_expr<T: diesel::expression::Expression<SqlType = Date>>(_: T) {}
        fn assert_time_expr<T: diesel::expression::Expression<SqlType = Time>>(_: T) {}
        fn assert_double_expr<T: diesel::expression::Expression<SqlType = Double>>(_: T) {}

        assert_timestamptz_expr(now_expr);
        assert_timestamptz_expr(current_timestamp_expr);
        assert_date_expr(current_date_expr);
        assert_time_expr(current_time_expr);

        // Test extract and date_part functions
        let timestamp_expr = diesel::dsl::sql::<Timestamp>("'2023-12-25'");
        let extract_expr = extract("YEAR", timestamp_expr);
        let date_part_expr = date_part("month", diesel::dsl::sql::<Timestamp>("'2023-12-25'"));

        assert_double_expr(extract_expr);
        assert_double_expr(date_part_expr);

        // This test verifies that the date/time expression system is working
        // and all functions have the correct SQL types
    }
}
