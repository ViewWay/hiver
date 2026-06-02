//! GraphQL error types
//! GraphQL 错误类型
//!
//! # Equivalent to Spring for GraphQL / 等价于 Spring for GraphQL
//!
//! - GraphQL error response format per spec
//! - Error extensions with codes and paths
//!
//! # GraphQL 错误响应格式
//!
//! ```json
//! {
//!   "errors": [{
//!     "message": "Something went wrong",
//!     "locations": [{"line": 1, "column": 5}],
//!     "path": ["user", "name"],
//!     "extensions": {"code": "INTERNAL"}
//!   }]
//! }
//! ```

use std::fmt;

/// GraphQL error structure following the GraphQL specification
/// GraphQL 错误结构，遵循 GraphQL 规范
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphQLError {
    /// Human-readable error message
    /// 人类可读的错误消息
    pub message: String,

    /// Source locations where the error occurred
    /// 错误发生的源代码位置
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<SourceLocation>,

    /// Path to the field that caused the error
    /// 导致错误的字段路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<serde_json::Value>,

    /// Additional error information
    /// 额外的错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

impl GraphQLError {
    /// Create a new GraphQL error with just a message
    /// `创建仅包含消息的新GraphQL错误`
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            locations: Vec::new(),
            path: None,
            extensions: None,
        }
    }

    /// Add a source location to the error
    /// 向错误添加源代码位置
    pub fn with_location(mut self, line: u32, column: u32) -> Self {
        self.locations.push(SourceLocation { line, column });
        self
    }

    /// Set the path that caused the error
    /// 设置导致错误的路径
    pub fn with_path(mut self, path: impl Into<serde_json::Value>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set path from a list of field names
    /// 从字段名列表设置路径
    pub fn with_path_segments(mut self, segments: &[&str]) -> Self {
        self.path = Some(serde_json::Value::Array(
            segments
                .iter()
                .map(|s| serde_json::Value::String(s.to_string()))
                .collect(),
        ));
        self
    }

    /// Set error extensions
    /// 设置错误扩展
    pub fn with_extensions(mut self, extensions: impl Into<serde_json::Value>) -> Self {
        self.extensions = Some(extensions.into());
        self
    }

    /// Set error code in extensions
    /// 在扩展中设置错误代码
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        let code = code.into();
        match &mut self.extensions {
            Some(serde_json::Value::Object(map)) => {
                map.insert("code".to_string(), serde_json::Value::String(code));
            },
            _ => {
                self.extensions = Some(serde_json::json!({ "code": code }));
            },
        }
        self
    }

    /// Create a syntax error
    /// 创建语法错误
    pub fn syntax_error(message: impl Into<String>, line: u32, column: u32) -> Self {
        Self::new(message)
            .with_location(line, column)
            .with_code("SYNTAX_ERROR")
    }

    /// Create a validation error
    /// 创建验证错误
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(message).with_code("VALIDATION_ERROR")
    }

    /// Create an execution error
    /// 创建执行错误
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::new(message).with_code("EXECUTION_ERROR")
    }

    /// Create a resolver not found error
    /// 创建解析器未找到错误
    pub fn resolver_not_found(type_name: &str, field_name: &str) -> Self {
        Self::new(format!("No resolver registered for {}.{}", type_name, field_name))
            .with_code("RESOLVER_NOT_FOUND")
            .with_path_segments(&[type_name, field_name])
    }

    /// Create a context error
    /// 创建上下文错误
    pub fn context_error(message: impl Into<String>) -> Self {
        Self::new(message).with_code("CONTEXT_ERROR")
    }

    /// Create a data loader error
    /// 创建数据加载器错误
    pub fn loader_error(message: impl Into<String>) -> Self {
        Self::new(message).with_code("LOADER_ERROR")
    }

    /// Convert to an HTTP response body as JSON
    /// 转换为HTTP响应体的JSON格式
    pub fn to_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self)
            .unwrap_or_else(|_| format!("{{\"message\":\"{}\"}}", self.message).into_bytes())
    }
}

impl fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(path) = &self.path {
            write!(f, " at path {}", path)?;
        }
        if let Some(loc) = self.locations.first() {
            write!(f, " (line {}, column {})", loc.line, loc.column)?;
        }
        Ok(())
    }
}

impl std::error::Error for GraphQLError {}

/// Source location for a GraphQL error
/// GraphQL 错误的源代码位置
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    /// Line number (1-based)
    /// 行号（从1开始）
    pub line: u32,
    /// Column number (1-based)
    /// 列号（从1开始）
    pub column: u32,
}

/// A collection of GraphQL errors (the spec allows multiple errors in a response)
/// GraphQL 错误集合（规范允许响应中包含多个错误）
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GraphQLErrors(pub Vec<GraphQLError>);

impl GraphQLErrors {
    /// Create an empty error collection
    /// 创建空的错误集合
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create from a single error
    /// 从单个错误创建
    pub fn single(error: GraphQLError) -> Self {
        Self(vec![error])
    }

    /// Create from a message
    /// 从消息创建
    pub fn from_message(message: impl Into<String>) -> Self {
        Self::single(GraphQLError::new(message))
    }

    /// Add an error to the collection
    /// 向集合添加错误
    pub fn push(&mut self, error: GraphQLError) {
        self.0.push(error);
    }

    /// Check if there are any errors
    /// 检查是否有错误
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of errors
    /// 获取错误数量
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Convert to a JSON value for inclusion in a GraphQL response
    /// `转换为JSON值以包含在GraphQL响应中`
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.0).unwrap_or(serde_json::Value::Null)
    }
}

impl From<Vec<GraphQLError>> for GraphQLErrors {
    fn from(errors: Vec<GraphQLError>) -> Self {
        Self(errors)
    }
}

impl From<GraphQLError> for GraphQLErrors {
    fn from(error: GraphQLError) -> Self {
        Self::single(error)
    }
}

/// Convert GraphQL errors to an HTTP response
/// `将GraphQL错误转换为HTTP响应`
pub fn errors_to_response(errors: &GraphQLErrors) -> hiver_http::Response {
    let body = serde_json::json!({
        "errors": errors.0,
    });
    hiver_http::Response::json(&body)
}

/// Convert a single GraphQL error to an HTTP response
/// `将单个GraphQL错误转换为HTTP响应`
pub fn error_to_response(error: &GraphQLError) -> hiver_http::Response {
    errors_to_response(&GraphQLErrors::single(error.clone()))
}
