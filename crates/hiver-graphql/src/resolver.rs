//! GraphQL resolver traits / `GraphQL解析器trait`
//! Equivalent to Spring for GraphQL's `DataFetcher` + @SchemaMapping/@QueryMapping

use async_trait::async_trait;
use serde_json::Value;

use crate::{context::GraphQLContext, error::GraphQLError};

/// Result type for resolver operations. / 解析器操作的结果类型。
pub type ResolverResult<T> = Result<T, GraphQLError>;

/// Trait for resolving GraphQL query fields. / GraphQL 查询字段解析 trait。
#[async_trait]
pub trait QueryResolver: Send + Sync
{
    /// Resolve a query field. / 解析查询字段。
    async fn resolve(
        &self,
        field_name: &str,
        args: &Value,
        ctx: &GraphQLContext,
    ) -> ResolverResult<Value>;
}

/// Trait for resolving GraphQL mutation fields. / GraphQL 变更字段解析 trait。
#[async_trait]
pub trait MutationResolver: Send + Sync
{
    /// Resolve a mutation field. / 解析变更字段。
    async fn resolve(
        &self,
        field_name: &str,
        args: &Value,
        ctx: &GraphQLContext,
    ) -> ResolverResult<Value>;
}

/// Trait for resolving GraphQL subscription fields. / GraphQL 订阅字段解析 trait。
#[async_trait]
pub trait SubscriptionResolver: Send + Sync
{
    /// Resolve a subscription field. / 解析订阅字段。
    async fn resolve(
        &self,
        field_name: &str,
        args: &Value,
        ctx: &GraphQLContext,
    ) -> ResolverResult<Value>;
}

/// Trait for resolving fields on a parent type. / 父类型字段解析 trait。
#[async_trait]
pub trait FieldResolver<T>: Send + Sync
{
    /// Resolve a field on the given parent. / 解析给定父对象上的字段。
    async fn resolve_field(
        &self,
        parent: &T,
        field_name: &str,
        args: &Value,
        ctx: &GraphQLContext,
    ) -> ResolverResult<Value>;
}

/// Central registry for query, mutation, and subscription resolvers. /
/// 查询、变更和订阅解析器的中央注册表。
#[derive(Default)]
#[allow(clippy::struct_field_names)]
pub struct ResolverRegistry
{
    query_resolvers: Vec<Box<dyn QueryResolver>>,
    mutation_resolvers: Vec<Box<dyn MutationResolver>>,
    subscription_resolvers: Vec<Box<dyn SubscriptionResolver>>,
}

impl ResolverRegistry
{
    /// Create a new empty registry. / 创建新的空注册表。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Register a query resolver. / 注册查询解析器。
    pub fn register_query(&mut self, resolver: impl QueryResolver + 'static)
    {
        self.query_resolvers.push(Box::new(resolver));
    }

    /// Register a mutation resolver. / 注册变更解析器。
    pub fn register_mutation(&mut self, resolver: impl MutationResolver + 'static)
    {
        self.mutation_resolvers.push(Box::new(resolver));
    }

    /// Register a subscription resolver. / 注册订阅解析器。
    pub fn register_subscription(&mut self, resolver: impl SubscriptionResolver + 'static)
    {
        self.subscription_resolvers.push(Box::new(resolver));
    }

    /// Return all registered query resolvers. / 返回所有已注册的查询解析器。
    pub fn query_resolvers(&self) -> &[Box<dyn QueryResolver>]
    {
        &self.query_resolvers
    }

    /// Return all registered mutation resolvers. / 返回所有已注册的变更解析器。
    pub fn mutation_resolvers(&self) -> &[Box<dyn MutationResolver>]
    {
        &self.mutation_resolvers
    }

    /// Return the number of registered query resolvers. / 返回已注册查询解析器的数量。
    pub fn query_count(&self) -> usize
    {
        self.query_resolvers.len()
    }

    /// Return the number of registered mutation resolvers. / 返回已注册变更解析器的数量。
    pub fn mutation_count(&self) -> usize
    {
        self.mutation_resolvers.len()
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    struct TestQuery;
    #[async_trait]
    impl QueryResolver for TestQuery
    {
        async fn resolve(&self, f: &str, _a: &Value, _c: &GraphQLContext) -> ResolverResult<Value>
        {
            Ok(serde_json::json!({f: "ok"}))
        }
    }

    #[hiver_macros::test]
    async fn test_registry()
    {
        let mut reg = ResolverRegistry::new();
        reg.register_query(TestQuery);
        assert_eq!(reg.query_count(), 1);
    }
}
