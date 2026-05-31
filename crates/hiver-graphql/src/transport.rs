//! GraphQL transport layer (HTTP) / GraphQL传输层
//! Equivalent to Spring for GraphQL Server Transports

use crate::context::GraphQLContext;
use crate::error::GraphQLError;
use crate::schema::Schema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

impl GraphQLRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self { query: query.into(), variables: None, operation_name: None, extensions: None }
    }
    pub fn with_variables(mut self, v: serde_json::Value) -> Self { self.variables = Some(v); self }
    pub fn with_operation_name(mut self, n: impl Into<String>) -> Self { self.operation_name = Some(n.into()); self }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<GraphQLError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

impl GraphQLResponse {
    pub fn ok(data: serde_json::Value) -> Self {
        Self { data: Some(data), errors: Vec::new(), extensions: None }
    }
    pub fn error(errors: Vec<GraphQLError>) -> Self {
        Self { data: None, errors, extensions: None }
    }
    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }
    pub fn into_http_body(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::json!({"errors": [{"message": "Serialization failed"}]}))
    }
}

#[derive(Clone)]
pub struct GraphQLHandler {
    schema: Arc<Schema>,
    context_factory: Arc<dyn Fn() -> GraphQLContext + Send + Sync>,
}

impl GraphQLHandler {
    pub fn new(schema: Schema) -> Self {
        Self { schema: Arc::new(schema), context_factory: Arc::new(GraphQLContext::new) }
    }
    pub fn with_context_factory(mut self, f: impl Fn() -> GraphQLContext + Send + Sync + 'static) -> Self {
        self.context_factory = Arc::new(f);
        self
    }

    pub async fn execute(&self, request: GraphQLRequest) -> GraphQLResponse {
        let ctx = (self.context_factory)();
        match self.schema.execute(&request.query, request.variables.as_ref(), request.operation_name.as_deref(), &ctx).await {
            Ok(data) => GraphQLResponse::ok(data),
            Err(errors) => GraphQLResponse::error(errors),
        }
    }

    pub async fn query(&self, query: impl Into<String>) -> GraphQLResponse {
        self.execute(GraphQLRequest::new(query)).await
    }
}

pub fn graphiql_html(endpoint: &str) -> String {
    format!(
        concat!(
            "<!DOCTYPE html><html><head><title>GraphiQL - Nexus</title>",
            "<meta charset=\"utf-8\"/><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"/>",
            "<script src=\"https://unpkg.com/react@17/umd/react.development.js\" crossorigin></script>",
            "<script src=\"https://unpkg.com/react-dom@17/umd/react-dom.development.js\" crossorigin></script>",
            "<script src=\"https://unpkg.com/graphiql@2.4.7/graphiql.min.js\"></script>",
            "<link rel=\"stylesheet\" href=\"https://unpkg.com/graphiql@2.4.7/graphiql.min.css\"/>",
            "<style>body{{height:100vh;margin:0;overflow:hidden}}#graphiql{{height:100vh}}</style>",
            "</head><body><div id=\"graphiql\">Loading...</div>",
            "<script>const r=React.createElement(GraphiQL,{{fetcher:GraphiQL.createFetcher({{url:'{}'}})}});",
            "ReactDOM.render(r,document.getElementById('graphiql'));</script></body></html>"
        ), endpoint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaBuilder;

    #[tokio::test]
    async fn test_handler() {
        let schema = SchemaBuilder::new().build();
        let handler = GraphQLHandler::new(schema);
        let resp = handler.query("{ hello }").await;
        assert!(!resp.has_errors());
    }

    #[test]
    fn test_graphiql_html() {
        let html = graphiql_html("/graphql");
        assert!(html.contains("GraphiQL"));
    }

    #[test]
    fn test_request_serde() {
        let req = GraphQLRequest::new("{ test }").with_variables(serde_json::json!({"id":1}));
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("test"));
    }
}
