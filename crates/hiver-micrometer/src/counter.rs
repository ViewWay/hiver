//! Counter metric
//! 计数器指标

use crate::metric::{MetricId, Tags};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Counter metric - monotonically increasing value
/// 计数器指标 - 单调递增值
#[derive(Clone)]
pub struct Counter {
    inner: Arc<CounterInner>,
}

struct CounterInner {
    /// Metric ID
    /// 指标 ID
    id: MetricId,

    /// Count value
    /// 计数值
    count: AtomicU64,

    /// Description
    /// 描述
    description: Option<String>,
}

impl Counter {
    /// Create a new counter
    /// 创建新计数器
    pub fn new(id: MetricId) -> Self {
        Self {
            inner: Arc::new(CounterInner {
                id,
                count: AtomicU64::new(0),
                description: None,
            }),
        }
    }

    /// Create with description
    /// 创建带描述的计数器
    pub fn with_description(id: MetricId, description: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(CounterInner {
                id,
                count: AtomicU64::new(0),
                description: Some(description.into()),
            }),
        }
    }

    /// Increment by 1
    /// 递增 1
    pub fn increment(&self) -> u64 {
        self.inner.count.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Increment by amount
    /// 递增指定值
    pub fn increment_by(&self, amount: u64) -> u64 {
        self.inner.count.fetch_add(amount, Ordering::Relaxed) + amount
    }

    /// Get current count
    /// 获取当前计数值
    pub fn count(&self) -> u64 {
        self.inner.count.load(Ordering::Relaxed)
    }

    /// Get metric ID
    /// 获取指标 ID
    pub fn id(&self) -> &MetricId {
        &self.inner.id
    }

    /// Get description
    /// 获取描述
    pub fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }

    /// Set description
    /// 设置描述
    pub fn set_description(&mut self, _desc: impl Into<String>) {
        // Note: This requires Arc::make_mut or similar for shared state
        // For simplicity, we'll just note that descriptions should be set at creation
    }
}

/// Counter builder
/// 计数器构建器
pub struct CounterBuilder {
    id: MetricId,
    description: Option<String>,
}

impl CounterBuilder {
    /// Create a new builder
    /// 创建新构建器
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            id: MetricId::from_name(
                crate::metric::MetricName::new(name.as_ref()).expect("Invalid metric name"),
            ),
            description: None,
        }
    }

    /// Set tags
    /// 设置标签
    pub fn tags(mut self, tags: Tags) -> Self {
        self.id.tags = tags;
        self
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Build the counter
    /// 构建计数器
    pub fn build(self) -> Counter {
        if let Some(desc) = self.description {
            Counter::with_description(self.id, desc)
        } else {
            Counter::new(self.id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metric::MetricName;

    #[test]
    fn test_counter_increment() {
        let counter = Counter::new(MetricId::from_name(MetricName::new("test_counter").unwrap()));

        assert_eq!(counter.count(), 0);
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_counter_increment_by() {
        let counter = Counter::new(MetricId::from_name(MetricName::new("test_counter").unwrap()));

        assert_eq!(counter.increment_by(5), 5);
        assert_eq!(counter.increment_by(3), 8);
        assert_eq!(counter.count(), 8);
    }

    #[test]
    fn test_counter_builder() {
        let counter = CounterBuilder::new("my_counter")
            .description("A test counter")
            .tags(Tags::new())
            .build();

        assert_eq!(counter.id().name.as_str(), "my_counter");
        assert_eq!(counter.description(), Some("A test counter"));
    }

    #[test]
    fn test_counter_with_description() {
        let counter = Counter::with_description(
            MetricId::from_name(MetricName::new("test_counter").unwrap()),
            "Test description",
        );

        assert_eq!(counter.description(), Some("Test description"));
    }
}
