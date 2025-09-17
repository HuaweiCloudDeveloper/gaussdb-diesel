//! Serialization support for GaussDB
//!
//! This module provides serialization functionality for GaussDB types.

mod write_tuple;


/// Re-export common serialization types
pub use diesel::serialize::{IsNull, Result};
pub use self::write_tuple::WriteTuple;
