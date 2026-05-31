//! Actuator 自动配置模块 / Actuator Auto-Configuration Module
//!
//! 自动配置监控端点（健康检查、指标等）。
//! Auto-configures monitoring endpoints (health check, metrics, etc.).

use crate::core::{AutoConfiguration, ApplicationContext};

// Re-export actuator types
// 重新导出 actuator 类型
pub use hiver_actuator::{
    Actuator, InfoBuilder, HealthCheck, HealthIndicator, HealthStatus,
    MetricsRegistry, Metric, MetricType,
    Environment, EnvironmentCollector,
};

// Re-export the handler function
// 重新导出处理函数
pub use hiver_actuator::routes::handle_request;

// ============================================================================
// ActuatorAutoConfiguration / Actuator 自动配置
// ============================================================================

/// Actuator 自动配置
/// Actuator auto-configuration
///
/// 参考 Spring Boot 的 `ActuatorAutoConfiguration`。
/// Based on Spring Boot's `ActuatorAutoConfiguration`.
#[derive(Debug)]
pub struct ActuatorAutoConfiguration {
    /// 是否启用 Actuator
    pub enabled: bool,

    /// 是否启用健康检查端点
    pub health_enabled: bool,

    /// 是否启用指标端点
    pub metrics_enabled: bool,

    /// Actuator 基础路径
    pub base_path: String,
}

impl ActuatorAutoConfiguration {
    /// 创建新的 Actuator 自动配置
    pub fn new() -> Self {
        Self {
            enabled: true,
            health_enabled: true,
            metrics_enabled: true,
            base_path: "/actuator".to_string(),
        }
    }

    /// 从配置创建
    pub fn from_config(ctx: &ApplicationContext) -> Self {
        Self {
            enabled: ctx
                .get_property("actuator.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(true),
            health_enabled: ctx
                .get_property("actuator.health.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(true),
            metrics_enabled: ctx
                .get_property("actuator.metrics.enabled")
                .and_then(|p| p.parse().ok())
                .unwrap_or(true),
            base_path: ctx
                .get_property_or("actuator.base_path", "/actuator"),
        }
    }
}

impl Default for ActuatorAutoConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoConfiguration for ActuatorAutoConfiguration {
    fn name(&self) -> &'static str {
        "ActuatorAutoConfiguration"
    }

    fn order(&self) -> i32 {
        200  // 较低优先级，在业务配置之后
    }

    fn configure(&self, ctx: &mut ApplicationContext) -> anyhow::Result<()> {
        tracing::info!("Configuring Actuator (base path: {})", self.base_path);

        // Create Actuator with configured settings
        // 使用配置的设置创建 Actuator
        let mut actuator = Actuator::new()
            .enable_health(self.health_enabled)
            .enable_metrics(self.metrics_enabled);

        // Set application info from configuration
        // 从配置设置应用信息
        let app_name = ctx.get_property_or("app.name", "hiver-app");
        let app_version = ctx.get_property_or("app.version", "0.1.0");
        actuator = actuator.info(app_name, app_version);

        // Register the actuator as a bean
        // 将 actuator 注册为 bean
        ctx.register_bean(actuator);

        // Log enabled endpoints
        // 记录启用的端点
        if self.health_enabled {
            tracing::info!("  Health endpoint: {}/health", self.base_path);
        }

        if self.metrics_enabled {
            tracing::info!("  Metrics endpoint: {}/metrics", self.base_path);
        }

        // Info and env are always enabled by default in Actuator
        // Info 和 env 在 Actuator 中默认总是启用
        tracing::info!("  Info endpoint: {}/info", self.base_path);
        tracing::info!("  Env endpoint: {}/env", self.base_path);

        tracing::info!("Registered Actuator bean");

        Ok(())
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actuator_auto_config() {
        let config = ActuatorAutoConfiguration::new();
        assert!(config.enabled);
        assert!(config.health_enabled);
        assert!(config.metrics_enabled);
        assert_eq!(config.base_path, "/actuator");
    }

    #[test]
    fn test_actuator_auto_config_registers_actuator() {
        let config = ActuatorAutoConfiguration {
            enabled: true,
            health_enabled: true,
            metrics_enabled: true,
            base_path: "/actuator".to_string(),
        };

        let mut ctx = ApplicationContext::new();
        config.configure(&mut ctx).unwrap();

        // Verify Actuator was registered
        // 验证 Actuator 已注册
        assert!(ctx.contains_bean::<Actuator>());
    }

    #[test]
    fn test_actuator_auto_config_with_custom_base_path() {
        let config = ActuatorAutoConfiguration {
            enabled: true,
            health_enabled: true,
            metrics_enabled: false,
            base_path: "/management".to_string(),
        };

        let mut ctx = ApplicationContext::new();
        config.configure(&mut ctx).unwrap();

        // Verify Actuator was registered even with custom base path
        // 验证即使使用自定义基础路径，Actuator 也已注册
        assert!(ctx.contains_bean::<Actuator>());
    }
}
