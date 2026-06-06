//! Request extension for `TransactionContext`
//! `TransactionContext的Request扩展`
//!
//! This module provides Request-based `TransactionContext` that works across async boundaries.
//! 本模块提供基于Request的TransactionContext，可在异步边界间工作。

use std::sync::Arc;

use hiver_http::Request;
use tokio::sync::RwLock;

use crate::{Transaction, TransactionStatus};

/// `TransactionContext` extension for Request
/// `Request的TransactionContext扩展`
///
/// This allows `TransactionContext` to be passed through Request extensions,
/// making it available across async boundaries without `ThreadLocal`.
///
/// `这允许TransactionContext通过Request扩展传递`，
/// 使其在异步边界间可用，无需ThreadLocal。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_tx::request_ext::TransactionContextExt;
/// use hiver_http::Request;
///
/// async fn handler(req: Request) -> Result<Response> {
///     // Get TransactionContext from Request
///     let ctx = TransactionContextExt::from_request(&req)?;
///     let tx = ctx.current_transaction().await;
///     Ok(Response::json(tx))
/// }
/// ```
#[derive(Clone)]
pub struct TransactionContextExt
{
    /// Current transaction
    /// 当前事务
    current: Arc<RwLock<Option<Transaction>>>,

    /// Transaction stack (for nested transactions)
    /// 事务栈（用于嵌套事务）
    stack: Arc<RwLock<Vec<Transaction>>>,
}

impl TransactionContextExt
{
    /// Create a new `TransactionContext` extension
    /// `创建新的TransactionContext扩展`
    pub fn new() -> Self
    {
        Self {
            current: Arc::new(RwLock::new(None)),
            stack: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get `TransactionContext` from Request extensions
    /// `从Request扩展中获取TransactionContext`
    ///
    /// Returns None if `TransactionContext` is not found in the request.
    /// 如果请求中未找到TransactionContext，则返回None。
    pub fn from_request(req: &Request) -> Option<Arc<Self>>
    {
        req.extensions().get::<Arc<Self>>().cloned()
    }

    /// Set `TransactionContext` to Request extensions
    /// `将TransactionContext设置到Request扩展`
    pub fn set_to_request(req: &mut Request) -> Arc<Self>
    {
        let ctx = Arc::new(Self::new());
        req.extensions_mut().insert(ctx.clone());
        ctx
    }

    /// Get current transaction
    /// 获取当前事务
    pub async fn current_transaction(&self) -> Option<Transaction>
    {
        self.current.read().await.clone()
    }

    /// Set current transaction
    /// 设置当前事务
    pub async fn set_current_transaction(&self, tx: Transaction)
    {
        let mut current = self.current.write().await;
        *current = Some(tx);
    }

    /// Clear current transaction
    /// 清除当前事务
    pub async fn clear(&self)
    {
        let mut current = self.current.write().await;
        *current = None;
    }

    /// Push transaction onto stack (for nested transactions)
    /// 将事务压入栈（用于嵌套事务）
    pub async fn push_transaction(&self, tx: Transaction)
    {
        let mut stack = self.stack.write().await;
        stack.push(tx);
    }

    /// Pop transaction from stack
    /// 从栈弹出事务
    pub async fn pop_transaction(&self) -> Option<Transaction>
    {
        let mut stack = self.stack.write().await;
        stack.pop()
    }

    /// Get stack depth
    /// 获取栈深度
    pub async fn stack_depth(&self) -> usize
    {
        self.stack.read().await.len()
    }

    /// Check if there is an active transaction
    /// 检查是否有活动事务
    pub async fn has_active_transaction(&self) -> bool
    {
        self.current
            .read()
            .await
            .as_ref()
            .is_some_and(Transaction::is_active)
    }

    /// Get transaction status
    /// 获取事务状态
    pub async fn transaction_status(&self) -> Option<TransactionStatus>
    {
        self.current
            .read()
            .await
            .as_ref()
            .map(|tx| tx.status().clone())
    }
}

impl Default for TransactionContextExt
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Convenience function: Get current transaction from Request
/// 便捷函数：从Request获取当前事务
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_tx::request_ext::get_transaction_from_request;
/// use hiver_http::Request;
///
/// async fn handler(req: Request) -> Result<Response> {
///     let tx = get_transaction_from_request(&req).await?;
///     Ok(Response::json(tx))
/// }
/// ```
pub async fn get_transaction_from_request(req: &Request) -> Option<Transaction>
{
    TransactionContextExt::from_request(req)?
        .current_transaction()
        .await
}

/// Convenience function: Check if request has active transaction
/// 便捷函数：检查请求是否有活动事务
pub async fn has_active_transaction_in_request(req: &Request) -> bool
{
    if let Some(ctx) = TransactionContextExt::from_request(req)
    {
        ctx.has_active_transaction().await
    }
    else
    {
        false
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use hiver_http::{Method, Request};

    use super::*;
    use crate::Transaction;

    #[tokio::test]
    async fn test_transaction_context_ext()
    {
        let mut req = Request::from_method_uri(Method::GET, "/test");

        // Set TransactionContext
        let ctx = TransactionContextExt::set_to_request(&mut req);

        // Get from Request
        let ctx2 = TransactionContextExt::from_request(&req).unwrap();
        assert_eq!(Arc::as_ptr(&ctx), Arc::as_ptr(&ctx2));

        // Test transaction
        let tx = Transaction::new("test");
        ctx.set_current_transaction(tx.clone()).await;

        assert!(ctx.has_active_transaction().await);

        // Get from Request again
        let tx_from_req = get_transaction_from_request(&req).await;
        assert_eq!(tx_from_req.map(|t| t.status().name().to_string()), Some("test".to_string()));
    }
}
