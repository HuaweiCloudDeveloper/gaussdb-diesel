//! Date and time functions for GaussDB
//!
//! This module provides PostgreSQL-compatible date and time functions
//! for GaussDB, including timestamp functions, date operations, and
//! time zone handling.

use crate::backend::GaussDB;
use crate::types::sql_types::Timestamptz;
use diesel::expression::{
    AppearsOnTable, AsExpression, Expression, SelectableExpression,
    ValidGrouping,
};
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::result::QueryResult;
use diesel::sql_types::{Date, Nullable, Time, Timestamp};

/// Represents the SQL `NOW()` function.
///
/// This function returns the current timestamp with time zone.
/// It's equivalent to `CURRENT_TIMESTAMP` in PostgreSQL.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::now;
/// // SELECT NOW()
/// let current_time = now;
/// ```
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, QueryId, ValidGrouping)]
pub struct now;

impl Expression for now {
    type SqlType = Timestamptz;
}

impl QueryFragment<GaussDB> for now {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("NOW()");
        Ok(())
    }
}

impl<QS> SelectableExpression<QS> for now {}
impl<QS> AppearsOnTable<QS> for now {}

// Note: Type coercion implementations would go here
// For now, we keep the basic Timestamptz type

/// Represents the SQL `CURRENT_TIMESTAMP` constant.
///
/// This is equivalent to the `NOW()` function and returns the current
/// timestamp with time zone.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::current_timestamp;
/// // SELECT CURRENT_TIMESTAMP
/// let current_time = current_timestamp;
/// ```
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, QueryId, ValidGrouping)]
pub struct current_timestamp;

impl Expression for current_timestamp {
    type SqlType = Timestamptz;
}

impl QueryFragment<GaussDB> for current_timestamp {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("CURRENT_TIMESTAMP");
        Ok(())
    }
}

impl<QS> SelectableExpression<QS> for current_timestamp {}
impl<QS> AppearsOnTable<QS> for current_timestamp {}

// Note: Type coercion implementations would go here
// For now, we keep the basic Timestamptz type

/// Represents the SQL `CURRENT_DATE` constant.
///
/// Returns the current date (without time).
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::current_date;
/// // SELECT CURRENT_DATE
/// let today = current_date;
/// ```
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, QueryId, ValidGrouping)]
pub struct current_date;

impl Expression for current_date {
    type SqlType = Date;
}

impl QueryFragment<GaussDB> for current_date {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("CURRENT_DATE");
        Ok(())
    }
}

impl<QS> SelectableExpression<QS> for current_date {}
impl<QS> AppearsOnTable<QS> for current_date {}

// Note: Type coercion implementations would go here
// For now, we keep the basic Date type

/// Represents the SQL `CURRENT_TIME` constant.
///
/// Returns the current time (without date).
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::current_time;
/// // SELECT CURRENT_TIME
/// let now_time = current_time;
/// ```
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, QueryId, ValidGrouping)]
pub struct current_time;

impl Expression for current_time {
    type SqlType = Time;
}

impl QueryFragment<GaussDB> for current_time {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("CURRENT_TIME");
        Ok(())
    }
}

impl<QS> SelectableExpression<QS> for current_time {}
impl<QS> AppearsOnTable<QS> for current_time {}

// Note: Type coercion implementations would go here
// For now, we keep the basic Time type

/// Creates a PostgreSQL `EXTRACT(field FROM source)` expression.
///
/// Extracts a specific field from a date/time value.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::extract;
/// # use diesel::sql_types::Timestamp;
/// // EXTRACT(YEAR FROM timestamp_col)
/// let year = extract("YEAR", diesel::dsl::sql::<Timestamp>("'2023-12-25'"));
/// ```
pub fn extract<T>(field: &str, source: T) -> ExtractFunction<T::Expression>
where
    T: AsExpression<Timestamp>,
{
    ExtractFunction::new(field.to_string(), source.as_expression())
}

/// PostgreSQL `EXTRACT` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct ExtractFunction<Expr> {
    field: String,
    source: Expr,
}

impl<Expr> ExtractFunction<Expr> {
    fn new(field: String, source: Expr) -> Self {
        ExtractFunction { field, source }
    }
}

impl<Expr> Expression for ExtractFunction<Expr>
where
    Expr: Expression,
{
    type SqlType = diesel::sql_types::Double;
}

impl<Expr> QueryFragment<GaussDB> for ExtractFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("EXTRACT(");
        out.push_sql(&self.field);
        out.push_sql(" FROM ");
        self.source.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for ExtractFunction<Expr>
where
    ExtractFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for ExtractFunction<Expr>
where
    Expr: AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `DATE_PART(field, source)` expression.
///
/// This is equivalent to `EXTRACT` but uses function syntax.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::date_part;
/// # use diesel::sql_types::Timestamp;
/// // DATE_PART('month', timestamp_col)
/// let month = date_part("month", diesel::dsl::sql::<Timestamp>("'2023-12-25'"));
/// ```
pub fn date_part<T>(field: &str, source: T) -> DatePartFunction<T::Expression>
where
    T: AsExpression<Timestamp>,
{
    DatePartFunction::new(field.to_string(), source.as_expression())
}

/// PostgreSQL `DATE_PART` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct DatePartFunction<Expr> {
    field: String,
    source: Expr,
}

impl<Expr> DatePartFunction<Expr> {
    fn new(field: String, source: Expr) -> Self {
        DatePartFunction { field, source }
    }
}

impl<Expr> Expression for DatePartFunction<Expr>
where
    Expr: Expression,
{
    type SqlType = diesel::sql_types::Double;
}

impl<Expr> QueryFragment<GaussDB> for DatePartFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("DATE_PART('");
        out.push_sql(&self.field);
        out.push_sql("', ");
        self.source.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for DatePartFunction<Expr>
where
    DatePartFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for DatePartFunction<Expr>
where
    Expr: AppearsOnTable<QS>,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::{Double, Timestamp};

    #[test]
    fn test_now_function() {
        let now_expr = now;
        let debug_str = format!("{:?}", now_expr);
        assert!(debug_str.contains("now"));
        
        // Test that it implements Expression with correct type
        fn assert_timestamptz_expr<T: Expression<SqlType = Timestamptz>>(_: T) {}
        assert_timestamptz_expr(now);
    }

    #[test]
    fn test_current_timestamp_function() {
        let current_expr = current_timestamp;
        let debug_str = format!("{:?}", current_expr);
        assert!(debug_str.contains("current_timestamp"));
        
        // Test that it implements Expression with correct type
        fn assert_timestamptz_expr<T: Expression<SqlType = Timestamptz>>(_: T) {}
        assert_timestamptz_expr(current_timestamp);
    }

    #[test]
    fn test_current_date_function() {
        let date_expr = current_date;
        let debug_str = format!("{:?}", date_expr);
        assert!(debug_str.contains("current_date"));
        
        // Test that it implements Expression with correct type
        fn assert_date_expr<T: Expression<SqlType = Date>>(_: T) {}
        assert_date_expr(current_date);
    }

    #[test]
    fn test_current_time_function() {
        let time_expr = current_time;
        let debug_str = format!("{:?}", time_expr);
        assert!(debug_str.contains("current_time"));
        
        // Test that it implements Expression with correct type
        fn assert_time_expr<T: Expression<SqlType = Time>>(_: T) {}
        assert_time_expr(current_time);
    }

    #[test]
    fn test_extract_function() {
        let timestamp_expr = diesel::dsl::sql::<Timestamp>("'2023-12-25'");
        let extract_expr = extract("YEAR", timestamp_expr);
        let debug_str = format!("{:?}", extract_expr);
        assert!(debug_str.contains("ExtractFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_double_expr<T: Expression<SqlType = Double>>(_: T) {}
        assert_double_expr(extract_expr);
    }

    #[test]
    fn test_date_part_function() {
        let timestamp_expr = diesel::dsl::sql::<Timestamp>("'2023-12-25'");
        let date_part_expr = date_part("month", timestamp_expr);
        let debug_str = format!("{:?}", date_part_expr);
        assert!(debug_str.contains("DatePartFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_double_expr<T: Expression<SqlType = Double>>(_: T) {}
        assert_double_expr(date_part_expr);
    }
}
