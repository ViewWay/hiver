//! GraphQL resolver traits / `GraphQL解析器trait`
//! Equivalent to Spring for GraphQL's `DataFetcher` + @SchemaMapping/@QueryMapping

use crate::context::GraphQLContext;
use crate::error::GraphQLError;
use async_trait::async_trait;
use serde_json::Value;

pub type ResolverResult<T> = Result<T, GraphQLError>;

#[async_trait]
pub trait QueryResolver: Send + Sync {
    async fn resolve(&self, field_name: &str, args: &Value, ctx: &GraphQLContext) -> ResolverResult<Value>;
}

#[async_trait]
pub trait MutationResolver: Send + Sync {
    async fn resolve(&self, field_name: &str, args: &Value, ctx: &GraphQLContext) -> ResolverResult<Value>;
}

#[async_trait]
pub trait SubscriptionResolver: Send + Sync {
    async fn resolve(&self, field_name: &str, args: &Value, ctx: &GraphQLContext) -> ResolverResult<Value>;
}

#[async_trait]
pub trait FieldResolver<T>: Send + Sync {
    async fn resolve_field(&self, parent: &T, field_name: &str, args: &Value, ctx: &GraphQLContext) -> ResolverResult<Value>;
}

#[derive(Default)]
pub struct ResolverRegistry {
    query_resolvers: Vec<Box<dyn QueryResolver>>,
    mutation_resolvers: Vec<Box<dyn MutationResolver>>,
    subscription_resolvers: Vec<Box<dyn SubscriptionResolver>>,
}

impl ResolverRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register_query(&mut self, resolver: impl QueryResolver + 'static) {
        self.query_resolvers.push(Box::new(resolver));
    }
    pub fn register_mutation(&mut self, resolver: impl MutationResolver + 'static) {
        self.mutation_resolvers.push(Box::new(resolver));
    }
    pub fn register_subscription(&mut self, resolver: impl SubscriptionResolver + 'static) {
        self.subscription_resolvers.push(Box::new(resolver));
    }
    pub fn query_resolvers(&self) -> &[Box<dyn QueryResolver>] { &self.query_resolvers }
    pub fn mutation_resolvers(&self) -> &[Box<dyn MutationResolver>] { &self.mutation_resolvers }
    pub fn query_count(&self) -> usize { self.query_resolvers.len() }
    pub fn mutation_count(&self) -> usize { self.mutation_resolvers.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestQuery;
    #[async_trait]
    impl QueryResolver for TestQuery {
        async fn resolve(&self, f: &str, _a: &Value, _c: &GraphQLContext) -> ResolverResult<Value> {
            Ok(serde_json::json!({f: "ok"}))
        }
    }

    #[tokio::test]
    async fn test_registry() {
        let mut reg = ResolverRegistry::new();
        reg.register_query(TestQuery);
        assert_eq!(reg.query_count(), 1);
    }
}
