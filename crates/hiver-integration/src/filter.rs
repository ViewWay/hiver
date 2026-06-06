//! Message filtering patterns
//! 消息过滤模式

use std::sync::Arc;

use async_trait::async_trait;

use crate::message::Message;

/// Message filter
/// 消息过滤器
#[async_trait]
pub trait MessageFilter: Send + Sync
{
    /// Test if a message should be accepted
    /// 测试消息是否应该被接受
    async fn test(&self, message: &Message) -> bool;

    /// Filter a message, returning Some if accepted, None if rejected
    /// 过滤消息，如果接受返回 Some，否则返回 None
    async fn filter(&self, message: Message) -> Option<Message>
    {
        if self.test(&message).await
        {
            Some(message)
        }
        else
        {
            None
        }
    }
}

/// Predicate-based filter
/// 基于谓词的过滤器
pub struct PredicateFilter
{
    predicate: Arc<dyn Fn(&Message) -> bool + Send + Sync>,
}

impl PredicateFilter
{
    /// Create a new predicate filter
    /// 创建新的谓词过滤器
    pub fn new<F>(predicate: F) -> Self
    where
        F: Fn(&Message) -> bool + Send + Sync + 'static,
    {
        Self {
            predicate: Arc::new(predicate),
        }
    }
}

#[async_trait]
impl MessageFilter for PredicateFilter
{
    async fn test(&self, message: &Message) -> bool
    {
        (self.predicate)(message)
    }
}

/// Header filter - filters based on header values
/// 头部过滤器 - 基于头部值过滤
pub struct HeaderFilter
{
    header_name: String,
    expected: Option<String>,
}

impl HeaderFilter
{
    /// Create a new header filter that checks for header presence
    /// 创建检查头部存在性的新头部过滤器
    pub fn has(header_name: impl Into<String>) -> Self
    {
        Self {
            header_name: header_name.into(),
            expected: None,
        }
    }

    /// Create a new header filter that checks for header value
    /// 创建检查头部值的新头部过滤器
    pub fn equals(header_name: impl Into<String>, expected: impl Into<String>) -> Self
    {
        Self {
            header_name: header_name.into(),
            expected: Some(expected.into()),
        }
    }
}

#[async_trait]
impl MessageFilter for HeaderFilter
{
    async fn test(&self, message: &Message) -> bool
    {
        match &self.expected
        {
            Some(expected) => message
                .header(&self.header_name)
                .and_then(|h| h.as_str())
                .map(|v| v == expected)
                .unwrap_or(false),
            None => message.header(&self.header_name).is_some(),
        }
    }
}

/// Payload type filter - filters based on payload type
/// 载荷类型过滤器 - 基于载荷类型过滤
pub struct PayloadTypeFilter {}

impl PayloadTypeFilter
{
    /// Create a new payload type filter
    /// 创建新的载荷类型过滤器
    pub fn new<T: 'static>() -> Self
    {
        Self {}
    }
}

#[async_trait]
impl MessageFilter for PayloadTypeFilter
{
    async fn test(&self, message: &Message) -> bool
    {
        // Check if payload can be downcast to the expected type
        // 检查载荷是否可以向下转换为期望的类型
        message.payload().downcast_ref::<String>().is_some()
            || message.payload().downcast_ref::<i32>().is_some()
            || message.payload().downcast_ref::<i64>().is_some()
            || message.payload().downcast_ref::<f64>().is_some()
            || message.payload().downcast_ref::<bool>().is_some()
    }
}

/// Negation filter - inverts another filter
/// 取反过滤器 - 反转另一个过滤器
pub struct NotFilter
{
    inner: Arc<dyn MessageFilter>,
}

impl NotFilter
{
    /// Create a new negation filter
    /// 创建新的取反过滤器
    pub fn new(filter: Arc<dyn MessageFilter>) -> Self
    {
        Self { inner: filter }
    }
}

#[async_trait]
impl MessageFilter for NotFilter
{
    async fn test(&self, message: &Message) -> bool
    {
        !self.inner.test(message).await
    }
}

/// Logical AND filter - combines filters with AND logic
/// 逻辑 AND 过滤器 - 使用 AND 逻辑组合过滤器
pub struct AndFilter
{
    filters: Vec<Arc<dyn MessageFilter>>,
}

impl AndFilter
{
    /// Create a new AND filter
    /// 创建新的 AND 过滤器
    pub fn new(filters: Vec<Arc<dyn MessageFilter>>) -> Self
    {
        Self { filters }
    }

    /// Add a filter to the combination
    /// 添加过滤器到组合
    pub fn add(mut self, filter: Arc<dyn MessageFilter>) -> Self
    {
        self.filters.push(filter);
        self
    }
}

#[async_trait]
impl MessageFilter for AndFilter
{
    async fn test(&self, message: &Message) -> bool
    {
        for filter in &self.filters
        {
            if !filter.test(message).await
            {
                return false;
            }
        }
        !self.filters.is_empty()
    }
}

/// Logical OR filter - combines filters with OR logic
/// 逻辑 OR 过滤器 - 使用 OR 逻辑组合过滤器
pub struct OrFilter
{
    filters: Vec<Arc<dyn MessageFilter>>,
}

impl OrFilter
{
    /// Create a new OR filter
    /// 创建新的 OR 过滤器
    pub fn new(filters: Vec<Arc<dyn MessageFilter>>) -> Self
    {
        Self { filters }
    }

    /// Add a filter to the combination
    /// 添加过滤器到组合
    pub fn add(mut self, filter: Arc<dyn MessageFilter>) -> Self
    {
        self.filters.push(filter);
        self
    }
}

#[async_trait]
impl MessageFilter for OrFilter
{
    async fn test(&self, message: &Message) -> bool
    {
        for filter in &self.filters
        {
            if filter.test(message).await
            {
                return true;
            }
        }
        false
    }
}

/// Threshold filter - filters numeric values
/// 阈值过滤器 - 过滤数值
pub enum ThresholdFilter
{
    LessThan(i64),
    GreaterThan(i64),
    Between(i64, i64),
    Equal(i64),
}

impl ThresholdFilter
{
    /// Create a less-than filter
    /// 创建小于过滤器
    pub fn less_than(value: i64) -> Self
    {
        Self::LessThan(value)
    }

    /// Create a greater-than filter
    /// 创建大于过滤器
    pub fn greater_than(value: i64) -> Self
    {
        Self::GreaterThan(value)
    }

    /// Create a between filter (inclusive)
    /// 创建区间过滤器（包含边界）
    pub fn between(min: i64, max: i64) -> Self
    {
        Self::Between(min, max)
    }

    /// Create an equal filter
    /// 创建等于过滤器
    pub fn equal(value: i64) -> Self
    {
        Self::Equal(value)
    }
}

#[async_trait]
impl MessageFilter for ThresholdFilter
{
    async fn test(&self, message: &Message) -> bool
    {
        let value = message.get_payload::<i64>();
        match (self, value)
        {
            (ThresholdFilter::LessThan(threshold), Some(v)) => v < *threshold,
            (ThresholdFilter::GreaterThan(threshold), Some(v)) => v > *threshold,
            (ThresholdFilter::Between(min, max), Some(v)) => v >= *min && v <= *max,
            (ThresholdFilter::Equal(expected), Some(v)) => v == *expected,
            (_, None) => false,
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_predicate_filter()
    {
        let filter = PredicateFilter::new(|msg| msg.get_payload::<i32>().is_some_and(|v| v > 10));

        assert!(!filter.test(&Message::new(5)).await);
        assert!(filter.test(&Message::new(15)).await);
    }

    #[tokio::test]
    async fn test_header_filter()
    {
        let filter_has = HeaderFilter::has("important");
        let filter_equals = HeaderFilter::equals("priority", "high");

        let mut msg1 = Message::new("test");
        msg1.set_header("important", "true");

        let mut msg2 = Message::new("test");
        msg2.set_header("priority", "low");

        let mut msg3 = Message::new("test");
        msg3.set_header("priority", "high");

        assert!(filter_has.test(&msg1).await);
        assert!(!filter_has.test(&msg2).await);
        assert!(!filter_equals.test(&msg2).await);
        assert!(filter_equals.test(&msg3).await);
    }

    #[tokio::test]
    async fn test_not_filter()
    {
        let inner = Arc::new(PredicateFilter::new(|msg| {
            msg.get_payload::<i32>().is_some_and(|v| v > 10)
        }));
        let filter = NotFilter::new(inner);

        assert!(filter.test(&Message::new(5i32)).await);
        assert!(!filter.test(&Message::new(15i32)).await);
    }

    #[tokio::test]
    async fn test_and_filter()
    {
        let filter = AndFilter::new(vec![
            Arc::new(PredicateFilter::new(|msg| msg.get_payload::<i32>().is_some_and(|v| v > 0))),
            Arc::new(PredicateFilter::new(|msg| {
                msg.get_payload::<i32>().is_some_and(|v| v < 100)
            })),
        ]);

        assert!(!filter.test(&Message::new(-1i32)).await);
        assert!(filter.test(&Message::new(50i32)).await);
        assert!(!filter.test(&Message::new(150i32)).await);
    }

    #[tokio::test]
    async fn test_or_filter()
    {
        let filter = OrFilter::new(vec![
            Arc::new(PredicateFilter::new(|msg| {
                msg.get_payload::<String>()
                    .is_some_and(|s| s == "special")
            })),
            Arc::new(PredicateFilter::new(|msg| {
                msg.get_payload::<i32>().is_some_and(|v| v > 100)
            })),
        ]);

        assert!(filter.test(&Message::new("special".to_string())).await);
        assert!(!filter.test(&Message::new("normal".to_string())).await);
        assert!(filter.test(&Message::new(150i32)).await);
        assert!(!filter.test(&Message::new(50i32)).await);
    }

    #[tokio::test]
    async fn test_threshold_filter()
    {
        let lt = ThresholdFilter::less_than(10);
        let gt = ThresholdFilter::greater_than(10);
        let between = ThresholdFilter::between(5, 15);
        let eq = ThresholdFilter::equal(10);

        assert!(lt.test(&Message::new(5i64)).await);
        assert!(!lt.test(&Message::new(15i64)).await);

        assert!(!gt.test(&Message::new(5i64)).await);
        assert!(gt.test(&Message::new(15i64)).await);

        assert!(!between.test(&Message::new(4i64)).await);
        assert!(between.test(&Message::new(10i64)).await);
        assert!(!between.test(&Message::new(16i64)).await);

        assert!(!eq.test(&Message::new(5i64)).await);
        assert!(eq.test(&Message::new(10i64)).await);
    }

    #[tokio::test]
    async fn test_filter_method()
    {
        let filter = PredicateFilter::new(|msg| msg.get_payload::<i32>().is_some_and(|v| v > 10));

        let good_msg = Message::new(15i32);
        let bad_msg = Message::new(5i32);

        assert!(filter.filter(good_msg).await.is_some());
        assert!(filter.filter(bad_msg).await.is_none());
    }
}
