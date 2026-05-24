//! Tests for nexus-aop proc-macro crate
//! nexus-aop 过程宏 crate 的测试
//!
//! Tests here verify the proc-macro infrastructure and runtime integration.
//! Runtime type tests (JoinPoint, PointcutExpression, AspectRegistry) live in runtime.rs.
//! 此处的测试验证过程宏基础设施和运行时集成。
//! 运行时类型测试（JoinPoint, PointcutExpression, AspectRegistry）位于 runtime.rs。

#[cfg(test)]
mod tests {
    /// Verify the test infrastructure is functional
    /// 验证测试基础设施正常工作
    #[test]
    fn smoke_test() {
        assert!(true, "nexus-aop test infrastructure is working");
    }

    /// Verify runtime module is accessible from proc-macro test context
    /// 验证运行时模块在 proc-macro 测试上下文中可访问
    #[test]
    fn test_runtime_module_accessible() {
        use crate::runtime::{AdviceType, JoinPoint, PointcutExpression};

        let advice = AdviceType::Before;
        assert_eq!(format!("{:?}", advice), "Before");
    }

    /// Verify global registry is accessible from proc-macro test context
    /// 验证全局注册表在 proc-macro 测试上下文中可访问
    #[test]
    fn test_global_registry_accessible() {
        let _registry = crate::runtime::global_registry();
    }
}
