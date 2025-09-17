//! LIMIT/OFFSET clause implementations for GaussDB backend
//!
//! This module provides QueryFragment implementations for diesel's internal
//! LimitOffsetClause types to support the GaussDB backend.

use crate::backend::GaussDB;
use diesel::query_builder::{AstPass, IntoBoxedClause, QueryFragment};
use diesel::query_builder::{BoxedLimitOffsetClause, LimitOffsetClause};
use diesel::result::QueryResult;

// 为diesel内部的LimitOffsetClause实现QueryFragment<GaussDB>
impl<L, O> QueryFragment<GaussDB> for LimitOffsetClause<L, O>
where
    L: QueryFragment<GaussDB>,
    O: QueryFragment<GaussDB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        self.limit_clause.walk_ast(out.reborrow())?;
        self.offset_clause.walk_ast(out.reborrow())?;
        Ok(())
    }
}

// 为BoxedLimitOffsetClause实现QueryFragment<GaussDB>
impl QueryFragment<GaussDB> for BoxedLimitOffsetClause<'_, GaussDB> {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, GaussDB>) -> QueryResult<()> {
        if let Some(ref limit) = self.limit {
            limit.walk_ast(out.reborrow())?;
        }
        if let Some(ref offset) = self.offset {
            offset.walk_ast(out.reborrow())?;
        }
        Ok(())
    }
}

// 为LimitOffsetClause实现IntoBoxedClause<GaussDB>
impl<'a, L, O> IntoBoxedClause<'a, GaussDB> for LimitOffsetClause<L, O>
where
    L: QueryFragment<GaussDB> + Send + 'a,
    O: QueryFragment<GaussDB> + Send + 'a,
{
    type BoxedClause = BoxedLimitOffsetClause<'a, GaussDB>;

    fn into_boxed(self) -> Self::BoxedClause {
        BoxedLimitOffsetClause {
            limit: Some(Box::new(self.limit_clause)),
            offset: Some(Box::new(self.offset_clause)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_builder::GaussDBQueryBuilder;
    use diesel::query_builder::QueryBuilder;
    use diesel::dsl::sql;
    use diesel::sql_types::BigInt;

    /// 测试辅助函数：生成QueryFragment的SQL
    fn generate_sql<T>(fragment: T) -> String 
    where 
        T: QueryFragment<GaussDB>
    {
        let mut query_builder = GaussDBQueryBuilder::new();
        fragment.to_sql(&mut query_builder, &GaussDB).unwrap();
        query_builder.finish()
    }

    #[test]
    fn test_limit_offset_clause_sql_generation() {
        use diesel::query_builder::{LimitClause, OffsetClause};

        // 使用diesel内部的构造方式
        let limit = LimitClause(sql::<BigInt>("10"));
        let offset = OffsetClause(sql::<BigInt>("20"));
        let clause = LimitOffsetClause {
            limit_clause: limit,
            offset_clause: offset,
        };

        let sql_result = generate_sql(clause);
        assert!(sql_result.contains("10"));
        assert!(sql_result.contains("20"));
    }

    #[test]
    fn test_boxed_limit_offset_clause() {
        let boxed_clause = BoxedLimitOffsetClause::<GaussDB> {
            limit: Some(Box::new(sql::<BigInt>("5"))),
            offset: Some(Box::new(sql::<BigInt>("15"))),
        };
        
        let sql_result = generate_sql(boxed_clause);
        assert!(sql_result.contains("5"));
        assert!(sql_result.contains("15"));
    }

    #[test]
    fn test_empty_boxed_limit_offset_clause() {
        let boxed_clause = BoxedLimitOffsetClause::<GaussDB> {
            limit: None,
            offset: None,
        };
        
        let sql_result = generate_sql(boxed_clause);
        assert_eq!(sql_result, "");
    }
}
