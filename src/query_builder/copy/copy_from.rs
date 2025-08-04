//! COPY FROM implementation for GaussDB
//!
//! This module provides support for PostgreSQL-style COPY FROM operations,
//! which are also supported by GaussDB for bulk data import.

use std::marker::PhantomData;

use super::{CommonOptions, CopyFormat, CopyTarget};
use crate::backend::GaussDB;
use diesel::query_builder::{QueryFragment, AstPass, QueryId};
use diesel::result::QueryResult;

/// Describes the different possible settings for the `HEADER` option
/// for `COPY FROM` statements
#[derive(Debug, Copy, Clone)]
pub enum CopyHeader {
    /// Is the header set?
    Set(bool),
    /// Match the header with the targeted table names
    /// and fail in the case of a mismatch
    Match,
}

/// Options specific to COPY FROM operations
#[derive(Debug, Default)]
pub struct CopyFromOptions {
    common: CommonOptions,
    default: Option<String>,
    header: Option<CopyHeader>,
}

impl QueryFragment<GaussDB> for CopyFromOptions {
    fn walk_ast<'b>(
        &'b self,
        mut pass: AstPass<'_, 'b, GaussDB>,
    ) -> QueryResult<()> {
        if self.any_set() {
            let mut comma = "";
            pass.push_sql(" WITH (");
            self.common.walk_ast(pass.reborrow(), &mut comma);
            if let Some(ref default) = self.default {
                pass.push_sql(comma);
                comma = ", ";
                pass.push_sql("DEFAULT '");
                // cannot use binds here :(
                pass.push_sql(default);
                pass.push_sql("'");
            }
            if let Some(ref header) = self.header {
                pass.push_sql(comma);
                // commented out because rustc complains otherwise
                //comma = ", ";
                pass.push_sql("HEADER ");
                match header {
                    CopyHeader::Set(true) => pass.push_sql("1"),
                    CopyHeader::Set(false) => pass.push_sql("0"),
                    CopyHeader::Match => pass.push_sql("MATCH"),
                }
            }

            pass.push_sql(")");
        }
        Ok(())
    }
}

impl CopyFromOptions {
    fn any_set(&self) -> bool {
        self.common.any_set() || self.default.is_some() || self.header.is_some()
    }
}

/// Represents a COPY FROM query
#[derive(Debug)]
pub struct CopyFromQuery<S, F> {
    options: CopyFromOptions,
    copy_callback: F,
    p: PhantomData<S>,
}

impl<S, F> CopyFromQuery<S, F> {
    /// Create a new COPY FROM query
    pub fn new(copy_callback: F) -> Self {
        Self {
            options: CopyFromOptions::default(),
            copy_callback,
            p: PhantomData,
        }
    }

    /// Set the format for the COPY FROM operation
    pub fn with_format(mut self, format: CopyFormat) -> Self {
        self.options.common.format = Some(format);
        self
    }

    /// Set the delimiter for the COPY FROM operation
    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.options.common.delimiter = Some(delimiter);
        self
    }

    /// Set the NULL string for the COPY FROM operation
    pub fn with_null(mut self, null: String) -> Self {
        self.options.common.null = Some(null);
        self
    }

    /// Set the quote character for the COPY FROM operation
    pub fn with_quote(mut self, quote: char) -> Self {
        self.options.common.quote = Some(quote);
        self
    }

    /// Set the escape character for the COPY FROM operation
    pub fn with_escape(mut self, escape: char) -> Self {
        self.options.common.escape = Some(escape);
        self
    }

    /// Enable or disable FREEZE option
    pub fn with_freeze(mut self, freeze: bool) -> Self {
        self.options.common.freeze = Some(freeze);
        self
    }

    /// Set the default value for missing columns
    pub fn with_default(mut self, default: String) -> Self {
        self.options.default = Some(default);
        self
    }

    /// Set the header option
    pub fn with_header(mut self, header: CopyHeader) -> Self {
        self.options.header = Some(header);
        self
    }
}

impl<S, F> QueryId for CopyFromQuery<S, F> {
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<S, F> QueryFragment<GaussDB> for CopyFromQuery<S, F>
where
    S: CopyTarget,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        pass.unsafe_to_cache_prepared();
        pass.push_sql("COPY ");
        S::walk_target(pass.reborrow())?;
        pass.push_sql(" FROM STDIN");
        self.options.walk_ast(pass.reborrow())?;
        Ok(())
    }
}

/// Internal representation of a COPY FROM query
pub(crate) struct InternalCopyFromQuery<S, T> {
    pub(crate) target: S,
    p: PhantomData<T>,
}

impl<S, T> InternalCopyFromQuery<S, T> {
    pub(crate) fn new(target: S) -> Self {
        Self {
            target,
            p: PhantomData,
        }
    }
}

impl<S, T> QueryId for InternalCopyFromQuery<S, T> {
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<S, T> QueryFragment<GaussDB> for InternalCopyFromQuery<S, T>
where
    S: CopyTarget,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        pass.unsafe_to_cache_prepared();
        pass.push_sql("COPY ");
        S::walk_target(pass.reborrow())?;
        pass.push_sql(" FROM STDIN BINARY");
        Ok(())
    }
}

/// A trait for executing COPY FROM operations
pub trait ExecuteCopyFromDsl<T> {
    /// Execute the COPY FROM operation
    fn execute_copy_from<F>(self, callback: F) -> QueryResult<usize>
    where
        F: FnMut() -> QueryResult<Option<Vec<u8>>>;
}

/// Helper function to create a COPY FROM query
pub fn copy_from<S>(_target: S) -> CopyFromQuery<S, ()>
where
    S: CopyTarget,
{
    CopyFromQuery::new(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_header_debug() {
        let header = CopyHeader::Set(true);
        let debug_str = format!("{:?}", header);
        assert!(debug_str.contains("Set"));
        assert!(debug_str.contains("true"));

        let header = CopyHeader::Match;
        let debug_str = format!("{:?}", header);
        assert!(debug_str.contains("Match"));
    }

    #[test]
    fn test_copy_from_options_any_set() {
        let mut options = CopyFromOptions::default();
        assert!(!options.any_set());

        options.default = Some("DEFAULT".to_string());
        assert!(options.any_set());

        options = CopyFromOptions::default();
        options.header = Some(CopyHeader::Set(true));
        assert!(options.any_set());
    }

    #[test]
    fn test_copy_from_query_builder() {
        let query: CopyFromQuery<(), ()> = CopyFromQuery::new(())
            .with_format(CopyFormat::Csv)
            .with_delimiter(',')
            .with_null("NULL".to_string())
            .with_quote('"')
            .with_escape('\\')
            .with_freeze(true)
            .with_default("DEFAULT".to_string())
            .with_header(CopyHeader::Set(true));

        assert!(query.options.common.format.is_some());
        assert!(query.options.common.delimiter.is_some());
        assert!(query.options.common.null.is_some());
        assert!(query.options.common.quote.is_some());
        assert!(query.options.common.escape.is_some());
        assert!(query.options.common.freeze.is_some());
        assert!(query.options.default.is_some());
        assert!(query.options.header.is_some());
    }

    #[test]
    fn test_copy_from_query_id() {
        let query = CopyFromQuery::<(), ()>::new(());
        
        // Test that QueryId is implemented correctly
        assert!(!CopyFromQuery::<(), ()>::HAS_STATIC_QUERY_ID);
        
        // Test that we can use it in generic contexts that require QueryId
        fn requires_query_id<T: QueryId>(_: T) {}
        requires_query_id(query);
    }

    #[test]
    fn test_internal_copy_from_query() {
        let query = InternalCopyFromQuery::<(), ()>::new(());
        
        // Test that QueryId is implemented correctly
        assert!(!InternalCopyFromQuery::<(), ()>::HAS_STATIC_QUERY_ID);
        
        // Test that we can use it in generic contexts that require QueryId
        fn requires_query_id<T: QueryId>(_: T) {}
        requires_query_id(query);
    }
}
