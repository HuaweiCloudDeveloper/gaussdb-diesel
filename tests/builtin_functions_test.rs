//! Tests for built-in functions (string and math functions)

use diesel_gaussdb::expression::functions::{
    // String functions
    length, lower, substring, trim, upper,
    // Math functions
    abs, ceil, floor, round, sqrt,
    // Date/time functions
    current_date, current_time, current_timestamp, extract, date_part, now,
};
use diesel::sql_types::{Double, Integer, Text, Timestamp};

#[test]
fn test_string_functions_compilation() {
    // Test that string functions compile correctly
    let text_expr = diesel::dsl::sql::<Text>("'Hello World'");
    
    // Test length function
    let length_expr = length(text_expr.clone());
    let debug_str = format!("{:?}", length_expr);
    assert!(debug_str.contains("LengthFunction"));
    
    // Test upper function
    let upper_expr = upper(text_expr.clone());
    let debug_str = format!("{:?}", upper_expr);
    assert!(debug_str.contains("UpperFunction"));
    
    // Test lower function
    let lower_expr = lower(text_expr.clone());
    let debug_str = format!("{:?}", lower_expr);
    assert!(debug_str.contains("LowerFunction"));
    
    // Test trim function
    let trim_expr = trim(text_expr.clone());
    let debug_str = format!("{:?}", trim_expr);
    assert!(debug_str.contains("TrimFunction"));
    
    // Test substring function
    let substring_expr = substring(text_expr, 2, 5);
    let debug_str = format!("{:?}", substring_expr);
    assert!(debug_str.contains("SubstringFunction"));
}

#[test]
fn test_math_functions_compilation() {
    // Test that math functions compile correctly
    let int_expr = diesel::dsl::sql::<Integer>("42");
    let double_expr = diesel::dsl::sql::<Double>("3.14");
    
    // Test abs function
    let abs_expr = abs(int_expr);
    let debug_str = format!("{:?}", abs_expr);
    assert!(debug_str.contains("AbsFunction"));
    
    // Test ceil function
    let ceil_expr = ceil(double_expr.clone());
    let debug_str = format!("{:?}", ceil_expr);
    assert!(debug_str.contains("CeilFunction"));
    
    // Test floor function
    let floor_expr = floor(double_expr.clone());
    let debug_str = format!("{:?}", floor_expr);
    assert!(debug_str.contains("FloorFunction"));
    
    // Test round function
    let round_expr = round(double_expr.clone(), 2);
    let debug_str = format!("{:?}", round_expr);
    assert!(debug_str.contains("RoundFunction"));
    
    // Test sqrt function
    let sqrt_expr = sqrt(double_expr);
    let debug_str = format!("{:?}", sqrt_expr);
    assert!(debug_str.contains("SqrtFunction"));
}

#[test]
fn test_datetime_functions_compilation() {
    // Test that date/time functions compile correctly
    let timestamp_expr = diesel::dsl::sql::<Timestamp>("'2023-12-25 10:30:00'");
    
    // Test now function
    let now_expr = now;
    let debug_str = format!("{:?}", now_expr);
    assert!(debug_str.contains("now"));

    // Test current_timestamp function
    let current_timestamp_expr = current_timestamp;
    let debug_str = format!("{:?}", current_timestamp_expr);
    assert!(debug_str.contains("current_timestamp"));

    // Test current_date function
    let current_date_expr = current_date;
    let debug_str = format!("{:?}", current_date_expr);
    assert!(debug_str.contains("current_date"));

    // Test current_time function
    let current_time_expr = current_time;
    let debug_str = format!("{:?}", current_time_expr);
    assert!(debug_str.contains("current_time"));
    
    // Test extract function
    let extract_expr = extract("YEAR", timestamp_expr.clone());
    let debug_str = format!("{:?}", extract_expr);
    assert!(debug_str.contains("ExtractFunction"));
    
    // Test date_part function
    let date_part_expr = date_part("month", timestamp_expr);
    let debug_str = format!("{:?}", date_part_expr);
    assert!(debug_str.contains("DatePartFunction"));
}

#[test]
fn test_function_type_safety() {
    use diesel::expression::Expression;
    use diesel::sql_types::{Nullable, Text, Integer, Double};
    
    // Test string function return types
    let text_expr = diesel::dsl::sql::<Text>("'test'");
    
    // length returns Nullable<Integer>
    fn assert_nullable_integer<T: Expression<SqlType = Nullable<Integer>>>(_: T) {}
    assert_nullable_integer(length(text_expr.clone()));
    
    // upper, lower, trim return Text
    fn assert_text<T: Expression<SqlType = Text>>(_: T) {}
    assert_text(upper(text_expr.clone()));
    assert_text(lower(text_expr.clone()));
    assert_text(trim(text_expr.clone()));
    assert_text(substring(text_expr, 1, 3));
    
    // Test math function return types
    let int_expr = diesel::dsl::sql::<Integer>("42");
    let double_expr = diesel::dsl::sql::<Double>("3.14");
    
    // abs returns Integer for Integer input
    fn assert_integer<T: Expression<SqlType = Integer>>(_: T) {}
    assert_integer(abs(int_expr));
    
    // ceil, floor, sqrt, round return Double for Double input
    fn assert_double<T: Expression<SqlType = Double>>(_: T) {}
    assert_double(ceil(double_expr.clone()));
    assert_double(floor(double_expr.clone()));
    assert_double(sqrt(double_expr.clone()));
    assert_double(round(double_expr, 2));
}

#[test]
fn test_dsl_imports() {
    // Test that functions are available through DSL
    use diesel_gaussdb::expression::dsl::*;
    
    let text_expr = diesel::dsl::sql::<Text>("'test'");
    let int_expr = diesel::dsl::sql::<Integer>("42");
    let double_expr = diesel::dsl::sql::<Double>("3.14");
    let timestamp_expr = diesel::dsl::sql::<Timestamp>("'2023-12-25 10:30:00'");
    
    // String functions
    let _ = length(text_expr.clone());
    let _ = upper(text_expr.clone());
    let _ = lower(text_expr.clone());
    let _ = trim(text_expr.clone());
    let _ = substring(text_expr, 1, 3);
    
    // Math functions
    let _ = abs(int_expr);
    let _ = ceil(double_expr.clone());
    let _ = floor(double_expr.clone());
    let _ = round(double_expr.clone(), 2);
    let _ = sqrt(double_expr);
    
    // Date/time functions
    let _ = now;
    let _ = current_timestamp;
    let _ = current_date;
    let _ = current_time;
    let _ = extract("YEAR", timestamp_expr.clone());
    let _ = date_part("month", timestamp_expr);
}

#[test]
fn test_function_chaining() {
    // Test that functions can be chained and combined
    let text_expr = diesel::dsl::sql::<Text>("'  Hello World  '");
    
    // Chain string functions: trim then upper then length
    let chained = length(upper(trim(text_expr)));
    let debug_str = format!("{:?}", chained);
    assert!(debug_str.contains("LengthFunction"));
    assert!(debug_str.contains("UpperFunction"));
    assert!(debug_str.contains("TrimFunction"));
    
    // Test math function chaining (with compatible types)
    let double_expr = diesel::dsl::sql::<Double>("3.7");
    let chained_math = sqrt(ceil(double_expr));
    let debug_str = format!("{:?}", chained_math);
    assert!(debug_str.contains("SqrtFunction"));
    assert!(debug_str.contains("CeilFunction"));
}

#[test]
fn test_function_expressions_in_queries() {
    // Test that functions can be used in query contexts
    use diesel::prelude::*;
    
    // This tests compilation, not execution
    let text_expr = diesel::dsl::sql::<Text>("'test'");
    let int_expr = diesel::dsl::sql::<Integer>("42");
    
    // Test in select clause
    let _query = diesel::select((
        upper(text_expr.clone()),
        length(text_expr),
        abs(int_expr),
        now,
    ));
    
    // Test that the query compiles
    let debug_str = format!("{:?}", _query);
    assert!(!debug_str.is_empty());
}
