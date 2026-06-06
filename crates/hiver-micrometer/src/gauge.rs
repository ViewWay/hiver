//! Gauge metric
//! 仪表盘指标

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::metric::{MetricId, Tags};

/// Convert f64 to u64 bits
fn f64_to_u64(value: f64) -> u64
{
    value.to_bits()
}

/// Convert u64 bits to f64
fn u64_to_f64(bits: u64) -> f64
{
    f64::from_bits(bits)
}

/// Gauge metric - can go up or down
/// 仪表盘指标 - 可增可减
#[derive(Clone)]
pub struct Gauge
{
    inner: Arc<GaugeInner>,
}

struct GaugeInner
{
    /// Metric ID
    /// 指标 ID
    id: MetricId,

    /// Current value (as u64 bits)
    /// 当前值（作为 u64 位数）
    value: AtomicU64,

    /// Description
    /// 描述
    description: Option<String>,
}

impl Gauge
{
    /// Create a new gauge
    /// 创建新仪表盘
    pub fn new(id: MetricId) -> Self
    {
        Self {
            inner: Arc::new(GaugeInner {
                id,
                value: AtomicU64::new(f64_to_u64(0.0)),
                description: None,
            }),
        }
    }

    /// Create with initial value
    /// 创建带初始值的仪表盘
    pub fn with_value(id: MetricId, initial: f64) -> Self
    {
        Self {
            inner: Arc::new(GaugeInner {
                id,
                value: AtomicU64::new(f64_to_u64(initial)),
                description: None,
            }),
        }
    }

    /// Create with description
    /// 创建带描述的仪表盘
    pub fn with_description(id: MetricId, description: impl Into<String>) -> Self
    {
        Self {
            inner: Arc::new(GaugeInner {
                id,
                value: AtomicU64::new(f64_to_u64(0.0)),
                description: Some(description.into()),
            }),
        }
    }

    /// Set the value
    /// 设置值
    pub fn set(&self, value: f64)
    {
        self.inner.value.store(f64_to_u64(value), Ordering::Relaxed);
    }

    /// Get current value
    /// 获取当前值
    pub fn value(&self) -> f64
    {
        u64_to_f64(self.inner.value.load(Ordering::Relaxed))
    }

    /// Increment by amount
    /// 递增指定值
    pub fn increment(&self, amount: f64) -> f64
    {
        let mut current = self.inner.value.load(Ordering::Relaxed);
        loop
        {
            let new_value = u64_to_f64(current) + amount;
            let new_bits = f64_to_u64(new_value);
            match self.inner.value.compare_exchange_weak(
                current,
                new_bits,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            {
                Ok(_) => return new_value,
                Err(actual) => current = actual,
            }
        }
    }

    /// Decrement by amount
    /// 递减指定值
    pub fn decrement(&self, amount: f64) -> f64
    {
        self.increment(-amount)
    }

    /// Get metric ID
    /// 获取指标 ID
    pub fn id(&self) -> &MetricId
    {
        &self.inner.id
    }

    /// Get description
    /// 获取描述
    pub fn description(&self) -> Option<&str>
    {
        self.inner.description.as_deref()
    }
}

/// Gauge builder
/// 仪表盘构建器
pub struct GaugeBuilder
{
    id: MetricId,
    description: Option<String>,
    initial_value: Option<f64>,
}

impl GaugeBuilder
{
    /// Create a new builder
    /// 创建新构建器
    pub fn new(name: impl AsRef<str>) -> Self
    {
        Self {
            id: MetricId::from_name(
                crate::metric::MetricName::new(name.as_ref()).expect("Invalid metric name"),
            ),
            description: None,
            initial_value: None,
        }
    }

    /// Set tags
    /// 设置标签
    pub fn tags(mut self, tags: Tags) -> Self
    {
        self.id.tags = tags;
        self
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self
    {
        self.description = Some(desc.into());
        self
    }

    /// Set initial value
    /// 设置初始值
    pub fn initial_value(mut self, value: f64) -> Self
    {
        self.initial_value = Some(value);
        self
    }

    /// Build the gauge
    /// 构建仪表盘
    pub fn build(self) -> Gauge
    {
        if let Some(initial) = self.initial_value
        {
            let mut gauge = Gauge::with_value(self.id.clone(), initial);
            if let Some(desc) = self.description
            {
                gauge = Gauge::with_description(self.id, desc);
                gauge.set(initial);
            }
            gauge
        }
        else if let Some(desc) = self.description
        {
            Gauge::with_description(self.id, desc)
        }
        else
        {
            Gauge::new(self.id)
        }
    }
}

/// Function gauge - a gauge that gets its value from a function
/// 函数仪表盘 - 从函数获取值的仪表盘
pub struct FunctionGauge<F>
where
    F: Fn() -> f64 + Send + Sync,
{
    id: MetricId,
    f: Arc<F>,
    description: Option<String>,
}

impl<F> FunctionGauge<F>
where
    F: Fn() -> f64 + Send + Sync,
{
    /// Create a new function gauge
    /// 创建新的函数仪表盘
    pub fn new(id: MetricId, f: F) -> Self
    {
        Self {
            id,
            f: Arc::new(f),
            description: None,
        }
    }

    /// Create with description
    /// 创建带描述的函数仪表盘
    pub fn with_description(id: MetricId, f: F, description: impl Into<String>) -> Self
    {
        Self {
            id,
            f: Arc::new(f),
            description: Some(description.into()),
        }
    }

    /// Get current value by calling the function
    /// 通过调用函数获取当前值
    pub fn value(&self) -> f64
    {
        (self.f)()
    }

    /// Get metric ID
    /// 获取指标 ID
    pub fn id(&self) -> &MetricId
    {
        &self.id
    }

    /// Get description
    /// 获取描述
    pub fn description(&self) -> Option<&str>
    {
        self.description.as_deref()
    }
}

impl<F> Clone for FunctionGauge<F>
where
    F: Fn() -> f64 + Send + Sync,
{
    fn clone(&self) -> Self
    {
        Self {
            id: self.id.clone(),
            f: Arc::clone(&self.f),
            description: self.description.clone(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_gauge_set()
    {
        let gauge =
            Gauge::new(MetricId::from_name(crate::metric::MetricName::new("test_gauge").unwrap()));

        gauge.set(42.0);
        assert_eq!(gauge.value(), 42.0);

        gauge.set(100.0);
        assert_eq!(gauge.value(), 100.0);
    }

    #[test]
    fn test_gauge_increment_decrement()
    {
        let gauge = Gauge::with_value(
            MetricId::from_name(crate::metric::MetricName::new("test_gauge").unwrap()),
            10.0,
        );

        assert_eq!(gauge.value(), 10.0);
        gauge.increment(5.0);
        assert_eq!(gauge.value(), 15.0);
        gauge.decrement(3.0);
        assert_eq!(gauge.value(), 12.0);
    }

    #[test]
    fn test_gauge_builder()
    {
        let gauge = GaugeBuilder::new("my_gauge")
            .description("A test gauge")
            .initial_value(25.0)
            .build();

        assert_eq!(gauge.id().name.as_str(), "my_gauge");
        assert_eq!(gauge.description(), Some("A test gauge"));
        assert_eq!(gauge.value(), 25.0);
    }

    #[test]
    fn test_function_gauge()
    {
        let value = Arc::new(std::sync::atomic::AtomicU64::new(f64_to_u64(42.0)));

        let gauge = FunctionGauge::new(
            MetricId::from_name(crate::metric::MetricName::new("func_gauge").unwrap()),
            {
                let value = value.clone();
                move || u64_to_f64(value.load(Ordering::Relaxed))
            },
        );

        assert_eq!(gauge.value(), 42.0);
        value.store(f64_to_u64(100.0), Ordering::Relaxed);
        assert_eq!(gauge.value(), 100.0);
    }
}
