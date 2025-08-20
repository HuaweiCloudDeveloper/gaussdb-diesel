//! String functions for GaussDB
//!
//! This module provides PostgreSQL-compatible string functions
//! for GaussDB, including text manipulation, pattern matching,
//! and string operations.

use crate::backend::GaussDB;
use diesel::expression::{
    AppearsOnTable, AsExpression, Expression, SelectableExpression,
    ValidGrouping,
};
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::result::QueryResult;
use diesel::sql_types::{Integer, Nullable, Text};

/// Creates a PostgreSQL `LENGTH(string)` expression.
///
/// Returns the number of characters in the string.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::length;
/// # use diesel::sql_types::Text;
/// // LENGTH('hello')
/// let len = length(diesel::dsl::sql::<Text>("'hello'"));
/// ```
pub fn length<T>(string: T) -> LengthFunction<T::Expression>
where
    T: AsExpression<Text>,
{
    LengthFunction::new(string.as_expression())
}

/// PostgreSQL `LENGTH` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct LengthFunction<Expr> {
    string: Expr,
}

impl<Expr> LengthFunction<Expr> {
    fn new(string: Expr) -> Self {
        LengthFunction { string }
    }
}

impl<Expr> Expression for LengthFunction<Expr>
where
    Expr: Expression<SqlType = Text>,
{
    type SqlType = Nullable<Integer>;
}

impl<Expr> QueryFragment<GaussDB> for LengthFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("LENGTH(");
        self.string.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for LengthFunction<Expr>
where
    LengthFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for LengthFunction<Expr>
where
    Expr: Expression<SqlType = Text> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `UPPER(string)` expression.
///
/// Converts the string to uppercase.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::upper;
/// # use diesel::sql_types::Text;
/// // UPPER('hello')
/// let upper_str = upper(diesel::dsl::sql::<Text>("'hello'"));
/// ```
pub fn upper<T>(string: T) -> UpperFunction<T::Expression>
where
    T: AsExpression<Text>,
{
    UpperFunction::new(string.as_expression())
}

/// PostgreSQL `UPPER` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct UpperFunction<Expr> {
    string: Expr,
}

impl<Expr> UpperFunction<Expr> {
    fn new(string: Expr) -> Self {
        UpperFunction { string }
    }
}

impl<Expr> Expression for UpperFunction<Expr>
where
    Expr: Expression<SqlType = Text>,
{
    type SqlType = Text;
}

impl<Expr> QueryFragment<GaussDB> for UpperFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("UPPER(");
        self.string.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for UpperFunction<Expr>
where
    UpperFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for UpperFunction<Expr>
where
    Expr: Expression<SqlType = Text> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `LOWER(string)` expression.
///
/// Converts the string to lowercase.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::lower;
/// # use diesel::sql_types::Text;
/// // LOWER('HELLO')
/// let lower_str = lower(diesel::dsl::sql::<Text>("'HELLO'"));
/// ```
pub fn lower<T>(string: T) -> LowerFunction<T::Expression>
where
    T: AsExpression<Text>,
{
    LowerFunction::new(string.as_expression())
}

/// PostgreSQL `LOWER` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct LowerFunction<Expr> {
    string: Expr,
}

impl<Expr> LowerFunction<Expr> {
    fn new(string: Expr) -> Self {
        LowerFunction { string }
    }
}

impl<Expr> Expression for LowerFunction<Expr>
where
    Expr: Expression<SqlType = Text>,
{
    type SqlType = Text;
}

impl<Expr> QueryFragment<GaussDB> for LowerFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("LOWER(");
        self.string.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for LowerFunction<Expr>
where
    LowerFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for LowerFunction<Expr>
where
    Expr: Expression<SqlType = Text> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `TRIM(string)` expression.
///
/// Removes leading and trailing whitespace from the string.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::trim;
/// # use diesel::sql_types::Text;
/// // TRIM('  hello  ')
/// let trimmed = trim(diesel::dsl::sql::<Text>("'  hello  '"));
/// ```
pub fn trim<T>(string: T) -> TrimFunction<T::Expression>
where
    T: AsExpression<Text>,
{
    TrimFunction::new(string.as_expression())
}

/// PostgreSQL `TRIM` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct TrimFunction<Expr> {
    string: Expr,
}

impl<Expr> TrimFunction<Expr> {
    fn new(string: Expr) -> Self {
        TrimFunction { string }
    }
}

impl<Expr> Expression for TrimFunction<Expr>
where
    Expr: Expression<SqlType = Text>,
{
    type SqlType = Text;
}

impl<Expr> QueryFragment<GaussDB> for TrimFunction<Expr>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("TRIM(");
        self.string.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for TrimFunction<Expr>
where
    TrimFunction<Expr>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for TrimFunction<Expr>
where
    Expr: Expression<SqlType = Text> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `SUBSTRING(string FROM start FOR length)` expression.
///
/// Extracts a substring from the string.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::substring;
/// # use diesel::sql_types::Text;
/// // SUBSTRING('hello' FROM 2 FOR 3)
/// let substr = substring(diesel::dsl::sql::<Text>("'hello'"), 2, 3);
/// ```
pub fn substring<T, S, L>(string: T, start: S, length: L) -> SubstringFunction<T::Expression, S::Expression, L::Expression>
where
    T: AsExpression<Text>,
    S: AsExpression<Integer>,
    L: AsExpression<Integer>,
{
    SubstringFunction::new(string.as_expression(), start.as_expression(), length.as_expression())
}

/// PostgreSQL `SUBSTRING` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct SubstringFunction<Str, Start, Len> {
    string: Str,
    start: Start,
    length: Len,
}

impl<Str, Start, Len> SubstringFunction<Str, Start, Len> {
    fn new(string: Str, start: Start, length: Len) -> Self {
        SubstringFunction { string, start, length }
    }
}

impl<Str, Start, Len> Expression for SubstringFunction<Str, Start, Len>
where
    Str: Expression<SqlType = Text>,
    Start: Expression<SqlType = Integer>,
    Len: Expression<SqlType = Integer>,
{
    type SqlType = Text;
}

impl<Str, Start, Len> QueryFragment<GaussDB> for SubstringFunction<Str, Start, Len>
where
    Str: QueryFragment<GaussDB>,
    Start: QueryFragment<GaussDB>,
    Len: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("SUBSTRING(");
        self.string.walk_ast(out.reborrow())?;
        out.push_sql(" FROM ");
        self.start.walk_ast(out.reborrow())?;
        out.push_sql(" FOR ");
        self.length.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<Str, Start, Len, QS> SelectableExpression<QS> for SubstringFunction<Str, Start, Len>
where
    SubstringFunction<Str, Start, Len>: AppearsOnTable<QS>,
{
}

impl<Str, Start, Len, QS> AppearsOnTable<QS> for SubstringFunction<Str, Start, Len>
where
    Str: Expression<SqlType = Text> + AppearsOnTable<QS>,
    Start: Expression<SqlType = Integer> + AppearsOnTable<QS>,
    Len: Expression<SqlType = Integer> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `CONCAT(string1, string2, ...)` expression.
///
/// Concatenates multiple strings together.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::concat;
/// # use diesel::sql_types::Text;
/// // CONCAT('Hello', ' ', 'World')
/// let concatenated = concat(vec![
///     diesel::dsl::sql::<Text>("'Hello'"),
///     diesel::dsl::sql::<Text>("' '"),
///     diesel::dsl::sql::<Text>("'World'")
/// ]);
/// ```
pub fn concat<T>(strings: Vec<T>) -> ConcatFunction<Vec<T::Expression>>
where
    T: AsExpression<Text>,
{
    let expressions = strings.into_iter().map(|s| s.as_expression()).collect();
    ConcatFunction::new(expressions)
}

/// PostgreSQL `CONCAT` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct ConcatFunction<Expr> {
    strings: Expr,
}

impl<Expr> ConcatFunction<Expr> {
    fn new(strings: Expr) -> Self {
        ConcatFunction { strings }
    }
}

impl<Expr> Expression for ConcatFunction<Vec<Expr>>
where
    Expr: Expression<SqlType = Text>,
{
    type SqlType = Text;
}

impl<Expr> QueryFragment<GaussDB> for ConcatFunction<Vec<Expr>>
where
    Expr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("CONCAT(");
        for (i, string) in self.strings.iter().enumerate() {
            if i > 0 {
                out.push_sql(", ");
            }
            string.walk_ast(out.reborrow())?;
        }
        out.push_sql(")");
        Ok(())
    }
}

impl<Expr, QS> SelectableExpression<QS> for ConcatFunction<Vec<Expr>>
where
    ConcatFunction<Vec<Expr>>: AppearsOnTable<QS>,
{
}

impl<Expr, QS> AppearsOnTable<QS> for ConcatFunction<Vec<Expr>>
where
    Expr: Expression<SqlType = Text> + AppearsOnTable<QS>,
{
}

/// Creates a PostgreSQL `POSITION(substring IN string)` expression.
///
/// Returns the position of the first occurrence of substring in string.
///
/// # Examples
///
/// ```rust
/// # use diesel_gaussdb::expression::functions::position;
/// # use diesel::sql_types::Text;
/// // POSITION('world' IN 'hello world')
/// let pos = position(
///     diesel::dsl::sql::<Text>("'world'"),
///     diesel::dsl::sql::<Text>("'hello world'")
/// );
/// ```
pub fn position<T, U>(substring: T, string: U) -> PositionFunction<T::Expression, U::Expression>
where
    T: AsExpression<Text>,
    U: AsExpression<Text>,
{
    PositionFunction::new(substring.as_expression(), string.as_expression())
}

/// PostgreSQL `POSITION` function
#[derive(Debug, Clone, QueryId, ValidGrouping)]
pub struct PositionFunction<SubExpr, StrExpr> {
    substring: SubExpr,
    string: StrExpr,
}

impl<SubExpr, StrExpr> PositionFunction<SubExpr, StrExpr> {
    fn new(substring: SubExpr, string: StrExpr) -> Self {
        PositionFunction { substring, string }
    }
}

impl<SubExpr, StrExpr> Expression for PositionFunction<SubExpr, StrExpr>
where
    SubExpr: Expression<SqlType = Text>,
    StrExpr: Expression<SqlType = Text>,
{
    type SqlType = Integer;
}

impl<SubExpr, StrExpr> QueryFragment<GaussDB> for PositionFunction<SubExpr, StrExpr>
where
    SubExpr: QueryFragment<GaussDB>,
    StrExpr: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.push_sql("POSITION(");
        self.substring.walk_ast(out.reborrow())?;
        out.push_sql(" IN ");
        self.string.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl<SubExpr, StrExpr, QS> SelectableExpression<QS> for PositionFunction<SubExpr, StrExpr>
where
    PositionFunction<SubExpr, StrExpr>: AppearsOnTable<QS>,
{
}

impl<SubExpr, StrExpr, QS> AppearsOnTable<QS> for PositionFunction<SubExpr, StrExpr>
where
    SubExpr: Expression<SqlType = Text> + AppearsOnTable<QS>,
    StrExpr: Expression<SqlType = Text> + AppearsOnTable<QS>,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::{Integer, Text};

    #[test]
    fn test_length_function() {
        let text_expr = diesel::dsl::sql::<Text>("'hello'");
        let length_expr = length(text_expr);
        let debug_str = format!("{:?}", length_expr);
        assert!(debug_str.contains("LengthFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_nullable_integer_expr<T: Expression<SqlType = Nullable<Integer>>>(_: T) {}
        assert_nullable_integer_expr(length_expr);
    }

    #[test]
    fn test_upper_function() {
        let text_expr = diesel::dsl::sql::<Text>("'hello'");
        let upper_expr = upper(text_expr);
        let debug_str = format!("{:?}", upper_expr);
        assert!(debug_str.contains("UpperFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_text_expr<T: Expression<SqlType = Text>>(_: T) {}
        assert_text_expr(upper_expr);
    }

    #[test]
    fn test_lower_function() {
        let text_expr = diesel::dsl::sql::<Text>("'HELLO'");
        let lower_expr = lower(text_expr);
        let debug_str = format!("{:?}", lower_expr);
        assert!(debug_str.contains("LowerFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_text_expr<T: Expression<SqlType = Text>>(_: T) {}
        assert_text_expr(lower_expr);
    }

    #[test]
    fn test_trim_function() {
        let text_expr = diesel::dsl::sql::<Text>("'  hello  '");
        let trim_expr = trim(text_expr);
        let debug_str = format!("{:?}", trim_expr);
        assert!(debug_str.contains("TrimFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_text_expr<T: Expression<SqlType = Text>>(_: T) {}
        assert_text_expr(trim_expr);
    }

    #[test]
    fn test_substring_function() {
        let text_expr = diesel::dsl::sql::<Text>("'hello'");
        let substring_expr = substring(text_expr, 2, 3);
        let debug_str = format!("{:?}", substring_expr);
        assert!(debug_str.contains("SubstringFunction"));
        
        // Test that it implements Expression with correct type
        fn assert_text_expr<T: Expression<SqlType = Text>>(_: T) {}
        assert_text_expr(substring_expr);
    }
}
