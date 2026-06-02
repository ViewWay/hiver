//! Bean registry — compile-time collected bean descriptors via `inventory`.
//! Bean 注册表 — 通过 `inventory` 在编译期收集 Bean 描述符。

use std::any::{Any, TypeId};

use super::container::ApplicationContext;

/// Bean lifecycle scope.
/// Bean 生命周期作用域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BeanScope {
    /// Single shared instance for the application.
    /// 应用内单例。
    #[default]
    Singleton,
    /// New instance on every `get_bean` call.
    /// 每次 `get_bean` 创建新实例。
    Prototype,
    /// Scoped to the current request/task.
    /// 绑定到当前请求/任务。
    Request,
}

/// Returns `true` when the bean should be registered (default: always).
/// 当应注册 Bean 时返回 `true`（默认：始终注册）。
pub type BeanConditionFn = fn(&ApplicationContext) -> bool;

/// Always-true condition used as default.
/// 默认始终为真的条件。
pub fn always_true(_ctx: &ApplicationContext) -> bool {
    true
}

/// Compile-time bean descriptor submitted by `#[service]` / `#[component]` macros.
/// 由 `#[service]` / `#[component]` 宏在编译期提交的 Bean 描述符。
pub struct BeanDescriptor {
    /// Bean name (camelCase type name by default).
    /// Bean 名称（默认为类型的 camelCase 名称）。
    pub name: &'static str,

    /// Returns the bean's `TypeId`.
    /// 返回 Bean 的 `TypeId`。
    pub type_id: fn() -> TypeId,

    /// Bean scope.
    /// Bean 作用域。
    pub scope: BeanScope,

    /// Factory that creates the bean from the application context.
    /// 从应用上下文创建 Bean 的工厂函数。
    pub factory: fn(&ApplicationContext) -> Box<dyn Any + Send + Sync>,

    /// Dependency type ids for topological ordering.
    /// 用于拓扑排序的依赖类型 ID。
    pub dep_type_ids: &'static [fn() -> TypeId],

    /// Optional registration condition.
    /// 可选的注册条件。
    pub condition: BeanConditionFn,
}

inventory::collect!(BeanDescriptor);

/// Trait for beans that need initialization after all dependencies are wired.
/// 所有依赖注入完成后需要初始化的 Bean 实现此 trait。
pub trait PostConstruct: Send + Sync {
    /// Called once after the bean is fully constructed.
    /// Bean 完全构造后调用一次。
    fn post_construct(&self);
}

/// Trait for beans that need cleanup on shutdown.
/// 关闭时需要清理的 Bean 实现此 trait。
pub trait PreDestroy: Send + Sync {
    /// Called before the application context shuts down.
    /// 应用上下文关闭前调用。
    fn pre_destroy(&self);
}

/// Convert a PascalCase ident to camelCase bean name.
/// 将 PascalCase 标识符转换为 camelCase Bean 名称。
pub fn to_bean_name(type_name: &str) -> String {
    let mut chars = type_name.chars();
    match chars.next() {
        Some(c) => c.to_ascii_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Topological sort of bean descriptors; detects circular dependencies.
/// 对 Bean 描述符进行拓扑排序；检测循环依赖。
pub fn topological_sort<'a>(
    descriptors: &[&'a BeanDescriptor],
) -> Result<Vec<&'a BeanDescriptor>, String> {
    let n = descriptors.len();
    if n == 0 {
        return Ok(Vec::new());
    }

    let mut providers: std::collections::HashMap<TypeId, usize> = std::collections::HashMap::new();
    for (idx, desc) in descriptors.iter().enumerate() {
        providers.insert((desc.type_id)(), idx);
    }

    let mut in_degree = vec![0usize; n];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];

    for (idx, desc) in descriptors.iter().enumerate() {
        for dep_fn in desc.dep_type_ids {
            let dep_id = dep_fn();
            if let Some(&provider_idx) = providers.get(&dep_id) {
                if provider_idx != idx {
                    in_degree[idx] += 1;
                    dependents[provider_idx].push(idx);
                }
            }
        }
    }

    let mut queue: std::collections::VecDeque<usize> =
        (0..n).filter(|&i| in_degree[i] == 0).collect();

    let mut sorted = Vec::with_capacity(n);
    while let Some(idx) = queue.pop_front() {
        sorted.push(descriptors[idx]);
        for &dep in &dependents[idx] {
            in_degree[dep] -= 1;
            if in_degree[dep] == 0 {
                queue.push_back(dep);
            }
        }
    }

    if sorted.len() != n {
        return Err(
            "Circular dependency detected among beans; cannot initialize ApplicationContext"
                .to_string(),
        );
    }

    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_factory(_ctx: &ApplicationContext) -> Box<dyn Any + Send + Sync> {
        Box::new(0i32)
    }

    #[test]
    fn test_to_bean_name() {
        assert_eq!(to_bean_name("UserService"), "userService");
    }

    #[test]
    fn test_topological_sort_empty() {
        let sorted = topological_sort(&[]).unwrap();
        assert!(sorted.is_empty());
    }
}
