//! Tests for complex types (arrays and ranges) in GaussDB
//!
//! This module tests the complex type functionality, focusing on
//! array types which are more stable than range types.

use diesel_gaussdb::backend::GaussDB;
use diesel_gaussdb::value::GaussDBValue;
use diesel::deserialize::FromSql;
use diesel::sql_types::*;
use byteorder::{NetworkEndian, WriteBytesExt};

#[test]
fn test_array_has_sql_type() {
    // This is a compile-time test to ensure the trait is implemented
    fn test_array_type<T: 'static>()
    where
        GaussDB: HasSqlType<Array<T>>,
        GaussDB: HasSqlType<T>,
    {
        // This function should compile if HasSqlType is properly implemented
    }
    
    test_array_type::<Integer>();
    test_array_type::<Text>();
}

#[test]
fn test_empty_array_deserialization() {
    // Create empty array bytes (PostgreSQL format)
    let mut bytes = Vec::new();
    bytes.write_i32::<NetworkEndian>(0).unwrap(); // num_dimensions = 0
    bytes.write_i32::<NetworkEndian>(0).unwrap(); // flags
    bytes.write_i32::<NetworkEndian>(23).unwrap(); // element type OID (int4)
    
    let value = GaussDBValue::new(Some(&bytes), 1007); // int4 array OID
    let result: Result<Vec<i32>, _> = <Vec<i32> as FromSql<Array<Integer>, GaussDB>>::from_sql(value);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Vec::<i32>::new());
}

#[test]
fn test_array_deserialization_error_multidimensional() {
    // Create multi-dimensional array bytes
    let mut bytes = Vec::new();
    bytes.write_i32::<NetworkEndian>(2).unwrap(); // num_dimensions = 2
    bytes.write_i32::<NetworkEndian>(0).unwrap(); // flags
    bytes.write_i32::<NetworkEndian>(23).unwrap(); // element type OID
    bytes.write_i32::<NetworkEndian>(4).unwrap(); // num_elements
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // lower_bound
    
    let value = GaussDBValue::new(Some(&bytes), 1007);
    let result: Result<Vec<i32>, _> = <Vec<i32> as FromSql<Array<Integer>, GaussDB>>::from_sql(value);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("multi-dimensional"));
}

#[test]
fn test_array_deserialization_null_value() {
    // Test null array value
    let value = GaussDBValue::new(None, 1007);
    let result: Result<Vec<i32>, _> = <Vec<i32> as FromSql<Array<Integer>, GaussDB>>::from_sql(value);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Array value is null"));
}

#[test]
fn test_simple_array_with_elements() {
    // Create a simple 1D array with 3 elements: [1, 2, 3]
    let mut bytes = Vec::new();
    
    // Array header
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // num_dimensions = 1
    bytes.write_i32::<NetworkEndian>(0).unwrap(); // flags (no nulls)
    bytes.write_i32::<NetworkEndian>(23).unwrap(); // element type OID (int4)
    bytes.write_i32::<NetworkEndian>(3).unwrap(); // num_elements = 3
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // lower_bound = 1
    
    // Element 1: value 1
    bytes.write_i32::<NetworkEndian>(4).unwrap(); // element size = 4 bytes
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // value = 1
    
    // Element 2: value 2
    bytes.write_i32::<NetworkEndian>(4).unwrap(); // element size = 4 bytes
    bytes.write_i32::<NetworkEndian>(2).unwrap(); // value = 2
    
    // Element 3: value 3
    bytes.write_i32::<NetworkEndian>(4).unwrap(); // element size = 4 bytes
    bytes.write_i32::<NetworkEndian>(3).unwrap(); // value = 3
    
    let value = GaussDBValue::new(Some(&bytes), 1007); // int4 array OID
    let result: Result<Vec<i32>, _> = <Vec<i32> as FromSql<Array<Integer>, GaussDB>>::from_sql(value);

    assert!(result.is_ok());
    let array = result.unwrap();
    assert_eq!(array.len(), 3);
    assert_eq!(array, vec![1, 2, 3]);
}

#[test]
fn test_array_with_null_elements() {
    // Create a 1D array with null elements: [1, NULL, 3]
    let mut bytes = Vec::new();
    
    // Array header
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // num_dimensions = 1
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // flags (has nulls)
    bytes.write_i32::<NetworkEndian>(23).unwrap(); // element type OID (int4)
    bytes.write_i32::<NetworkEndian>(3).unwrap(); // num_elements = 3
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // lower_bound = 1
    
    // Element 1: value 1
    bytes.write_i32::<NetworkEndian>(4).unwrap(); // element size = 4 bytes
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // value = 1
    
    // Element 2: NULL
    bytes.write_i32::<NetworkEndian>(-1).unwrap(); // element size = -1 (NULL)
    
    // Element 3: value 3
    bytes.write_i32::<NetworkEndian>(4).unwrap(); // element size = 4 bytes
    bytes.write_i32::<NetworkEndian>(3).unwrap(); // value = 3
    
    let value = GaussDBValue::new(Some(&bytes), 1007); // int4 array OID
    let result: Result<Vec<i32>, _> = <Vec<i32> as FromSql<Array<Integer>, GaussDB>>::from_sql(value);

    // Note: Our current implementation skips NULL elements
    // In a full implementation, we'd need Option<T> support
    assert!(result.is_ok());
    let array = result.unwrap();
    assert_eq!(array.len(), 2); // Only non-null elements
    assert_eq!(array, vec![1, 3]);
}

// Note: ToSql tests are disabled since we don't implement ToSql for arrays yet

#[test]
fn test_array_type_metadata() {
    // Test that we can create type metadata for arrays
    // This is mainly a compile-time test
    use diesel::sql_types::{Array, Integer, Text};
    
    fn test_metadata<T: 'static>()
    where
        GaussDB: HasSqlType<Array<T>>,
        GaussDB: HasSqlType<T>,
    {
        // This should compile if metadata is properly implemented
    }
    
    test_metadata::<Integer>();
    test_metadata::<Text>();
}

#[test]
fn test_array_error_handling() {
    // Test various error conditions
    
    // Test with malformed array data (too short)
    let short_bytes = vec![0, 0]; // Only 2 bytes
    let value = GaussDBValue::new(Some(&short_bytes), 1007);
    let result: Result<Vec<i32>, _> = <Vec<i32> as FromSql<Array<Integer>, GaussDB>>::from_sql(value);
    assert!(result.is_err());
    
    // Test with inconsistent element size
    let mut bad_bytes = Vec::new();
    bad_bytes.write_i32::<NetworkEndian>(1).unwrap(); // num_dimensions = 1
    bad_bytes.write_i32::<NetworkEndian>(0).unwrap(); // flags
    bad_bytes.write_i32::<NetworkEndian>(23).unwrap(); // element type OID
    bad_bytes.write_i32::<NetworkEndian>(1).unwrap(); // num_elements = 1
    bad_bytes.write_i32::<NetworkEndian>(1).unwrap(); // lower_bound = 1
    bad_bytes.write_i32::<NetworkEndian>(100).unwrap(); // element size = 100 (too big)
    // But we don't have 100 bytes of data
    
    let value = GaussDBValue::new(Some(&bad_bytes), 1007);
    let result: Result<Vec<i32>, _> = <Vec<i32> as FromSql<Array<Integer>, GaussDB>>::from_sql(value);
    assert!(result.is_err());
}

#[test]
fn test_array_boundary_conditions() {
    // Test with zero elements but valid header
    let mut bytes = Vec::new();
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // num_dimensions = 1
    bytes.write_i32::<NetworkEndian>(0).unwrap(); // flags
    bytes.write_i32::<NetworkEndian>(23).unwrap(); // element type OID
    bytes.write_i32::<NetworkEndian>(0).unwrap(); // num_elements = 0
    bytes.write_i32::<NetworkEndian>(1).unwrap(); // lower_bound = 1
    
    let value = GaussDBValue::new(Some(&bytes), 1007);
    let result: Result<Vec<i32>, _> = <Vec<i32> as FromSql<Array<Integer>, GaussDB>>::from_sql(value);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Vec::<i32>::new());
}
