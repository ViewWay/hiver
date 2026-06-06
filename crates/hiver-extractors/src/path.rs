//! Path extractor module
//! 路径提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Path<T>` - `@PathVariable`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::Path;
//!
//! // GET /users/:id
//! async fn get_user(Path(id): Path<u64>) -> String {
//!     format!("User ID: {}", id)
//! }
//!
//! // Multiple path variables
//! // GET /users/:user_id/posts/:post_id
//! async fn get_post(Path((user_id, post_id)): Path<(u64, u64)>) -> String {
//!     format!("User: {}, Post: {}", user_id, post_id)
//! }
//! ```

use std::str::FromStr;

use crate::{ExtractorError, ExtractorFuture, FromRequest, Request};

/// Path parameter extractor
/// 路径参数提取器
///
/// Equivalent to Spring's `@PathVariable`.
/// 等价于Spring的`@PathVariable`。
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type to extract. Can be:
///   - A single value: `Path<u64>`, `Path<String>`, etc.
///   - A tuple: `Path<(u64, String)>`, `Path<(u64, u64, String)>`, etc.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// // Single parameter
/// async fn user(Path(id): Path<u64>) -> String {
///     format!("User {}", id)
/// }
///
/// // Multiple parameters
/// async fn item(Path((user, item)): Path<(u64, u64)>) -> String {
///     format!("User {} Item {}", user, item)
/// }
/// ```
pub struct Path<T>(pub T);

impl<T> Path<T>
{
    /// Consume the path extractor and get the inner value
    /// 消耗路径提取器并获取内部值
    pub fn into_inner(self) -> T
    {
        self.0
    }

    /// Get reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T
    {
        &self.0
    }
}

impl<T> std::fmt::Debug for Path<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_tuple("Path").field(&self.0).finish()
    }
}

impl<T> Clone for Path<T>
where
    T: Clone,
{
    fn clone(&self) -> Self
    {
        Self(self.0.clone())
    }
}

// Implement FromRequest for single values using FromStr
macro_rules! impl_from_request_single {
    ($($ty:ty),*) => {
        $(
            impl FromRequest for Path<$ty> {
                fn from_request(req: &Request) -> ExtractorFuture<Self> {
                    let path_vars = req.path_vars().clone();

                    Box::pin(async move {
                        // Get the first path variable
                        let value = path_vars
                            .values()
                            .next()
                            .ok_or_else(|| ExtractorError::Missing("path parameter".to_string()))?;

                        <$ty as FromStr>::from_str(value)
                            .map_err(|e| ExtractorError::Invalid(format!("{}: {}", value, e)))
                            .map(Path)
                    })
                }
            }
        )*
    };
}

impl_from_request_single!(String, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, bool);

// Implement FromRequest for tuples
impl<T1, T2> FromRequest for Path<(T1, T2)>
where
    T1: FromStr + Send + 'static,
    T1::Err: std::fmt::Display,
    T2: FromStr + Send + 'static,
    T2::Err: std::fmt::Display,
{
    fn from_request(req: &Request) -> ExtractorFuture<Self>
    {
        let path_vars = req.path_vars().clone();
        let var_names: Vec<_> = path_vars.keys().cloned().collect();

        Box::pin(async move {
            if var_names.len() < 2
            {
                return Err(ExtractorError::Missing("expected 2 path parameters".to_string()));
            }

            let v1 = T1::from_str(
                path_vars
                    .get(var_names.first().ok_or_else(|| {
                        ExtractorError::Missing("expected 2 path parameters".to_string())
                    })?)
                    .ok_or_else(|| {
                        ExtractorError::Missing("expected 2 path parameters".to_string())
                    })?,
            )
            .map_err(|e| ExtractorError::Invalid(format!("parameter 0: {}", e)))?;
            let v2 = T2::from_str(
                path_vars
                    .get(var_names.get(1).ok_or_else(|| {
                        ExtractorError::Missing("expected 2 path parameters".to_string())
                    })?)
                    .ok_or_else(|| {
                        ExtractorError::Missing("expected 2 path parameters".to_string())
                    })?,
            )
            .map_err(|e| ExtractorError::Invalid(format!("parameter 1: {}", e)))?;

            Ok(Path((v1, v2)))
        })
    }
}

// Implement FromRequest for 3-tuples
impl<T1, T2, T3> FromRequest for Path<(T1, T2, T3)>
where
    T1: FromStr + Send + 'static,
    T1::Err: std::fmt::Display,
    T2: FromStr + Send + 'static,
    T2::Err: std::fmt::Display,
    T3: FromStr + Send + 'static,
    T3::Err: std::fmt::Display,
{
    fn from_request(req: &Request) -> ExtractorFuture<Self>
    {
        let path_vars = req.path_vars().clone();
        let var_names: Vec<_> = path_vars.keys().cloned().collect();

        Box::pin(async move {
            if var_names.len() < 3
            {
                return Err(ExtractorError::Missing("expected 3 path parameters".to_string()));
            }

            let v1 = T1::from_str(
                path_vars
                    .get(var_names.first().ok_or_else(|| {
                        ExtractorError::Missing("expected 3 path parameters".to_string())
                    })?)
                    .ok_or_else(|| {
                        ExtractorError::Missing("expected 3 path parameters".to_string())
                    })?,
            )
            .map_err(|e| ExtractorError::Invalid(format!("parameter 0: {}", e)))?;
            let v2 = T2::from_str(
                path_vars
                    .get(var_names.get(1).ok_or_else(|| {
                        ExtractorError::Missing("expected 3 path parameters".to_string())
                    })?)
                    .ok_or_else(|| {
                        ExtractorError::Missing("expected 3 path parameters".to_string())
                    })?,
            )
            .map_err(|e| ExtractorError::Invalid(format!("parameter 1: {}", e)))?;
            let v3 = T3::from_str(
                path_vars
                    .get(var_names.get(2).ok_or_else(|| {
                        ExtractorError::Missing("expected 3 path parameters".to_string())
                    })?)
                    .ok_or_else(|| {
                        ExtractorError::Missing("expected 3 path parameters".to_string())
                    })?,
            )
            .map_err(|e| ExtractorError::Invalid(format!("parameter 2: {}", e)))?;

            Ok(Path((v1, v2, v3)))
        })
    }
}

/// Get a path parameter by name
/// 按名称获取路径参数
///
/// Equivalent to Spring's `@PathVariable("id")`.
/// 等价于Spring的`@PathVariable("id")`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::path::get_path_var;
///
/// async fn handler(req: &Request) -> String {
///     let id = get_path_var(req, "id").unwrap_or_default();
///     format!("ID: {}", id)
/// }
/// ```
pub fn get_path_var(req: &Request, name: &str) -> Option<String>
{
    req.path_var(name).map(ToString::to_string)
}

/// Get all path variables
/// 获取所有路径变量
pub fn get_all_path_vars(req: &Request) -> std::collections::HashMap<String, String>
{
    req.path_vars().clone()
}

/// Check if a path variable exists
/// 检查路径变量是否存在
pub fn has_path_var(req: &Request, name: &str) -> bool
{
    req.path_var(name).is_some()
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_path_into_inner()
    {
        let path: Path<u64> = Path(123);
        assert_eq!(path.into_inner(), 123);
    }
}
