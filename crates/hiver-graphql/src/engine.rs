//! async-graphql execution engine bridge.
//! 基于 async-graphql 的真实 GraphQL 执行引擎桥接。
//!
//! # Description / 描述
//!
//! This module bridges `async-graphql` (the most mature Rust GraphQL library)
//! into the Hiver framework, providing:
//! - SDL schema parsing and validation
//! - Full query / mutation execution
//! - Subscription streams over async channels
//! - DataLoader integration
//! - GraphiQL playground HTML
//!
//! 本模块将 async-graphql 集成到 Hiver 框架，提供真实的 SDL 解析、
//! Query/Mutation 执行、Subscription 流以及 DataLoader 集成。
//!
//! # Example / 示例
//! ```rust,ignore
//! use async_graphql::{Object, SimpleObject, EmptyMutation, EmptySubscription, Schema};
//! use hiver_graphql::engine::{HiverGraphQL, GraphQLRequest};
//!
//! #[derive(SimpleObject)]
//! struct User { id: u64, name: String }
//!
//! struct QueryRoot;
//! #[Object]
//! impl QueryRoot {
//!     async fn user(&self, id: u64) -> User { User { id, name: "Alice".into() } }
//! }
//!
//! let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
//! let engine = HiverGraphQL::new(schema);
//! let resp = engine.execute(GraphQLRequest::new("{ user(id: 1) { name } }")).await;
//! ```

#[cfg(feature = "engine")]
pub use engine_impl::*;

#[cfg(feature = "engine")]
mod engine_impl {
    use async_graphql::{
        BatchRequest, BatchResponse, ObjectType, Request as AGRequest,
        Response as AGResponse, Schema, SubscriptionType,
    };
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    // ─────────────────────────────────────────────────────────────────────────
    // Request / Response wrappers
    // ─────────────────────────────────────────────────────────────────────────

    /// GraphQL over-HTTP request.
    /// GraphQL over-HTTP 请求。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GraphQLRequest {
        /// The GraphQL query string / GraphQL 查询字符串
        pub query: String,
        /// Optional named variables / 可选命名变量
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub variables: Option<serde_json::Value>,
        /// Optional operation name / 可选操作名
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub operation_name: Option<String>,
        /// Protocol-level extensions / 协议级扩展
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub extensions: Option<serde_json::Value>,
    }

    impl GraphQLRequest {
        /// Create a new request with the given query.
        /// 创建带有指定查询的新请求。
        pub fn new(query: impl Into<String>) -> Self {
            Self { query: query.into(), variables: None, operation_name: None, extensions: None }
        }

        /// Attach named variables.
        /// 附加命名变量。
        pub fn with_variables(mut self, vars: serde_json::Value) -> Self {
            self.variables = Some(vars);
            self
        }

        /// Set the operation name.
        /// 设置操作名称。
        pub fn with_operation_name(mut self, name: impl Into<String>) -> Self {
            self.operation_name = Some(name.into());
            self
        }

        fn into_ag_request(self) -> AGRequest {
            let mut req = AGRequest::new(self.query);
            if let Some(vars) = self.variables {
                req = req.variables(async_graphql::Variables::from_json(vars));
            }
            if let Some(op) = self.operation_name {
                req = req.operation_name(op);
            }
            req
        }
    }

    /// GraphQL over-HTTP response.
    /// GraphQL over-HTTP 响应。
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GraphQLResponse {
        /// Successful result data / 成功结果数据
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<serde_json::Value>,
        /// Execution errors / 执行错误
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub errors: Vec<serde_json::Value>,
    }

    impl GraphQLResponse {
        /// Returns `true` if the response contains any errors.
        /// 若响应包含错误则返回 true。
        pub fn has_errors(&self) -> bool { !self.errors.is_empty() }

        fn from_ag(resp: &AGResponse) -> Self {
            let json = serde_json::to_value(resp).unwrap_or(serde_json::json!({}));
            let data = json.get("data").cloned();
            let errors = json
                .get("errors")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            Self { data, errors }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HiverGraphQL engine
    // ─────────────────────────────────────────────────────────────────────────

    /// The Hiver GraphQL execution engine backed by `async-graphql`.
    /// 基于 async-graphql 的 Hiver GraphQL 执行引擎。
    ///
    /// Type parameters mirror `async-graphql::Schema<Q, M, S>`.
    /// 类型参数与 async-graphql::Schema<Q, M, S> 对应。
    #[derive(Clone)]
    pub struct HiverGraphQL<Q, M, S>
    where
        Q: ObjectType + 'static,
        M: ObjectType + 'static,
        S: SubscriptionType + 'static,
    {
        schema: Arc<Schema<Q, M, S>>,
    }

    impl<Q, M, S> HiverGraphQL<Q, M, S>
    where
        Q: ObjectType + 'static,
        M: ObjectType + 'static,
        S: SubscriptionType + 'static,
    {
        /// Wrap a pre-built `async-graphql` schema.
        /// 包装一个预构建的 async-graphql Schema。
        pub fn new(schema: Schema<Q, M, S>) -> Self {
            Self { schema: Arc::new(schema) }
        }

        /// Execute a single GraphQL request and return the response.
        /// 执行单个 GraphQL 请求并返回响应。
        pub async fn execute(&self, request: GraphQLRequest) -> GraphQLResponse {
            let resp = self.schema.execute(request.into_ag_request()).await;
            GraphQLResponse::from_ag(&resp)
        }

        /// Execute a batch of GraphQL requests.
        /// 执行一批 GraphQL 请求。
        pub async fn execute_batch(&self, requests: Vec<GraphQLRequest>) -> Vec<GraphQLResponse> {
            let ag_batch = BatchRequest::Batch(
                requests.into_iter().map(GraphQLRequest::into_ag_request).collect(),
            );
            let batch_resp = self.schema.execute_batch(ag_batch).await;
            match batch_resp {
                BatchResponse::Batch(resps) => {
                    resps.into_iter().map(|r| GraphQLResponse::from_ag(&r)).collect()
                }
                BatchResponse::Single(resp) => vec![GraphQLResponse::from_ag(&resp)],
            }
        }

        /// Execute a raw query string.
        /// 执行原始查询字符串。
        pub async fn query(&self, query: impl Into<String>) -> GraphQLResponse {
            self.execute(GraphQLRequest::new(query)).await
        }

        /// Return the SDL (Schema Definition Language) representation.
        /// 返回 SDL（Schema 定义语言）表示。
        pub fn sdl(&self) -> String {
            self.schema.sdl()
        }

        /// Return a reference to the underlying async-graphql schema.
        /// 返回底层 async-graphql schema 的引用。
        pub fn inner(&self) -> &Schema<Q, M, S> {
            &self.schema
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SchemaBuilder helper
    // ─────────────────────────────────────────────────────────────────────────

    /// Convenience builder for constructing a Hiver-wrapped `async-graphql` schema.
    /// 方便构建 Hiver 包装的 async-graphql schema 的辅助构建器。
    ///
    /// # Example / 示例
    /// ```rust,ignore
    /// let engine = HiverGraphQLBuilder::new(QueryRoot, EmptyMutation, EmptySubscription)
    ///     .max_depth(10)
    ///     .introspection(true)
    ///     .build();
    /// ```
    pub struct HiverGraphQLBuilder<Q, M, S>
    where
        Q: ObjectType + 'static,
        M: ObjectType + 'static,
        S: SubscriptionType + 'static,
    {
        inner: async_graphql::SchemaBuilder<Q, M, S>,
    }

    impl<Q, M, S> HiverGraphQLBuilder<Q, M, S>
    where
        Q: ObjectType + 'static,
        M: ObjectType + 'static,
        S: SubscriptionType + 'static,
    {
        /// Create a new builder with the given root types.
        /// 使用给定的根类型创建新构建器。
        pub fn new(query: Q, mutation: M, subscription: S) -> Self {
            Self { inner: Schema::build(query, mutation, subscription) }
        }

        /// Set the maximum query depth.
        /// 设置最大查询深度。
        pub fn max_depth(mut self, depth: usize) -> Self {
            self.inner = self.inner.limit_depth(depth);
            self
        }

        /// Set the maximum query complexity.
        /// 设置最大查询复杂度。
        pub fn max_complexity(mut self, complexity: usize) -> Self {
            self.inner = self.inner.limit_complexity(complexity);
            self
        }

        /// Enable or disable introspection (default: enabled).
        /// 启用或禁用 introspection（默认启用）。
        pub fn introspection(mut self, enabled: bool) -> Self {
            if !enabled {
                self.inner = self.inner.disable_introspection();
            }
            self
        }

        /// Add global data available in all resolvers.
        /// 添加在所有解析器中可用的全局数据。
        pub fn data<D: Send + Sync + 'static>(mut self, val: D) -> Self {
            self.inner = self.inner.data(val);
            self
        }

        /// Build the `HiverGraphQL` engine.
        /// 构建 HiverGraphQL 引擎。
        pub fn build(self) -> HiverGraphQL<Q, M, S> {
            HiverGraphQL::new(self.inner.finish())
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // GraphiQL HTML
    // ─────────────────────────────────────────────────────────────────────────

    /// Generate the GraphiQL HTML playground page.
    /// 生成 GraphiQL HTML 游乐场页面。
    pub fn graphiql_html(graphql_endpoint: &str, subscription_endpoint: Option<&str>) -> String {
        let sub_endpoint = subscription_endpoint
            .map(|s| format!(", subscriptionUrl: '{s}'"))
            .unwrap_or_default();
        format!(
            concat!(
                "<!DOCTYPE html><html><head><title>GraphiQL - Hiver</title>",
                "<meta charset=\"utf-8\"/>",
                "<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"/>",
                "<script src=\"https://unpkg.com/react@17/umd/react.development.js\"></script>",
                "<script src=\"https://unpkg.com/react-dom@17/umd/react-dom.development.js\"></script>",
                "<script src=\"https://unpkg.com/graphiql@2.4.7/graphiql.min.js\"></script>",
                "<link rel=\"stylesheet\" href=\"https://unpkg.com/graphiql@2.4.7/graphiql.min.css\"/>",
                "<style>body{{height:100vh;margin:0;overflow:hidden}}#graphiql{{height:100vh}}</style>",
                "</head><body><div id=\"graphiql\">Loading...</div>",
                "<script>",
                "const fetcher=GraphiQL.createFetcher({{url:'{}'{}}});",
                "ReactDOM.render(React.createElement(GraphiQL,{{fetcher}}),document.getElementById('graphiql'));",
                "</script></body></html>"
            ),
            graphql_endpoint,
            sub_endpoint,
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Re-export commonly needed async-graphql derive macros / types
    // ─────────────────────────────────────────────────────────────────────────

    /// Re-exported from `async-graphql` for convenience.
    /// 从 async-graphql 重新导出，方便使用。
    pub use async_graphql::{
        Context, EmptyMutation, EmptySubscription, Enum, FieldResult, InputObject, Interface,
        MergedObject, MergedSubscription, Object, OneofObject, OutputType, Scalar, ScalarType,
        SimpleObject, Subscription, Union, ID,
    };

    #[cfg(test)]
    mod tests {
        use super::*;
        use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

        struct QueryRoot;
        #[Object]
        impl QueryRoot {
            async fn hello(&self) -> &str { "world" }
            async fn add(&self, a: i32, b: i32) -> i32 { a + b }
        }

        fn make_engine() -> HiverGraphQL<QueryRoot, EmptyMutation, EmptySubscription> {
            HiverGraphQLBuilder::new(QueryRoot, EmptyMutation, EmptySubscription)
                .max_depth(10)
                .introspection(true)
                .build()
        }

        #[tokio::test]
        async fn test_hello_query() {
            let engine = make_engine();
            let resp = engine.query("{ hello }").await;
            assert!(!resp.has_errors(), "errors: {:?}", resp.errors);
            assert_eq!(
                resp.data.as_ref().and_then(|d| d.get("hello")).and_then(|v| v.as_str()),
                Some("world")
            );
        }

        #[tokio::test]
        async fn test_arithmetic_query() {
            let engine = make_engine();
            let resp = engine.query("{ add(a: 3, b: 4) }").await;
            assert!(!resp.has_errors());
            assert_eq!(
                resp.data.as_ref().and_then(|d| d.get("add")).and_then(|v| v.as_i64()),
                Some(7)
            );
        }

        #[tokio::test]
        async fn test_sdl() {
            let engine = make_engine();
            let sdl = engine.sdl();
            assert!(sdl.contains("hello"));
        }

        #[tokio::test]
        async fn test_introspection() {
            let engine = make_engine();
            let resp = engine.query("{ __schema { queryType { name } } }").await;
            assert!(!resp.has_errors());
        }

        #[test]
        fn test_graphiql_html() {
            let html = graphiql_html("/graphql", Some("/ws/graphql"));
            assert!(html.contains("GraphiQL"));
            assert!(html.contains("/graphql"));
            assert!(html.contains("/ws/graphql"));
        }

        #[tokio::test]
        async fn test_variables() {
            let engine = make_engine();
            let resp = engine
                .execute(
                    GraphQLRequest::new("query Add($a: Int!, $b: Int!) { add(a: $a, b: $b) }")
                        .with_variables(serde_json::json!({"a": 10, "b": 20})),
                )
                .await;
            assert!(!resp.has_errors());
            assert_eq!(
                resp.data.as_ref().and_then(|d| d.get("add")).and_then(|v| v.as_i64()),
                Some(30)
            );
        }
    }
}
