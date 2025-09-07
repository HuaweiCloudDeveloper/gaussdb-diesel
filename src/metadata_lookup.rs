//! Metadata lookup for GaussDB connections
//!
//! This module provides type metadata lookup functionality for GaussDB,
//! adapted from PostgreSQL's metadata lookup system.

use crate::backend::{FailedToLookupTypeError, InnerGaussDBTypeMetadata, GaussDB, GaussDBTypeMetadata};
use diesel::connection::{DefaultLoadingMode, LoadConnection};
use diesel::prelude::*;
use diesel::result::QueryResult;
use diesel::sql_types::{Text, Integer, Bool};
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

// 获取当前临时模式的 OID
define_sql_function!(fn gaussdb_my_temp_schema() -> diesel::sql_types::Oid);

// 获取数据库版本信息
define_sql_function!(fn version() -> diesel::sql_types::Text);

// 获取当前数据库名称
define_sql_function!(fn current_database() -> diesel::sql_types::Text);

// 获取当前用户名
define_sql_function!(fn current_user() -> diesel::sql_types::Text);

// 获取会话用户名
define_sql_function!(fn session_user() -> diesel::sql_types::Text);

// 获取当前模式名称
define_sql_function!(fn current_schema() -> diesel::sql_types::Text);

/// 表存在性查询结果
#[derive(Debug, diesel::QueryableByName)]
struct TableExistsResult {
    #[diesel(sql_type = Bool)]
    exists: bool,
}

/// 检查表是否存在的辅助函数
///
/// 这个函数可以用来动态检查表的存在性，对于迁移和动态查询很有用
pub fn table_exists(
    conn: &mut crate::connection::GaussDBConnection,
    table_name: &str,
    schema_name: Option<&str>,
) -> diesel::result::QueryResult<bool> {
    use diesel::prelude::*;

    let schema = schema_name.unwrap_or("public");

    // 查询 information_schema.tables 来检查表是否存在
    let query = diesel::sql_query(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = $1 AND table_name = $2
        ) as exists"
    )
    .bind::<Text, _>(schema)
    .bind::<Text, _>(table_name);

    // 执行查询并返回结果
    let result: Vec<TableExistsResult> = query.load(conn)?;
    Ok(result.first().map(|r| r.exists).unwrap_or(false))
}

/// 获取表的列信息
///
/// 返回指定表的所有列的详细信息，包括列名、数据类型、是否可空等
pub fn get_table_columns(
    conn: &mut crate::connection::GaussDBConnection,
    table_name: &str,
    schema_name: Option<&str>,
) -> diesel::result::QueryResult<Vec<ColumnInfo>> {
    use diesel::prelude::*;

    let schema = schema_name.unwrap_or("public");

    let query = diesel::sql_query(
        "SELECT
            column_name,
            data_type,
            is_nullable::boolean,
            ordinal_position,
            column_default
        FROM information_schema.columns
        WHERE table_schema = $1 AND table_name = $2
        ORDER BY ordinal_position"
    )
    .bind::<Text, _>(schema)
    .bind::<Text, _>(table_name);

    query.load(conn)
}

/// 列信息结构体
///
/// 包含数据库表列的详细信息，用于动态查询和元数据分析
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct ColumnInfo {
    /// 列名
    #[diesel(sql_type = Text)]
    pub column_name: String,
    /// 数据类型
    #[diesel(sql_type = Text)]
    pub data_type: String,
    /// 是否可为空
    #[diesel(sql_type = Bool)]
    pub is_nullable: bool,
    /// 列在表中的位置
    #[diesel(sql_type = Integer)]
    pub ordinal_position: i32,
    /// 列的默认值
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    pub column_default: Option<String>,
}

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

    #[test]
    fn test_column_info_structure() {
        // 测试 ColumnInfo 结构体的创建和访问
        let column = ColumnInfo {
            column_name: "id".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            ordinal_position: 1,
            column_default: Some("nextval('seq')".to_string()),
        };

        assert_eq!(column.column_name, "id");
        assert_eq!(column.data_type, "integer");
        assert!(!column.is_nullable);
        assert_eq!(column.ordinal_position, 1);
        assert!(column.column_default.is_some());

        println!("✅ ColumnInfo 结构体测试通过");
    }
}
