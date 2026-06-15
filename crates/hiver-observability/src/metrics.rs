//! Metrics module
//! 指标模块
//!
//! # Overview / 概述
//!
//! This module provides metrics collection functionality compatible with Prometheus.
//! It includes counters, gauges, histograms, and a metrics registry.
//!
//! 本模块提供与 Prometheus 兼容的指标收集功能。
//! 包括计数器、仪表、直方图和指标注册表。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Micrometer Metrics
//! - Spring Boot Actuator Metrics
//! - Prometheus `MeterRegistry`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_observability::metrics::{MetricsRegistry, Counter, Gauge, Histogram};
//!
//! let registry = MetricsRegistry::new();
//!
//! // Counter - monotonically increasing
//! let counter = registry.counter("http_requests_total");
//! counter.increment();
//! counter.increment_by(5);
//!
//! // Gauge - can go up or down
//! let gauge = registry.gauge("active_connections");
//! gauge.set(10);
//! gauge.increment();
//! gauge.decrement();
//!
//! // Histogram - observe distributions
//! let histogram = registry.histogram("request_duration_seconds");
//! histogram.observe(0.123);
//! histogram.observe(0.456);
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{
    collections::HashMap,
    sync::{
        Arc, RwLock,
        atomic::{AtomicI64, AtomicU64, Ordering},
    },
};

/// Metric ID
/// 指标ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetricId
{
    /// Metric name
    /// 指标名称
    pub name: String,

    /// Metric labels
    /// 指标标签
    pub labels: Vec<(String, String)>,
}

impl MetricId
{
    /// Create a new metric ID
    /// 创建新的指标ID
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            labels: Vec::new(),
        }
    }

    /// Create a new metric ID from a reference
    /// 从引用创建新的指标ID
    fn new_from_ref(name: &str) -> Self
    {
        Self {
            name: name.to_string(),
            labels: Vec::new(),
        }
    }

    /// Add a label
    /// 添加标签
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.labels.push((key.into(), value.into()));
        self
    }

    /// Create a labeled metric ID
    /// 创建带标签的指标ID
    pub fn with_labels(mut self, labels: Vec<(String, String)>) -> Self
    {
        self.labels = labels;
        self
    }

    /// Create a new `MetricId` with a suffix added to the name
    /// 创建名称带后缀的新 `MetricId`
    fn with_suffix(&self, suffix: &str) -> MetricId
    {
        MetricId {
            name: format!("{}_{}", self.name, suffix),
            labels: self.labels.clone(),
        }
    }
}

/// Metric type
/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType
{
    /// Counter (monotonically increasing)
    /// 计数器（单调递增）
    Counter,

    /// Gauge (can go up or down)
    /// 仪表（可增可减）
    Gauge,

    /// Histogram (distribution of values)
    /// 直方图（值分布）
    Histogram,
}

impl MetricType
{
    /// Get the Prometheus type string
    /// 获取Prometheus类型字符串
    pub fn as_str(&self) -> &str
    {
        match self
        {
            Self::Counter => "counter",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
        }
    }
}

impl std::fmt::Display for MetricType
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.as_str())
    }
}

/// Counter metric - monotonically increasing value
/// 计数器指标 - 单调递增值
///
/// Counters are used for things that only increase (e.g., number of requests).
/// 计数器用于只增加的内容（例如请求数）。
#[derive(Clone)]
pub struct Counter
{
    /// Metric ID
    /// 指标ID
    id: MetricId,

    /// Counter value
    /// 计数器值
    value: Arc<AtomicU64>,

    /// Help text (interior mutable for shared updates)
    /// 帮助文本（内部可变以支持共享更新）
    help: Arc<RwLock<Option<String>>>,
}

impl Counter
{
    /// Create a new counter
    /// 创建新的计数器
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            id: MetricId::new(name),
            value: Arc::new(AtomicU64::new(0)),
            help: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new labeled counter
    /// 创建带标签的计数器
    pub fn with_labels(name: impl Into<String>, labels: Vec<(String, String)>) -> Self
    {
        Self {
            id: MetricId::new(name).with_labels(labels),
            value: Arc::new(AtomicU64::new(0)),
            help: Arc::new(RwLock::new(None)),
        }
    }

    /// Set help text
    /// 设置帮助文本
    pub fn with_help(&self, help: impl Into<String>) -> Self
    {
        if let Ok(mut h) = self.help.write()
        {
            *h = Some(help.into());
        }
        self.clone()
    }

    /// Get the metric ID
    /// 获取指标ID
    pub fn id(&self) -> &MetricId
    {
        &self.id
    }

    /// Get the current value
    /// 获取当前值
    pub fn get(&self) -> u64
    {
        self.value.load(Ordering::Relaxed)
    }

    /// Increment by 1
    /// 增加1
    pub fn increment(&self)
    {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment by a specific amount
    /// 增加特定数量
    pub fn increment_by(&self, amount: u64)
    {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }

    /// Reset to zero
    /// 重置为零
    pub fn reset(&self)
    {
        self.value.store(0, Ordering::Relaxed);
    }

    /// Get help text
    /// 获取帮助文本
    pub fn help(&self) -> Option<String>
    {
        self.help.read().ok().and_then(|h| h.clone())
    }
}

/// Gauge metric - can go up or down
/// 仪表指标 - 可增可减
///
/// Gauges are used for values that can increase or decrease (e.g., temperature, connections).
/// 仪表用于可增可减的值（例如温度、连接数）。
#[derive(Clone)]
pub struct Gauge
{
    /// Metric ID
    /// 指标ID
    id: MetricId,

    /// Gauge value (can be negative)
    /// 仪表值（可以为负）
    value: Arc<AtomicI64>,

    /// Help text (interior mutable for shared updates)
    /// 帮助文本（内部可变以支持共享更新）
    help: Arc<RwLock<Option<String>>>,
}

impl Gauge
{
    /// Create a new gauge
    /// 创建新的仪表
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            id: MetricId::new(name),
            value: Arc::new(AtomicI64::new(0)),
            help: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new labeled gauge
    /// 创建带标签的仪表
    pub fn with_labels(name: impl Into<String>, labels: Vec<(String, String)>) -> Self
    {
        Self {
            id: MetricId::new(name).with_labels(labels),
            value: Arc::new(AtomicI64::new(0)),
            help: Arc::new(RwLock::new(None)),
        }
    }

    /// Set help text
    /// 设置帮助文本
    pub fn with_help(&self, help: impl Into<String>) -> Self
    {
        if let Ok(mut h) = self.help.write()
        {
            *h = Some(help.into());
        }
        self.clone()
    }

    /// Get the metric ID
    /// 获取指标ID
    pub fn id(&self) -> &MetricId
    {
        &self.id
    }

    /// Get the current value
    /// 获取当前值
    pub fn get(&self) -> i64
    {
        self.value.load(Ordering::Relaxed)
    }

    /// Set to a specific value
    /// 设置为特定值
    pub fn set(&self, value: i64)
    {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment by 1
    /// 增加1
    pub fn increment(&self)
    {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement by 1 (saturating, won't underflow).
    /// 减少1（饱和操作，不会下溢）。
    pub fn decrement(&self)
    {
        self.value
            .fetch_update(Ordering::AcqRel, Ordering::Relaxed, |v| {
                if v > 0 { Some(v - 1) } else { None }
            })
            .ok();
    }

    /// Add a specific amount
    /// 添加特定数量
    pub fn add(&self, amount: i64)
    {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }

    /// Subtract a specific amount
    /// 减去特定数量
    pub fn sub(&self, amount: i64)
    {
        self.value.fetch_sub(amount, Ordering::Relaxed);
    }

    /// Get help text
    /// 获取帮助文本
    pub fn help(&self) -> Option<String>
    {
        self.help.read().ok().and_then(|h| h.clone())
    }
}

/// Histogram bucket
/// 直方图桶
#[derive(Debug, Clone)]
pub struct Bucket
{
    /// Upper bound (None means +Inf)
    /// 上界（None表示+Inf）
    pub upper_bound: Option<f64>,

    /// Cumulative count
    /// 累积计数
    pub count: u64,
}

impl Bucket
{
    /// Create a new bucket
    /// 创建新的桶
    pub fn new(upper_bound: Option<f64>) -> Self
    {
        Self {
            upper_bound,
            count: 0,
        }
    }

    /// Get the label for this bucket
    /// 获取此桶的标签
    pub fn label(&self) -> String
    {
        match self.upper_bound
        {
            Some(bound) => format!("{:.6}", bound),
            None => "+Inf".to_string(),
        }
    }

    /// Check if this is the infinity bucket
    /// 检查这是否是无穷大桶
    pub fn is_infinity(&self) -> bool
    {
        self.upper_bound.is_none()
    }
}

/// Histogram metric - distribution of values
/// 直方图指标 - 值分布
///
/// Histograms observe values and count them into configurable buckets.
/// 直方图观察值并将其计数到可配置的桶中。
#[derive(Clone)]
pub struct Histogram
{
    /// Metric ID
    /// 指标ID
    id: MetricId,

    /// Histogram data
    /// 直方图数据
    data: Arc<RwLock<HistogramData>>,

    /// Help text (interior mutable for shared updates)
    /// 帮助文本（内部可变以支持共享更新）
    help: Arc<RwLock<Option<String>>>,
}

/// Histogram data (interior mutable)
/// 直方图数据（内部可变）
#[derive(Debug)]
struct HistogramData
{
    /// Total sum of observed values
    /// 观察值的总和
    sum: f64,

    /// Total count of observations
    /// 观察总数
    count: u64,

    /// Bucket definitions
    /// 桶定义
    buckets: Vec<Option<f64>>,

    /// Bucket counts
    /// 桶计数
    bucket_counts: Vec<u64>,
}

impl HistogramData
{
    /// Create new histogram data with default buckets
    /// 使用默认桶创建新的直方图数据
    fn new() -> Self
    {
        // Default Prometheus buckets
        let buckets = vec![
            Some(0.005),
            Some(0.01),
            Some(0.025),
            Some(0.05),
            Some(0.1),
            Some(0.25),
            Some(0.5),
            Some(1.0),
            Some(2.5),
            Some(5.0),
            Some(10.0),
            None, // +Inf
        ];
        Self {
            sum: 0.0,
            count: 0,
            buckets: buckets.clone(),
            bucket_counts: vec![0; buckets.len()],
        }
    }

    /// Create new histogram data with custom buckets
    /// 使用自定义桶创建新的直方图数据
    fn with_buckets(mut buckets: Vec<Option<f64>>) -> Self
    {
        // Ensure buckets are sorted and end with +Inf
        let has_infinity = buckets.iter().any(Option::is_none);
        buckets.sort_by(|a, b| match (a, b)
        {
            (Some(va), Some(vb)) => va.partial_cmp(vb).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        if !has_infinity
        {
            buckets.push(None);
        }

        Self {
            sum: 0.0,
            count: 0,
            bucket_counts: vec![0; buckets.len()],
            buckets,
        }
    }

    /// Observe a value
    /// 观察值
    fn observe(&mut self, value: f64)
    {
        self.sum += value;
        self.count += 1;

        // Find the appropriate bucket
        for (i, upper_bound) in self.buckets.iter().enumerate()
        {
            if upper_bound.is_none_or(|ub| value <= ub)
                && let Some(count) = self.bucket_counts.get_mut(i)
            {
                *count += 1;
            }
        }
    }

    /// Get all buckets with cumulative counts
    /// 获取所有桶及其累积计数
    fn get_buckets(&self) -> Vec<Bucket>
    {
        let mut cumulative = 0u64;
        self.buckets
            .iter()
            .zip(self.bucket_counts.iter())
            .map(|(upper_bound, &count)| {
                cumulative += count;
                Bucket {
                    upper_bound: *upper_bound,
                    count: cumulative,
                }
            })
            .collect()
    }
}

impl Default for HistogramData
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Histogram
{
    /// Create a new histogram with default buckets
    /// 使用默认桶创建新的直方图
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            id: MetricId::new(name),
            data: Arc::new(RwLock::new(HistogramData::new())),
            help: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new histogram with custom buckets
    /// 使用自定义桶创建新的直方图
    pub fn with_buckets(name: impl Into<String>, buckets: Vec<f64>) -> Self
    {
        let bucket_options = buckets.into_iter().map(Some).collect();
        Self {
            id: MetricId::new(name),
            data: Arc::new(RwLock::new(HistogramData::with_buckets(bucket_options))),
            help: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new labeled histogram
    /// 创建带标签的直方图
    pub fn with_labels(name: impl Into<String>, labels: Vec<(String, String)>) -> Self
    {
        Self {
            id: MetricId::new(name).with_labels(labels),
            data: Arc::new(RwLock::new(HistogramData::new())),
            help: Arc::new(RwLock::new(None)),
        }
    }

    /// Set help text
    /// 设置帮助文本
    pub fn with_help(&self, help: impl Into<String>) -> Self
    {
        if let Ok(mut h) = self.help.write()
        {
            *h = Some(help.into());
        }
        self.clone()
    }

    /// Get the metric ID
    /// 获取指标ID
    pub fn id(&self) -> &MetricId
    {
        &self.id
    }

    /// Observe a value
    /// 观察值
    pub fn observe(&self, value: f64)
    {
        if let Ok(mut data) = self.data.write()
        {
            data.observe(value);
        }
    }

    /// Get the total count of observations
    /// 获取观察总数
    pub fn count(&self) -> u64
    {
        self.data.read().map_or(0, |d| d.count)
    }

    /// Get the sum of observed values
    /// 获取观察值的总和
    pub fn sum(&self) -> f64
    {
        self.data.read().map_or(0.0, |d| d.sum)
    }

    /// Get the buckets with cumulative counts
    /// 获取桶及其累积计数
    pub fn buckets(&self) -> Vec<Bucket>
    {
        self.data
            .read()
            .map(|d| d.get_buckets())
            .unwrap_or_default()
    }

    /// Reset the histogram
    /// 重置直方图
    pub fn reset(&self)
    {
        if let Ok(mut data) = self.data.write()
        {
            *data = HistogramData::with_buckets(data.buckets.clone());
        }
    }

    /// Get help text
    /// 获取帮助文本
    pub fn help(&self) -> Option<String>
    {
        self.help.read().ok().and_then(|h| h.clone())
    }
}

/// Metric - enum of all metric types
/// 指标 - 所有指标类型的枚举
#[derive(Clone)]
pub enum Metric
{
    /// Counter metric
    /// 计数器指标
    Counter(Counter),

    /// Gauge metric
    /// 仪表指标
    Gauge(Gauge),

    /// Histogram metric
    /// 直方图指标
    Histogram(Histogram),
}

impl Metric
{
    /// Get the metric ID
    /// 获取指标ID
    pub fn id(&self) -> &MetricId
    {
        match self
        {
            Self::Counter(c) => c.id(),
            Self::Gauge(g) => g.id(),
            Self::Histogram(h) => h.id(),
        }
    }

    /// Get the metric type
    /// 获取指标类型
    pub fn metric_type(&self) -> MetricType
    {
        match self
        {
            Self::Counter(_) => MetricType::Counter,
            Self::Gauge(_) => MetricType::Gauge,
            Self::Histogram(_) => MetricType::Histogram,
        }
    }

    /// Get help text
    /// 获取帮助文本
    pub fn help(&self) -> Option<String>
    {
        match self
        {
            Self::Counter(c) => c.help(),
            Self::Gauge(g) => g.help(),
            Self::Histogram(h) => h.help(),
        }
    }
}

/// Metrics registry - central storage for metrics
/// 指标注册表 - 指标的中央存储
///
/// The registry manages all metrics and provides export functionality.
/// 注册表管理所有指标并提供导出功能。
#[derive(Clone)]
pub struct MetricsRegistry
{
    /// Registered metrics
    /// 已注册的指标
    metrics: Arc<RwLock<HashMap<MetricId, Metric>>>,

    /// Common labels to add to all metrics
    /// 添加到所有指标的公共标签
    common_labels: Arc<Vec<(String, String)>>,
}

impl Default for MetricsRegistry
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl MetricsRegistry
{
    /// Create a new metrics registry
    /// 创建新的指标注册表
    pub fn new() -> Self
    {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            common_labels: Arc::new(Vec::new()),
        }
    }

    /// Create a registry with common labels
    /// 创建带有公共标签的注册表
    pub fn with_common_labels(labels: Vec<(String, String)>) -> Self
    {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            common_labels: Arc::new(labels),
        }
    }

    /// Register or retrieve a counter
    /// 注册或检索计数器
    pub fn counter(&self, name: impl Into<String>) -> Counter
    {
        self.counter_with_labels(name, Vec::new())
    }

    /// Register or retrieve a labeled counter
    /// 注册或检索带标签的计数器
    pub fn counter_with_labels(
        &self,
        name: impl Into<String>,
        labels: Vec<(String, String)>,
    ) -> Counter
    {
        let name = name.into();
        let mut all_labels = (*self.common_labels).clone();
        all_labels.extend(labels);

        let id = MetricId::new_from_ref(&name).with_labels(all_labels.clone());

        let mut metrics = self
            .metrics
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(Metric::Counter(counter)) = metrics.get(&id)
        {
            return counter.clone();
        }

        let counter = Counter::with_labels(name, all_labels);
        metrics.insert(id, Metric::Counter(counter.clone()));
        counter
    }

    /// Register or retrieve a gauge
    /// 注册或检索仪表
    pub fn gauge(&self, name: impl Into<String>) -> Gauge
    {
        self.gauge_with_labels(name, Vec::new())
    }

    /// Register or retrieve a labeled gauge
    /// 注册或检索带标签的仪表
    pub fn gauge_with_labels(&self, name: impl Into<String>, labels: Vec<(String, String)>)
    -> Gauge
    {
        let name = name.into();
        let mut all_labels = (*self.common_labels).clone();
        all_labels.extend(labels);

        let id = MetricId::new_from_ref(&name).with_labels(all_labels.clone());

        let mut metrics = self
            .metrics
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(Metric::Gauge(gauge)) = metrics.get(&id)
        {
            return gauge.clone();
        }

        let gauge = Gauge::with_labels(name, all_labels);
        metrics.insert(id, Metric::Gauge(gauge.clone()));
        gauge
    }

    /// Register or retrieve a histogram
    /// 注册或检索直方图
    pub fn histogram(&self, name: impl Into<String>) -> Histogram
    {
        self.histogram_with_labels(name, Vec::new())
    }

    /// Register or retrieve a labeled histogram
    /// 注册或检索带标签的直方图
    pub fn histogram_with_labels(
        &self,
        name: impl Into<String>,
        labels: Vec<(String, String)>,
    ) -> Histogram
    {
        let name = name.into();
        let mut all_labels = (*self.common_labels).clone();
        all_labels.extend(labels);

        let id = MetricId::new_from_ref(&name).with_labels(all_labels.clone());

        let mut metrics = self
            .metrics
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(Metric::Histogram(histogram)) = metrics.get(&id)
        {
            return histogram.clone();
        }

        let histogram = Histogram::with_labels(name, all_labels);
        metrics.insert(id, Metric::Histogram(histogram.clone()));
        histogram
    }

    /// Get all metrics
    /// 获取所有指标
    pub fn get_all(&self) -> Vec<Metric>
    {
        self.metrics
            .read()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Export metrics in Prometheus text format
    /// 以Prometheus文本格式导出指标
    pub fn export_prometheus(&self) -> String
    {
        let mut output = String::new();

        let metrics = self.get_all();

        // Group by name (ignoring labels)
        let mut by_name: HashMap<String, Vec<&Metric>> = HashMap::new();
        for metric in &metrics
        {
            by_name
                .entry(metric.id().name.clone())
                .or_default()
                .push(metric);
        }

        for (name, group) in by_name
        {
            // Get help text from first metric in group
            if let Some(help) = group.first().and_then(|m| m.help())
            {
                output.push_str("# HELP ");
                output.push_str(&name);
                output.push(' ');
                output.push_str(&help);
                output.push('\n');
            }

            // Get type
            if let Some(first) = group.first()
            {
                output.push_str("# TYPE ");
                output.push_str(&name);
                output.push(' ');
                output.push_str(first.metric_type().as_str());
                output.push('\n');
            }

            // Export each metric
            for metric in group
            {
                match metric
                {
                    Metric::Counter(counter) =>
                    {
                        output.push_str(&export_metric_line(counter.id(), counter.get()));
                    },
                    Metric::Gauge(gauge) =>
                    {
                        output.push_str(&export_metric_line(gauge.id(), gauge.get() as u64));
                    },
                    Metric::Histogram(histogram) =>
                    {
                        // Export bucket counts with _bucket suffix
                        let bucket_name = histogram.id().with_suffix("bucket");
                        for bucket in histogram.buckets()
                        {
                            let mut bucket_id = bucket_name.clone();
                            bucket_id.labels.push(("le".to_string(), bucket.label()));
                            output.push_str(&export_metric_line(&bucket_id, bucket.count));
                        }
                        // Export sum and count with proper suffixes
                        let sum_id = histogram.id().with_suffix("sum");
                        output.push_str(&export_metric_line(&sum_id, histogram.sum() as u64));

                        let count_id = histogram.id().with_suffix("count");
                        output.push_str(&export_metric_line(&count_id, histogram.count()));
                    },
                }
            }
        }

        output
    }

    /// Clear all metrics
    /// 清除所有指标
    pub fn clear(&self)
    {
        if let Ok(mut metrics) = self.metrics.write()
        {
            metrics.clear();
        }
    }
}

/// Export a single metric line in Prometheus format
/// 以Prometheus格式导出单个指标行
fn export_metric_line(id: &MetricId, value: u64) -> String
{
    let mut line = id.name.clone();

    if !id.labels.is_empty()
    {
        line.push('{');
        let labels: Vec<String> = id
            .labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect();
        line.push_str(&labels.join(","));
        line.push('}');
    }

    // Handle suffixes like _sum, _count which have empty value labels
    let labels: Vec<_> = id
        .labels
        .iter()
        .filter(|(k, _)| !k.starts_with('_'))
        .collect();
    if !labels.is_empty() && id.labels.len() != labels.len()
    {
        // Rebuild line without empty labels
        line.clone_from(&id.name);
        line.push('{');
        let label_strings: Vec<String> = labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect();
        line.push_str(&label_strings.join(","));
        line.push('}');
    }
    else if labels.is_empty() && !id.labels.is_empty()
    {
        // Only suffix labels, remove braces
        line.clone_from(&id.name);
    }

    // Handle histogram suffixes
    for (k, _) in &id.labels
    {
        if k.starts_with('_') && k != "_sum" && k != "_count"
        {
            // This is a bucket label, don't add to name
            break;
        }
        if k == "_sum" || k == "_count"
        {
            line = format!("{}{}", id.name, k);
        }
    }

    // Format value
    let formatted_value = value.to_string();

    line.push(' ');
    line.push_str(&formatted_value);
    line.push('\n');

    line
}

/// Global metrics registry
/// 全局指标注册表
static GLOBAL_REGISTRY: std::sync::LazyLock<MetricsRegistry> =
    std::sync::LazyLock::new(MetricsRegistry::new);

/// Get the global metrics registry
/// 获取全局指标注册表
pub fn global_registry() -> &'static MetricsRegistry
{
    &GLOBAL_REGISTRY
}

/// Get a counter from the global registry
/// 从全局注册表获取计数器
pub fn counter(name: impl Into<String>) -> Counter
{
    GLOBAL_REGISTRY.counter(name)
}

/// Get a gauge from the global registry
/// 从全局注册表获取仪表
pub fn gauge(name: impl Into<String>) -> Gauge
{
    GLOBAL_REGISTRY.gauge(name)
}

/// Get a histogram from the global registry
/// 从全局注册表获取直方图
pub fn histogram(name: impl Into<String>) -> Histogram
{
    GLOBAL_REGISTRY.histogram(name)
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
    fn test_metric_id_new()
    {
        let id = MetricId::new("test_metric");
        assert_eq!(id.name, "test_metric");
        assert!(id.labels.is_empty());
    }

    #[test]
    fn test_metric_id_with_label()
    {
        let id = MetricId::new("test_metric")
            .with_label("key", "value")
            .with_label("foo", "bar");
        assert_eq!(id.labels.len(), 2);
        assert_eq!(id.labels[0], ("key".to_string(), "value".to_string()));
    }

    #[test]
    fn test_counter_increment()
    {
        let counter = Counter::new("test");
        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);

        counter.increment_by(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_counter_reset()
    {
        let counter = Counter::new("test");
        counter.increment_by(10);
        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_counter_with_help()
    {
        let counter = Counter::new("test").with_help("Test counter");
        assert_eq!(counter.help(), Some("Test counter".to_string()));
    }

    #[test]
    fn test_gauge_operations()
    {
        let gauge = Gauge::new("test");
        assert_eq!(gauge.get(), 0);

        gauge.set(10);
        assert_eq!(gauge.get(), 10);

        gauge.increment();
        assert_eq!(gauge.get(), 11);

        gauge.decrement();
        assert_eq!(gauge.get(), 10);

        gauge.add(5);
        assert_eq!(gauge.get(), 15);

        gauge.sub(3);
        assert_eq!(gauge.get(), 12);

        gauge.set(-5);
        assert_eq!(gauge.get(), -5);
    }

    #[test]
    fn test_histogram_observe()
    {
        let histogram = Histogram::new("test");

        histogram.observe(0.1);
        histogram.observe(0.5);
        histogram.observe(1.5);

        assert_eq!(histogram.count(), 3);
        assert!((histogram.sum() - 2.1).abs() < 0.001);
    }

    #[test]
    fn test_histogram_buckets()
    {
        let histogram = Histogram::new("test");

        histogram.observe(0.01);
        histogram.observe(0.1);
        histogram.observe(1.0);

        let buckets = histogram.buckets();
        assert!(!buckets.is_empty());

        // Check cumulative counts
        let bucket_100 = buckets.iter().find(|b| b.upper_bound == Some(0.1));
        assert!(bucket_100.is_some());
        // Should have at least 2 observations (0.01 and 0.1)
        assert!(bucket_100.unwrap().count >= 2);
    }

    #[test]
    fn test_histogram_custom_buckets()
    {
        let histogram = Histogram::with_buckets("test", vec![1.0, 5.0, 10.0]);

        histogram.observe(0.5);
        histogram.observe(3.0);
        histogram.observe(7.0);

        assert_eq!(histogram.count(), 3);
    }

    #[test]
    fn test_histogram_reset()
    {
        let histogram = Histogram::new("test");

        histogram.observe(1.0);
        histogram.observe(2.0);
        assert_eq!(histogram.count(), 2);

        histogram.reset();
        assert_eq!(histogram.count(), 0);
        assert_eq!(histogram.sum(), 0.0);
    }

    #[test]
    fn test_registry_counter()
    {
        let registry = MetricsRegistry::new();
        let counter = registry.counter("requests");

        counter.increment();
        counter.increment_by(5);

        // Same counter should be returned
        let counter2 = registry.counter("requests");
        assert_eq!(counter2.get(), 6);
    }

    #[test]
    fn test_registry_gauge()
    {
        let registry = MetricsRegistry::new();
        let gauge = registry.gauge("connections");

        gauge.set(10);
        gauge.increment();

        let gauge2 = registry.gauge("connections");
        assert_eq!(gauge2.get(), 11);
    }

    #[test]
    fn test_registry_histogram()
    {
        let registry = MetricsRegistry::new();
        let histogram = registry.histogram("duration");

        histogram.observe(0.1);
        histogram.observe(0.5);

        let histogram2 = registry.histogram("duration");
        assert_eq!(histogram2.count(), 2);
    }

    #[test]
    fn test_registry_labeled_metrics()
    {
        let registry = MetricsRegistry::new();

        let counter1 = registry
            .counter_with_labels("requests", vec![("method".to_string(), "GET".to_string())]);
        let counter2 = registry
            .counter_with_labels("requests", vec![("method".to_string(), "POST".to_string())]);

        counter1.increment();
        counter1.increment();
        counter2.increment();

        assert_eq!(counter1.get(), 2);
        assert_eq!(counter2.get(), 1);
    }

    #[test]
    fn test_registry_common_labels()
    {
        let registry = MetricsRegistry::with_common_labels(vec![
            ("service".to_string(), "test".to_string()),
            ("env".to_string(), "dev".to_string()),
        ]);

        let counter = registry.counter("requests");
        assert!(
            counter
                .id()
                .labels
                .iter()
                .any(|(k, v)| k == "service" && v == "test")
        );
        assert!(
            counter
                .id()
                .labels
                .iter()
                .any(|(k, v)| k == "env" && v == "dev")
        );
    }

    #[test]
    fn test_export_prometheus_counter()
    {
        let registry = MetricsRegistry::new();
        let counter = registry.counter("test_total");
        counter.increment_by(42);

        let exported = registry.export_prometheus();
        assert!(exported.contains("# TYPE test_total counter"));
        assert!(exported.contains("test_total 42"));
    }

    #[test]
    fn test_export_prometheus_gauge()
    {
        let registry = MetricsRegistry::new();
        let gauge = registry.gauge("temperature");
        gauge.set(23);

        let exported = registry.export_prometheus();
        assert!(exported.contains("# TYPE temperature gauge"));
        assert!(exported.contains("temperature 23"));
    }

    #[test]
    fn test_export_prometheus_histogram()
    {
        let registry = MetricsRegistry::new();
        let histogram = registry.histogram("duration_seconds");
        histogram.observe(0.1);
        histogram.observe(0.5);

        let exported = registry.export_prometheus();
        assert!(exported.contains("# TYPE duration_seconds histogram"));
        assert!(exported.contains("duration_seconds_bucket"));
        assert!(exported.contains("duration_seconds_sum"));
        assert!(exported.contains("duration_seconds_count"));
        assert!(exported.contains("le=\"+Inf\""));
    }

    #[test]
    fn test_export_prometheus_with_help()
    {
        let registry = MetricsRegistry::new();
        let counter = registry.counter("test_total").with_help("Total test count");
        counter.increment();

        let exported = registry.export_prometheus();
        assert!(exported.contains("# HELP test_total Total test count"));
    }

    #[test]
    fn test_export_prometheus_labeled()
    {
        let registry = MetricsRegistry::new();
        let counter = registry.counter_with_labels("requests", vec![
            ("method".to_string(), "GET".to_string()),
            ("status".to_string(), "200".to_string()),
        ]);
        counter.increment();

        let exported = registry.export_prometheus();
        assert!(exported.contains("requests{method=\"GET\",status=\"200\"}"));
    }

    #[test]
    fn test_global_registry()
    {
        let counter = counter("global_test");
        counter.increment();

        let counter2 = global_registry().counter("global_test");
        assert_eq!(counter2.get(), 1);
    }

    #[test]
    fn test_metric_type_display()
    {
        assert_eq!(MetricType::Counter.to_string(), "counter");
        assert_eq!(MetricType::Gauge.to_string(), "gauge");
        assert_eq!(MetricType::Histogram.to_string(), "histogram");
    }

    #[test]
    fn test_registry_clear()
    {
        let registry = MetricsRegistry::new();
        registry.counter("test").increment();
        registry.gauge("test_gauge").set(5);

        assert!(!registry.get_all().is_empty());

        registry.clear();
        assert!(registry.get_all().is_empty());
    }
}
