//! Table sampling support for GaussDB
//!
//! This module provides support for PostgreSQL's TABLESAMPLE clause, which allows
//! you to retrieve a random sample of rows from a table.

use crate::backend::GaussDB;
use diesel::expression::{Expression, ValidGrouping};
use diesel::query_builder::{AsQuery, AstPass, FromClause, QueryFragment, QueryId, SelectStatement};
use diesel::query_source::QuerySource;
use diesel::result::QueryResult;
use diesel::sql_types::{Double, SmallInt};
use diesel::{JoinTo, SelectableExpression, Table};
use std::marker::PhantomData;

/// Trait for table sampling methods
#[doc(hidden)]
pub trait TablesampleMethod: Clone {
    /// Returns the SQL name for this sampling method
    fn method_name_sql() -> &'static str;
}

/// Used to specify the `BERNOULLI` sampling method.
///
/// The BERNOULLI method scans the whole table and selects or ignores individual rows
/// independently with the specified probability. This method is slower but gives a
/// more random sample.
#[derive(Clone, Copy, Debug)]
pub struct BernoulliMethod;

impl TablesampleMethod for BernoulliMethod {
    fn method_name_sql() -> &'static str {
        "BERNOULLI"
    }
}

/// Used to specify the `SYSTEM` sampling method.
///
/// The SYSTEM method reads each physical storage page for the table and selects
/// or ignores it based on the specified probability. This method is faster but
/// may not give a truly random sample.
#[derive(Clone, Copy, Debug)]
pub struct SystemMethod;

impl TablesampleMethod for SystemMethod {
    fn method_name_sql() -> &'static str {
        "SYSTEM"
    }
}

/// Represents a query with a `TABLESAMPLE` clause.
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
/// use diesel_gaussdb::query_builder::tablesample::*;
/// 
/// // Get a 10% sample using BERNOULLI method
/// let sample = users::table
///     .tablesample_bernoulli(10)
///     .select(users::all_columns)
///     .load::<User>(&mut conn)?;
/// 
/// // Get a 5% sample using SYSTEM method with a seed for repeatability
/// let repeatable_sample = users::table
///     .tablesample_system(5)
///     .with_seed(42.0)
///     .select(users::all_columns)
///     .load::<User>(&mut conn)?;
/// #     Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Tablesample<S, TSM>
where
    TSM: TablesampleMethod,
{
    source: S,
    method: PhantomData<TSM>,
    portion: i16,
    seed: Option<f64>,
}

impl<S, TSM> Tablesample<S, TSM>
where
    TSM: TablesampleMethod,
{
    /// Creates a new tablesample clause with the specified portion percentage
    pub(crate) fn new(source: S, portion: i16) -> Tablesample<S, TSM> {
        Tablesample {
            source,
            method: PhantomData,
            portion,
            seed: None,
        }
    }

    /// Specifies the random number generator seed to use in the sampling method.
    /// 
    /// This allows you to obtain repeatable results across multiple queries.
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
    /// use diesel_gaussdb::query_builder::tablesample::*;
    /// 
    /// let query = users::table
    ///     .tablesample_bernoulli(10)
    ///     .with_seed(42.0);
    /// ```
    pub fn with_seed(self, seed: f64) -> Tablesample<S, TSM> {
        Tablesample {
            source: self.source,
            method: self.method,
            portion: self.portion,
            seed: Some(seed),
        }
    }
}

impl<S, TSM> QueryId for Tablesample<S, TSM>
where
    S: QueryId,
    TSM: TablesampleMethod,
{
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<S, TSM> QuerySource for Tablesample<S, TSM>
where
    S: Table + Clone,
    TSM: TablesampleMethod,
    <S as QuerySource>::DefaultSelection:
        ValidGrouping<()> + SelectableExpression<Tablesample<S, TSM>>,
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

impl<S, TSM> QueryFragment<GaussDB> for Tablesample<S, TSM>
where
    S: QueryFragment<GaussDB>,
    TSM: TablesampleMethod,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        self.source.walk_ast(out.reborrow())?;
        out.push_sql(" TABLESAMPLE ");
        out.push_sql(TSM::method_name_sql());
        out.push_sql("(");
        out.push_bind_param::<SmallInt, _>(&self.portion)?;
        out.push_sql(")");
        if let Some(f) = &self.seed {
            out.push_sql(" REPEATABLE(");
            out.push_bind_param::<Double, _>(f)?;
            out.push_sql(")");
        }
        Ok(())
    }
}

impl<S, TSM> AsQuery for Tablesample<S, TSM>
where
    S: Table + Clone,
    TSM: TablesampleMethod,
    <S as QuerySource>::DefaultSelection:
        ValidGrouping<()> + SelectableExpression<Tablesample<S, TSM>>,
{
    type SqlType = <<Self as QuerySource>::DefaultSelection as Expression>::SqlType;
    type Query = SelectStatement<FromClause<Self>>;
    
    fn as_query(self) -> Self::Query {
        SelectStatement::simple(self)
    }
}

impl<S, T, TSM> JoinTo<T> for Tablesample<S, TSM>
where
    S: JoinTo<T>,
    T: Table,
    S: Table,
    TSM: TablesampleMethod,
{
    type FromClause = <S as JoinTo<T>>::FromClause;
    type OnClause = <S as JoinTo<T>>::OnClause;

    fn join_target(rhs: T) -> (Self::FromClause, Self::OnClause) {
        <S as JoinTo<T>>::join_target(rhs)
    }
}

impl<S, TSM> Table for Tablesample<S, TSM>
where
    S: Table + Clone + AsQuery,
    TSM: TablesampleMethod,
    <S as Table>::PrimaryKey: SelectableExpression<Tablesample<S, TSM>>,
    <S as Table>::AllColumns: SelectableExpression<Tablesample<S, TSM>>,
    <S as QuerySource>::DefaultSelection:
        ValidGrouping<()> + SelectableExpression<Tablesample<S, TSM>>,
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

/// Creates a TABLESAMPLE clause using the BERNOULLI method
pub fn tablesample_bernoulli<S>(source: S, portion: i16) -> Tablesample<S, BernoulliMethod> {
    Tablesample::new(source, portion)
}

/// Creates a TABLESAMPLE clause using the SYSTEM method
pub fn tablesample_system<S>(source: S, portion: i16) -> Tablesample<S, SystemMethod> {
    Tablesample::new(source, portion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::GaussDB;
    use diesel::query_builder::QueryBuilder;

    #[test]
    fn test_bernoulli_method() {
        assert_eq!(BernoulliMethod::method_name_sql(), "BERNOULLI");
    }

    #[test]
    fn test_system_method() {
        assert_eq!(SystemMethod::method_name_sql(), "SYSTEM");
    }

    #[test]
    fn test_tablesample_sql_generation() {
        // Create a mock table for testing
        #[derive(Debug, Clone, Copy)]
        struct MockTable;
        
        impl QueryFragment<GaussDB> for MockTable {
            fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
                pass.push_sql("users");
                Ok(())
            }
        }
        
        let tablesample = Tablesample::<_, BernoulliMethod>::new(MockTable, 10);
        let mut query_builder = <GaussDB as diesel::backend::Backend>::QueryBuilder::default();
        tablesample.to_sql(&mut query_builder, &GaussDB).unwrap();
        
        let sql = query_builder.finish();
        assert!(sql.contains("users TABLESAMPLE BERNOULLI"));
    }

    #[test]
    fn test_tablesample_with_seed() {
        #[derive(Debug, Clone, Copy)]
        struct MockTable;
        
        impl QueryFragment<GaussDB> for MockTable {
            fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
                pass.push_sql("users");
                Ok(())
            }
        }
        
        let tablesample = Tablesample::<_, SystemMethod>::new(MockTable, 5).with_seed(42.0);
        let mut query_builder = <GaussDB as diesel::backend::Backend>::QueryBuilder::default();
        tablesample.to_sql(&mut query_builder, &GaussDB).unwrap();
        
        let sql = query_builder.finish();
        assert!(sql.contains("users TABLESAMPLE SYSTEM"));
        assert!(sql.contains("REPEATABLE"));
    }

    #[test]
    fn test_helper_functions() {
        #[derive(Debug, Clone, Copy)]
        struct MockTable;
        
        let bernoulli_sample = tablesample_bernoulli(MockTable, 10);
        assert!(matches!(bernoulli_sample.method, PhantomData::<BernoulliMethod>));
        
        let system_sample = tablesample_system(MockTable, 5);
        assert!(matches!(system_sample.method, PhantomData::<SystemMethod>));
    }
}
