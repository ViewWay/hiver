//! Metrics endpoint
//! 指标端点
//!
//! # Equivalent to Spring Boot Actuator /metrics
//! # 等价于 Spring Boot Actuator /metrics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

/// Metric type
/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// A gauge can go up or down
    /// 仪表盘，可以上下变化
    Gauge,

    /// A counter can only increase
    /// 计数器，只能增加
    Counter,

    /// A timing measurement
    /// 时间测量
    Timer,
}

/// Metric value
/// 指标值
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    /// Integer value
    /// 整数值
    Integer(i64),

    /// Floating point value
    /// 浮点数值
    Float(f64),

    /// String value
    /// 字符串值
    String(String),
}

/// A metric
/// 指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    /// 指标名称
    pub name: String,

    /// Metric description
    /// 指标描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Metric type
    /// 指标类型
    #[serde(rename = "type")]
    pub metric_type: MetricType,

    /// Metric value
    /// 指标值
    pub value: MetricValue,

    /// Available measurements (for timers/gauges)
    /// 可用的测量值（用于计时器/仪表盘）
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub measurements: HashMap<String, f64>,

    /// Available tags
    /// 可用的标签
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, String>,
}

impl Metric {
    /// Create a new gauge metric
    /// 创建新的仪表盘指标
    pub fn gauge(name: impl Into<String>, value: i64) -> Self {
        Self {
            name: name.into(),
            description: None,
            metric_type: MetricType::Gauge,
            value: MetricValue::Integer(value),
            measurements: HashMap::new(),
            tags: HashMap::new(),
        }
    }

    /// Create a new counter metric
    /// 创建新的计数器指标
    pub fn counter(name: impl Into<String>, value: u64) -> Self {
        Self {
            name: name.into(),
            description: None,
            metric_type: MetricType::Counter,
            value: MetricValue::Integer(value as i64),
            measurements: HashMap::new(),
            tags: HashMap::new(),
        }
    }

    /// Set the description
    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a tag
    /// 添加标签
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }
}

/// Metrics registry
/// 指标注册表
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    counters: HashMap<String, Arc<AtomicU64>>,
    gauges: HashMap<String, Arc<AtomicI64>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    /// 创建新的指标注册表
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
        }
    }

    /// Increment a counter
    /// 增加计数器
    pub fn increment(&mut self, name: &str) -> u64 {
        let counter = self
            .counters
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));
        counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Increment a counter by a specific amount
    /// 按指定数量增加计数器
    pub fn increment_by(&mut self, name: &str, amount: u64) -> u64 {
        let counter = self
            .counters
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));
        counter.fetch_add(amount, Ordering::Relaxed)
    }

    /// Get a counter value
    /// 获取计数器值
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.get(name).map(|c| c.load(Ordering::Relaxed))
    }

    /// Set a gauge value
    /// 设置仪表盘值
    pub fn set_gauge(&mut self, name: &str, value: i64) {
        let gauge = self
            .gauges
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicI64::new(0)));
        gauge.store(value, Ordering::Relaxed);
    }

    /// Get a gauge value
    /// 获取仪表盘值
    pub fn get_gauge(&self, name: &str) -> Option<i64> {
        self.gauges.get(name).map(|g| g.load(Ordering::Relaxed))
    }

    /// Collect all metrics
    /// 收集所有指标
    pub fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();

        // Collect counters
        for (name, counter) in &self.counters {
            let value = counter.load(Ordering::Relaxed);
            metrics.push(Metric::counter(name, value).with_description("Counter metric"));
        }

        // Collect gauges
        for (name, gauge) in &self.gauges {
            let value = gauge.load(Ordering::Relaxed);
            metrics.push(Metric::gauge(name, value).with_description("Gauge metric"));
        }

        metrics
    }

    /// Get metric names list
    /// 获取指标名称列表
    pub fn names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.counters.keys().cloned());
        names.extend(self.gauges.keys().cloned());
        names.sort();
        names.dedup();
        names
    }
}

/// System metrics collector
/// 系统指标收集器
pub struct SystemMetrics {
    _registry: Arc<MetricsRegistry>,
}

impl SystemMetrics {
    /// Create new system metrics
    /// 创建新的系统指标
    pub fn new() -> Self {
        Self {
            _registry: Arc::new(MetricsRegistry::new()),
        }
    }

    /// Collect system metrics
    /// 收集系统指标
    pub fn collect(&self) -> Vec<Metric> {
        vec![
            Metric::gauge("jvm.memory.max", 1024 * 1024 * 512).with_description("Maximum memory"),
            Metric::gauge("jvm.memory.used", 1024 * 1024 * 128).with_description("Used memory"),
            Metric::gauge("process.cpu.usage", 5).with_description("Process CPU usage"),
            Metric::gauge("system.cpu.count", 4).with_description("Number of processors"),
        ]
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_gauge() {
        let metric = Metric::gauge("test.gauge", 42);
        assert_eq!(metric.name, "test.gauge");
        assert_eq!(metric.metric_type, MetricType::Gauge);
    }

    #[test]
    fn test_metric_counter() {
        let metric = Metric::counter("test.counter", 100);
        assert_eq!(metric.name, "test.counter");
        assert_eq!(metric.metric_type, MetricType::Counter);
    }

    #[test]
    fn test_metrics_registry() {
        let mut registry = MetricsRegistry::new();

        registry.increment("test.counter");
        registry.increment("test.counter");

        assert_eq!(registry.get_counter("test.counter"), Some(2));

        registry.set_gauge("test.gauge", 42);
        assert_eq!(registry.get_gauge("test.gauge"), Some(42));
    }

    #[test]
    fn test_metrics_collect() {
        let mut registry = MetricsRegistry::new();

        registry.increment("requests.total");
        registry.set_gauge("connections.active", 10);

        let metrics = registry.collect();
        assert_eq!(metrics.len(), 2);
    }
}
