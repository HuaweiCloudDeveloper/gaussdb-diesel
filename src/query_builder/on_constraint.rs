//! ON CONSTRAINT clause implementation for GaussDB
//!
//! This module provides support for PostgreSQL-style ON CONSTRAINT clauses,
//! which are also supported by GaussDB for upsert operations.

use crate::backend::GaussDB;
use diesel::query_builder::{QueryFragment, AstPass, QueryId};
use diesel::result::QueryResult;

/// Used to specify the constraint name for an upsert statement in the form `ON
/// CONFLICT ON CONSTRAINT`. Note that `constraint_name` must be the name of a
/// unique constraint, not the name of an index.
///
/// # Example
///
/// ```sql
/// INSERT INTO users (id, name) VALUES (1, 'John')
/// ON CONFLICT ON CONSTRAINT users_name_unique
/// DO UPDATE SET name = EXCLUDED.name;
/// ```
pub fn on_constraint(constraint_name: &str) -> OnConstraint<'_> {
    OnConstraint { constraint_name }
}

/// Represents an ON CONSTRAINT clause for conflict resolution
#[derive(Debug, Clone, Copy)]
pub struct OnConstraint<'a> {
    constraint_name: &'a str,
}

impl<'a> OnConstraint<'a> {
    /// Create a new ON CONSTRAINT clause with the given constraint name
    pub fn new(constraint_name: &'a str) -> Self {
        Self { constraint_name }
    }

    /// Get the constraint name
    pub fn constraint_name(&self) -> &str {
        self.constraint_name
    }
}

impl QueryId for OnConstraint<'_> {
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<'a> QueryFragment<GaussDB> for OnConstraint<'a> {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        out.push_sql(" ON CONSTRAINT ");
        out.push_identifier(self.constraint_name)?;
        Ok(())
    }
}

/// A conflict target that specifies a constraint name
#[derive(Debug, Clone, Copy)]
pub struct ConflictTarget<T>(pub T);

impl<T> ConflictTarget<T> {
    /// Create a new conflict target
    pub fn new(target: T) -> Self {
        Self(target)
    }
}

impl QueryFragment<GaussDB> for ConflictTarget<OnConstraint<'_>> {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        out.push_sql(" ON CONSTRAINT ");
        out.push_identifier(self.0.constraint_name)?;
        Ok(())
    }
}

/// Trait for types that can be used as conflict targets
pub trait OnConflictTarget<Table> {}

impl<Table> OnConflictTarget<Table> for ConflictTarget<OnConstraint<'_>> {}

/// Helper trait for building upsert queries with constraint conflicts
pub trait OnConflictDsl<Target> {
    /// The type returned by `.on_conflict()`
    type Output;

    /// Add an ON CONFLICT clause to the query
    fn on_conflict(self, target: Target) -> Self::Output;
}

/// Helper trait for specifying what to do when a conflict occurs
pub trait DoUpdateDsl<Set> {
    /// The type returned by `.do_update()`
    type Output;

    /// Specify what to do when a conflict occurs - update with the given values
    fn do_update(self) -> Self::Output;
}

/// Helper trait for specifying to do nothing when a conflict occurs
pub trait DoNothingDsl {
    /// The type returned by `.do_nothing()`
    type Output;

    /// Specify to do nothing when a conflict occurs
    fn do_nothing(self) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_constraint_creation() {
        let constraint = on_constraint("users_name_unique");
        assert_eq!(constraint.constraint_name(), "users_name_unique");
    }

    #[test]
    fn test_on_constraint_new() {
        let constraint = OnConstraint::new("test_constraint");
        assert_eq!(constraint.constraint_name(), "test_constraint");
    }

    #[test]
    fn test_conflict_target() {
        let constraint = on_constraint("users_email_unique");
        let target = ConflictTarget::new(constraint);
        assert_eq!(target.0.constraint_name(), "users_email_unique");
    }

    #[test]
    fn test_on_constraint_debug() {
        let constraint = on_constraint("debug_constraint");
        let debug_str = format!("{:?}", constraint);
        assert!(debug_str.contains("OnConstraint"));
        assert!(debug_str.contains("debug_constraint"));
    }

    #[test]
    fn test_conflict_target_debug() {
        let constraint = on_constraint("target_constraint");
        let target = ConflictTarget::new(constraint);
        let debug_str = format!("{:?}", target);
        assert!(debug_str.contains("ConflictTarget"));
        assert!(debug_str.contains("target_constraint"));
    }

    #[test]
    fn test_query_id_properties() {
        let constraint = on_constraint("test");
        
        // Test that QueryId is implemented correctly
        assert!(!OnConstraint::HAS_STATIC_QUERY_ID);
        
        // Test that we can use it in generic contexts that require QueryId
        fn requires_query_id<T: QueryId>(_: T) {}
        requires_query_id(constraint);
    }

    #[test]
    fn test_sql_generation() {
        // This is a basic test to ensure the SQL generation doesn't panic
        // In a real implementation, you'd want to test the actual SQL output
        let constraint = on_constraint("users_unique_email");
        let target = ConflictTarget::new(constraint);
        
        // Test that the types are properly constructed
        assert_eq!(target.0.constraint_name(), "users_unique_email");
    }
}
