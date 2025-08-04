//! Statement implementation for GaussDB
//!
//! This module provides prepared statement functionality for GaussDB connections.

use diesel::result::{QueryResult, Error as DieselError, DatabaseErrorKind};
use std::fmt;

/// A prepared statement for GaussDB
///
/// This represents a prepared SQL statement that can be executed multiple times
/// with different parameters.
pub struct Statement {
    sql: String,
    param_count: usize,
    prepared: bool,
}

impl Statement {
    /// Create a new statement
    pub fn new(sql: String) -> Self {
        // Count the number of parameters in the SQL
        let param_count = sql.matches('$').count();
        
        Statement {
            sql,
            param_count,
            prepared: false,
        }
    }

    /// Prepare the statement
    pub fn prepare(&mut self) -> QueryResult<()> {
        if self.prepared {
            return Ok(());
        }

        // Mock preparation - in reality this would prepare the statement on the server
        println!("Preparing statement: {}", self.sql);
        self.prepared = true;
        Ok(())
    }

    /// Execute the statement with parameters
    pub fn execute(&self, params: &[&dyn std::fmt::Debug]) -> QueryResult<usize> {
        if !self.prepared {
            return Err(DieselError::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand,
                Box::new("Statement is not prepared".to_string())
            ));
        }

        if params.len() != self.param_count {
            return Err(DieselError::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand,
                Box::new(format!(
                    "Parameter count mismatch: expected {}, got {}",
                    self.param_count,
                    params.len()
                ))
            ));
        }

        // Mock execution - in reality this would execute the prepared statement
        println!("Executing prepared statement with params: {:?}", params);
        Ok(0) // Return 0 affected rows for now
    }

    /// Query with the statement and return results
    pub fn query(&self, params: &[&dyn std::fmt::Debug]) -> QueryResult<Vec<Vec<Option<String>>>> {
        if !self.prepared {
            return Err(DieselError::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand,
                Box::new("Statement is not prepared".to_string())
            ));
        }

        if params.len() != self.param_count {
            return Err(DieselError::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand,
                Box::new(format!(
                    "Parameter count mismatch: expected {}, got {}",
                    self.param_count,
                    params.len()
                ))
            ));
        }

        // Mock query - in reality this would execute the query and return results
        println!("Querying with prepared statement, params: {:?}", params);
        Ok(vec![]) // Return empty result set for now
    }

    /// Get the SQL text of this statement
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Get the number of parameters this statement expects
    pub fn param_count(&self) -> usize {
        self.param_count
    }

    /// Check if this statement is prepared
    pub fn is_prepared(&self) -> bool {
        self.prepared
    }
}

impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Statement")
            .field("sql", &self.sql)
            .field("param_count", &self.param_count)
            .field("prepared", &self.prepared)
            .finish()
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Statement({})", self.sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_creation() {
        let stmt = Statement::new("SELECT * FROM users WHERE id = $1".to_string());
        assert_eq!(stmt.sql(), "SELECT * FROM users WHERE id = $1");
        assert_eq!(stmt.param_count(), 1);
        assert!(!stmt.is_prepared());
    }

    #[test]
    fn test_statement_with_multiple_params() {
        let stmt = Statement::new("INSERT INTO users (name, email) VALUES ($1, $2)".to_string());
        assert_eq!(stmt.param_count(), 2);
    }

    #[test]
    fn test_statement_with_no_params() {
        let stmt = Statement::new("SELECT COUNT(*) FROM users".to_string());
        assert_eq!(stmt.param_count(), 0);
    }

    #[test]
    fn test_statement_preparation() {
        let mut stmt = Statement::new("SELECT * FROM users WHERE id = $1".to_string());
        assert!(!stmt.is_prepared());
        
        let result = stmt.prepare();
        assert!(result.is_ok());
        assert!(stmt.is_prepared());
    }

    #[test]
    fn test_execute_unprepared_statement() {
        let stmt = Statement::new("SELECT * FROM users WHERE id = $1".to_string());
        let result = stmt.execute(&[&1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_with_wrong_param_count() {
        let mut stmt = Statement::new("SELECT * FROM users WHERE id = $1".to_string());
        stmt.prepare().unwrap();
        
        let result = stmt.execute(&[&1, &2]); // Too many params
        assert!(result.is_err());
        
        let result = stmt.execute(&[]); // Too few params
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_with_correct_params() {
        let mut stmt = Statement::new("SELECT * FROM users WHERE id = $1".to_string());
        stmt.prepare().unwrap();
        
        let result = stmt.execute(&[&1]);
        assert!(result.is_ok());
    }
}
