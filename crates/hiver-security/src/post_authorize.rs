//! `PostAuthorize` module
//! PostAuthorize模块（@PostAuthorize等价物）

#![allow(dead_code)]
use std::{future::Future, pin::Pin};

use crate::{SecurityContext, pre_authorize::SecurityExpression};

/// `PostAuthorize` trait
/// `PostAuthorize` trait
///
/// Equivalent to Spring's @`PostAuthorize` annotation.
/// Unlike PreAuthorize, PostAuthorize evaluates AFTER method execution
/// and can access the return value via `#returnObject`.
///
/// 等价于Spring的@PostAuthorize注解。
/// 与PreAuthorize不同，PostAuthorize在方法执行后评估，
/// 可以通过 `#returnObject` 访问返回值。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @PostAuthorize("returnObject.owner == authentication.name")
/// public Document getDocument(Long id) { }
///
/// @PostAuthorize("hasPermission(returnObject, 'READ')")
/// public Document readDocument(Long id) { }
/// ```
pub trait PostAuthorize<T>
{
    /// Check authorization after method execution, with access to the return value
    /// 在方法执行后检查授权，可访问返回值
    fn check_post_authorize(
        &self,
        context: &SecurityContext,
        return_value: &T,
    ) -> Pin<Box<dyn Future<Output = bool> + Send>>;
}

/// `PostAuthorize` options
/// `PostAuthorize` 选项
#[derive(Debug, Clone)]
pub struct PostAuthorizeOptions
{
    /// Security expressions
    /// 安全表达式
    pub expressions: Vec<SecurityExpression>,

    /// All expressions must pass (AND logic)
    /// 所有表达式必须通过（AND逻辑）
    pub require_all: bool,

    /// Expression referencing return object (e.g. "returnObject.owner == 'admin'")
    /// 引用返回对象的表达式
    pub return_object_filter: Option<String>,
}

impl PostAuthorizeOptions
{
    /// Create new options
    /// 创建新选项
    pub fn new() -> Self
    {
        Self {
            expressions: Vec::new(),
            require_all: true,
            return_object_filter: None,
        }
    }

    /// Add expression
    /// 添加表达式
    pub fn add_expression(mut self, expr: SecurityExpression) -> Self
    {
        self.expressions.push(expr);
        self
    }

    /// Parse and add expression string
    /// 解析并添加表达式字符串
    pub fn add_expression_string(mut self, expr: impl Into<String>) -> Self
    {
        let parsed = SecurityExpression::parse(&expr.into());
        self.expressions.extend(parsed);
        self
    }

    /// Set a return object filter expression
    /// 设置返回对象过滤表达式
    pub fn return_object_filter(mut self, filter: impl Into<String>) -> Self
    {
        self.return_object_filter = Some(filter.into());
        self
    }

    /// Set require all (AND) or require any (OR)
    /// 设置需要全部（AND）或需要任一（OR）
    pub fn require_all(mut self, require_all: bool) -> Self
    {
        self.require_all = require_all;
        self
    }

    /// Evaluate all expressions against security context
    /// 根据安全上下文评估所有表达式
    pub async fn evaluate(&self, context: &SecurityContext) -> bool
    {
        if self.expressions.is_empty() && self.return_object_filter.is_none()
        {
            return true;
        }

        if self.require_all
        {
            for expr in &self.expressions
            {
                if !expr.evaluate(context).await
                {
                    return false;
                }
            }
            true
        }
        else
        {
            for expr in &self.expressions
            {
                if expr.evaluate(context).await
                {
                    return true;
                }
            }
            false
        }
    }
}

impl Default for PostAuthorizeOptions
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Helper function to check post-authorize
/// 检查post-authorize的助手函数
pub async fn check_post_authorize(
    context: &SecurityContext,
    expression: &str,
) -> Result<bool, crate::SecurityError>
{
    let options = PostAuthorizeOptions::new().add_expression_string(expression);
    Ok(options.evaluate(context).await)
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_post_authorize_options_builder()
    {
        let opts = PostAuthorizeOptions::new()
            .add_expression_string("hasRole('ADMIN')")
            .return_object_filter("returnObject.owner == 'admin'");

        assert_eq!(opts.expressions.len(), 1);
        assert!(opts.return_object_filter.is_some());
    }

    #[test]
    fn test_post_authorize_options_default()
    {
        let opts = PostAuthorizeOptions::default();
        assert!(opts.expressions.is_empty());
        assert!(opts.require_all);
        assert!(opts.return_object_filter.is_none());
    }

    #[tokio::test]
    async fn test_post_authorize_empty_expressions()
    {
        let context = SecurityContext::new();
        let opts = PostAuthorizeOptions::new();
        assert!(opts.evaluate(&context).await);
    }

    #[tokio::test]
    async fn test_post_authorize_check()
    {
        let context = SecurityContext::new();
        let result = check_post_authorize(&context, "hasRole('ADMIN')").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
