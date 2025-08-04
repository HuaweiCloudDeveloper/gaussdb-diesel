//! Metadata lookup for GaussDB connections
//!
//! This module provides type metadata lookup functionality for GaussDB,
//! adapted from PostgreSQL's metadata lookup system.

use crate::backend::{FailedToLookupTypeError, InnerGaussDBTypeMetadata, GaussDB, GaussDBTypeMetadata};
use diesel::connection::{DefaultLoadingMode, LoadConnection};
use diesel::prelude::*;
use diesel::result::QueryResult;
use diesel::define_sql_function;

use std::borrow::Cow;
use std::collections::HashMap;

/// Determines the OID of types at runtime for GaussDB
///
/// Custom implementations of `Connection<Backend = GaussDB>` should not implement this trait directly.
/// Instead `GetGaussDBMetadataCache` should be implemented, afterwards the generic implementation will provide
/// the necessary functions to perform the type lookup.
#[cfg(feature = "gaussdb")]
pub trait GaussDBMetadataLookup {
    /// Determine the type metadata for the given `type_name`
    ///
    /// This function should only be used for user defined types, or types which
    /// come from an extension. This function may perform a SQL query to look
    /// up the type. For built-in types, a static OID should be preferred.
    fn lookup_type(&mut self, type_name: &str, schema: Option<&str>) -> GaussDBTypeMetadata;

    /// Convert this lookup instance to a `std::any::Any` pointer
    ///
    /// Implementing this method is required to support `#[derive(MultiConnection)]`
    fn as_any<'a>(&mut self) -> &mut (dyn std::any::Any + 'a)
    where
        Self: 'a,
    {
        unimplemented!()
    }
}

impl<T> crate::backend::GaussDBMetadataLookup for T
where
    T: Connection<Backend = GaussDB> + GetGaussDBMetadataCache + LoadConnection<DefaultLoadingMode>,
{
    fn lookup_type(&mut self, type_name: &str, schema: Option<&str>) -> GaussDBTypeMetadata {
        let cache_key = GaussDBMetadataCacheKey {
            schema: schema.map(Cow::Borrowed),
            type_name: Cow::Borrowed(type_name),
        };

        {
            let metadata_cache = self.get_metadata_cache();

            if let Some(metadata) = metadata_cache.lookup_type(&cache_key) {
                return metadata;
            }
        }

        let r = lookup_type(&cache_key, self);

        match r {
            Ok(type_metadata) => {
                self.get_metadata_cache()
                    .store_type(cache_key, type_metadata);
                GaussDBTypeMetadata::from_result(Ok((type_metadata.oid, type_metadata.array_oid)))
            }
            Err(_e) => GaussDBTypeMetadata::from_result(Err(FailedToLookupTypeError::new_internal(
                cache_key.into_owned(),
            ))),
        }
    }

    fn as_any<'a>(&mut self) -> &mut (dyn std::any::Any + 'a)
    where
        Self: 'a,
    {
        self
    }
}

/// Gets the `GaussDBMetadataCache` for a `Connection<Backend=GaussDB>`
/// so that the lookup of user defined types, or types which come from an extension can be cached.
///
/// Implementing this trait for a `Connection<Backend=GaussDB>` will cause `GaussDBMetadataLookup` to be auto implemented.
pub trait GetGaussDBMetadataCache {
    /// Get the `GaussDBMetadataCache`
    fn get_metadata_cache(&mut self) -> &mut GaussDBMetadataCache;
}

fn lookup_type<T: Connection<Backend = GaussDB> + LoadConnection<DefaultLoadingMode>>(
    cache_key: &GaussDBMetadataCacheKey<'_>,
    _conn: &mut T,
) -> QueryResult<InnerGaussDBTypeMetadata> {
    // TODO: Implement actual type lookup from GaussDB system tables
    // For now, return a default metadata for common types
    let metadata = match cache_key.type_name.as_ref() {
        "text" => InnerGaussDBTypeMetadata { oid: 25, array_oid: 1009 },
        "int4" => InnerGaussDBTypeMetadata { oid: 23, array_oid: 1007 },
        "int8" => InnerGaussDBTypeMetadata { oid: 20, array_oid: 1016 },
        "bool" => InnerGaussDBTypeMetadata { oid: 16, array_oid: 1000 },
        "bytea" => InnerGaussDBTypeMetadata { oid: 17, array_oid: 1001 },
        _ => {
            // Return an error for unknown types
            return Err(diesel::result::Error::NotFound);
        }
    };

    Ok(metadata)
}

/// The key used to lookup cached type oid's inside of
/// a [GaussDBMetadataCache].
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct GaussDBMetadataCacheKey<'a> {
    pub(crate) schema: Option<Cow<'a, str>>,
    pub(crate) type_name: Cow<'a, str>,
}

impl<'a> GaussDBMetadataCacheKey<'a> {
    /// Construct a new cache key from an optional schema name and
    /// a type name
    pub fn new(schema: Option<Cow<'a, str>>, type_name: Cow<'a, str>) -> Self {
        Self { schema, type_name }
    }

    /// Convert the possibly borrowed version of this metadata cache key
    /// into a lifetime independent owned version
    pub fn into_owned(self) -> GaussDBMetadataCacheKey<'static> {
        let GaussDBMetadataCacheKey { schema, type_name } = self;
        GaussDBMetadataCacheKey {
            schema: schema.map(|s| Cow::Owned(s.into_owned())),
            type_name: Cow::Owned(type_name.into_owned()),
        }
    }
}

/// Cache for the [OIDs] of custom GaussDB types
///
/// [OIDs]: https://www.postgresql.org/docs/current/static/datatype-oid.html
#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct GaussDBMetadataCache {
    cache: HashMap<GaussDBMetadataCacheKey<'static>, InnerGaussDBTypeMetadata>,
}

impl GaussDBMetadataCache {
    /// Construct a new `GaussDBMetadataCache`
    pub fn new() -> Self {
        Default::default()
    }

    /// Lookup the OID of a custom type
    pub fn lookup_type(&self, type_name: &GaussDBMetadataCacheKey<'_>) -> Option<GaussDBTypeMetadata> {
        let metadata = *self.cache.get(type_name)?;
        Some(GaussDBTypeMetadata::from_result(Ok((metadata.oid, metadata.array_oid))))
    }

    /// Store the OID of a custom type
    pub fn store_type(
        &mut self,
        type_name: GaussDBMetadataCacheKey<'_>,
        type_metadata: impl Into<InnerGaussDBTypeMetadata>,
    ) {
        self.cache
            .insert(type_name.into_owned(), type_metadata.into());
    }

    /// Clear all cached metadata
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get the number of cached types
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

// GaussDB system tables (PostgreSQL-compatible)
diesel::table! {
    gaussdb_type (oid) {
        oid -> diesel::sql_types::Oid,
        typname -> diesel::sql_types::Text,
        typarray -> diesel::sql_types::Oid,
        typnamespace -> diesel::sql_types::Oid,
    }
}

diesel::table! {
    gaussdb_namespace (oid) {
        oid -> diesel::sql_types::Oid,
        nspname -> diesel::sql_types::Text,
    }
}

diesel::joinable!(gaussdb_type -> gaussdb_namespace(typnamespace));
diesel::allow_tables_to_appear_in_same_query!(gaussdb_type, gaussdb_namespace);

// GaussDB-specific functions


define_sql_function!(fn gaussdb_my_temp_schema() -> diesel::sql_types::Oid);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_creation() {
        let key = GaussDBMetadataCacheKey::new(
            Some(Cow::Borrowed("public")),
            Cow::Borrowed("custom_type"),
        );
        
        assert_eq!(key.schema.as_deref(), Some("public"));
        assert_eq!(key.type_name.as_ref(), "custom_type");
    }

    #[test]
    fn test_cache_key_into_owned() {
        let key = GaussDBMetadataCacheKey::new(
            Some(Cow::Borrowed("public")),
            Cow::Borrowed("custom_type"),
        );
        
        let owned_key = key.into_owned();
        assert_eq!(owned_key.schema.as_deref(), Some("public"));
        assert_eq!(owned_key.type_name.as_ref(), "custom_type");
    }

    #[test]
    fn test_metadata_cache() {
        let mut cache = GaussDBMetadataCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        let key = GaussDBMetadataCacheKey::new(
            None,
            Cow::Borrowed("test_type"),
        );
        
        let metadata = InnerGaussDBTypeMetadata { oid: 12345, array_oid: 12346 };
        cache.store_type(key.clone(), metadata);
        
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);
        
        let retrieved = cache.lookup_type(&key);
        assert!(retrieved.is_some());
        
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = GaussDBMetadataCacheKey::new(
            Some(Cow::Borrowed("public")),
            Cow::Borrowed("type1"),
        );
        
        let key2 = GaussDBMetadataCacheKey::new(
            Some(Cow::Borrowed("public")),
            Cow::Borrowed("type1"),
        );
        
        let key3 = GaussDBMetadataCacheKey::new(
            Some(Cow::Borrowed("private")),
            Cow::Borrowed("type1"),
        );
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
