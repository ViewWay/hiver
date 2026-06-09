//! Beans endpoint - list all registered IoC beans
//! Beans 端点 - 列出所有注册的 IoC Bean
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! `/actuator/beans` - Lists all beans in the IoC container.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Descriptor for a single bean.
/// 单个 Bean 的描述符。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeanDescriptor {
    /// Bean scope (singleton / prototype).
    /// Bean 作用域。
    pub scope: String,
    /// Bean type name.
    /// Bean 类型名。
    pub r#type: String,
    /// Dependencies (bean names this bean depends on).
    /// 依赖项。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

/// Response for /actuator/beans.
/// /actuator/beans 的响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeansResponse {
    /// Map of bean name → descriptor.
    /// Bean 名称到描述符的映射。
    pub beans: HashMap<String, BeanDescriptor>,
}

/// Builder for collecting bean descriptors.
/// Bean 描述符收集器构建器。
#[derive(Debug, Clone, Default)]
pub struct BeansBuilder {
    beans: HashMap<String, BeanDescriptor>,
}

impl BeansBuilder {
    /// Create a new builder.
    /// 创建新构建器。
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a bean.
    /// 注册一个 Bean。
    pub fn bean(
        mut self,
        name: impl Into<String>,
        r#type: impl Into<String>,
        scope: impl Into<String>,
    ) -> Self {
        self.beans.insert(
            name.into(),
            BeanDescriptor {
                scope: scope.into(),
                r#type: r#type.into(),
                dependencies: Vec::new(),
            },
        );
        self
    }

    /// Register a bean with dependencies.
    /// 注册带依赖的 Bean。
    pub fn bean_with_deps(
        mut self,
        name: impl Into<String>,
        r#type: impl Into<String>,
        scope: impl Into<String>,
        dependencies: Vec<String>,
    ) -> Self {
        self.beans.insert(
            name.into(),
            BeanDescriptor {
                scope: scope.into(),
                r#type: r#type.into(),
                dependencies,
            },
        );
        self
    }

    /// Build the response.
    /// 构建响应。
    pub fn build(self) -> BeansResponse {
        BeansResponse { beans: self.beans }
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
mod tests {
    use super::*;

    #[test]
    fn test_beans_builder() {
        let resp = BeansBuilder::new()
            .bean("userController", "UserController", "singleton")
            .bean_with_deps("userService", "UserService", "singleton", vec!["userRepo".into()])
            .build();

        assert_eq!(resp.beans.len(), 2);
        assert!(resp.beans.contains_key("userController"));
        let svc = resp.beans.get("userService").unwrap();
        assert_eq!(svc.dependencies.len(), 1);
    }

    #[test]
    fn test_beans_serialize() {
        let resp = BeansBuilder::new()
            .bean("app", "Application", "singleton")
            .build();
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"beans\""));
        assert!(json.contains("\"app\""));
    }
}
