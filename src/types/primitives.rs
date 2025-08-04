//! Primitive type support for GaussDB
//!
//! This module provides support for basic PostgreSQL-compatible types
//! that are supported by GaussDB, following the same patterns as PostgreSQL.

use crate::backend::GaussDB;
use crate::value::GaussDBValue;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::*;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

// Helper function for size errors (following PostgreSQL pattern)
#[cold]
#[inline(never)]
fn emit_size_error<T>(msg: &str) -> deserialize::Result<T> {
    Err(msg.into())
}

// OID type implementation
impl FromSql<Oid, GaussDB> for u32 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("OID value is null")?;
        let mut cursor = std::io::Cursor::new(bytes);
        cursor.read_u32::<NetworkEndian>().map_err(Into::into)
    }
}

impl ToSql<Oid, GaussDB> for u32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_u32::<NetworkEndian>(*self)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

// SmallInt (i16) implementation with proper error handling
impl FromSql<SmallInt, GaussDB> for i16 {
    #[inline(always)]
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("SmallInt value is null")?;
        if bytes.len() < 2 {
            return emit_size_error(
                "Received less than 2 bytes while decoding an i16. \
                Was an expression of a different type accidentally marked as SmallInt?"
            );
        }
        if bytes.len() > 2 {
            return emit_size_error(
                "Received more than 2 bytes while decoding an i16. \
                Was an Integer expression accidentally marked as SmallInt?"
            );
        }
        let mut cursor = std::io::Cursor::new(bytes);
        cursor.read_i16::<NetworkEndian>()
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

impl ToSql<SmallInt, GaussDB> for i16 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_i16::<NetworkEndian>(*self)
            .map(|_| IsNull::No)
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

// Integer (i32) implementation with proper error handling
impl FromSql<Integer, GaussDB> for i32 {
    #[inline(always)]
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Integer value is null")?;
        if bytes.len() < 4 {
            return emit_size_error(
                "Received less than 4 bytes while decoding an i32. \
                Was a SmallInt expression accidentally marked as Integer?"
            );
        }
        if bytes.len() > 4 {
            return emit_size_error(
                "Received more than 4 bytes while decoding an i32. \
                Was a BigInt expression accidentally marked as Integer?"
            );
        }
        let mut cursor = std::io::Cursor::new(bytes);
        cursor.read_i32::<NetworkEndian>()
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

impl ToSql<Integer, GaussDB> for i32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_i32::<NetworkEndian>(*self)
            .map(|_| IsNull::No)
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

// BigInt (i64) implementation with proper error handling
impl FromSql<BigInt, GaussDB> for i64 {
    #[inline(always)]
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("BigInt value is null")?;
        if bytes.len() < 8 {
            return emit_size_error(
                "Received less than 8 bytes while decoding an i64. \
                Was an Integer expression accidentally marked as BigInt?"
            );
        }
        if bytes.len() > 8 {
            return emit_size_error(
                "Received more than 8 bytes while decoding an i64. \
                Was an expression of a different type accidentally marked as BigInt?"
            );
        }
        let mut cursor = std::io::Cursor::new(bytes);
        cursor.read_i64::<NetworkEndian>()
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

impl ToSql<BigInt, GaussDB> for i64 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_i64::<NetworkEndian>(*self)
            .map(|_| IsNull::No)
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

// Float (f32) implementation
impl FromSql<Float, GaussDB> for f32 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Float value is null")?;
        if bytes.len() < 4 {
            return emit_size_error(
                "Received less than 4 bytes while decoding an f32. \
                Got {} bytes"
            );
        }
        if bytes.len() > 4 {
            return emit_size_error(
                "Received more than 4 bytes while decoding an f32. \
                Was a double accidentally marked as float? Got {} bytes"
            );
        }
        let mut cursor = std::io::Cursor::new(bytes);
        cursor.read_f32::<NetworkEndian>()
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

impl ToSql<Float, GaussDB> for f32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_f32::<NetworkEndian>(*self)
            .map(|_| IsNull::No)
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

// Double (f64) implementation
impl FromSql<Double, GaussDB> for f64 {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Double value is null")?;
        if bytes.len() < 8 {
            return emit_size_error(
                "Received less than 8 bytes while decoding an f64. \
                Was a float accidentally marked as double? Got {} bytes"
            );
        }
        if bytes.len() > 8 {
            return emit_size_error(
                "Received more than 8 bytes while decoding an f64. \
                Was a numeric accidentally marked as double? Got {} bytes"
            );
        }
        let mut cursor = std::io::Cursor::new(bytes);
        cursor.read_f64::<NetworkEndian>()
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

impl ToSql<Double, GaussDB> for f64 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, GaussDB>) -> serialize::Result {
        out.write_f64::<NetworkEndian>(*self)
            .map(|_| IsNull::No)
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

// Boolean implementation
impl FromSql<Bool, GaussDB> for bool {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Bool value is null")?;
        Ok(bytes[0] != 0)
    }
}

// Text implementation (following PostgreSQL pattern)
impl FromSql<Text, GaussDB> for *const str {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        use std::str;
        let bytes = value.as_bytes().ok_or("Text value is null")?;
        let string = str::from_utf8(bytes)?;
        Ok(string as *const _)
    }
}

// Binary data implementation
impl FromSql<Binary, GaussDB> for Vec<u8> {
    fn from_sql(value: GaussDBValue<'_>) -> deserialize::Result<Self> {
        let bytes = value.as_bytes().ok_or("Binary value is null")?;
        Ok(bytes.to_vec())
    }
}


