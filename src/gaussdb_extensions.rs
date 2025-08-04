//! GaussDB-specific extensions and features
//!
//! This module provides GaussDB-specific functionality that extends beyond
//! standard PostgreSQL compatibility.

use diesel::query_builder::{QueryFragment, AstPass};
use diesel::result::QueryResult;
use crate::backend::GaussDB;

/// GaussDB-specific SQL functions and operators
pub mod functions {
    use super::*;

    /// The `ROWNUM` pseudo-column function (GaussDB specific)
    ///
    /// This function provides Oracle-style ROWNUM functionality in GaussDB.
    /// It returns a number indicating the order in which a row was selected.
    ///
    /// # Example
    ///
    /// ```sql
    /// SELECT ROWNUM, name FROM users WHERE ROWNUM <= 10;
    /// ```
    #[derive(Debug, Clone, Copy)]
    pub struct Rownum;

    impl QueryFragment<GaussDB> for Rownum {
        fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            out.push_sql("ROWNUM");
            Ok(())
        }
    }

    /// The `LEVEL` pseudo-column function for hierarchical queries
    ///
    /// This function is used in hierarchical queries with CONNECT BY clauses.
    /// It returns the level of a node in a tree structure.
    ///
    /// # Example
    ///
    /// ```sql
    /// SELECT LEVEL, name FROM employees 
    /// START WITH manager_id IS NULL 
    /// CONNECT BY PRIOR employee_id = manager_id;
    /// ```
    #[derive(Debug, Clone, Copy)]
    pub struct Level;

    impl QueryFragment<GaussDB> for Level {
        fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            out.push_sql("LEVEL");
            Ok(())
        }
    }

    /// The `CONNECT_BY_ROOT` function for hierarchical queries
    ///
    /// This function returns the root value of a column in a hierarchical query.
    ///
    /// # Example
    ///
    /// ```sql
    /// SELECT CONNECT_BY_ROOT name, name FROM employees 
    /// START WITH manager_id IS NULL 
    /// CONNECT BY PRIOR employee_id = manager_id;
    /// ```
    #[derive(Debug, Clone)]
    pub struct ConnectByRoot<T> {
        expr: T,
    }

    impl<T> ConnectByRoot<T> {
        /// Create a new CONNECT_BY_ROOT expression
        pub fn new(expr: T) -> Self {
            Self { expr }
        }
    }

    impl<T> QueryFragment<GaussDB> for ConnectByRoot<T>
    where
        T: QueryFragment<GaussDB>,
    {
        fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            out.push_sql("CONNECT_BY_ROOT ");
            self.expr.walk_ast(out.reborrow())?;
            Ok(())
        }
    }

    /// The `SYS_CONNECT_BY_PATH` function for hierarchical queries
    ///
    /// This function returns the path from root to the current row.
    ///
    /// # Example
    ///
    /// ```sql
    /// SELECT SYS_CONNECT_BY_PATH(name, '/') FROM employees 
    /// START WITH manager_id IS NULL 
    /// CONNECT BY PRIOR employee_id = manager_id;
    /// ```
    #[derive(Debug, Clone)]
    pub struct SysConnectByPath<T, S> {
        column: T,
        separator: S,
    }

    impl<T, S> SysConnectByPath<T, S> {
        /// Create a new SYS_CONNECT_BY_PATH expression
        pub fn new(column: T, separator: S) -> Self {
            Self { column, separator }
        }
    }

    impl<T, S> QueryFragment<GaussDB> for SysConnectByPath<T, S>
    where
        T: QueryFragment<GaussDB>,
        S: QueryFragment<GaussDB>,
    {
        fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            out.push_sql("SYS_CONNECT_BY_PATH(");
            self.column.walk_ast(out.reborrow())?;
            out.push_sql(", ");
            self.separator.walk_ast(out.reborrow())?;
            out.push_sql(")");
            Ok(())
        }
    }
}

/// GaussDB-specific clauses and query constructs
pub mod clauses {
    use super::*;

    /// The `START WITH` clause for hierarchical queries
    ///
    /// This clause specifies the root rows for a hierarchical query.
    ///
    /// # Example
    ///
    /// ```sql
    /// SELECT * FROM employees 
    /// START WITH manager_id IS NULL 
    /// CONNECT BY PRIOR employee_id = manager_id;
    /// ```
    #[derive(Debug, Clone)]
    pub struct StartWith<T> {
        condition: T,
    }

    impl<T> StartWith<T> {
        /// Create a new START WITH clause
        pub fn new(condition: T) -> Self {
            Self { condition }
        }
    }

    impl<T> QueryFragment<GaussDB> for StartWith<T>
    where
        T: QueryFragment<GaussDB>,
    {
        fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            out.push_sql(" START WITH ");
            self.condition.walk_ast(out.reborrow())?;
            Ok(())
        }
    }

    /// The `CONNECT BY` clause for hierarchical queries
    ///
    /// This clause specifies how rows are connected in a hierarchical query.
    ///
    /// # Example
    ///
    /// ```sql
    /// SELECT * FROM employees 
    /// START WITH manager_id IS NULL 
    /// CONNECT BY PRIOR employee_id = manager_id;
    /// ```
    #[derive(Debug, Clone)]
    pub struct ConnectBy<T> {
        condition: T,
        prior: bool,
    }

    impl<T> ConnectBy<T> {
        /// Create a new CONNECT BY clause
        pub fn new(condition: T) -> Self {
            Self {
                condition,
                prior: false,
            }
        }

        /// Create a new CONNECT BY PRIOR clause
        pub fn prior(condition: T) -> Self {
            Self {
                condition,
                prior: true,
            }
        }
    }

    impl<T> QueryFragment<GaussDB> for ConnectBy<T>
    where
        T: QueryFragment<GaussDB>,
    {
        fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            out.push_sql(" CONNECT BY ");
            if self.prior {
                out.push_sql("PRIOR ");
            }
            self.condition.walk_ast(out.reborrow())?;
            Ok(())
        }
    }

    /// The `MERGE` statement for upsert operations
    ///
    /// This provides GaussDB's MERGE statement functionality.
    ///
    /// # Example
    ///
    /// ```sql
    /// MERGE INTO target_table t
    /// USING source_table s ON (t.id = s.id)
    /// WHEN MATCHED THEN UPDATE SET t.value = s.value
    /// WHEN NOT MATCHED THEN INSERT (id, value) VALUES (s.id, s.value);
    /// ```
    #[derive(Debug, Clone)]
    pub struct MergeInto<T, S, C> {
        target: T,
        source: S,
        condition: C,
    }

    impl<T, S, C> MergeInto<T, S, C> {
        /// Create a new MERGE INTO statement
        pub fn new(target: T, source: S, condition: C) -> Self {
            Self {
                target,
                source,
                condition,
            }
        }
    }

    impl<T, S, C> QueryFragment<GaussDB> for MergeInto<T, S, C>
    where
        T: QueryFragment<GaussDB>,
        S: QueryFragment<GaussDB>,
        C: QueryFragment<GaussDB>,
    {
        fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
            out.push_sql("MERGE INTO ");
            self.target.walk_ast(out.reborrow())?;
            out.push_sql(" USING ");
            self.source.walk_ast(out.reborrow())?;
            out.push_sql(" ON (");
            self.condition.walk_ast(out.reborrow())?;
            out.push_sql(")");
            Ok(())
        }
    }
}

/// GaussDB-specific data types and type extensions
pub mod types {

    /// The `CLOB` (Character Large Object) type
    ///
    /// This type is used for storing large text data in GaussDB.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Clob;

    /// The `BLOB` (Binary Large Object) type
    ///
    /// This type is used for storing large binary data in GaussDB.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Blob;

    /// The `RAW` type for binary data
    ///
    /// This type is used for storing raw binary data with a specified length.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Raw;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussdb_functions() {
        let _rownum = functions::Rownum;
        let _level = functions::Level;
        let _connect_by_root = functions::ConnectByRoot::new("column_name");
        let _sys_connect_by_path = functions::SysConnectByPath::new("name", "/");
    }

    #[test]
    fn test_gaussdb_clauses() {
        let _start_with = clauses::StartWith::new("manager_id IS NULL");
        let _connect_by = clauses::ConnectBy::new("employee_id = manager_id");
        let _connect_by_prior = clauses::ConnectBy::prior("employee_id = manager_id");
        let _merge_into = clauses::MergeInto::new("target", "source", "t.id = s.id");
    }

    #[test]
    fn test_gaussdb_types() {
        let _clob = types::Clob;
        let _blob = types::Blob;
        let _raw = types::Raw;
    }

    #[test]
    fn test_connect_by_root_creation() {
        let _connect_by_root = functions::ConnectByRoot::new("employee_name");
        // Test that the structure can be created without errors
    }

    #[test]
    fn test_sys_connect_by_path_creation() {
        let _sys_connect_by_path = functions::SysConnectByPath::new("name", "/");
        // Test that the structure can be created without errors
    }

    #[test]
    fn test_start_with_creation() {
        let _start_with = clauses::StartWith::new("manager_id IS NULL");
        // Test that the structure can be created without errors
    }

    #[test]
    fn test_connect_by_creation() {
        let _connect_by = clauses::ConnectBy::new("employee_id = manager_id");
        let _connect_by_prior = clauses::ConnectBy::prior("employee_id = manager_id");
        // Test that both structures can be created without errors
    }

    #[test]
    fn test_merge_into_creation() {
        let _merge_into = clauses::MergeInto::new("target_table", "source_table", "t.id = s.id");
        // Test that the structure can be created without errors
    }

    #[test]
    fn test_gaussdb_functions_debug() {
        // Test that all functions implement Debug trait
        use std::fmt::Debug;

        let rownum = functions::Rownum;
        let level = functions::Level;
        let connect_by_root = functions::ConnectByRoot::new("test");
        let sys_connect_by_path = functions::SysConnectByPath::new("test", "/");

        let _: &dyn Debug = &rownum;
        let _: &dyn Debug = &level;
        let _: &dyn Debug = &connect_by_root;
        let _: &dyn Debug = &sys_connect_by_path;
    }

    #[test]
    fn test_gaussdb_clauses_debug() {
        // Test that all clauses implement Debug trait
        use std::fmt::Debug;

        let start_with = clauses::StartWith::new("test");
        let connect_by = clauses::ConnectBy::new("test");
        let merge_into = clauses::MergeInto::new("target", "source", "condition");

        let _: &dyn Debug = &start_with;
        let _: &dyn Debug = &connect_by;
        let _: &dyn Debug = &merge_into;
    }
}
