//! Cursor support for GaussDB connections
//!
//! This module provides cursor functionality for handling large result sets
//! efficiently by fetching data in batches rather than loading everything
//! into memory at once.

use crate::backend::GaussDB;
use crate::connection::{GaussDBConnection, row::GaussDBRow};
use diesel::result::{QueryResult, Error as DieselError};
use diesel::query_builder::{QueryFragment, QueryId, QueryBuilder};
use diesel::connection::SimpleConnection;
use std::fmt;

/// A cursor for iterating over large result sets in batches
///
/// Cursors allow you to process large query results without loading
/// all data into memory at once. This is particularly useful for
/// data processing pipelines and ETL operations.
///
/// # Example
///
/// ```rust,no_run
/// # use diesel_gaussdb::prelude::*;
/// # use diesel_gaussdb::connection::cursor::GaussDBCursor;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
/// // Declare a cursor for a large query
/// let mut cursor = GaussDBCursor::declare(
///     &mut conn,
///     "large_data_cursor",
///     "SELECT * FROM large_table ORDER BY id"
/// )?;
///
/// // Process data in batches
/// loop {
///     let batch = cursor.fetch(1000)?;
///     if batch.is_empty() {
///         break;
///     }
///     
///     for row in batch {
///         // Process each row
///         println!("Processing row: {:?}", row);
///     }
/// }
///
/// // Close the cursor
/// cursor.close()?;
/// # Ok(())
/// # }
/// ```
pub struct GaussDBCursor<'conn> {
    name: String,
    connection: &'conn mut GaussDBConnection,
    is_closed: bool,
}

impl<'conn> GaussDBCursor<'conn> {
    /// Declare a new cursor with the given name and query
    ///
    /// This creates a server-side cursor that can be used to fetch
    /// results in batches. The cursor must be closed when done to
    /// free server resources.
    ///
    /// # Arguments
    ///
    /// * `connection` - The database connection to use
    /// * `name` - A unique name for the cursor
    /// * `query` - The SQL query to execute
    ///
    /// # Errors
    ///
    /// Returns an error if the cursor declaration fails or if a cursor
    /// with the same name already exists.
    pub fn declare(
        connection: &'conn mut GaussDBConnection,
        name: &str,
        query: &str,
    ) -> QueryResult<Self> {
        let declare_sql = format!("DECLARE {} CURSOR FOR {}", name, query);
        
        connection.batch_execute(&declare_sql)?;
        
        Ok(GaussDBCursor {
            name: name.to_string(),
            connection,
            is_closed: false,
        })
    }

    /// Declare a cursor from a Diesel query
    ///
    /// This is a convenience method for declaring cursors from Diesel queries
    /// rather than raw SQL strings.
    ///
    /// # Arguments
    ///
    /// * `connection` - The database connection to use
    /// * `name` - A unique name for the cursor
    /// * `query` - The Diesel query to execute
    pub fn declare_query<T>(
        connection: &'conn mut GaussDBConnection,
        name: &str,
        query: T,
    ) -> QueryResult<Self>
    where
        T: QueryFragment<GaussDB> + QueryId,
    {
        // Build the SQL from the Diesel query
        let mut query_builder = crate::query_builder::GaussDBQueryBuilder::new();
        query.to_sql(&mut query_builder, &GaussDB)?;
        let sql = query_builder.finish();
        
        Self::declare(connection, name, &sql)
    }

    /// Fetch the next batch of rows from the cursor
    ///
    /// # Arguments
    ///
    /// * `count` - The maximum number of rows to fetch
    ///
    /// # Returns
    ///
    /// A vector of rows. If the vector is empty, there are no more rows
    /// to fetch from the cursor.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetch operation fails or if the cursor
    /// has been closed.
    pub fn fetch(&mut self, count: i32) -> QueryResult<Vec<GaussDBRow<'static>>> {
        if self.is_closed {
            return Err(DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new("Cursor has been closed".to_string())
            ));
        }

        let fetch_sql = format!("FETCH {} FROM {}", count, self.name);
        
        #[cfg(feature = "gaussdb")]
        {
            let empty_params: Vec<&(dyn gaussdb::types::ToSql + Sync)> = vec![];
            let rows = self.connection.raw_connection().query(&fetch_sql, &empty_params)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand,
                    Box::new(format!("GaussDB cursor fetch error: {}", e))
                ))?;

            // Convert gaussdb::Row to GaussDBRow
            let mut result = Vec::new();
            for row in rows {
                result.push(GaussDBRow::new_owned(row));
            }
            Ok(result)
        }
        #[cfg(not(feature = "gaussdb"))]
        {
            // Mock implementation for testing
            use crate::connection::result::MockRow;
            
            // Simulate fetching some mock data
            if count > 0 {
                let mock_row = MockRow {
                    columns: vec![
                        ("id".to_string(), Some(b"1".to_vec())),
                        ("data".to_string(), Some(b"test_data".to_vec())),
                    ],
                };
                Ok(vec![GaussDBRow::new_mock_owned(mock_row)])
            } else {
                Ok(vec![])
            }
        }
    }

    /// Fetch all remaining rows from the cursor
    ///
    /// This is a convenience method that fetches all remaining rows
    /// in the cursor. Use with caution on large result sets as it
    /// will load all data into memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetch operation fails or if the cursor
    /// has been closed.
    pub fn fetch_all(&mut self) -> QueryResult<Vec<GaussDBRow<'static>>> {
        let fetch_sql = format!("FETCH ALL FROM {}", self.name);
        
        #[cfg(feature = "gaussdb")]
        {
            let empty_params: Vec<&(dyn gaussdb::types::ToSql + Sync)> = vec![];
            let rows = self.connection.raw_connection().query(&fetch_sql, &empty_params)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand,
                    Box::new(format!("GaussDB cursor fetch all error: {}", e))
                ))?;

            // Convert gaussdb::Row to GaussDBRow
            let mut result = Vec::new();
            for row in rows {
                result.push(GaussDBRow::new_owned(row));
            }
            Ok(result)
        }
        #[cfg(not(feature = "gaussdb"))]
        {
            // Mock implementation - return empty for testing
            Ok(vec![])
        }
    }

    /// Move the cursor to a specific position
    ///
    /// # Arguments
    ///
    /// * `position` - The position to move to. Can be:
    ///   - A positive number to move forward
    ///   - A negative number to move backward
    ///   - "FIRST" to move to the beginning
    ///   - "LAST" to move to the end
    ///
    /// # Errors
    ///
    /// Returns an error if the move operation fails or if the cursor
    /// has been closed.
    pub fn move_cursor(&mut self, position: &str) -> QueryResult<()> {
        if self.is_closed {
            return Err(DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new("Cursor has been closed".to_string())
            ));
        }

        let move_sql = format!("MOVE {} FROM {}", position, self.name);
        self.connection.batch_execute(&move_sql)
    }

    /// Close the cursor and free server resources
    ///
    /// This should be called when you're done with the cursor to free
    /// server-side resources. Once closed, the cursor cannot be used again.
    ///
    /// # Errors
    ///
    /// Returns an error if the close operation fails. The cursor will
    /// be marked as closed regardless of whether the operation succeeds.
    pub fn close(mut self) -> QueryResult<()> {
        if self.is_closed {
            return Ok(());
        }

        let close_sql = format!("CLOSE {}", self.name);
        let result = self.connection.batch_execute(&close_sql);
        self.is_closed = true;
        result
    }

    /// Get the name of the cursor
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if the cursor is closed
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }
}

impl<'conn> fmt::Debug for GaussDBCursor<'conn> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GaussDBCursor")
            .field("name", &self.name)
            .field("is_closed", &self.is_closed)
            .finish()
    }
}

// Implement Drop to ensure cursors are closed when dropped
impl<'conn> Drop for GaussDBCursor<'conn> {
    fn drop(&mut self) {
        if !self.is_closed {
            // Try to close the cursor, but ignore errors since we're in Drop
            let close_sql = format!("CLOSE {}", self.name);
            let _ = self.connection.batch_execute(&close_sql);
            self.is_closed = true;
        }
    }
}

/// Extension trait for GaussDBConnection to provide cursor functionality
pub trait CursorDsl {
    /// Declare a cursor with the given name and query
    fn declare_cursor(&mut self, name: &str, query: &str) -> QueryResult<GaussDBCursor<'_>>;
    
    /// Declare a cursor from a Diesel query
    fn declare_cursor_query<T>(&mut self, name: &str, query: T) -> QueryResult<GaussDBCursor<'_>>
    where
        T: QueryFragment<GaussDB> + QueryId;
}

impl CursorDsl for GaussDBConnection {
    fn declare_cursor(&mut self, name: &str, query: &str) -> QueryResult<GaussDBCursor<'_>> {
        GaussDBCursor::declare(self, name, query)
    }
    
    fn declare_cursor_query<T>(&mut self, name: &str, query: T) -> QueryResult<GaussDBCursor<'_>>
    where
        T: QueryFragment<GaussDB> + QueryId,
    {
        GaussDBCursor::declare_query(self, name, query)
    }
}
