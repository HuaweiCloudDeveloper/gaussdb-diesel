//! The GaussDB backend implementation
//!
//! This module provides the core backend implementation for GaussDB,
//! which is PostgreSQL-compatible but with some GaussDB-specific features.

use diesel::backend::*;
use diesel::query_builder::bind_collector::RawBytesBindCollector;
use diesel::sql_types::{HasSqlType, TypeMetadata};

use crate::query_builder::GaussDBQueryBuilder;
use crate::value::GaussDBValue;

/// The GaussDB backend
///
/// This backend is PostgreSQL-compatible and supports GaussDB-specific
/// authentication methods and data types.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct GaussDB;

/// Inner type metadata for GaussDB types
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct InnerGaussDBTypeMetadata {
    pub(crate) oid: u32,
    pub(crate) array_oid: u32,
}

impl From<(u32, u32)> for InnerGaussDBTypeMetadata {
    fn from((oid, array_oid): (u32, u32)) -> Self {
        Self { oid, array_oid }
    }
}

/// Error type for failed type lookups
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FailedToLookupTypeError {
    type_name: String,
    schema: Option<String>,
}

impl FailedToLookupTypeError {
    /// Create a new lookup error with a simple message
    pub fn new(message: &str) -> Self {
        Self {
            type_name: message.to_string(),
            schema: None,
        }
    }

    /// Create a new internal lookup error
    pub(crate) fn new_internal(cache_key: crate::metadata_lookup::GaussDBMetadataCacheKey<'static>) -> Self {
        Self {
            type_name: cache_key.type_name.into_owned(),
            schema: cache_key.schema.map(|s| s.into_owned()),
        }
    }

    /// Get the type name that failed to be looked up
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Get the schema name if specified
    pub fn schema(&self) -> Option<&str> {
        self.schema.as_deref()
    }
}

impl std::fmt::Display for FailedToLookupTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.schema {
            Some(schema) => write!(f, "Failed to lookup type {}.{}", schema, self.type_name),
            None => write!(f, "Failed to lookup type {}", self.type_name),
        }
    }
}

impl std::error::Error for FailedToLookupTypeError {}



/// Type metadata for GaussDB types
///
/// Since GaussDB is PostgreSQL-compatible, we use similar metadata structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub struct GaussDBTypeMetadata(Result<InnerGaussDBTypeMetadata, FailedToLookupTypeError>);

impl GaussDBTypeMetadata {
    /// Create new type metadata based on known constant OIDs
    pub fn new(type_oid: u32, array_oid: u32) -> Self {
        Self(Ok(InnerGaussDBTypeMetadata {
            oid: type_oid,
            array_oid,
        }))
    }

    /// Create a new instance based on a result
    pub fn from_result(r: Result<(u32, u32), FailedToLookupTypeError>) -> Self {
        Self(r.map(|(oid, array_oid)| InnerGaussDBTypeMetadata { oid, array_oid }))
    }

    /// Get the OID of this type
    pub fn oid(&self) -> Result<u32, impl std::error::Error + Send + Sync + use<>> {
        self.0.as_ref().map(|i| i.oid).map_err(Clone::clone)
    }

    /// Get the array OID of this type
    pub fn array_oid(&self) -> Result<u32, impl std::error::Error + Send + Sync + use<>> {
        self.0.as_ref().map(|i| i.array_oid).map_err(Clone::clone)
    }
}

impl From<(u32, u32)> for GaussDBTypeMetadata {
    fn from((oid, array_oid): (u32, u32)) -> Self {
        Self::new(oid, array_oid)
    }
}

impl Backend for GaussDB {
    type QueryBuilder = GaussDBQueryBuilder;
    type RawValue<'a> = GaussDBValue<'a>;
    type BindCollector<'a> = RawBytesBindCollector<GaussDB>;
}

impl TypeMetadata for GaussDB {
    type TypeMetadata = GaussDBTypeMetadata;
    type MetadataLookup = dyn GaussDBMetadataLookup;
}

/// Trait for looking up type metadata in GaussDB
pub trait GaussDBMetadataLookup {
    /// Look up metadata for a type
    fn lookup_type(&mut self, type_name: &str, schema: Option<&str>) -> GaussDBTypeMetadata;

    /// Cast to Any for downcasting
    fn as_any<'a>(&mut self) -> &mut (dyn std::any::Any + 'a)
    where
        Self: 'a;
}

impl SqlDialect for GaussDB {
    type ReturningClause = GaussDBReturningClause;
    type OnConflictClause = GaussDBOnConflictClause;
    type InsertWithDefaultKeyword = sql_dialect::default_keyword_for_insert::IsoSqlDefaultKeyword;
    type BatchInsertSupport = sql_dialect::batch_insert_support::PostgresLikeBatchInsertSupport;
    type ConcatClause = sql_dialect::concat_clause::ConcatWithPipesClause;
    type DefaultValueClauseForInsert = sql_dialect::default_value_clause::AnsiDefaultValueClause;
    type EmptyFromClauseSyntax = sql_dialect::from_clause_syntax::AnsiSqlFromClauseSyntax;
    type SelectStatementSyntax = sql_dialect::select_statement_syntax::AnsiSqlSelectStatement;
    type ExistsSyntax = sql_dialect::exists_syntax::AnsiSqlExistsSyntax;
    type ArrayComparison = sql_dialect::array_comparison::AnsiSqlArrayComparison;
    type AliasSyntax = sql_dialect::alias_syntax::AsAliasSyntax;
    // Note: These types are not available in diesel 2.2.12
    // type WindowFrameClauseGroupSupport = sql_dialect::window_frame_clause_group_support::IsoGroupWindowFrameUnit;
    // type WindowFrameExclusionSupport = sql_dialect::window_frame_exclusion_support::FrameExclusionSupport;
    // type AggregateFunctionExpressions = sql_dialect::aggregate_function_expressions::PostgresLikeAggregateFunctionExpressions;
    // type BuiltInWindowFunctionRequireOrder = sql_dialect::built_in_window_function_require_order::NoOrderRequired;
}

// Basic SQL type support - we'll implement these similar to PostgreSQL
impl HasSqlType<diesel::sql_types::SmallInt> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(21, 1005) // smallint, _int2
    }
}

impl HasSqlType<diesel::sql_types::Integer> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(23, 1007) // int4, _int4
    }
}

impl HasSqlType<diesel::sql_types::BigInt> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(20, 1016) // int8, _int8
    }
}

impl HasSqlType<diesel::sql_types::Numeric> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(1700, 1231) // numeric, _numeric
    }
}

impl HasSqlType<diesel::sql_types::Json> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(114, 199) // json, _json
    }
}

impl HasSqlType<diesel::sql_types::Jsonb> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(3802, 3807) // jsonb, _jsonb
    }
}

impl HasSqlType<diesel::sql_types::Float> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(700, 1021) // float4, _float4
    }
}

impl HasSqlType<diesel::sql_types::Double> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(701, 1022) // float8, _float8
    }
}

impl HasSqlType<diesel::sql_types::Text> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(25, 1009) // text, _text
    }
}

impl HasSqlType<diesel::sql_types::Binary> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(17, 1001) // bytea, _bytea
    }
}

impl HasSqlType<diesel::sql_types::Date> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(1082, 1182) // date, _date
    }
}

impl HasSqlType<diesel::sql_types::Time> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(1083, 1183) // time, _time
    }
}

impl HasSqlType<diesel::sql_types::Timestamp> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(1114, 1115) // timestamp, _timestamp
    }
}

impl HasSqlType<diesel::sql_types::Bool> for GaussDB {
    fn metadata(_: &mut (dyn GaussDBMetadataLookup + 'static)) -> GaussDBTypeMetadata {
        GaussDBTypeMetadata::new(16, 1000) // bool, _bool
    }
}

impl DieselReserveSpecialization for GaussDB {}
impl TrustedBackend for GaussDB {}

// GaussDB-specific types for SQL dialect
/// GaussDB-specific ON CONFLICT clause support
#[derive(Debug, Copy, Clone)]
pub struct GaussDBOnConflictClause;

impl sql_dialect::on_conflict_clause::SupportsOnConflictClause for GaussDBOnConflictClause {}

/// GaussDB-specific RETURNING clause support
#[derive(Debug, Copy, Clone)]
pub struct GaussDBReturningClause;

impl sql_dialect::returning_clause::SupportsReturningClause for GaussDBReturningClause {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussdb_backend_creation() {
        let backend = GaussDB::default();
        assert_eq!(backend, GaussDB);
    }

    #[test]
    fn test_type_metadata() {
        let metadata = GaussDBTypeMetadata::new(23, 1007);
        assert_eq!(metadata.oid().unwrap(), 23);
        assert_eq!(metadata.array_oid().unwrap(), 1007);
    }

    #[test]
    fn test_type_metadata_from_tuple() {
        let metadata: GaussDBTypeMetadata = (23, 1007).into();
        assert_eq!(metadata.oid().unwrap(), 23);
        assert_eq!(metadata.array_oid().unwrap(), 1007);
    }
}
