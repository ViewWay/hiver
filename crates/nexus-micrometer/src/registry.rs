//! Metric registry
//! 指标注册表

use crate::counter::Counter;
use crate::error::{Result, MicrometerError};
use crate::gauge::Gauge;
use crate::metric::{MetricId, MetricName, Tags};
use crate::timer::{LongTaskTimer, Timer};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Metric registry
/// 指标注册表
#[derive(Clone)]
pub struct MetricRegistry {
    inner: Arc<RegistryInner>,
}

struct RegistryInner {
    /// Counters
    /// 计数器
    counters: RwLock<HashMap<String, Counter>>,

    /// Gauges
    /// 仪表盘
    gauges: RwLock<HashMap<String, Gauge>>,

    /// Timers
    /// 计时器
    timers: RwLock<HashMap<String, Timer>>,

    /// Long task timers
    /// 长任务计时器
    long_task_timers: RwLock<HashMap<String, LongTaskTimer>>,
}

impl MetricRegistry {
    /// Create a new registry
    /// 创建新注册表
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RegistryInner {
                counters: RwLock::new(HashMap::new()),
                gauges: RwLock::new(HashMap::new()),
                timers: RwLock::new(HashMap::new()),
                long_task_timers: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// Register a counter
    /// 注册计数器
    pub fn counter(&self, name: &str) -> Result<Counter> {
        self.counter_with_tags(name, Tags::new())
    }

    /// Register a counter with tags
    /// 注册带标签的计数器
    pub fn counter_with_tags(&self, name: &str, tags: Tags) -> Result<Counter> {
        let metric_name = MetricName::new(name)?;
        let key = self.metric_key(&metric_name, &tags);
        let id = MetricId::new(metric_name.clone(), tags);

        let mut counters = self.inner.counters.write().unwrap();
        if let Some(counter) = counters.get(&key) {
            Ok(counter.clone())
        } else {
            let counter = Counter::new(id);
            counters.insert(key.clone(), counter.clone());
            Ok(counter)
        }
    }

    /// Register a gauge
    /// 注册仪表盘
    pub fn gauge(&self, name: &str) -> Result<Gauge> {
        self.gauge_with_tags(name, Tags::new())
    }

    /// Register a gauge with tags
    /// 注册带标签的仪表盘
    pub fn gauge_with_tags(&self, name: &str, tags: Tags) -> Result<Gauge> {
        let metric_name = MetricName::new(name)?;
        let key = self.metric_key(&metric_name, &tags);
        let id = MetricId::new(metric_name.clone(), tags);

        let mut gauges = self.inner.gauges.write().unwrap();
        if let Some(gauge) = gauges.get(&key) {
            Ok(gauge.clone())
        } else {
            let gauge = Gauge::new(id);
            gauges.insert(key.clone(), gauge.clone());
            Ok(gauge)
        }
    }

    /// Register a function gauge
    /// 注册函数仪表盘
    pub fn function_gauge<F>(&self, name: &str, f: F) -> Result<Gauge>
    where
        F: Fn() -> f64 + Send + Sync + 'static,
    {
        self.function_gauge_with_tags(name, Tags::new(), f)
    }

    /// Register a function gauge with tags
    /// 注册带标签的函数仪表盘
    pub fn function_gauge_with_tags<F>(
        &self,
        name: &str,
        tags: Tags,
        f: F,
    ) -> Result<Gauge>
    where
        F: Fn() -> f64 + Send + Sync + 'static,
    {
        let metric_name = MetricName::new(name)?;
        let key = self.metric_key(&metric_name, &tags);
        let id = MetricId::new(metric_name.clone(), tags);

        let mut gauges = self.inner.gauges.write().unwrap();

        // For function gauges, we create a regular gauge that wraps the function
        // In a real implementation, we'd store the FunctionGauge separately
        if let Some(gauge) = gauges.get(&key) {
            Ok(gauge.clone())
        } else {
            // Create a gauge that samples the function value
            let gauge = Gauge::with_value(id, f());
            gauges.insert(key.clone(), gauge.clone());
            Ok(gauge)
        }
    }

    /// Register a timer
    /// 注册计时器
    pub fn timer(&self, name: &str) -> Result<Timer> {
        self.timer_with_tags(name, Tags::new())
    }

    /// Register a timer with tags
    /// 注册带标签的计时器
    pub fn timer_with_tags(&self, name: &str, tags: Tags) -> Result<Timer> {
        let metric_name = MetricName::new(name)?;
        let key = self.metric_key(&metric_name, &tags);
        let id = MetricId::new(metric_name.clone(), tags);

        let mut timers = self.inner.timers.write().unwrap();
        if let Some(timer) = timers.get(&key) {
            Ok(timer.clone())
        } else {
            let timer = Timer::new(id);
            timers.insert(key.clone(), timer.clone());
            Ok(timer)
        }
    }

    /// Register a long task timer
    /// 注册长任务计时器
    pub fn long_task_timer(&self, name: &str) -> Result<LongTaskTimer> {
        self.long_task_timer_with_tags(name, Tags::new())
    }

    /// Register a long task timer with tags
    /// 注册带标签的长任务计时器
    pub fn long_task_timer_with_tags(&self, name: &str, tags: Tags) -> Result<LongTaskTimer> {
        let metric_name = MetricName::new(name)?;
        let key = self.metric_key(&metric_name, &tags);
        let id = MetricId::new(metric_name.clone(), tags);

        let mut long_timers = self.inner.long_task_timers.write().unwrap();
        if let Some(timer) = long_timers.get(&key) {
            Ok(timer.clone())
        } else {
            let timer = LongTaskTimer::new(id);
            long_timers.insert(key.clone(), timer.clone());
            Ok(timer)
        }
    }

    /// Get all counters
    /// 获取所有计数器
    pub fn get_counters(&self) -> Vec<Counter> {
        self.inner
            .counters
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Get all gauges
    /// 获取所有仪表盘
    pub fn get_gauges(&self) -> Vec<Gauge> {
        self.inner
            .gauges
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Get all timers
    /// 获取所有计时器
    pub fn get_timers(&self) -> Vec<Timer> {
        self.inner
            .timers
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Remove a metric by name
    /// 按名称删除指标
    pub fn remove(&self, name: &str) -> Result<()> {
        let mut counters = self.inner.counters.write().unwrap();
        let mut gauges = self.inner.gauges.write().unwrap();
        let mut timers = self.inner.timers.write().unwrap();
        let mut long_timers = self.inner.long_task_timers.write().unwrap();

        let mut removed = false;

        if counters.remove(name).is_some() {
            removed = true;
        }
        if gauges.remove(name).is_some() {
            removed = true;
        }
        if timers.remove(name).is_some() {
            removed = true;
        }
        if long_timers.remove(name).is_some() {
            removed = true;
        }

        if removed {
            Ok(())
        } else {
            Err(MicrometerError::MetricNotFound(name.to_string()))
        }
    }

    /// Clear all metrics
    /// 清除所有指标
    pub fn clear(&self) {
        self.inner.counters.write().unwrap().clear();
        self.inner.gauges.write().unwrap().clear();
        self.inner.timers.write().unwrap().clear();
        self.inner.long_task_timers.write().unwrap().clear();
    }

    /// Get metric count
    /// 获取指标数量
    pub fn metric_count(&self) -> usize {
        self.inner.counters.read().unwrap().len()
            + self.inner.gauges.read().unwrap().len()
            + self.inner.timers.read().unwrap().len()
            + self.inner.long_task_timers.read().unwrap().len()
    }

    /// Generate metric key from name and tags
    /// 从名称和标签生成指标键
    fn metric_key(&self, name: &MetricName, tags: &Tags) -> String {
        if tags.is_empty() {
            name.to_string()
        } else {
            let tag_str = tags
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            format!("{}{{{}}}", name, tag_str)
        }
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metric registry
/// 全局指标注册表
static GLOBAL_REGISTRY: std::sync::OnceLock<MetricRegistry> = std::sync::OnceLock::new();

/// Get the global metric registry
/// 获取全局指标注册表
pub fn global_registry() -> &'static MetricRegistry {
    GLOBAL_REGISTRY.get_or_init(|| MetricRegistry::new())
}

/// Convenience function to get or create a counter
/// 便捷函数：获取或创建计数器
pub fn counter(name: &str) -> Result<Counter> {
    global_registry().counter(name)
}

/// Convenience function to get or create a gauge
/// 便捷函数：获取或创建仪表盘
pub fn gauge(name: &str) -> Result<Gauge> {
    global_registry().gauge(name)
}

/// Convenience function to get or create a timer
/// 便捷函数：获取或创建计时器
pub fn timer(name: &str) -> Result<Timer> {
    global_registry().timer(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_counter() {
        let registry = MetricRegistry::new();
        let counter = registry.counter("test_counter").unwrap();

        counter.increment();
        assert_eq!(counter.count(), 1);

        // Same counter should be returned
        let counter2 = registry.counter("test_counter").unwrap();
        assert_eq!(counter2.count(), 1);
    }

    #[test]
    fn test_registry_gauge() {
        let registry = MetricRegistry::new();
        let gauge = registry.gauge("test_gauge").unwrap();

        gauge.set(42.0);
        assert_eq!(gauge.value(), 42.0);
    }

    #[test]
    fn test_registry_timer() {
        let registry = MetricRegistry::new();
        let timer = registry.timer("test_timer").unwrap();

        timer.record(Duration::from_millis(100));
        assert_eq!(timer.count(), 1);
    }

    #[test]
    fn test_registry_clear() {
        let registry = MetricRegistry::new();
        registry.counter("test").unwrap();
        registry.gauge("test2").unwrap();

        assert_eq!(registry.metric_count(), 2);
        registry.clear();
        assert_eq!(registry.metric_count(), 0);
    }

    #[test]
    fn test_global_registry() {
        let counter = counter("global_test").unwrap();
        counter.increment();

        let counter2 = global_registry().counter("global_test").unwrap();
        assert_eq!(counter2.count(), 1);
    }

    #[test]
    fn test_metric_key_with_tags() {
        let registry = MetricRegistry::new();
        let name = MetricName::new("test").unwrap();
        let mut tags = Tags::new();
        tags.add("key1", "value1").unwrap();

        let key = registry.metric_key(&name, &tags);
        assert_eq!(key, "test{key1=value1}");
    }
}
