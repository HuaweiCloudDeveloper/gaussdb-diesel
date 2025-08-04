//! Transaction support for GaussDB
//!
//! This module provides transaction building functionality compatible with PostgreSQL.

use crate::backend::GaussDB;
use diesel::backend::Backend;
use diesel::connection::{AnsiTransactionManager, TransactionManager};
use diesel::prelude::*;
use diesel::query_builder::{AstPass, QueryBuilder, QueryFragment};
use diesel::result::Error;

/// Used to build a transaction, specifying additional details.
///
/// This struct is returned by [`.build_transaction`].
/// See the documentation for methods on this struct for usage examples.
/// See [the PostgreSQL documentation for `SET TRANSACTION`][pg-docs]
/// for details on the behavior of each option.
///
/// [`.build_transaction`]: GaussDBConnection::build_transaction()
/// [pg-docs]: https://www.postgresql.org/docs/current/static/sql-set-transaction.html
#[allow(missing_debug_implementations)] // False positive. Connection isn't Debug.
#[must_use = "Transaction builder does nothing unless you call `run` on it"]
pub struct TransactionBuilder<'a, C> {
    connection: &'a mut C,
    isolation_level: Option<IsolationLevel>,
    read_mode: Option<ReadMode>,
    deferrable: Option<Deferrable>,
}

impl<'a, C> TransactionBuilder<'a, C>
where
    C: Connection<Backend = GaussDB, TransactionManager = AnsiTransactionManager>,
{
    /// Creates a new TransactionBuilder.
    pub(crate) fn new(connection: &'a mut C) -> Self {
        Self {
            connection,
            isolation_level: None,
            read_mode: None,
            deferrable: None,
        }
    }

    /// Makes the transaction `READ ONLY`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use diesel_gaussdb::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
    /// conn.build_transaction()
    ///     .read_only()
    ///     .run::<_, diesel::result::Error, _>(|conn| {
    ///         // Read operations only
    ///         Ok(())
    ///     })?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn read_only(mut self) -> Self {
        self.read_mode = Some(ReadMode::ReadOnly);
        self
    }

    /// Makes the transaction `READ WRITE`
    ///
    /// This is the default, unless you've changed the
    /// `default_transaction_read_only` configuration parameter.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use diesel_gaussdb::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
    /// conn.build_transaction().read_write().run::<_, diesel::result::Error, _>(|conn| {
    ///     // Read and write operations
    ///     Ok(())
    /// })?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn read_write(mut self) -> Self {
        self.read_mode = Some(ReadMode::ReadWrite);
        self
    }

    /// Makes the transaction `DEFERRABLE`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use diesel_gaussdb::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
    /// conn.build_transaction().deferrable().run::<_, diesel::result::Error, _>(|conn| Ok(()))?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn deferrable(mut self) -> Self {
        self.deferrable = Some(Deferrable::Deferrable);
        self
    }

    /// Makes the transaction `NOT DEFERRABLE`
    ///
    /// This is the default, unless you've changed the
    /// `default_transaction_deferrable` configuration parameter.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use diesel_gaussdb::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
    /// conn.build_transaction().not_deferrable().run::<_, diesel::result::Error, _>(|conn| Ok(()))?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn not_deferrable(mut self) -> Self {
        self.deferrable = Some(Deferrable::NotDeferrable);
        self
    }

    /// Makes the transaction `ISOLATION LEVEL READ COMMITTED`
    ///
    /// This is the default, unless you've changed the
    /// `default_transaction_isolation_level` configuration parameter.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use diesel_gaussdb::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
    /// conn.build_transaction().read_committed().run::<_, diesel::result::Error, _>(|conn| Ok(()))?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn read_committed(mut self) -> Self {
        self.isolation_level = Some(IsolationLevel::ReadCommitted);
        self
    }

    /// Makes the transaction `ISOLATION LEVEL REPEATABLE READ`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use diesel_gaussdb::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
    /// conn.build_transaction()
    ///     .repeatable_read()
    ///     .run::<_, diesel::result::Error, _>(|conn| Ok(()))?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn repeatable_read(mut self) -> Self {
        self.isolation_level = Some(IsolationLevel::RepeatableRead);
        self
    }

    /// Makes the transaction `ISOLATION LEVEL SERIALIZABLE`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use diesel_gaussdb::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let mut conn = GaussDBConnection::establish("gaussdb://localhost/test")?;
    /// conn.build_transaction().serializable().run::<_, diesel::result::Error, _>(|conn| Ok(()))?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn serializable(mut self) -> Self {
        self.isolation_level = Some(IsolationLevel::Serializable);
        self
    }

    /// Runs the given function inside of the transaction
    /// with the parameters given to this builder.
    ///
    /// This function executes the provided closure `f` inside a database
    /// transaction. If there is already an open transaction for the current
    /// connection it will return an error. The connection is committed if
    /// the closure returns `Ok(_)`, it will be rolled back if it returns `Err(_)`.
    /// For both cases the original result value will be returned from this function.
    ///
    /// If the transaction fails to commit and requires a rollback according to GaussDB,
    /// (e.g. serialization failure) a rollback will be attempted.
    /// If the rollback fails, the error will be returned in a
    /// [`Error::RollbackErrorOnCommit`](diesel::result::Error::RollbackErrorOnCommit),
    /// from which you will be able to extract both the original commit error and
    /// the rollback error.
    /// In addition, the connection will be considered broken
    /// as it contains a uncommitted unabortable open transaction. Any further
    /// interaction with the transaction system will result in an returned error
    /// in this case.
    pub fn run<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut C) -> Result<T, E>,
        E: From<Error>,
    {
        let mut query_builder = <GaussDB as Backend>::QueryBuilder::default();
        self.to_sql(&mut query_builder, &GaussDB)?;
        let sql = query_builder.finish();

        AnsiTransactionManager::begin_transaction_sql(&mut *self.connection, &sql)?;
        match f(&mut *self.connection) {
            Ok(value) => {
                AnsiTransactionManager::commit_transaction(&mut *self.connection)?;
                Ok(value)
            }
            Err(user_error) => {
                match AnsiTransactionManager::rollback_transaction(&mut *self.connection) {
                    Ok(()) => Err(user_error),
                    Err(Error::BrokenTransactionManager) => {
                        // In this case we are probably more interested by the
                        // original error, which likely caused this
                        Err(user_error)
                    }
                    Err(rollback_error) => Err(rollback_error.into()),
                }
            }
        }
    }
}

impl<C> QueryFragment<GaussDB> for TransactionBuilder<'_, C> {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> diesel::QueryResult<()> {
        out.push_sql("BEGIN TRANSACTION");
        if let Some(ref isolation_level) = self.isolation_level {
            isolation_level.walk_ast(out.reborrow())?;
        }
        if let Some(ref read_mode) = self.read_mode {
            read_mode.walk_ast(out.reborrow())?;
        }
        if let Some(ref deferrable) = self.deferrable {
            deferrable.walk_ast(out.reborrow())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum IsolationLevel {
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl QueryFragment<GaussDB> for IsolationLevel {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> diesel::QueryResult<()> {
        out.push_sql(" ISOLATION LEVEL ");
        match *self {
            IsolationLevel::ReadCommitted => out.push_sql("READ COMMITTED"),
            IsolationLevel::RepeatableRead => out.push_sql("REPEATABLE READ"),
            IsolationLevel::Serializable => out.push_sql("SERIALIZABLE"),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum ReadMode {
    ReadOnly,
    ReadWrite,
}

impl QueryFragment<GaussDB> for ReadMode {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> diesel::QueryResult<()> {
        match *self {
            ReadMode::ReadOnly => out.push_sql(" READ ONLY"),
            ReadMode::ReadWrite => out.push_sql(" READ WRITE"),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum Deferrable {
    Deferrable,
    NotDeferrable,
}

impl QueryFragment<GaussDB> for Deferrable {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> diesel::QueryResult<()> {
        match *self {
            Deferrable::Deferrable => out.push_sql(" DEFERRABLE"),
            Deferrable::NotDeferrable => out.push_sql(" NOT DEFERRABLE"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::GaussDB;
    use diesel::query_builder::QueryBuilder;

    #[test]
    fn test_isolation_levels() {
        let read_committed = IsolationLevel::ReadCommitted;
        let repeatable_read = IsolationLevel::RepeatableRead;
        let serializable = IsolationLevel::Serializable;

        let mut query_builder = <GaussDB as Backend>::QueryBuilder::default();
        read_committed.to_sql(&mut query_builder, &GaussDB).unwrap();
        assert_eq!(query_builder.finish(), " ISOLATION LEVEL READ COMMITTED");

        let mut query_builder = <GaussDB as Backend>::QueryBuilder::default();
        repeatable_read.to_sql(&mut query_builder, &GaussDB).unwrap();
        assert_eq!(query_builder.finish(), " ISOLATION LEVEL REPEATABLE READ");

        let mut query_builder = <GaussDB as Backend>::QueryBuilder::default();
        serializable.to_sql(&mut query_builder, &GaussDB).unwrap();
        assert_eq!(query_builder.finish(), " ISOLATION LEVEL SERIALIZABLE");
    }

    #[test]
    fn test_read_modes() {
        let read_only = ReadMode::ReadOnly;
        let read_write = ReadMode::ReadWrite;

        let mut query_builder = <GaussDB as Backend>::QueryBuilder::default();
        read_only.to_sql(&mut query_builder, &GaussDB).unwrap();
        assert_eq!(query_builder.finish(), " READ ONLY");

        let mut query_builder = <GaussDB as Backend>::QueryBuilder::default();
        read_write.to_sql(&mut query_builder, &GaussDB).unwrap();
        assert_eq!(query_builder.finish(), " READ WRITE");
    }

    #[test]
    fn test_deferrable_modes() {
        let deferrable = Deferrable::Deferrable;
        let not_deferrable = Deferrable::NotDeferrable;

        let mut query_builder = <GaussDB as Backend>::QueryBuilder::default();
        deferrable.to_sql(&mut query_builder, &GaussDB).unwrap();
        assert_eq!(query_builder.finish(), " DEFERRABLE");

        let mut query_builder = <GaussDB as Backend>::QueryBuilder::default();
        not_deferrable.to_sql(&mut query_builder, &GaussDB).unwrap();
        assert_eq!(query_builder.finish(), " NOT DEFERRABLE");
    }

    #[test]
    fn test_transaction_builder_basic() {
        // Test that transaction builder can be created and configured
        // We can't easily test the full functionality without a real connection
        // but we can test the builder pattern

        // This test ensures the API compiles correctly
        fn _test_builder_api<C>(_conn: &mut C)
        where
            C: Connection<Backend = GaussDB, TransactionManager = AnsiTransactionManager>
        {
            let _builder = TransactionBuilder::new(_conn)
                .read_only()
                .serializable()
                .deferrable();
        }
    }
}