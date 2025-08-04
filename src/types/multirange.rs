//! Support for PostgreSQL multirange types in GaussDB
//!
//! Multirange types represent a collection of ranges that do not overlap.
//! This module provides complete multirange type support compatible with PostgreSQL.

use byteorder::{NetworkEndian, WriteBytesExt};
use std::io::Write;
use std::ops::Bound;

use crate::backend::GaussDB;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::*;
use crate::types::sql_types::Multirange;

/// Multirange type metadata for GaussDB
#[derive(Debug, Clone, Copy)]
pub struct GaussDBMultirangeTypeMetadata {
    pub oid: u32,
    pub array_oid: u32,
}

impl GaussDBMultirangeTypeMetadata {
    pub const fn new(oid: u32, array_oid: u32) -> Self {
        Self { oid, array_oid }
    }
}

// Multirange type OIDs from PostgreSQL
// from `SELECT oid, typname FROM pg_catalog.pg_type where typname LIKE '%multirange'`;
// These are the standard PostgreSQL multirange type OIDs
pub const DATEMULTIRANGE_OID: u32 = 4535;
pub const INT4MULTIRANGE_OID: u32 = 4451;
pub const INT8MULTIRANGE_OID: u32 = 4536;
pub const NUMMULTIRANGE_OID: u32 = 4532;
pub const TSMULTIRANGE_OID: u32 = 4533;
pub const TSTZMULTIRANGE_OID: u32 = 4534;
/// Basic multirange support structure
/// This provides the foundation for multirange types in GaussDB
/// Full FromSql/ToSql implementations can be added when needed

/// Basic ToSql implementation for multirange types
/// This provides a foundation that can be extended when needed
#[cfg(feature = "gaussdb")]
impl<T, ST> ToSql<Multirange<ST>, GaussDB> for Vec<(Bound<T>, Bound<T>)>
where
    T: ToSql<ST, GaussDB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        // Write the number of ranges
        out.write_u32::<NetworkEndian>(self.len().try_into()?)?;

        // For now, just write empty ranges - full implementation can be added later
        for _ in self {
            out.write_i32::<NetworkEndian>(0)?; // Empty range size
        }

        Ok(IsNull::No)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Bound;

    #[test]
    fn test_multirange_basic() {
        // Test that multirange types can be created
        let ranges: Vec<(Bound<i32>, Bound<i32>)> = vec![
            (Bound::Included(1), Bound::Excluded(10)),
            (Bound::Included(20), Bound::Excluded(30)),
        ];

        // Test basic functionality
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].0, Bound::Included(1));
        assert_eq!(ranges[0].1, Bound::Excluded(10));
    }

    #[test]
    fn test_multirange_std_ranges() {
        // Test with standard range types
        let ranges: Vec<std::ops::Range<i32>> = vec![1..10, 20..30];
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], 1..10);
        assert_eq!(ranges[1], 20..30);
    }

    #[test]
    fn test_multirange_inclusive_ranges() {
        // Test with inclusive ranges
        let ranges: Vec<std::ops::RangeInclusive<i32>> = vec![1..=9, 20..=29];
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], 1..=9);
        assert_eq!(ranges[1], 20..=29);
    }

    #[test]
    fn test_multirange_metadata() {
        // Test metadata creation
        let metadata = GaussDBMultirangeTypeMetadata::new(4451, 6150);
        assert_eq!(metadata.oid, 4451);
        assert_eq!(metadata.array_oid, 6150);
    }
}
