//! gRPC health check service.
//! gRPC 健康检查服务。
//!
//! Equivalent to Spring Cloud gRPC health checking / Kubernetes readiness probes.
//! 等价于 Spring Cloud gRPC 健康检查 / Kubernetes 就绪探针。

use std::{collections::HashMap, sync::RwLock};

/// Serving status for a gRPC service.
/// gRPC 服务的服务状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServingStatus
{
    /// Unknown state.
    Unknown,
    /// Service is serving.
    Serving,
    /// Service is not serving.
    NotServing,
}

impl ServingStatus
{
    /// Numeric code matching grpc.health.v1.HealthCheckResponse.
    /// 与 grpc.health.v1.HealthCheckResponse 匹配的数字编码。
    pub fn code(self) -> i32
    {
        match self
        {
            Self::Unknown => 0,
            Self::Serving => 1,
            Self::NotServing => 2,
        }
    }
}

/// Overall health status (empty-string key = overall server health).
/// 整体健康状态（空字符串键 = 服务器整体健康）。
const OVERALL: &str = "";

/// In-memory health check service for tracking serving status of gRPC services.
/// 基于 gRPC 服务内存的健康状态跟踪器。
///
/// # Example / 示例
/// ```rust,ignore
/// use hiver_grpc::health::{HealthService, ServingStatus};
///
/// let health = HealthService::new();
/// health.set_serving("my.package.MyService");
/// assert_eq!(health.check("my.package.MyService"), ServingStatus::Serving);
/// ```
pub struct HealthService
{
    statuses: RwLock<HashMap<String, ServingStatus>>,
}

impl HealthService
{
    /// Create a new health service with overall status set to `Serving`.
    /// 创建新的健康服务，整体状态设为 Serving。
    pub fn new() -> Self
    {
        let mut map = HashMap::new();
        map.insert(OVERALL.to_string(), ServingStatus::Serving);
        Self {
            statuses: RwLock::new(map),
        }
    }

    /// Mark a service as serving.
    /// 将服务标记为 Serving。
    pub fn set_serving(&self, service: &str)
    {
        self.statuses
            .write()
            .expect("health service lock poisoned")
            .insert(service.to_string(), ServingStatus::Serving);
    }

    /// Mark a service as not serving.
    /// 将服务标记为 NotServing。
    pub fn set_not_serving(&self, service: &str)
    {
        self.statuses
            .write()
            .expect("health service lock poisoned")
            .insert(service.to_string(), ServingStatus::NotServing);
    }

    /// Set the status for a service explicitly.
    /// 显式设置服务的状态。
    pub fn set_status(&self, service: &str, status: ServingStatus)
    {
        self.statuses
            .write()
            .expect("health service lock poisoned")
            .insert(service.to_string(), status);
    }

    /// Query the serving status of a service.
    /// 查询服务的服务状态。
    pub fn check(&self, service: &str) -> ServingStatus
    {
        self.statuses
            .read()
            .expect("health service lock poisoned")
            .get(service)
            .copied()
            .unwrap_or(ServingStatus::Unknown)
    }

    /// Remove a service from tracking.
    /// 从跟踪中移除服务。
    pub fn remove(&self, service: &str)
    {
        self.statuses
            .write()
            .expect("health service lock poisoned")
            .remove(service);
    }

    /// Returns true if the overall server is in Serving state.
    /// 若整体服务器处于 Serving 状态则返回 true。
    pub fn is_serving(&self) -> bool
    {
        self.check(OVERALL) == ServingStatus::Serving
    }
}

impl Default for HealthService
{
    fn default() -> Self
    {
        Self::new()
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
    fn test_overall_serving_by_default()
    {
        let hs = HealthService::new();
        assert!(hs.is_serving());
        assert_eq!(hs.check(""), ServingStatus::Serving);
    }

    #[test]
    fn test_service_lifecycle()
    {
        let hs = HealthService::new();
        hs.set_serving("pkg.ServiceA");
        assert_eq!(hs.check("pkg.ServiceA"), ServingStatus::Serving);

        hs.set_not_serving("pkg.ServiceA");
        assert_eq!(hs.check("pkg.ServiceA"), ServingStatus::NotServing);

        hs.remove("pkg.ServiceA");
        assert_eq!(hs.check("pkg.ServiceA"), ServingStatus::Unknown);
    }

    #[test]
    fn test_unknown_service()
    {
        let hs = HealthService::new();
        assert_eq!(hs.check("nonexistent"), ServingStatus::Unknown);
    }

    #[test]
    fn test_overall_not_serving()
    {
        let hs = HealthService::new();
        hs.set_not_serving("");
        assert!(!hs.is_serving());
    }

    #[test]
    fn test_status_code()
    {
        assert_eq!(ServingStatus::Unknown.code(), 0);
        assert_eq!(ServingStatus::Serving.code(), 1);
        assert_eq!(ServingStatus::NotServing.code(), 2);
    }
}
