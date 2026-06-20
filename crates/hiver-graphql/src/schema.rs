//! GraphQL schema management / GraphQL schema管理
//! Equivalent to Spring for GraphQL's GraphQLSource + RuntimeWiringConfigurer

use crate::error::GraphQLError;
use crate::resolver::{MutationResolver, QueryResolver, SubscriptionResolver};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    pub definition: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TypeRegistry {
    types: HashMap<String, TypeDef>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self { types: HashMap::new() }
    }
    pub fn register(&mut self, type_def: TypeDef) {
        self.types.insert(type_def.name.clone(), type_def);
    }
    pub fn get(&self, name: &str) -> Option<&TypeDef> {
        self.types.get(name)
    }
    pub fn names(&self) -> Vec<&String> {
        self.types.keys().collect()
    }
    pub fn len(&self) -> usize {
        self.types.len()
    }
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResult {
    pub data: serde_json::Value,
}

#[derive(Clone)]
pub struct Schema {
    sdl: Arc<String>,
    type_registry: TypeRegistry,
    introspection_enabled: bool,
}

impl Schema {
    pub fn sdl(&self) -> &str { &self.sdl }
    pub fn type_registry(&self) -> &TypeRegistry { &self.type_registry }
    pub fn is_introspection_enabled(&self) -> bool { self.introspection_enabled }

    pub async fn execute(
        &self,
        query: &str,
        _variables: Option<&serde_json::Value>,
        _operation_name: Option<&str>,
        _context: &crate::context::GraphQLContext,
    ) -> Result<serde_json::Value, Vec<GraphQLError>> {
        if query.trim().is_empty() {
            return Err(vec![GraphQLError::new("Empty query").with_code("EMPTY_QUERY")]);
        }
        if query.contains("__schema") || query.contains("__type") {
            if !self.introspection_enabled {
                return Err(vec![GraphQLError::new("Introspection disabled").with_code("FORBIDDEN")]);
            }
            return Ok(serde_json::json!({"data": {"__schema": {"queryType": {"name": "Query"}}}}));
        }
        Ok(serde_json::json!({"data": {}}))
    }
}

#[derive(Default)]
pub struct SchemaBuilder {
    sdl_source: Option<String>,
    type_registry: TypeRegistry,
    introspection_enabled: bool,
    _query_resolver: Option<Arc<dyn QueryResolver>>,
    _mutation_resolver: Option<Arc<dyn MutationResolver>>,
    _subscription_resolver: Option<Arc<dyn SubscriptionResolver>>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self { introspection_enabled: true, ..Default::default() }
    }
    pub fn sdl(mut self, sdl: impl Into<String>) -> Self {
        self.sdl_source = Some(sdl.into());
        self
    }
    pub fn register_type(mut self, type_def: TypeDef) -> Self {
        self.type_registry.register(type_def);
        self
    }
    pub fn introspection(mut self, enabled: bool) -> Self {
        self.introspection_enabled = enabled;
        self
    }
    pub fn query_resolver(mut self, resolver: impl QueryResolver + 'static) -> Self {
        self._query_resolver = Some(Arc::new(resolver));
        self
    }
    pub fn mutation_resolver(mut self, resolver: impl MutationResolver + 'static) -> Self {
        self._mutation_resolver = Some(Arc::new(resolver));
        self
    }
    pub fn build(self) -> Schema {
        Schema {
            sdl: Arc::new(self.sdl_source.unwrap_or_else(|| "type Query { _empty: String }".to_string())),
            type_registry: self.type_registry,
            introspection_enabled: self.introspection_enabled,
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests {
    use super::*;

    #[hiver_macros::test]
    async fn test_empty_schema() {
        let schema = SchemaBuilder::new().build();
        let ctx = crate::context::GraphQLContext::new();
        let result = schema.execute("", None, None, &ctx).await;
        assert!(result.is_err());
    }

    #[hiver_macros::test]
    async fn test_type_registry() {
        let mut reg = TypeRegistry::new();
        reg.register(TypeDef { name: "User".into(), definition: "type User { id: ID! }".into(), description: None });
        assert!(reg.get("User").is_some());
    }
}
