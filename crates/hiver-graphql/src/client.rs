//! GraphQL client for querying GraphQL APIs / GraphQL客户端
//! Equivalent to Spring for GraphQL's GraphQlClient / HttpGraphQlClient

use crate::error::GraphQLError;
use crate::transport::{GraphQLRequest, GraphQLResponse};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct GraphQlClient {
    endpoint: String,
}

impl GraphQlClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self { endpoint: endpoint.into() }
    }
    pub fn endpoint(&self) -> &str { &self.endpoint }
    pub fn document(&self, query: impl Into<String>) -> GraphQlRequest<'_> {
        GraphQlRequest { client: self, query: query.into(), variables: None, operation_name: None }
    }
    pub async fn execute(&self, request: &GraphQLRequest) -> Result<GraphQLResponse, GraphQLError> {
        let client = reqwest::Client::new();
        let resp = client.post(&self.endpoint).json(request).send().await
            .map_err(|e| GraphQLError::new(format!("HTTP: {e}")).with_code("HTTP_ERROR"))?;
        resp.json().await
            .map_err(|e| GraphQLError::new(format!("Parse: {e}")).with_code("PARSE_ERROR"))
    }
}

pub struct GraphQlRequest<'a> {
    client: &'a GraphQlClient,
    query: String,
    variables: Option<serde_json::Value>,
    operation_name: Option<String>,
}

impl<'a> GraphQlRequest<'a> {
    pub fn variables(mut self, vars: serde_json::Value) -> Self { self.variables = Some(vars); self }
    pub fn operation_name(mut self, name: impl Into<String>) -> Self { self.operation_name = Some(name.into()); self }
    pub fn build(self) -> GraphQLRequest {
        GraphQLRequest { query: self.query, variables: self.variables, operation_name: self.operation_name, extensions: None }
    }
    pub async fn retrieve(self) -> Result<GraphQLResponse, GraphQLError> {
        self.client.execute(&self.build()).await
    }
    pub async fn retrieve_as<T: for<'de> Deserialize<'de>>(self) -> Result<Option<T>, GraphQLError> {
        let response = self.retrieve().await?;
        match response.data {
            Some(data) => serde_json::from_value(data)
                .map(Some)
                .map_err(|e| GraphQLError::new(format!("Deser: {e}")).with_code("PARSE_ERROR")),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests {
    use super::*;

    #[test]
    fn test_client_construction() {
        let client = GraphQlClient::new("http://localhost:8080/graphql");
        assert_eq!(client.endpoint(), "http://localhost:8080/graphql");
    }

    #[test]
    fn test_request_builder() {
        let client = GraphQlClient::new("http://localhost/graphql");
        let req = client.document("{ hello }").variables(serde_json::json!({"n":"w"})).build();
        assert!(req.query.contains("hello"));
        assert!(req.variables.is_some());
    }
}
