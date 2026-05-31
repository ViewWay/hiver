//! Actuator routes
//! Actuator 路由
//!
//! # Equivalent to Spring Boot Actuator endpoints
//! # 等价于 Spring Boot Actuator 端点

use crate::beans::{BeansBuilder, BeansResponse};
use crate::env::{EnvironmentCollector, EnvironmentResponse};
use crate::health::{HealthCheck, SystemHealthIndicator};
use crate::info::{AppInfo, InfoBuilder};
use crate::loggers::LoggerManager;
use crate::mappings::{MappingsBuilder, MappingsResponse};
use crate::metrics::MetricsRegistry;
use hiver_http::{Body, Request, Response, StatusCode};
use std::sync::Arc;

/// Actuator routes
/// Actuator 路由
///
/// # Spring Equivalent / Spring 等价物
///
/// Equivalent to Spring Boot Actuator with endpoints: /health, /info, /metrics
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_actuator::Actuator;
///
/// let actuator = Actuator::new()
///     .info("my-app", "1.0.0")
///     .enable_health(true)
///     .enable_metrics(true);
/// ```
#[derive(Clone)]
pub struct Actuator {
    /// Application information
    /// 应用信息
    app_info: AppInfo,

    /// Health check registry
    /// 健康检查注册表
    health_check: Arc<HealthCheck>,

    /// Metrics registry
    /// 指标注册表
    metrics_registry: Arc<MetricsRegistry>,

    /// Enable health endpoint
    /// 启用健康端点
    enable_health: bool,

    /// Enable info endpoint
    /// 启用信息端点
    enable_info: bool,

    /// Enable metrics endpoint
    /// 启用指标端点
    enable_metrics: bool,

    /// Enable env endpoint
    /// 启用环境端点
    enable_env: bool,

    /// Enable beans endpoint
    /// 启用 beans 端点
    enable_beans: bool,

    /// Enable loggers endpoint
    /// 启用 loggers 端点
    enable_loggers: bool,

    /// Enable mappings endpoint
    /// 启用 mappings 端点
    enable_mappings: bool,

    /// Environment collector
    /// 环境收集器
    env_collector: Arc<EnvironmentCollector>,

    /// Logger manager
    /// 日志管理器
    logger_manager: Arc<std::sync::Mutex<LoggerManager>>,

    /// Cached beans response
    /// 缓存的 beans 响应
    beans_response: Arc<BeansResponse>,

    /// Cached mappings response
    /// 缓存的 mappings 响应
    mappings_response: Arc<MappingsResponse>,
}

impl Actuator {
    /// Create a new actuator
    /// 创建新的 actuator
    pub fn new() -> Self {
        let mut health_check = HealthCheck::new();
        health_check = health_check.indicator(Box::new(SystemHealthIndicator));

        Self {
            app_info: AppInfo::new(),
            health_check: Arc::new(health_check),
            metrics_registry: Arc::new(MetricsRegistry::new()),
            enable_health: true,
            enable_info: true,
            enable_metrics: true,
            enable_env: true,
            enable_beans: true,
            enable_loggers: true,
            enable_mappings: true,
            env_collector: Arc::new(EnvironmentCollector::new()),
            logger_manager: Arc::new(std::sync::Mutex::new(LoggerManager::new())),
            beans_response: Arc::new(BeansBuilder::new().build()),
            mappings_response: Arc::new(MappingsBuilder::new().build()),
        }
    }

    /// Set application information
    /// 设置应用信息
    pub fn info(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.app_info = InfoBuilder::new()
            .name(name.into())
            .version(version.into())
            .build();
        self
    }

    /// Set full application info
    /// 设置完整的应用信息
    pub fn with_app_info(mut self, info: AppInfo) -> Self {
        self.app_info = info;
        self
    }

    /// Set the metrics registry
    /// 设置指标注册表
    pub fn with_metrics_registry(mut self, registry: Arc<MetricsRegistry>) -> Self {
        self.metrics_registry = registry;
        self
    }

    /// Enable or disable health endpoint
    /// 启用或禁用健康端点
    pub fn enable_health(mut self, enable: bool) -> Self {
        self.enable_health = enable;
        self
    }

    /// Enable or disable info endpoint
    /// 启用或禁用信息端点
    pub fn enable_info(mut self, enable: bool) -> Self {
        self.enable_info = enable;
        self
    }

    /// Enable or disable metrics endpoint
    /// 启用或禁用指标端点
    pub fn enable_metrics(mut self, enable: bool) -> Self {
        self.enable_metrics = enable;
        self
    }

    /// Enable or disable env endpoint
    /// 启用或禁用环境端点
    pub fn enable_env(mut self, enable: bool) -> Self {
        self.enable_env = enable;
        self
    }

    /// Set the environment collector
    /// 设置环境收集器
    pub fn with_env_collector(mut self, collector: EnvironmentCollector) -> Self {
        self.env_collector = Arc::new(collector);
        self
    }

    /// Enable or disable beans endpoint
    /// 启用或禁用 beans 端点
    pub fn enable_beans(mut self, enable: bool) -> Self {
        self.enable_beans = enable;
        self
    }

    /// Enable or disable loggers endpoint
    /// 启用或禁用 loggers 端点
    pub fn enable_loggers(mut self, enable: bool) -> Self {
        self.enable_loggers = enable;
        self
    }

    /// Enable or disable mappings endpoint
    /// 启用或禁用 mappings 端点
    pub fn enable_mappings(mut self, enable: bool) -> Self {
        self.enable_mappings = enable;
        self
    }

    /// Set the beans data.
    /// 设置 beans 数据。
    pub fn with_beans(mut self, builder: BeansBuilder) -> Self {
        self.beans_response = Arc::new(builder.build());
        self
    }

    /// Set the mappings data.
    /// 设置 mappings 数据。
    pub fn with_mappings(mut self, builder: MappingsBuilder) -> Self {
        self.mappings_response = Arc::new(builder.build());
        self
    }

    /// Get the logger manager.
    /// 获取日志管理器。
    pub fn get_logger_manager(&self) -> &Arc<std::sync::Mutex<LoggerManager>> {
        &self.logger_manager
    }

    /// Get the metrics registry
    /// 获取指标注册表
    pub fn get_metrics(&self) -> Arc<MetricsRegistry> {
        Arc::clone(&self.metrics_registry)
    }

    /// Get the application info
    /// 获取应用信息
    pub fn get_app_info(&self) -> &AppInfo {
        &self.app_info
    }

    /// Get the environment collector
    /// 获取环境收集器
    pub fn get_env_collector(&self) -> Arc<EnvironmentCollector> {
        Arc::clone(&self.env_collector)
    }

    /// Handle the actuator index request
    /// 处理 actuator 索引请求
    pub fn handle_index(&self, _req: &Request) -> Response {
        let mut links = serde_json::Map::new();
        let self_link = serde_json::json!({
            "href": "/actuator",
            "templated": false
        });
        links.insert("self".to_string(), self_link);

        if self.enable_health {
            let health_link = serde_json::json!({
                "href": "/actuator/health",
                "templated": false
            });
            links.insert("health".to_string(), health_link);
        }

        if self.enable_info {
            let info_link = serde_json::json!({
                "href": "/actuator/info",
                "templated": false
            });
            links.insert("info".to_string(), info_link);
        }

        if self.enable_metrics {
            let metrics_link = serde_json::json!({
                "href": "/actuator/metrics",
                "templated": false
            });
            links.insert("metrics".to_string(), metrics_link);
        }

        if self.enable_env {
            let env_link = serde_json::json!({
                "href": "/actuator/env",
                "templated": false
            });
            links.insert("env".to_string(), env_link);
        }

        if self.enable_beans {
            let beans_link = serde_json::json!({
                "href": "/actuator/beans",
                "templated": false
            });
            links.insert("beans".to_string(), beans_link);
        }

        if self.enable_loggers {
            let loggers_link = serde_json::json!({
                "href": "/actuator/loggers",
                "templated": false
            });
            links.insert("loggers".to_string(), loggers_link);
        }

        if self.enable_mappings {
            let mappings_link = serde_json::json!({
                "href": "/actuator/mappings",
                "templated": false
            });
            links.insert("mappings".to_string(), mappings_link);
        }

        let body = serde_json::to_vec(&links).unwrap_or_default();
        Response::new(StatusCode::OK).with_body(Body::from(body))
    }

    /// Handle the health request
    /// 处理健康请求
    pub fn handle_health(&self, _req: &Request) -> Response {
        if !self.enable_health {
            return Response::new(StatusCode::NOT_FOUND);
        }

        let health = self.health_check.check();
        let status = if health.status.is_healthy() {
            StatusCode::OK
        } else {
            StatusCode::from_u16(503)
        };

        let body = serde_json::to_vec(&health).unwrap_or_default();
        Response::new(status).with_body(Body::from(body))
    }

    /// Handle the info request
    /// 处理信息请求
    pub fn handle_info(&self, _req: &Request) -> Response {
        if !self.enable_info {
            return Response::new(StatusCode::NOT_FOUND);
        }

        let body = serde_json::to_vec(&self.app_info).unwrap_or_default();
        Response::new(StatusCode::OK).with_body(Body::from(body))
    }

    /// Handle the metrics list request
    /// 处理指标列表请求
    pub fn handle_metrics(&self, _req: &Request) -> Response {
        if !self.enable_metrics {
            return Response::new(StatusCode::NOT_FOUND);
        }

        let names = self.metrics_registry.names();
        let body = serde_json::to_vec(&names).unwrap_or_default();
        Response::new(StatusCode::OK).with_body(Body::from(body))
    }

    /// Handle the specific metric request
    /// 处理特定指标请求
    pub fn handle_metric(&self, name: &str, _req: &Request) -> Response {
        if !self.enable_metrics {
            return Response::new(StatusCode::NOT_FOUND);
        }

        // Check counters
        if let Some(value) = self.metrics_registry.get_counter(name) {
            let metric = crate::metrics::Metric::counter(name, value);
            let body = serde_json::to_vec(&metric).unwrap_or_default();
            return Response::new(StatusCode::OK).with_body(Body::from(body));
        }

        // Check gauges
        if let Some(value) = self.metrics_registry.get_gauge(name) {
            let metric = crate::metrics::Metric::gauge(name, value);
            let body = serde_json::to_vec(&metric).unwrap_or_default();
            return Response::new(StatusCode::OK).with_body(Body::from(body));
        }

        Response::new(StatusCode::NOT_FOUND)
    }

    /// Handle the env request
    /// 处理环境请求
    pub fn handle_env(&self, _req: &Request) -> Response {
        if !self.enable_env {
            return Response::new(StatusCode::NOT_FOUND);
        }

        let env = self.env_collector.collect();
        let response: EnvironmentResponse = env.into();
        let body = serde_json::to_vec(&response).unwrap_or_default();
        Response::new(StatusCode::OK).with_body(Body::from(body))
    }

    /// Handle the specific property request (e.g., /env/some.property)
    /// 处理特定属性请求（例如 /env/some.property）
    pub fn handle_property(&self, key: &str, _req: &Request) -> Response {
        if !self.enable_env {
            return Response::new(StatusCode::NOT_FOUND);
        }

        if let Some(value) = self.env_collector.get_property(key) {
            let property = serde_json::json!({
                "property": {
                    "value": value,
                    "origin": "unknown"
                }
            });
            let body = serde_json::to_vec(&property).unwrap_or_default();
            Response::new(StatusCode::OK).with_body(Body::from(body))
        } else {
            Response::new(StatusCode::NOT_FOUND)
                .with_body(Body::from(r#"{"error":"Property not found"}"#))
        }
    }

    /// Handle the beans request.
    /// 处理 beans 请求。
    pub fn handle_beans(&self, _req: &Request) -> Response {
        if !self.enable_beans {
            return Response::new(StatusCode::NOT_FOUND);
        }
        let body = serde_json::to_vec(&*self.beans_response).unwrap_or_default();
        Response::new(StatusCode::OK).with_body(Body::from(body))
    }

    /// Handle the loggers request.
    /// 处理 loggers 请求。
    pub fn handle_loggers(&self, _req: &Request) -> Response {
        if !self.enable_loggers {
            return Response::new(StatusCode::NOT_FOUND);
        }
        let mgr = self.logger_manager.lock().unwrap();
        let resp = mgr.to_response();
        let body = serde_json::to_vec(&resp).unwrap_or_default();
        Response::new(StatusCode::OK).with_body(Body::from(body))
    }

    /// Handle the mappings request.
    /// 处理 mappings 请求。
    pub fn handle_mappings(&self, _req: &Request) -> Response {
        if !self.enable_mappings {
            return Response::new(StatusCode::NOT_FOUND);
        }
        let body = serde_json::to_vec(&*self.mappings_response).unwrap_or_default();
        Response::new(StatusCode::OK).with_body(Body::from(body))
    }
}

impl Default for Actuator {
    fn default() -> Self {
        Self::new()
    }
}

/// Route handler for the actuator
/// Actuator 的路由处理器
pub fn handle_request(actuator: Arc<Actuator>, req: &Request) -> Response {
    let path = req.uri().to_string();

    // Remove /actuator prefix for matching
    let subpath = path.strip_prefix("/actuator").unwrap_or(&path);

    match subpath {
        "" | "/" => actuator.handle_index(req),
        "/health" => actuator.handle_health(req),
        "/info" => actuator.handle_info(req),
        "/metrics" => actuator.handle_metrics(req),
        "/env" => actuator.handle_env(req),
        "/beans" => actuator.handle_beans(req),
        "/loggers" => actuator.handle_loggers(req),
        "/mappings" => actuator.handle_mappings(req),
        path if path.starts_with("/metrics/") => {
            let name = &path[10..]; // Remove "/metrics/"
            actuator.handle_metric(name, req)
        },
        path if path.starts_with("/env/") => {
            let key = &path[6..]; // Remove "/env/"
            actuator.handle_property(key, req)
        },
        _ => {
            Response::new(StatusCode::NOT_FOUND).with_body(Body::from("{\"error\":\"Not found\"}"))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actuator_new() {
        let actuator = Actuator::new().info("test-app", "1.0.0");

        assert_eq!(actuator.app_info.name, Some("test-app".to_string()));
        assert_eq!(actuator.app_info.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_actuator_default() {
        let actuator = Actuator::default();
        assert!(actuator.enable_health);
        assert!(actuator.enable_info);
        assert!(actuator.enable_metrics);
        assert!(actuator.enable_env);
    }

    #[test]
    fn test_actuator_enable_disable() {
        let actuator = Actuator::new()
            .enable_health(false)
            .enable_info(false)
            .enable_metrics(false);

        assert!(!actuator.enable_health);
        assert!(!actuator.enable_info);
        assert!(!actuator.enable_metrics);
    }

    #[test]
    fn test_handle_index() {
        let actuator = Actuator::new();
        let request = Request::from_method_uri(hiver_http::Method::GET, "/actuator");
        let response = actuator.handle_index(&request);

        assert_eq!(response.status(), StatusCode::OK);
    }
}
