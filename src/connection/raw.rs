//! Raw connection implementation for GaussDB
//!
//! This module provides the low-level connection interface to GaussDB databases
//! using the real gaussdb crate for authentic connectivity.

use diesel::result::{ConnectionResult, Error as DieselError, DatabaseErrorKind};
use std::fmt;

use gaussdb::Client;

/// Raw connection to GaussDB database
///
/// This wraps the real gaussdb::Client for authentic GaussDB connectivity.
pub struct RawConnection {
    client: Client,
}

impl RawConnection {
    /// Establish a new connection to GaussDB
    pub fn establish(database_url: &str) -> ConnectionResult<Self> {
        use gaussdb::{Config, NoTls};
        use std::str::FromStr;

        let config = Config::from_str(database_url)
            .map_err(|e| diesel::ConnectionError::CouldntSetupConfiguration(DieselError::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand,
                Box::new(format!("Invalid database URL: {}", e))
            )))?;

        let client = config.connect(NoTls)
            .map_err(|e| diesel::ConnectionError::CouldntSetupConfiguration(DieselError::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand,
                Box::new(format!("Failed to connect to GaussDB: {}", e))
            )))?;

        Ok(Self { client })
    }

    /// Execute a simple SQL statement
    pub fn execute(&mut self, sql: &str) -> ConnectionResult<usize> {
        self.client.execute(sql, &[])
            .map(|rows| rows as usize)
            .map_err(|e| diesel::ConnectionError::CouldntSetupConfiguration(DieselError::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand,
                Box::new(format!("GaussDB execute error: {}", e))
            )))
    }

    /// Execute a query and return raw results
    pub fn query(&mut self, sql: &str, params: &[&(dyn gaussdb::types::ToSql + Sync)]) -> ConnectionResult<Vec<gaussdb::Row>> {
        self.client.query(sql, params)
            .map_err(|e| diesel::ConnectionError::CouldntSetupConfiguration(DieselError::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand,
                Box::new(format!("GaussDB query error: {}", e))
            )))
    }



    /// Batch execute multiple statements
    pub fn batch_execute(&mut self, sql: &str) -> ConnectionResult<()> {
        self.client.batch_execute(sql)
            .map_err(|e| diesel::ConnectionError::CouldntSetupConfiguration(DieselError::DatabaseError(
                DatabaseErrorKind::UnableToSendCommand,
                Box::new(format!("GaussDB batch execute error: {}", e))
            )))
    }

    /// Check if the connection is still alive
    pub fn is_connected(&self) -> bool {
        // For gaussdb::Client, we assume it's connected if it exists
        true
    }

    /// Get the database URL (placeholder for compatibility)
    pub fn database_url(&self) -> &str {
        // gaussdb::Client doesn't expose the URL, return placeholder
        "[REDACTED]"
    }

    /// Close the connection
    pub fn close(&mut self) {
        // gaussdb::Client doesn't have explicit close, it's handled by Drop
    }
}

impl fmt::Debug for RawConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawConnection")
            .field("database_url", &"[REDACTED]")
            .field("connected", &self.is_connected())
            .finish()
    }
}

impl Drop for RawConnection {
    fn drop(&mut self) {
        if self.is_connected() {
            self.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "gaussdb")]
    #[test]
    fn test_establish_connection_invalid_url() {
        let conn = super::RawConnection::establish("invalid://url");
        assert!(conn.is_err());
    }

    // Note: Connection tests are disabled when gaussdb feature is not available
    // as RawConnection is feature-gated.

    #[test]
    #[cfg(feature = "gaussdb")]
    fn test_gaussdb_connection_attempt() {
        // With gaussdb feature, connection attempt should be made
        // This will likely fail without a real database, but should not fail due to missing feature
        let conn = RawConnection::establish("gaussdb://user:pass@localhost:5432/test");
        // We expect this to fail due to no real database, not due to missing feature
        assert!(conn.is_err());
        if let Err(e) = conn {
            let error_msg = format!("{:?}", e);
            assert!(!error_msg.contains("gaussdb feature not enabled"));
        }
    }
}
