//! Matrix Variable Extractor
//! 矩阵变量提取器
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@MatrixVariable` - Extract matrix variables from path segments
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::{MatrixVariable, MatrixVariables};
//!
//! // URL: /users;color=red;size=large/123
//! async fn handler(vars: MatrixVariables) -> String {
//!     format!("{:?}", vars.0)  // {"color": "red", "size": "large"}
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{collections::HashMap, future::Future, pin::Pin};

use crate::{ExtractorError, FromRequest, Request};

/// Matrix variable extractor - extracts the first matrix variable value
/// 矩阵变量提取器 - 提取第一个矩阵变量值
///
/// # Description / 描述
///
/// Extracts the first matrix variable value from the path.
/// Matrix variables appear after semicolons in path segments.
/// 从路径中提取第一个矩阵变量值。矩阵变量出现在路径段的分号之后。
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type to deserialize the value into. Must implement `serde::Deserialize`.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::MatrixVariable;
///
/// // For URL: /users;color=red/123
/// // color.0 will be "red"
/// async fn handler(color: MatrixVariable<String>) -> String {
///     color.0
/// }
/// ```
///
/// # Note / 注意
///
/// Matrix variables are defined in RFC 3986 and allow parameters to be
/// placed within path segments using semicolons.
/// 矩阵变量在 RFC 3986 中定义，允许使用分号在路径段中放置参数。
///
/// Path format: `/path;key1=value1;key2=value2/more`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixVariable<T>(pub T);

/// Extract all matrix variables from the path as a `HashMap`.
/// 从路径提取所有矩阵变量为 `HashMap`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::MatrixVariables;
///
/// // For URL: /users;color=red;size=large/123
/// // variables.0 will be {"color": "red", "size": "large"}
/// async fn handler(variables: MatrixVariables) -> String {
///     format!("{:?}", variables.0)
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixVariables(pub HashMap<String, String>);

/// Extract all matrix variables from a specific path segment as a `Vec`.
/// 从特定路径段提取所有矩阵变量为 `Vec`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::MatrixPath;
///
/// // For URL: /users;color=red;size=large/123
/// // path.0 will be [("color", "red"), ("size", "large")]
/// async fn handler(path: MatrixPath) -> String {
///     format!("{:?}", path.0)
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixPath(pub Vec<(String, String)>);

impl<T> FromRequest for MatrixVariable<T>
where
    T: for<'de> serde::Deserialize<'de> + Send + 'static,
{
    fn from_request(
        req: &Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, ExtractorError>> + Send>> {
        let path = req.path().to_string();

        Box::pin(async move {
            // Find the first matrix variable in the path
            let value = extract_first_matrix_value(&path)
                .ok_or_else(|| ExtractorError::Missing("No matrix variable found".to_string()))?;

            // Deserialize to target type
            let parsed = if value.contains(',') {
                // Handle comma-separated values as JSON array
                serde_json::from_str(format!("[{}]", value).as_str())
            } else {
                // Single value - try to deserialize directly
                serde_json::from_value(serde_json::Value::String(value))
            }
            .map_err(|e| {
                ExtractorError::Invalid(format!("Invalid matrix variable value: {}", e))
            })?;

            Ok(MatrixVariable(parsed))
        })
    }
}

impl FromRequest for MatrixVariables {
    fn from_request(
        req: &Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, ExtractorError>> + Send>> {
        let path = req.path().to_string();

        Box::pin(async move {
            let variables = parse_matrix_variables(&path);
            Ok(MatrixVariables(variables))
        })
    }
}

impl FromRequest for MatrixPath {
    fn from_request(
        req: &Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, ExtractorError>> + Send>> {
        let path = req.path().to_string();

        Box::pin(async move {
            let variables = parse_matrix_variables_as_vec(&path);
            Ok(MatrixPath(variables))
        })
    }
}

impl MatrixVariables {
    /// Get a specific matrix variable by name
    /// 通过名称获取特定的矩阵变量
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    /// Get a specific matrix variable by name, returning a default if not found
    /// 通过名称获取特定的矩阵变量，如果未找到则返回默认值
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.0.get(key).map_or(default, String::as_str).to_string()
    }
}

/// Extract the first matrix variable value from a path
/// 从路径中提取第一个矩阵变量值
fn extract_first_matrix_value(path: &str) -> Option<String> {
    for segment in path.split('/') {
        if let Some(semi_pos) = segment.find(';') {
            let matrix_part = &segment[semi_pos + 1..];
            if let Some(eq_pos) = matrix_part.find('=') {
                let value_part = &matrix_part[eq_pos + 1..];
                // Stop at the next semicolon if present
                let value = if let Some(semicolon_pos) = value_part.find(';') {
                    &value_part[..semicolon_pos]
                } else {
                    value_part
                };
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Parse all matrix variables from a path into a `HashMap`.
/// 将路径中的所有矩阵变量解析为 `HashMap`。
fn parse_matrix_variables(path: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for segment in path.split('/') {
        if let Some(semi_pos) = segment.find(';') {
            let matrix_part = &segment[semi_pos + 1..];
            // Parse key=value pairs separated by ;
            for pair in matrix_part.split(';') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = pair[..eq_pos].to_string();
                    let value = pair[eq_pos + 1..].to_string();
                    result.insert(key, value);
                }
            }
        }
    }

    result
}

/// Parse all matrix variables from a path into a Vec
/// 将路径中的所有矩阵变量解析为Vec
fn parse_matrix_variables_as_vec(path: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();

    for segment in path.split('/') {
        if let Some(semi_pos) = segment.find(';') {
            let matrix_part = &segment[semi_pos + 1..];
            // Parse key=value pairs separated by ;
            for pair in matrix_part.split(';') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = pair[..eq_pos].to_string();
                    let value = pair[eq_pos + 1..].to_string();
                    result.push((key, value));
                }
            }
        }
    }

    result
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_first_matrix_value() {
        let path = "/users;color=red;size=large/123";
        let value = extract_first_matrix_value(path);
        assert_eq!(value, Some("red".to_string()));
    }

    #[test]
    fn test_extract_first_matrix_value_no_semicolon() {
        let path = "/users/123";
        let value = extract_first_matrix_value(path);
        assert_eq!(value, None);
    }

    #[test]
    fn test_parse_matrix_variables() {
        let path = "/users;color=red;size=large/123";
        let vars = parse_matrix_variables(path);
        assert_eq!(vars.get("color"), Some(&"red".to_string()));
        assert_eq!(vars.get("size"), Some(&"large".to_string()));
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_parse_matrix_variables_multiple_segments() {
        let path = "/users;color=blue/123;id=456";
        let vars = parse_matrix_variables(path);
        assert_eq!(vars.get("color"), Some(&"blue".to_string()));
        assert_eq!(vars.get("id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_parse_matrix_variables_as_vec() {
        let path = "/users;color=red;size=large/123";
        let vars = parse_matrix_variables_as_vec(path);
        assert_eq!(
            vars,
            vec![
                ("color".to_string(), "red".to_string()),
                ("size".to_string(), "large".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_matrix_variables_empty() {
        let path = "/users/123";
        let vars = parse_matrix_variables(path);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_matrix_variables_get() {
        let vars = MatrixVariables(
            vec![
                ("color".to_string(), "red".to_string()),
                ("size".to_string(), "large".to_string()),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(vars.get("color"), Some(&"red".to_string()));
        assert_eq!(vars.get("missing"), None);
    }

    #[test]
    fn test_matrix_variables_get_or() {
        let vars = MatrixVariables(
            vec![("color".to_string(), "red".to_string())]
                .into_iter()
                .collect(),
        );

        assert_eq!(vars.get_or("color", "blue"), "red");
        assert_eq!(vars.get_or("missing", "default"), "default");
    }
}
