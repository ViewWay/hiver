//! Validated JSON extractor — Spring's `@Valid @RequestBody`.
//! 验证 JSON 提取器 — Spring 的 `@Valid @RequestBody`。

use hiver_http::HttpBody;
use serde::Deserialize;

use crate::{ExtractorError, ExtractorFuture, FromRequest, Request};

/// Maximum request body size (1 MB). Prevents OOM from unbounded body reads.
/// 最大请求体大小（1 MB）。防止无限制的请求体读取导致 OOM。
const MAX_BODY_SIZE: usize = 1024 * 1024;

/// Validation trait — matches what `hiver-validation-annotations` derive generates.
/// 验证 trait — 匹配 `hiver-validation-annotations` derive 生成的方法签名。
pub trait Validate
{
    /// Validate this value.
    /// 验证此值。
    fn validate(&self) -> Result<(), String>;
}

/// Validated JSON extractor — `@Valid @RequestBody` equivalent.
/// 验证 JSON 提取器 — `@Valid @RequestBody` 等价物。
pub struct Valid<T>(pub T);

impl<T> Valid<T>
{
    /// Consume and get inner value.
    pub fn into_inner(self) -> T { self.0 }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Valid<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_tuple("Valid").field(&self.0).finish()
    }
}

impl<T> FromRequest for Valid<T>
where
    T: for<'de> Deserialize<'de> + Validate + Send + 'static,
{
    fn from_request(req: &Request) -> ExtractorFuture<Self>
    {
        let body_bytes = req.body().as_bytes().map(<[u8]>::to_vec);
        let content_type = req.header("content-type").unwrap_or("").to_string();

        Box::pin(async move {
            if !content_type.starts_with("application/json")
                && !content_type.starts_with("application/")
                && !content_type.is_empty()
            {
                return Err(ExtractorError::Invalid(format!(
                    "Expected JSON content type, got: {content_type}"
                )));
            }

            let body = body_bytes.ok_or_else(|| {
                ExtractorError::Invalid("Request body is not available".to_string())
            })?;

            if body.len() > MAX_BODY_SIZE
            {
                return Err(ExtractorError::Invalid(format!(
                    "Request body too large: {} bytes (max {MAX_BODY_SIZE})",
                    body.len()
                )));
            }

            let value: T = serde_json::from_slice(&body).map_err(ExtractorError::from)?;

            value.validate().map_err(|e| {
                ExtractorError::Invalid(format!("Validation failed: {e}"))
            })?;

            Ok(Valid(value))
        })
    }
}
