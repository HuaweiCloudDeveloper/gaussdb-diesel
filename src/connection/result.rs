//! Result handling for GaussDB connections
//!
//! This module provides result processing for GaussDB queries,
//! adapted from PostgreSQL's result handling.

use crate::connection::row::GaussDBRow;
use diesel::result::{DatabaseErrorInformation, DatabaseErrorKind, Error, QueryResult};
use std::fmt;

/// A query result from GaussDB
///
/// This represents the result of executing a query against a GaussDB database.
/// It provides access to the rows returned by the query and metadata about the result.
#[derive(Debug)]
pub struct GaussDBResult {
    #[cfg(feature = "gaussdb")]
    rows: Vec<gaussdb::Row>,
    #[cfg(not(feature = "gaussdb"))]
    rows: Vec<MockRow>,
    column_count: usize,
    row_count: usize,
    rows_affected: usize,
}

#[cfg(not(feature = "gaussdb"))]
#[derive(Debug, Clone)]
pub(crate) struct MockRow {
    pub(crate) columns: Vec<(String, Option<Vec<u8>>)>,
}

impl GaussDBResult {
    /// Create a new GaussDBResult from raw query results
    #[cfg(feature = "gaussdb")]
    pub fn new(rows: Vec<gaussdb::Row>) -> QueryResult<Self> {
        let row_count = rows.len();
        let column_count = rows.first().map(|row| row.len()).unwrap_or(0);
        
        Ok(GaussDBResult {
            rows,
            column_count,
            row_count,
            rows_affected: row_count, // For SELECT queries, affected = returned
        })
    }

    /// Create a new GaussDBResult for non-query operations (INSERT, UPDATE, DELETE)
    #[cfg(feature = "gaussdb")]
    pub fn new_command_result(rows_affected: u64) -> QueryResult<Self> {
        Ok(GaussDBResult {
            rows: Vec::new(),
            column_count: 0,
            row_count: 0,
            rows_affected: rows_affected as usize,
        })
    }

    /// Create a mock result for testing
    #[cfg(not(feature = "gaussdb"))]
    pub fn new_mock(mock_rows: Vec<MockRow>) -> QueryResult<Self> {
        let row_count = mock_rows.len();
        let column_count = mock_rows.first().map(|row| row.columns.len()).unwrap_or(0);
        
        Ok(GaussDBResult {
            rows: mock_rows,
            column_count,
            row_count,
            rows_affected: row_count,
        })
    }

    /// Get the number of rows returned by the query
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Get the number of columns in the result set
    pub fn column_count(&self) -> usize {
        self.column_count
    }

    /// Get the number of rows affected by the query (for INSERT/UPDATE/DELETE)
    pub fn rows_affected(&self) -> usize {
        self.rows_affected
    }

    /// Check if the result set is empty
    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }

    /// Get an iterator over the rows in the result set
    pub fn iter(&self) -> GaussDBResultIterator<'_> {
        GaussDBResultIterator {
            result: self,
            current_row: 0,
        }
    }

    /// Get a specific row by index
    pub fn get_row(&self, index: usize) -> Option<GaussDBRow<'_>> {
        if index < self.row_count {
            #[cfg(feature = "gaussdb")]
            {
                Some(GaussDBRow::new(&self.rows[index]))
            }
            #[cfg(not(feature = "gaussdb"))]
            {
                Some(GaussDBRow::new_mock(&self.rows[index]))
            }
        } else {
            None
        }
    }

    /// Convert the result into a vector of rows
    pub fn into_rows(self) -> Vec<GaussDBRow<'static>> {
        #[cfg(feature = "gaussdb")]
        {
            self.rows.into_iter().map(|row| GaussDBRow::new_owned(row)).collect()
        }
        #[cfg(not(feature = "gaussdb"))]
        {
            self.rows.into_iter().map(|row| GaussDBRow::new_mock_owned(row)).collect()
        }
    }
}

/// Iterator over rows in a GaussDBResult
pub struct GaussDBResultIterator<'a> {
    result: &'a GaussDBResult,
    current_row: usize,
}

impl<'a> Iterator for GaussDBResultIterator<'a> {
    type Item = GaussDBRow<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row < self.result.row_count {
            let row = self.result.get_row(self.current_row);
            self.current_row += 1;
            row
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.result.row_count.saturating_sub(self.current_row);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for GaussDBResultIterator<'a> {
    fn len(&self) -> usize {
        self.result.row_count.saturating_sub(self.current_row)
    }
}

/// Error information for GaussDB database errors
#[derive(Debug)]
pub struct GaussDBErrorInformation {
    message: String,
    details: Option<String>,
    hint: Option<String>,
    table_name: Option<String>,
    column_name: Option<String>,
    constraint_name: Option<String>,
}

impl GaussDBErrorInformation {
    /// Create new error information from a GaussDB error
    #[cfg(feature = "gaussdb")]
    pub fn new(error: &gaussdb::Error) -> Self {
        Self {
            message: error.to_string(),
            details: None, // gaussdb crate doesn't expose detailed error info
            hint: None,
            table_name: None,
            column_name: None,
            constraint_name: None,
        }
    }

    /// Create new error information from a string message
    pub fn new_from_message(message: String) -> Self {
        Self {
            message,
            details: None,
            hint: None,
            table_name: None,
            column_name: None,
            constraint_name: None,
        }
    }
}

impl DatabaseErrorInformation for GaussDBErrorInformation {
    fn message(&self) -> &str {
        &self.message
    }

    fn details(&self) -> Option<&str> {
        self.details.as_deref()
    }

    fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    fn table_name(&self) -> Option<&str> {
        self.table_name.as_deref()
    }

    fn column_name(&self) -> Option<&str> {
        self.column_name.as_deref()
    }

    fn constraint_name(&self) -> Option<&str> {
        self.constraint_name.as_deref()
    }

    fn statement_position(&self) -> Option<i32> {
        None // GaussDB doesn't provide statement position information
    }
}

impl fmt::Display for GaussDBErrorInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GaussDBErrorInformation {}

/// Convert a GaussDB error to a Diesel error
#[cfg(feature = "gaussdb")]
pub fn convert_gaussdb_error(error: gaussdb::Error) -> Error {
    // Map GaussDB errors to Diesel error kinds
    let error_kind = match error.to_string().as_str() {
        s if s.contains("unique") => DatabaseErrorKind::UniqueViolation,
        s if s.contains("foreign key") => DatabaseErrorKind::ForeignKeyViolation,
        s if s.contains("not null") => DatabaseErrorKind::NotNullViolation,
        s if s.contains("check") => DatabaseErrorKind::CheckViolation,
        s if s.contains("connection") => DatabaseErrorKind::ClosedConnection,
        _ => DatabaseErrorKind::Unknown,
    };

    let error_info = Box::new(GaussDBErrorInformation::new(&error));
    Error::DatabaseError(error_kind, error_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "gaussdb"))]
    fn test_mock_result_creation() {
        let mock_rows = vec![
            MockRow {
                columns: vec![
                    ("id".to_string(), Some(b"1".to_vec())),
                    ("name".to_string(), Some(b"test".to_vec())),
                ],
            },
            MockRow {
                columns: vec![
                    ("id".to_string(), Some(b"2".to_vec())),
                    ("name".to_string(), Some(b"test2".to_vec())),
                ],
            },
        ];

        let result = GaussDBResult::new_mock(mock_rows).unwrap();
        assert_eq!(result.row_count(), 2);
        assert_eq!(result.column_count(), 2);
        assert_eq!(result.rows_affected(), 2);
        assert!(!result.is_empty());
    }

    #[test]
    #[cfg(not(feature = "gaussdb"))]
    fn test_result_iterator() {
        let mock_rows = vec![
            MockRow {
                columns: vec![("id".to_string(), Some(b"1".to_vec()))],
            },
            MockRow {
                columns: vec![("id".to_string(), Some(b"2".to_vec()))],
            },
        ];

        let result = GaussDBResult::new_mock(mock_rows).unwrap();
        let mut iter = result.iter();
        
        assert_eq!(iter.len(), 2);
        assert!(iter.next().is_some());
        assert_eq!(iter.len(), 1);
        assert!(iter.next().is_some());
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_error_information() {
        let error_info = GaussDBErrorInformation::new_from_message("Test error".to_string());
        assert_eq!(error_info.message(), "Test error");
        assert!(error_info.details().is_none());
        assert!(error_info.hint().is_none());
    }
}
