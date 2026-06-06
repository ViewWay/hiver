//! GraphQL context management / GraphQL 上下文管理
//! Equivalent to Spring for GraphQL context propagation

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use serde_json::Value;

/// Shared context for GraphQL request execution, carrying string-keyed values and typed data.
/// GraphQL 请求执行的共享上下文，携带字符串键值和类型化数据。
#[derive(Clone)]
pub struct GraphQLContext
{
    values: Arc<HashMap<String, Value>>,
    typed_data: Arc<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl GraphQLContext
{
    /// Create a new empty context. / 创建新的空上下文。
    pub fn new() -> Self
    {
        Self {
            values: Arc::new(HashMap::new()),
            typed_data: Arc::new(HashMap::new()),
        }
    }

    /// Attach a typed value, retrievable via `get_typed`. / 附加类型化值，可通过 `get_typed` 获取。
    pub fn with<T: 'static + Send + Sync>(mut self, value: T) -> Self
    {
        // Rebuild typed_data with the new value (dyn Any cannot be cloned)
        let mut new_typed: HashMap<TypeId, Box<dyn Any + Send + Sync>> = HashMap::new();
        new_typed.insert(TypeId::of::<T>(), Box::new(value));
        self.typed_data = Arc::new(new_typed);
        self
    }

    /// Retrieve a previously attached typed value. / 获取之前附加的类型化值。
    pub fn get_typed<T: 'static + Send + Sync>(&self) -> Option<&T>
    {
        self.typed_data
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }

    /// Insert a string-keyed value, returning a new context. / 插入字符串键值，返回新上下文。
    pub fn insert(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self
    {
        let values = Arc::make_mut(&mut self.values);
        values.insert(key.into(), value.into());
        self
    }

    /// Look up a value by key. / 按键查找值。
    pub fn get(&self, key: &str) -> Option<&Value>
    {
        self.values.get(key)
    }

    /// Look up a string value by key. / 按键查找字符串值。
    pub fn get_string(&self, key: &str) -> Option<&str>
    {
        self.values.get(key).and_then(|v| v.as_str())
    }

    /// Look up a boolean value by key. / 按键查找布尔值。
    pub fn get_bool(&self, key: &str) -> Option<bool>
    {
        self.values.get(key).and_then(Value::as_bool)
    }

    /// Look up an i64 value by key. / 按键查找 i64 值。
    pub fn get_i64(&self, key: &str) -> Option<i64>
    {
        self.values.get(key).and_then(Value::as_i64)
    }

    /// Check whether a key exists. / 检查键是否存在。
    pub fn contains_key(&self, key: &str) -> bool
    {
        self.values.contains_key(key)
    }

    /// Iterate over all keys. / 遍历所有键。
    pub fn keys(&self) -> impl Iterator<Item = &String>
    {
        self.values.keys()
    }

    /// Merge string-keyed values from another context (other wins on conflict)
    pub fn merge(&self, other: &GraphQLContext) -> GraphQLContext
    {
        let mut new_values: HashMap<String, Value> = HashMap::new();
        for (k, v) in self.values.iter()
        {
            new_values.insert(k.clone(), v.clone());
        }
        for (k, v) in other.values.iter()
        {
            new_values.insert(k.clone(), v.clone());
        }
        GraphQLContext {
            values: Arc::new(new_values),
            typed_data: Arc::clone(&self.typed_data),
        }
    }
}

impl Default for GraphQLContext
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl std::fmt::Debug for GraphQLContext
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("GraphQLContext")
            .field("values", &self.values)
            .finish_non_exhaustive()
    }
}

/// Builder for constructing GraphQL context
pub struct GraphQLContextBuilder
{
    values: HashMap<String, Value>,
}

impl GraphQLContextBuilder
{
    /// Create a new builder. / 创建新的构建器。
    pub fn new() -> Self
    {
        Self {
            values: HashMap::new(),
        }
    }

    /// Add a string-keyed value. / 添加字符串键值对。
    pub fn value(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self
    {
        self.values.insert(key.into(), value.into());
        self
    }

    /// Build the context. / 构建上下文。
    pub fn build(self) -> GraphQLContext
    {
        GraphQLContext {
            values: Arc::new(self.values),
            typed_data: Arc::new(HashMap::new()),
        }
    }
}

impl Default for GraphQLContextBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_basic_context()
    {
        let ctx = GraphQLContext::new()
            .insert("user_id", "123")
            .insert("authenticated", true);
        assert_eq!(ctx.get_string("user_id"), Some("123"));
        assert_eq!(ctx.get_bool("authenticated"), Some(true));
    }

    #[test]
    fn test_typed_context()
    {
        let ctx = GraphQLContext::new().with(42u64);
        assert_eq!(ctx.get_typed::<u64>(), Some(&42u64));
    }

    #[test]
    fn test_merge()
    {
        let ctx1 = GraphQLContext::new().insert("a", "1");
        let ctx2 = GraphQLContext::new()
            .insert("b", "2")
            .insert("a", "overridden");
        let merged = ctx1.merge(&ctx2);
        assert_eq!(merged.get_string("a"), Some("overridden"));
        assert_eq!(merged.get_string("b"), Some("2"));
    }

    #[test]
    fn test_builder()
    {
        let ctx = GraphQLContextBuilder::new().value("role", "admin").build();
        assert_eq!(ctx.get_string("role"), Some("admin"));
    }
}
