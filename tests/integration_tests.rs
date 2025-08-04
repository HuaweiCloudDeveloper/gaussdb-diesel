//! Integration tests for diesel-gaussdb
//!
//! These tests verify that the basic functionality works correctly.

use diesel_gaussdb::{GaussDB, GaussDBQueryBuilder, backend::GaussDBTypeMetadata};
use diesel::query_builder::QueryBuilder;

#[test]
fn test_query_builder_basic_functionality() {
    let mut builder = GaussDBQueryBuilder::new();
    
    builder.push_sql("SELECT * FROM ");
    builder.push_identifier("test_table").unwrap();
    builder.push_sql(" WHERE ");
    builder.push_identifier("id").unwrap();
    builder.push_sql(" = ");
    builder.push_bind_param();
    
    let sql = builder.finish();
    assert_eq!(sql, "SELECT * FROM \"test_table\" WHERE \"id\" = $1");
}

#[test]
fn test_query_builder_multiple_params() {
    let mut builder = GaussDBQueryBuilder::new();
    
    builder.push_sql("UPDATE ");
    builder.push_identifier("users").unwrap();
    builder.push_sql(" SET ");
    builder.push_identifier("name").unwrap();
    builder.push_sql(" = ");
    builder.push_bind_param();
    builder.push_sql(", ");
    builder.push_identifier("email").unwrap();
    builder.push_sql(" = ");
    builder.push_bind_param();
    builder.push_sql(" WHERE ");
    builder.push_identifier("id").unwrap();
    builder.push_sql(" = ");
    builder.push_bind_param();
    
    let sql = builder.finish();
    assert_eq!(sql, "UPDATE \"users\" SET \"name\" = $1, \"email\" = $2 WHERE \"id\" = $3");
}

#[test]
fn test_identifier_escaping() {
    let mut builder = GaussDBQueryBuilder::new();
    
    builder.push_sql("SELECT ");
    builder.push_identifier("table\"with\"quotes").unwrap();
    
    let sql = builder.finish();
    assert_eq!(sql, "SELECT \"table\"\"with\"\"quotes\"");
}

#[test]
fn test_type_metadata() {
    // Test that our type metadata works correctly
    let metadata = GaussDBTypeMetadata::new(23, 1007);
    assert_eq!(metadata.oid().unwrap(), 23);
    assert_eq!(metadata.array_oid().unwrap(), 1007);

    // Test conversion from tuple
    let metadata2: GaussDBTypeMetadata = (25, 1009).into();
    assert_eq!(metadata2.oid().unwrap(), 25);
    assert_eq!(metadata2.array_oid().unwrap(), 1009);
}

#[test]
fn test_backend_type_support() {
    // This test verifies that our backend supports the basic SQL types
    // Note: We can't actually call the metadata functions without a lookup instance,
    // but we can verify the types compile correctly
    
    // These should compile without errors
    fn _test_type_support() {
        use diesel::sql_types::*;
        
        // Verify that GaussDB implements HasSqlType for basic types
        fn _has_sql_type<T>() where GaussDB: HasSqlType<T> {}
        
        _has_sql_type::<SmallInt>();
        _has_sql_type::<Integer>();
        _has_sql_type::<BigInt>();
        _has_sql_type::<Float>();
        _has_sql_type::<Double>();
        _has_sql_type::<Text>();
        _has_sql_type::<Binary>();
        _has_sql_type::<Date>();
        _has_sql_type::<Time>();
        _has_sql_type::<Timestamp>();
        _has_sql_type::<Bool>();
    }
    
    _test_type_support();
}

#[test]
fn test_backend_creation() {
    let backend1 = GaussDB::default();
    let backend2 = GaussDB;
    
    assert_eq!(backend1, backend2);
    assert_eq!(format!("{:?}", backend1), "GaussDB");
}

#[test]
fn test_complex_query_building() {
    let mut builder = GaussDBQueryBuilder::new();
    
    // Build a complex query with JOINs
    builder.push_sql("SELECT ");
    builder.push_identifier("u").unwrap();
    builder.push_sql(".");
    builder.push_identifier("name").unwrap();
    builder.push_sql(", ");
    builder.push_identifier("p").unwrap();
    builder.push_sql(".");
    builder.push_identifier("title").unwrap();
    builder.push_sql(" FROM ");
    builder.push_identifier("users").unwrap();
    builder.push_sql(" ");
    builder.push_identifier("u").unwrap();
    builder.push_sql(" JOIN ");
    builder.push_identifier("posts").unwrap();
    builder.push_sql(" ");
    builder.push_identifier("p").unwrap();
    builder.push_sql(" ON ");
    builder.push_identifier("u").unwrap();
    builder.push_sql(".");
    builder.push_identifier("id").unwrap();
    builder.push_sql(" = ");
    builder.push_identifier("p").unwrap();
    builder.push_sql(".");
    builder.push_identifier("user_id").unwrap();
    builder.push_sql(" WHERE ");
    builder.push_identifier("u").unwrap();
    builder.push_sql(".");
    builder.push_identifier("active").unwrap();
    builder.push_sql(" = ");
    builder.push_bind_param();
    
    let sql = builder.finish();
    let expected = "SELECT \"u\".\"name\", \"p\".\"title\" FROM \"users\" \"u\" JOIN \"posts\" \"p\" ON \"u\".\"id\" = \"p\".\"user_id\" WHERE \"u\".\"active\" = $1";
    assert_eq!(sql, expected);
}
