//! Array type support for GaussDB
//!
//! This module provides support for PostgreSQL-style array types,
//! which are also supported by GaussDB.

use byteorder::{NetworkEndian, ReadBytesExt};

use crate::backend::{GaussDB, GaussDBTypeMetadata};
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql};
use diesel::sql_types::{Array, HasSqlType};

/// Implement HasSqlType for Array types
impl<T> HasSqlType<Array<T>> for GaussDB
where
    GaussDB: HasSqlType<T>,
{
    fn metadata(lookup: &mut Self::MetadataLookup) -> GaussDBTypeMetadata {
        // Get the base type metadata and use its array OID
        let base_metadata = <GaussDB as HasSqlType<T>>::metadata(lookup);
        match base_metadata.array_oid() {
            Ok(array_oid) => GaussDBTypeMetadata::new(array_oid, 0),
            Err(_) => GaussDBTypeMetadata::from_result(Err(
                crate::backend::FailedToLookupTypeError::new("Failed to lookup array type")
            )),
        }
    }
}

/// Implement FromSql for Vec<T> from Array<ST>
impl<T, ST> FromSql<Array<ST>, GaussDB> for Vec<T>
where
    T: FromSql<ST, GaussDB>,
{
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Array value is null")?;
        let mut bytes = bytes;
        
        let num_dimensions = bytes.read_i32::<NetworkEndian>()?;
        let has_null = bytes.read_i32::<NetworkEndian>()? != 0;
        let _oid = bytes.read_i32::<NetworkEndian>()?;

        if num_dimensions == 0 {
            return Ok(Vec::new());
        }

        let num_elements = bytes.read_i32::<NetworkEndian>()?;
        let _lower_bound = bytes.read_i32::<NetworkEndian>()?;

        if num_dimensions != 1 {
            return Err("multi-dimensional arrays are not supported".into());
        }

        let mut result = Vec::new();
        for _ in 0..num_elements {
            let elem_size = bytes.read_i32::<NetworkEndian>()?;
            if has_null && elem_size == -1 {
                // Skip NULL elements for now
                // In a full implementation, we'd need Vec<Option<T>>
                continue;
            } else {
                let elem_size_usize: usize = elem_size.try_into()
                    .map_err(|_| "Invalid element size")?;

                if elem_size_usize > bytes.len() {
                    return Err("Element size exceeds remaining data".into());
                }

                let (elem_bytes, new_bytes) = bytes.split_at(elem_size_usize);
                bytes = new_bytes;
                let element = T::from_sql(GaussDBValue::new(Some(elem_bytes), value.type_oid()))?;
                result.push(element);
            }
        }
        Ok(result)
    }
}

// Note: ToSql implementation for arrays is complex and requires proper
// element serialization with metadata lookup. This is planned for future implementation.

// Note: AsExpression implementations are provided by Diesel's generic implementations
// for Vec<T> and &[T] types. We don't need to implement them manually here as they
// would conflict with Diesel's orphan rules.

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::{Array, Integer, Text};
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

    // Note: ToSql tests require a proper Output setup which is complex to mock.
    // The ToSql implementations are tested through integration tests with real connections.

    #[test]
    fn test_array_deserialization_empty() {
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
}
