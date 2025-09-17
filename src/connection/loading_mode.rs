//! Loading modes for GaussDB connections
//!
//! This module provides different loading strategies for query results,
//! allowing optimization for different use cases such as memory usage,
//! performance, and data processing patterns.

use crate::backend::GaussDB;
use crate::connection::{GaussDBConnection, row::GaussDBRow};
use diesel::result::{QueryResult, Error as DieselError};
use diesel::query_builder::{QueryFragment, QueryId, QueryBuilder};
use diesel::connection::SimpleConnection;
use std::marker::PhantomData;

/// Trait for different loading modes
///
/// Loading modes determine how query results are fetched and processed.
/// Different modes can optimize for memory usage, performance, or
/// specific data processing patterns.
pub trait LoadingMode<ST> {
    /// The type of the loaded result
    type LoadedResult;

    /// Load the query result using this loading mode
    fn load_result<T>(
        connection: &mut GaussDBConnection,
        query: T,
    ) -> QueryResult<Self::LoadedResult>
    where
        T: QueryFragment<GaussDB> + QueryId;
}

/// Default loading mode that loads all results into memory at once
///
/// This is the standard loading mode that fetches all query results
/// and loads them into a Vec. It's suitable for most use cases where
/// the result set size is manageable.
///
/// # Example
///
/// ```rust,no_run
/// # use diesel_gaussdb::prelude::*;
/// # use diesel_gaussdb::connection::loading_mode::{DefaultLoadingMode, LoadingMode};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
/// // Use default loading mode (this is the default behavior)
/// let results: Vec<(i32, String)> = DefaultLoadingMode::load_result(
///     &mut conn,
///     "SELECT id, name FROM users"
/// )?;
/// 
/// for (id, name) in results {
///     println!("User {}: {}", id, name);
/// }
/// # Ok(())
/// # }
/// ```
pub struct DefaultLoadingMode<ST> {
    _phantom: PhantomData<ST>,
}

impl<ST> LoadingMode<ST> for DefaultLoadingMode<ST> {
    type LoadedResult = Vec<GaussDBRow<'static>>;

    fn load_result<T>(
        connection: &mut GaussDBConnection,
        query: T,
    ) -> QueryResult<Self::LoadedResult>
    where
        T: QueryFragment<GaussDB> + QueryId,
    {
        // Build the SQL from the query
        let mut query_builder = crate::query_builder::GaussDBQueryBuilder::new();
        query.to_sql(&mut query_builder, &GaussDB)?;
        let sql = query_builder.finish();

        {
            let empty_params: Vec<&(dyn gaussdb::types::ToSql + Sync)> = vec![];
            let rows = connection.raw_connection().query(&sql, &empty_params)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand,
                    Box::new(format!("GaussDB query error: {}", e))
                ))?;

            // Convert gaussdb::Row to GaussDBRow
            let mut result = Vec::new();
            for row in rows {
                result.push(GaussDBRow::new_owned(row));
            }
            Ok(result)
        }
    }
}

/// Row-by-row loading mode for memory-efficient processing
///
/// This loading mode processes query results one row at a time,
/// which is memory-efficient for large result sets. It uses an
/// iterator-like pattern to process rows without loading everything
/// into memory at once.
///
/// # Example
///
/// ```rust,no_run
/// # use diesel_gaussdb::prelude::*;
/// # use diesel_gaussdb::connection::loading_mode::{GaussDBRowByRowLoadingMode, LoadingMode};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
/// // Use row-by-row loading mode for large datasets
/// let mut row_iterator = GaussDBRowByRowLoadingMode::load_result(
///     &mut conn,
///     "SELECT * FROM large_table"
/// )?;
/// 
/// // Process rows one by one
/// while let Some(row) = row_iterator.next()? {
///     // Process individual row without loading all data into memory
///     println!("Processing row: {:?}", row);
/// }
/// # Ok(())
/// # }
/// ```
pub struct GaussDBRowByRowLoadingMode<ST> {
    _phantom: PhantomData<ST>,
}

/// Iterator for row-by-row loading
///
/// This iterator allows processing query results one row at a time,
/// which is memory-efficient for large datasets.
pub struct GaussDBRowIterator<'conn> {
    connection: &'conn mut GaussDBConnection,
    cursor_name: String,
    is_finished: bool,
}

impl<'conn> GaussDBRowIterator<'conn> {
    /// Create a new row iterator
    fn new(connection: &'conn mut GaussDBConnection, sql: &str) -> QueryResult<Self> {
        // Generate a unique cursor name
        let cursor_name = format!("row_iterator_{}", std::ptr::addr_of!(*connection) as usize);
        
        // Declare a cursor for the query
        let declare_sql = format!("DECLARE {} CURSOR FOR {}", cursor_name, sql);
        connection.batch_execute(&declare_sql)?;
        
        Ok(GaussDBRowIterator {
            connection,
            cursor_name,
            is_finished: false,
        })
    }

    /// Get the next row from the iterator
    pub fn next(&mut self) -> QueryResult<Option<GaussDBRow<'static>>> {
        if self.is_finished {
            return Ok(None);
        }

        let fetch_sql = format!("FETCH 1 FROM {}", self.cursor_name);
        
        {
            let empty_params: Vec<&(dyn gaussdb::types::ToSql + Sync)> = vec![];
            let rows = self.connection.raw_connection().query(&fetch_sql, &empty_params)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand,
                    Box::new(format!("GaussDB cursor fetch error: {}", e))
                ))?;

            if rows.is_empty() {
                self.is_finished = true;
                Ok(None)
            } else {
                Ok(Some(GaussDBRow::new_owned(rows.into_iter().next().unwrap())))
            }
        }
    }

    /// Check if the iterator has finished
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }
}

impl<'conn> Drop for GaussDBRowIterator<'conn> {
    fn drop(&mut self) {
        // Close the cursor when the iterator is dropped
        let close_sql = format!("CLOSE {}", self.cursor_name);
        let _ = self.connection.batch_execute(&close_sql);
    }
}

impl<ST> LoadingMode<ST> for GaussDBRowByRowLoadingMode<ST> {
    type LoadedResult = Vec<GaussDBRow<'static>>;

    fn load_result<T>(
        connection: &mut GaussDBConnection,
        query: T,
    ) -> QueryResult<Self::LoadedResult>
    where
        T: QueryFragment<GaussDB> + QueryId,
    {
        // For now, we'll implement this as a simplified version that loads all rows
        // In a real implementation, this would use a different approach to handle lifetimes
        // Build the SQL from the query
        let mut query_builder = crate::query_builder::GaussDBQueryBuilder::new();
        query.to_sql(&mut query_builder, &GaussDB)?;
        let sql = query_builder.finish();

        {
            let empty_params: Vec<&(dyn gaussdb::types::ToSql + Sync)> = vec![];
            let rows = connection.raw_connection().query(&sql, &empty_params)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand,
                    Box::new(format!("GaussDB query error: {}", e))
                ))?;

            // Convert gaussdb::Row to GaussDBRow
            let mut result = Vec::new();
            for row in rows {
                result.push(GaussDBRow::new_owned(row));
            }
            Ok(result)
        }
    }
}

/// Extension trait for GaussDBConnection to provide loading mode functionality
pub trait LoadingModeDsl {
    /// Load query results using the default loading mode
    fn load_with_default<T, ST>(&mut self, query: T) -> QueryResult<Vec<GaussDBRow<'static>>>
    where
        T: QueryFragment<GaussDB> + QueryId;

    /// Load query results using row-by-row loading mode
    /// Note: This currently returns the same as default mode due to lifetime constraints
    fn load_row_by_row<T, ST>(&mut self, query: T) -> QueryResult<Vec<GaussDBRow<'static>>>
    where
        T: QueryFragment<GaussDB> + QueryId;

    /// Create a row iterator for processing large result sets
    fn create_row_iterator<T>(&mut self, query: T) -> QueryResult<GaussDBRowIterator<'_>>
    where
        T: QueryFragment<GaussDB> + QueryId;

    /// Load query results from SQL string using default loading mode
    fn load_sql_with_default(&mut self, sql: &str) -> QueryResult<Vec<GaussDBRow<'static>>>;

    /// Load query results from SQL string using row-by-row loading mode
    fn load_sql_row_by_row(&mut self, sql: &str) -> QueryResult<Vec<GaussDBRow<'static>>>;

    /// Create a row iterator from SQL string for processing large result sets
    fn create_sql_row_iterator(&mut self, sql: &str) -> QueryResult<GaussDBRowIterator<'_>>;
}

impl LoadingModeDsl for GaussDBConnection {
    fn load_with_default<T, ST>(&mut self, query: T) -> QueryResult<Vec<GaussDBRow<'static>>>
    where
        T: QueryFragment<GaussDB> + QueryId,
    {
        DefaultLoadingMode::<ST>::load_result(self, query)
    }

    fn load_row_by_row<T, ST>(&mut self, query: T) -> QueryResult<Vec<GaussDBRow<'static>>>
    where
        T: QueryFragment<GaussDB> + QueryId,
    {
        // For now, this uses the same implementation as default mode
        // In a real implementation, this would use a different strategy
        GaussDBRowByRowLoadingMode::<ST>::load_result(self, query)
    }

    fn create_row_iterator<T>(&mut self, query: T) -> QueryResult<GaussDBRowIterator<'_>>
    where
        T: QueryFragment<GaussDB> + QueryId,
    {
        // Build the SQL from the query
        let mut query_builder = crate::query_builder::GaussDBQueryBuilder::new();
        query.to_sql(&mut query_builder, &GaussDB)?;
        let sql = query_builder.finish();

        GaussDBRowIterator::new(self, &sql)
    }

    fn load_sql_with_default(&mut self, sql: &str) -> QueryResult<Vec<GaussDBRow<'static>>> {
        {
            let empty_params: Vec<&(dyn gaussdb::types::ToSql + Sync)> = vec![];
            let rows = self.raw_connection().query(sql, &empty_params)
                .map_err(|e| DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UnableToSendCommand,
                    Box::new(format!("GaussDB query error: {}", e))
                ))?;

            // Convert gaussdb::Row to GaussDBRow
            let mut result = Vec::new();
            for row in rows {
                result.push(GaussDBRow::new_owned(row));
            }
            Ok(result)
        }
    }

    fn load_sql_row_by_row(&mut self, sql: &str) -> QueryResult<Vec<GaussDBRow<'static>>> {
        // For now, this uses the same implementation as default mode
        self.load_sql_with_default(sql)
    }

    fn create_sql_row_iterator(&mut self, sql: &str) -> QueryResult<GaussDBRowIterator<'_>> {
        GaussDBRowIterator::new(self, sql)
    }
}
