//! Mathematical functions for GaussDB
//!
//! This module provides PostgreSQL-compatible mathematical functions
//! for GaussDB, including arithmetic operations, trigonometric functions,
//! and statistical functions.

use crate::backend::GaussDB;
use diesel::expression::{
    AppearsOnTable, AsExpression, Expression, SelectableExpression,
    ValidGrouping,
};
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::result::QueryResult;
use diesel::sql_types::{Double, Integer, Nullable};

/// Creates a PostgreSQL `ABS(number)` expression.
///
/// Returns the absolute value of the number.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::abs;
/// # use diesel::sql_types::Integer;
/// // ABS(-5)
/// let absolute = abs(diesel::dsl::sql::<Integer>("-5"));
/// ```
pub fn abs<T>(number: T) -> AbsFunction<T::Expression>
where
    T: AsExpression<Integer>,
{
    AbsFunction::new(number.as_expression())
}

/// PostgreSQL `ABS` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct AbsFunction<Expr> {
    number: Expr,
}

impl<Expr> AbsFunction<Expr> {
    fn new(number: Expr) -> Self {
        AbsFunction { number }
    }
}

impl<Expr> Expression for AbsFunction<Expr>
where
    Expr: Expression<SqlType = Integer>,
{
    type SqlType = Integer;
}

impl<Expr> QueryFragment<GaussDB> for AbsFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("ABS(");
        self.number.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for AbsFunction<Expr>
where
    AbsFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for AbsFunction<Expr>
where
    Expr: Expression<SqlType = Integer> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `CEIL(number)` expression.
///
/// Returns the smallest integer greater than or equal to the number.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::ceil;
/// # use diesel::sql_types::Double;
/// // CEIL(4.2)
/// let ceiling = ceil(diesel::dsl::sql::<Double>("4.2"));
/// ```
pub fn ceil<T>(number: T) -> CeilFunction<T::Expression>
where
    T: AsExpression<Double>,
{
    CeilFunction::new(number.as_expression())
}

/// PostgreSQL `CEIL` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct CeilFunction<Expr> {
    number: Expr,
}

impl<Expr> CeilFunction<Expr> {
    fn new(number: Expr) -> Self {
        CeilFunction { number }
    }
}

impl<Expr> Expression for CeilFunction<Expr>
where
    Expr: Expression<SqlType = Double>,
{
    type SqlType = Double;
}

impl<Expr> QueryFragment<GaussDB> for CeilFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("CEIL(");
        self.number.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for CeilFunction<Expr>
where
    CeilFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for CeilFunction<Expr>
where
    Expr: Expression<SqlType = Double> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `FLOOR(number)` expression.
///
/// Returns the largest integer less than or equal to the number.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::floor;
/// # use diesel::sql_types::Double;
/// // FLOOR(4.8)
/// let floored = floor(diesel::dsl::sql::<Double>("4.8"));
/// ```
pub fn floor<T>(number: T) -> FloorFunction<T::Expression>
where
    T: AsExpression<Double>,
{
    FloorFunction::new(number.as_expression())
}

/// PostgreSQL `FLOOR` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct FloorFunction<Expr> {
    number: Expr,
}

impl<Expr> FloorFunction<Expr> {
    fn new(number: Expr) -> Self {
        FloorFunction { number }
    }
}

impl<Expr> Expression for FloorFunction<Expr>
where
    Expr: Expression<SqlType = Double>,
{
    type SqlType = Double;
}

impl<Expr> QueryFragment<GaussDB> for FloorFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("FLOOR(");
        self.number.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for FloorFunction<Expr>
where
    FloorFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for FloorFunction<Expr>
where
    Expr: Expression<SqlType = Double> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `ROUND(number, precision)` expression.
///
/// Rounds the number to the specified number of decimal places.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::round;
/// # use diesel::sql_types::Double;
/// // ROUND(4.567, 2)
/// let rounded = round(diesel::dsl::sql::<Double>("4.567"), 2);
/// ```
pub fn round<T, P>(number: T, precision: P) -> RoundFunction<T::Expression, P::Expression>
where
    T: AsExpression<Double>,
    P: AsExpression<Integer>,
{
    RoundFunction::new(number.as_expression(), precision.as_expression())
}

/// PostgreSQL `ROUND` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct RoundFunction<Num, Prec> {
    number: Num,
    precision: Prec,
}

impl<Num, Prec> RoundFunction<Num, Prec> {
    fn new(number: Num, precision: Prec) -> Self {
        RoundFunction { number, precision }
    }
}

impl<Num, Prec> Expression for RoundFunction<Num, Prec>
where
    Num: Expression<SqlType = Double>,
    Prec: Expression<SqlType = Integer>,
{
    type SqlType = Double;
}

impl<Num, Prec> QueryFragment<GaussDB> for RoundFunction<Num, Prec>
where
    Num: QueryFragment<GaussDB>,
    Prec: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("ROUND(");
        self.number.walk_ast(out.reborrow())?;
        out.push_sql(", ");
        self.precision.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Num, Prec, QS> SelectableExpression<QS> for RoundFunction<Num, Prec>
where
    RoundFunction<Num, Prec>: AppearsOnTable<QS>,
{
}

impl<Num, Prec, QS> AppearsOnTable<QS> for RoundFunction<Num, Prec>
where
    Num: Expression<SqlType = Double> + AppearsOnTable<QS>,
    Prec: Expression<SqlType = Integer> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `SQRT(number)` expression.
///
/// Returns the square root of the number.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::sqrt;
/// # use diesel::sql_types::Double;
/// // SQRT(16)
/// let square_root = sqrt(diesel::dsl::sql::<Double>("16"));
/// ```
pub fn sqrt<T>(number: T) -> SqrtFunction<T::Expression>
where
    T: AsExpression<Double>,
{
    SqrtFunction::new(number.as_expression())
}

/// PostgreSQL `SQRT` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct SqrtFunction<Expr> {
    number: Expr,
}

impl<Expr> SqrtFunction<Expr> {
    fn new(number: Expr) -> Self {
        SqrtFunction { number }
    }
}

impl<Expr> Expression for SqrtFunction<Expr>
where
    Expr: Expression<SqlType = Double>,
{
    type SqlType = Double;
}

impl<Expr> QueryFragment<GaussDB> for SqrtFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("SQRT(");
        self.number.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for SqrtFunction<Expr>
where
    SqrtFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for SqrtFunction<Expr>
where
    Expr: Expression<SqlType = Double> + AppearsOnTable<QS>,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::{Double, Integer};

    #[test]
    fn test_abs_function() {
        let int_expr = diesel::dsl::sql::<Integer>("-5");
        let abs_expr = abs(int_expr);
        let debug_str = format!("{:?}", abs_expr);
        assert!(debug_str.contains("AbsFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_integer_expr<T: Expression<SqlType = Integer>>(_: T) {}
        assert_integer_expr(abs_expr);
    }

    #[test]
    fn test_ceil_function() {
        let double_expr = diesel::dsl::sql::<Double>("4.2");
        let ceil_expr = ceil(double_expr);
        let debug_str = format!("{:?}", ceil_expr);
        assert!(debug_str.contains("CeilFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_double_expr<T: Expression<SqlType = Double>>(_: T) {}
        assert_double_expr(ceil_expr);
    }

    #[test]
    fn test_floor_function() {
        let double_expr = diesel::dsl::sql::<Double>("4.8");
        let floor_expr = floor(double_expr);
        let debug_str = format!("{:?}", floor_expr);
        assert!(debug_str.contains("FloorFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_double_expr<T: Expression<SqlType = Double>>(_: T) {}
        assert_double_expr(floor_expr);
    }

    #[test]
    fn test_round_function() {
        let double_expr = diesel::dsl::sql::<Double>("4.567");
        let round_expr = round(double_expr, 2);
        let debug_str = format!("{:?}", round_expr);
        assert!(debug_str.contains("RoundFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_double_expr<T: Expression<SqlType = Double>>(_: T) {}
        assert_double_expr(round_expr);
    }

    #[test]
    fn test_sqrt_function() {
        let double_expr = diesel::dsl::sql::<Double>("16");
        let sqrt_expr = sqrt(double_expr);
        let debug_str = format!("{:?}", sqrt_expr);
        assert!(debug_str.contains("SqrtFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_double_expr<T: Expression<SqlType = Double>>(_: T) {}
        assert_double_expr(sqrt_expr);
    }
}
