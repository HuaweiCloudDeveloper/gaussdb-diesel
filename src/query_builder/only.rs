//! ONLY clause support for GaussDB
//!
//! This module provides support for PostgreSQL's ONLY clause, which is used
//! to query only the specified table and not its inheritance children.

use crate::backend::GaussDB;
use diesel::expression::{Expression, ValidGrouping};
use diesel::query_builder::{AsQuery, AstPass, FromClause, QueryFragment, QueryId, SelectStatement};
use diesel::query_source::QuerySource;
use diesel::result::QueryResult;
use diesel::{JoinTo, SelectableExpression, Table};

/// Represents a query with an `ONLY` clause.
///
/// The ONLY clause is used in PostgreSQL (and GaussDB) to query only the specified
/// table and not its inheritance children. This is particularly useful when working
/// with table inheritance hierarchies.
///
/// # Example
///
/// ```rust
/// # use diesel_gaussdb::prelude::*;
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #     let mut conn = establish_connection();
/// use diesel_gaussdb::query_builder::only;
/// 
/// // Query only the users table, not any inherited tables
/// let results = only(users::table)
///     .select(users::all_columns)
///     .load::<User>(&mut conn)?;
/// #     Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Only<S> {
    pub(crate) source: S,
}

impl<S> QueryId for Only<S>
where
    Self: 'static,
    S: QueryId,
{
    type QueryId = Self;
    const HAS_STATIC_QUERY_ID: bool = <S as QueryId>::HAS_STATIC_QUERY_ID;
}

impl<S> QuerySource for Only<S>
where
    S: Table + Clone,
    <S as QuerySource>::DefaultSelection: ValidGrouping<()> + SelectableExpression<Only<S>>,
{
    type FromClause = Self;
    type DefaultSelection = <S as QuerySource>::DefaultSelection;

    fn from_clause(&self) -> Self::FromClause {
        self.clone()
    }

    fn default_selection(&self) -> Self::DefaultSelection {
        self.source.default_selection()
    }
}

impl<S> QueryFragment<GaussDB> for Only<S>
where
    S: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        pass.push_sql(" ONLY ");
        self.source.walk_ast(pass.reborrow())?;
        Ok(())
    }
}

impl<S> AsQuery for Only<S>
where
    S: Table + Clone,
    <S as QuerySource>::DefaultSelection: ValidGrouping<()> + SelectableExpression<Only<S>>,
{
    type SqlType = <<Self as QuerySource>::DefaultSelection as Expression>::SqlType;
    type Query = SelectStatement<FromClause<Self>>;

    fn as_query(self) -> Self::Query {
        SelectStatement::simple(self)
    }
}

impl<S, T> JoinTo<T> for Only<S>
where
    S: JoinTo<T>,
    T: Table,
    S: Table,
{
    type FromClause = <S as JoinTo<T>>::FromClause;
    type OnClause = <S as JoinTo<T>>::OnClause;

    fn join_target(rhs: T) -> (Self::FromClause, Self::OnClause) {
        <S as JoinTo<T>>::join_target(rhs)
    }
}

impl<S> Table for Only<S>
where
    S: Table + Clone + AsQuery,
    <S as Table>::PrimaryKey: SelectableExpression<Only<S>>,
    <S as Table>::AllColumns: SelectableExpression<Only<S>>,
    <S as QuerySource>::DefaultSelection: ValidGrouping<()> + SelectableExpression<Only<S>>,
{
    type PrimaryKey = <S as Table>::PrimaryKey;
    type AllColumns = <S as Table>::AllColumns;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.source.primary_key()
    }

    fn all_columns() -> Self::AllColumns {
        S::all_columns()
    }
}

/// Creates an ONLY clause for the given table.
///
/// This function creates an ONLY clause that can be used to query only the
/// specified table and not its inheritance children.
///
/// # Example
///
/// ```rust
/// # use diesel_gaussdb::prelude::*;
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// use diesel_gaussdb::query_builder::only;
/// 
/// let query = only(users::table).select(users::all_columns);
/// ```
pub fn only<T>(source: T) -> Only<T> {
    Only { source }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::GaussDB;
    use diesel::query_builder::QueryBuilder;

    #[test]
    fn test_only_clause_sql_generation() {
        // Create a mock table for testing
        #[derive(Debug, Clone, Copy)]
        struct MockTable;
        
        impl QueryFragment<GaussDB> for MockTable {
            fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
                pass.push_sql("users");
                Ok(())
            }
        }
        
        let only_clause = Only { source: MockTable };
        let mut query_builder = <GaussDB as diesel::backend::Backend>::QueryBuilder::default();
        only_clause.to_sql(&mut query_builder, &GaussDB).unwrap();
        
        assert_eq!(query_builder.finish(), " ONLY users");
    }

    #[test]
    fn test_only_function() {
        #[derive(Debug, Clone, Copy)]
        struct MockTable;
        
        let only_clause = only(MockTable);
        // Test that the function creates the correct structure
        assert!(matches!(only_clause, Only { source: MockTable }));
    }

    #[test]
    fn test_only_query_id() {
        #[derive(Debug, Clone, Copy)]
        struct MockTable;
        
        impl QueryId for MockTable {
            type QueryId = Self;
            const HAS_STATIC_QUERY_ID: bool = true;
        }
        
        let only_clause = Only { source: MockTable };
        assert_eq!(Only::<MockTable>::HAS_STATIC_QUERY_ID, true);
    }
}
