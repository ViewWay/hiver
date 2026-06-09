//! Health check endpoint
//! 健康检查端点
//!
//! # Equivalent to Spring Boot Actuator /health
//! # 等价于 Spring Boot Actuator /health

use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

/// Health status
/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HealthStatus
{
    /// The component is healthy / 组件健康
    Up,

    /// The component is unhealthy / 组件不健康
    Down,

    /// The component is out of service / 组件不可用
    OutOfService,

    /// The component status is unknown / 组件状态未知
    Unknown,
}

impl HealthStatus
{
    /// Check if the status represents a healthy state
    /// 检查状态是否表示健康
    pub fn is_healthy(self) -> bool
    {
        matches!(self, HealthStatus::Up)
    }
}

impl fmt::Display for HealthStatus
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            HealthStatus::Up => write!(f, "UP"),
            HealthStatus::Down => write!(f, "DOWN"),
            HealthStatus::OutOfService => write!(f, "OUT_OF_SERVICE"),
            HealthStatus::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Health check result
/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health
{
    /// Overall status
    /// 整体状态
    pub status: HealthStatus,

    /// Component-specific health details
    /// 组件特定的健康详情
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub components: HashMap<String, ComponentHealth>,
}

impl Health
{
    /// Create a new health check result
    /// 创建新的健康检查结果
    pub fn new(status: HealthStatus) -> Self
    {
        Self {
            status,
            components: HashMap::new(),
        }
    }

    /// Create an UP health status
    /// 创建 UP 健康状态
    pub fn up() -> Self
    {
        Self::new(HealthStatus::Up)
    }

    /// Create a DOWN health status
    /// 创建 DOWN 健康状态
    pub fn down() -> Self
    {
        Self::new(HealthStatus::Down)
    }

    /// Add a component health check result
    /// 添加组件健康检查结果
    pub fn with_component(mut self, name: impl Into<String>, health: ComponentHealth) -> Self
    {
        self.components.insert(name.into(), health);
        self
    }
}

/// Component health check result
/// 组件健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth
{
    /// Component status
    /// 组件状态
    pub status: HealthStatus,

    /// Additional details about the component
    /// 组件的额外详情
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub details: HashMap<String, serde_json::Value>,
}

impl ComponentHealth
{
    /// Create a new component health
    /// 创建新的组件健康
    pub fn new(status: HealthStatus) -> Self
    {
        Self {
            status,
            details: HashMap::new(),
        }
    }

    /// Add a detail
    /// 添加详情
    pub fn with_detail(mut self, key: impl Into<String>, value: serde_json::Value) -> Self
    {
        self.details.insert(key.into(), value);
        self
    }
}

/// Health indicator trait
/// 健康指标 trait
///
/// # Spring Equivalent / Spring 等价物
///
/// Equivalent to Spring's `HealthIndicator` interface.
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// struct DatabaseHealthIndicator;
///
/// impl HealthIndicator for DatabaseHealthIndicator {
///     fn name(&self) -> &str {
///         "db"
///     }
///
///     fn check(&self) -> ComponentHealth {
///         // Check database connection...
///         ComponentHealth::new(HealthStatus::Up)
///             .with_detail("database", "postgres")
///             .with_detail("latency", 5)
///     }
/// }
/// ```
pub trait HealthIndicator: Send + Sync
{
    /// Get the indicator name
    /// 获取指标名称
    fn name(&self) -> &str;

    /// Perform the health check
    /// 执行健康检查
    fn check(&self) -> ComponentHealth;
}

/// Health check registry
/// 健康检查注册表
#[derive(Default)]
pub struct HealthCheck
{
    indicators: Vec<Box<dyn HealthIndicator>>,
}

impl HealthCheck
{
    /// Create a new health check registry
    /// 创建新的健康检查注册表
    pub fn new() -> Self
    {
        Self {
            indicators: Vec::new(),
        }
    }

    /// Add a health indicator
    /// 添加健康指标
    pub fn indicator(mut self, indicator: Box<dyn HealthIndicator>) -> Self
    {
        self.indicators.push(indicator);
        self
    }

    /// Perform all health checks
    /// 执行所有健康检查
    pub fn check(&self) -> Health
    {
        let mut components = HashMap::new();
        let mut overall_status = HealthStatus::Up;

        for indicator in &self.indicators
        {
            let health = indicator.check();
            if !health.status.is_healthy()
            {
                overall_status = HealthStatus::Down;
            }
            components.insert(indicator.name().to_string(), health);
        }

        Health {
            status: overall_status,
            components,
        }
    }
}

/// Default system health indicator
/// 默认系统健康指标
pub struct SystemHealthIndicator;

impl HealthIndicator for SystemHealthIndicator
{
    fn name(&self) -> &'static str
    {
        "system"
    }

    fn check(&self) -> ComponentHealth
    {
        // Simple system check - always up for now
        // A full implementation would check disk space, memory, etc.
        ComponentHealth::new(HealthStatus::Up)
            .with_detail("status", serde_json::json!("operational"))
    }
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
    fn test_health_status()
    {
        assert!(HealthStatus::Up.is_healthy());
        assert!(!HealthStatus::Down.is_healthy());
        assert!(!HealthStatus::OutOfService.is_healthy());
        assert!(!HealthStatus::Unknown.is_healthy());
    }

    #[test]
    fn test_health_display()
    {
        assert_eq!(HealthStatus::Up.to_string(), "UP");
        assert_eq!(HealthStatus::Down.to_string(), "DOWN");
    }

    #[test]
    fn test_health()
    {
        let health = Health::up().with_component("db", ComponentHealth::new(HealthStatus::Up));

        assert_eq!(health.status, HealthStatus::Up);
        assert!(health.components.contains_key("db"));
    }
}
