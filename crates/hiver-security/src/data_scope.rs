//! Data scope / permission annotations for row-level access control.
//! 数据范围/权限注解，用于行级访问控制。
//!
//! Equivalent to Spring's `@DataScope` annotation used in RuoYi-style
//! admin systems to restrict SQL queries based on the current user's
//! department and role hierarchy.
//!
//! 等价于 Spring 的 `@DataScope` 注解，用于在若依风格的
//! 管理系统中根据当前用户的部门和角色层级限制 SQL 查询。
//!
//! # Spring Equivalent / Spring 等价物
//!
//! ```java
//! @DataScope(deptAlias = "d", userAlias = "u")
//! public List<User> selectUserList(User user) { ... }
//!
//! @DataScope(deptAlias = "d", userAlias = "u", scopeType = DataScopeType.SELF_ONLY)
//! public User selectUserById(Long id) { ... }
//! ```
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_security::data_scope::{DataScope, DataScopeRule, DataScopeType, DataScopeApply};
//!
//! // Define a rule describing which table columns carry department/user info
//! let rule = DataScopeRule::new("d", "dept_id", "user_id");
//!
//! // Create a scope for the current user
//! let scope = DataScope::new(DataScopeType::DeptAndSub)
//!     .with_dept_ids(vec![100, 101, 102]);
//!
//! // Generate the SQL WHERE fragment
//! let condition = rule.apply_scope(&scope, 1001, 100);
//! // => "d.dept_id IN (100, 101, 102) OR d.user_id = 1001"
//! ```

use crate::SecurityContext;
use std::sync::Arc;
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// DataScopeType
// ---------------------------------------------------------------------------

/// Data scope type for row-level access control.
/// 行级访问控制的数据范围类型。
///
/// Determines which rows a user is allowed to access.
/// 决定用户被允许访问哪些行。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataScopeType {
    /// Access all data — no filtering applied.
    /// 访问所有数据——不进行过滤。
    All,

    /// Custom SQL conditions provided by the administrator.
    /// 由管理员提供的自定义 SQL 条件。
    Custom,

    /// Access only rows belonging to the user's own department.
    /// 仅访问属于用户自己部门的行。
    Department,

    /// Access rows belonging to the user's department and all sub-departments.
    /// 访问属于用户部门及所有子部门的行。
    DeptAndSub,

    /// Access only rows created by the current user.
    /// 仅访问当前用户创建的行。
    SelfOnly,
}

impl DataScopeType {
    /// Returns `true` when no SQL filtering is needed.
    /// 当不需要 SQL 过滤时返回 `true`。
    pub fn requires_filtering(&self) -> bool {
        !matches!(self, DataScopeType::All)
    }
}

impl std::fmt::Display for DataScopeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataScopeType::All => write!(f, "ALL"),
            DataScopeType::Custom => write!(f, "CUSTOM"),
            DataScopeType::Department => write!(f, "DEPARTMENT"),
            DataScopeType::DeptAndSub => write!(f, "DEPT_AND_SUB"),
            DataScopeType::SelfOnly => write!(f, "SELF_ONLY"),
        }
    }
}

// ---------------------------------------------------------------------------
// DataScope
// ---------------------------------------------------------------------------

/// Data scope specification for row-level access control.
/// 行级访问控制的数据范围规范。
///
/// Holds the scope type along with the department IDs and any
/// custom conditions needed to build a SQL WHERE clause fragment.
///
/// 保存范围类型以及用于构建 SQL WHERE 子句片段的部门 ID 和自定义条件。
#[derive(Clone, Debug)]
pub struct DataScope {
    /// Scope type determines which rows are visible.
    /// 范围类型决定可见的行。
    pub scope_type: DataScopeType,

    /// Department IDs for department-based scopes.
    /// 用于基于部门的范围的部门 ID。
    pub dept_ids: Vec<u64>,

    /// Custom SQL conditions (only used when `scope_type` is `Custom`).
    /// 自定义 SQL 条件（仅在 `scope_type` 为 `Custom` 时使用）。
    pub custom_conditions: Vec<String>,
}

impl DataScope {
    /// Create a new data scope with the given type.
    /// 创建具有给定类型的新数据范围。
    pub fn new(scope_type: DataScopeType) -> Self {
        Self {
            scope_type,
            dept_ids: Vec::new(),
            custom_conditions: Vec::new(),
        }
    }

    /// Create a data scope that grants access to all data.
    /// 创建授予对所有数据访问权限的数据范围。
    pub fn all() -> Self {
        Self::new(DataScopeType::All)
    }

    /// Create a data scope restricted to the user's own department.
    /// 创建限制为用户自己部门的数据范围。
    pub fn department(dept_id: u64) -> Self {
        Self::new(DataScopeType::Department).with_dept_ids(vec![dept_id])
    }

    /// Create a data scope restricted to the user's department and sub-departments.
    /// 创建限制为用户部门及子部门的数据范围。
    pub fn dept_and_sub(dept_ids: Vec<u64>) -> Self {
        Self::new(DataScopeType::DeptAndSub).with_dept_ids(dept_ids)
    }

    /// Create a data scope restricted to the current user only.
    /// 创建仅限制为当前用户的数据范围。
    pub fn self_only() -> Self {
        Self::new(DataScopeType::SelfOnly)
    }

    /// Create a data scope with custom SQL conditions.
    /// 创建具有自定义 SQL 条件的数据范围。
    pub fn custom(conditions: Vec<String>) -> Self {
        Self::new(DataScopeType::Custom).with_custom_conditions(conditions)
    }

    /// Add department IDs (builder-style).
    /// 添加部门 ID（构建器风格）。
    pub fn with_dept_ids(mut self, ids: Vec<u64>) -> Self {
        self.dept_ids = ids;
        self
    }

    /// Add a single department ID (builder-style).
    /// 添加单个部门 ID（构建器风格）。
    pub fn add_dept_id(mut self, id: u64) -> Self {
        self.dept_ids.push(id);
        self
    }

    /// Set custom SQL conditions (builder-style).
    /// 设置自定义 SQL 条件（构建器风格）。
    pub fn with_custom_conditions(mut self, conditions: Vec<String>) -> Self {
        self.custom_conditions = conditions;
        self
    }

    /// Returns `true` if this scope requires SQL filtering.
    /// 如果此范围需要 SQL 过滤，则返回 `true`。
    pub fn requires_filtering(&self) -> bool {
        self.scope_type.requires_filtering()
    }
}

impl Default for DataScope {
    fn default() -> Self {
        Self::all()
    }
}

// ---------------------------------------------------------------------------
// DataScopeRule
// ---------------------------------------------------------------------------

/// Data scope rule that describes how to apply a scope to a SQL query.
/// 描述如何将范围应用到 SQL 查询的数据范围规则。
///
/// Maps the logical data scope to concrete table columns so that a
/// WHERE clause fragment can be generated.
///
/// 将逻辑数据范围映射到具体的表列，以便生成 WHERE 子句片段。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_security::data_scope::DataScopeRule;
///
/// // "d" is the table alias for the department table,
/// // "dept_id" is the column name for department ID,
/// // "create_by" is the column name for the user who created the row.
/// let rule = DataScopeRule::new("d", "dept_id", "create_by");
/// ```
#[derive(Clone, Debug)]
pub struct DataScopeRule {
    /// Table alias used in the SQL query (e.g. `"d"` for `department d`).
    /// SQL 查询中使用的表别名（例如 `department d` 的 `"d"`）。
    pub table_alias: String,

    /// Column name that holds the department ID.
    /// 保存部门 ID 的列名。
    pub dept_column: String,

    /// Column name that holds the user ID (creator/owner).
    /// 保存用户 ID（创建者/所有者）的列名。
    pub user_column: String,
}

impl DataScopeRule {
    /// Create a new data scope rule.
    /// 创建新的数据范围规则。
    pub fn new(
        table_alias: impl Into<String>,
        dept_column: impl Into<String>,
        user_column: impl Into<String>,
    ) -> Self {
        Self {
            table_alias: table_alias.into(),
            dept_column: dept_column.into(),
            user_column: user_column.into(),
        }
    }

    /// Fully qualified department column reference (e.g. `d.dept_id`).
    /// 完全限定的部门列引用（例如 `d.dept_id`）。
    pub fn dept_ref(&self) -> String {
        format!("{}.{}", self.table_alias, self.dept_column)
    }

    /// Fully qualified user column reference (e.g. `d.create_by`).
    /// 完全限定的用户列引用（例如 `d.create_by`）。
    pub fn user_ref(&self) -> String {
        format!("{}.{}", self.table_alias, self.user_column)
    }
}

// ---------------------------------------------------------------------------
// DataScopeApply trait
// ---------------------------------------------------------------------------

/// Trait for applying data scope to queries.
/// 用于将数据范围应用到查询的 trait。
///
/// Implement this trait for your query builder to automatically
/// generate SQL WHERE clause fragments based on the current user's
/// data scope.
///
/// 为你的查询构建器实现此 trait，以根据当前用户的数据范围
/// 自动生成 SQL WHERE 子句片段。
pub trait DataScopeApply {
    /// Generate a SQL WHERE clause fragment for the given data scope.
    /// 为给定的数据范围生成 SQL WHERE 子句片段。
    ///
    /// The returned string is a condition that can be appended to a
    /// WHERE clause. Returns an empty string when no filtering is needed.
    ///
    /// 返回的字符串是可以附加到 WHERE 子句的条件。
    /// 当不需要过滤时返回空字符串。
    fn apply_scope(&self, scope: &DataScope, user_id: u64, dept_id: u64) -> String;
}

/// Blanket implementation: `DataScopeRule` can generate SQL conditions.
/// Blanket实现：`DataScopeRule` 可以生成 SQL 条件。
impl DataScopeApply for DataScopeRule {
    fn apply_scope(&self, scope: &DataScope, user_id: u64, dept_id: u64) -> String {
        match &scope.scope_type {
            DataScopeType::All => String::new(),

            DataScopeType::Custom => {
                if scope.custom_conditions.is_empty() {
                    String::new()
                } else {
                    format!("({})", scope.custom_conditions.join(" AND "))
                }
            },

            DataScopeType::Department => {
                let col = self.dept_ref();
                format!("{} = {}", col, dept_id)
            },

            DataScopeType::DeptAndSub => {
                let col = self.dept_ref();
                if scope.dept_ids.is_empty() {
                    // Fall back to own department when no sub-dept IDs are provided.
                    // 当没有提供子部门 ID 时，回退到自己的部门。
                    format!("{} = {}", col, dept_id)
                } else {
                    let ids: Vec<String> = scope.dept_ids.iter().map(|id| id.to_string()).collect();
                    format!("{} IN ({})", col, ids.join(", "))
                }
            },

            DataScopeType::SelfOnly => {
                let col = self.user_ref();
                format!("{} = {}", col, user_id)
            },
        }
    }
}

// ---------------------------------------------------------------------------
// DataScopeContext — async task-local storage
// ---------------------------------------------------------------------------

/// Per-request data scope context.
/// 每请求的数据范围上下文。
///
/// Stores the current user's data scope, user ID, and department ID
/// so they can be accessed by the data layer when building queries.
///
/// 存储当前用户的数据范围、用户 ID 和部门 ID，
/// 以便数据层在构建查询时可以访问。
#[derive(Clone, Debug)]
pub struct DataScopeContext {
    /// The data scope definition.
    /// 数据范围定义。
    scope: DataScope,

    /// Current user's ID.
    /// 当前用户的 ID。
    user_id: u64,

    /// Current user's department ID.
    /// 当前用户的部门 ID。
    dept_id: u64,
}

impl DataScopeContext {
    /// Create a new data scope context.
    /// 创建新的数据范围上下文。
    pub fn new(scope: DataScope, user_id: u64, dept_id: u64) -> Self {
        Self {
            scope,
            user_id,
            dept_id,
        }
    }

    /// Get a reference to the data scope.
    /// 获取数据范围的引用。
    pub fn scope(&self) -> &DataScope {
        &self.scope
    }

    /// Get the current user ID.
    /// 获取当前用户 ID。
    pub fn user_id(&self) -> u64 {
        self.user_id
    }

    /// Get the current department ID.
    /// 获取当前部门 ID。
    pub fn dept_id(&self) -> u64 {
        self.dept_id
    }

    /// Apply a rule to generate a SQL WHERE clause fragment.
    /// 应用规则生成 SQL WHERE 子句片段。
    pub fn apply_rule(&self, rule: &DataScopeRule) -> String {
        rule.apply_scope(&self.scope, self.user_id, self.dept_id)
    }
}

// Task-local storage for the per-request data scope context.
// 每请求数据范围上下文的任务本地存储。
tokio::task_local! {
    static CURRENT_DATA_SCOPE: Arc<RwLock<Option<DataScopeContext>>>;
}

/// Set the data scope context for the current async task and run a closure.
/// 设置当前异步任务的数据范围上下文并运行闭包。
///
/// The context is automatically removed when the closure returns.
/// 闭包返回时上下文会自动移除。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_security::data_scope::{DataScope, DataScopeContext};
///
/// let scope = DataScope::dept_and_sub(vec![100, 101, 102]);
/// let ctx = DataScopeContext::new(scope, 1001, 100);
///
/// let result = hiver_security::data_scope::with_data_scope(ctx, || {
///     // Inside this block, `get_data_scope()` returns the context.
///     42
/// });
/// ```
pub fn with_data_scope<F, R>(ctx: DataScopeContext, f: F) -> R
where
    F: FnOnce() -> R,
{
    CURRENT_DATA_SCOPE.sync_scope(Arc::new(RwLock::new(Some(ctx))), f)
}

/// Set the data scope context for the current async task and run an async block.
/// 设置当前异步任务的数据范围上下文并运行异步块。
///
/// The context propagates through all `.await` points inside the future.
/// 上下文在 future 内所有 `.await` 点传播。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_security::data_scope::{DataScope, DataScopeContext};
///
/// let scope = DataScope::self_only();
/// let ctx = DataScopeContext::new(scope, 1001, 100);
///
/// let result = hiver_security::data_scope::with_data_scope_async(ctx, || async {
///     some_async_fn().await;
///     let current = hiver_security::data_scope::get_data_scope().await;
///     current.unwrap().apply_rule(&rule)
/// }).await;
/// ```
pub async fn with_data_scope_async<F, Fut, R>(ctx: DataScopeContext, f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    CURRENT_DATA_SCOPE
        .scope(Arc::new(RwLock::new(Some(ctx))), f())
        .await
}

/// Get the current data scope context from task-local storage.
/// 从任务本地存储获取当前数据范围上下文。
///
/// Returns `None` if no context has been set for the current task.
/// 如果当前任务未设置上下文，则返回 `None`。
pub async fn get_data_scope() -> Option<DataScopeContext> {
    CURRENT_DATA_SCOPE
        .try_with(|lock| {
            // We need a synchronous clone from the RwLock.
            // Since this is behind `try_with`, we are inside the task-local scope.
            let guard = lock.try_read().ok()?;
            guard.clone()
        })
        .ok()
        .flatten()
}

/// Apply a data scope rule using the current task-local context.
/// 使用当前任务本地上下文应用数据范围规则。
///
/// Returns an empty string if no context is set or no filtering is needed.
/// 如果未设置上下文或不需要过滤，则返回空字符串。
pub async fn apply_data_scope(rule: &DataScopeRule) -> String {
    match get_data_scope().await {
        Some(ctx) => ctx.apply_rule(rule),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// DataScopeMiddleware — HTTP middleware integration
// ---------------------------------------------------------------------------

/// Middleware that resolves the current user's data scope from the
/// [`SecurityContext`] and installs it as the task-local data scope.
/// 中间件，从 [`SecurityContext`] 解析当前用户的数据范围，
/// 并将其安装为任务本地数据范围。
///
/// This middleware should be placed after the authentication middleware
/// so that the `SecurityContext` already contains the user's info.
///
/// 此中间件应放置在认证中间件之后，
/// 以便 `SecurityContext` 已包含用户信息。
///
/// # Spring Equivalent / Spring 等价物
///
/// ```java
/// @DataScope(deptAlias = "d", userAlias = "u")
/// public List<SysUser> selectUserList(SysUser user) { ... }
/// ```
pub struct DataScopeMiddleware {
    /// Function that extracts the data scope from an `Authentication`.
    /// 从 `Authentication` 提取数据范围的函数。
    scope_resolver:
        Arc<dyn Fn(&crate::Authentication) -> Option<(DataScope, u64, u64)> + Send + Sync>,

    /// Optional fallback user ID when authentication is absent.
    /// 当认证不存在时的可选回退用户 ID。
    fallback_user_id: u64,

    /// Optional fallback department ID when authentication is absent.
    /// 当认证不存在时的可选回退部门 ID。
    fallback_dept_id: u64,
}

impl DataScopeMiddleware {
    /// Create a new middleware with a custom scope resolver.
    /// 使用自定义范围解析器创建新中间件。
    ///
    /// The resolver receives the current `Authentication` and should
    /// return `(DataScope, user_id, dept_id)` when a scope can be
    /// determined, or `None` to skip data scope filtering.
    ///
    /// 解析器接收当前的 `Authentication`，当可以确定范围时
    /// 应返回 `(DataScope, user_id, dept_id)`，否则返回 `None` 以跳过数据范围过滤。
    pub fn new<F>(resolver: F) -> Self
    where
        F: Fn(&crate::Authentication) -> Option<(DataScope, u64, u64)> + Send + Sync + 'static,
    {
        Self {
            scope_resolver: Arc::new(resolver),
            fallback_user_id: 0,
            fallback_dept_id: 0,
        }
    }

    /// Set the fallback user ID (defaults to 0).
    /// 设置回退用户 ID（默认为 0）。
    pub fn fallback_user_id(mut self, id: u64) -> Self {
        self.fallback_user_id = id;
        self
    }

    /// Set the fallback department ID (defaults to 0).
    /// 设置回退部门 ID（默认为 0）。
    pub fn fallback_dept_id(mut self, id: u64) -> Self {
        self.fallback_dept_id = id;
        self
    }

    /// Process a request: extract data scope from the security context,
    /// install it as the task-local value, and invoke the handler.
    /// 处理请求：从安全上下文提取数据范围，
    /// 将其安装为任务本地值，并调用处理程序。
    pub async fn handle<F, Fut, R>(&self, security_context: &SecurityContext, handler: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = R>,
    {
        // Try to resolve the data scope from the current authentication.
        // 尝试从当前认证解析数据范围。
        if let Some(auth) = security_context.get_authentication().await {
            if let Some((scope, user_id, dept_id)) = (self.scope_resolver)(&auth) {
                let ctx = DataScopeContext::new(scope, user_id, dept_id);
                return with_data_scope_async(ctx, handler).await;
            }
        }

        // No scope could be resolved — proceed without filtering.
        // 无法解析范围——继续进行而不进行过滤。
        handler().await
    }
}

/// Create a default scope resolver that looks up scope information
/// from the authentication's authorities.
/// 创建从认证的权限中查找范围信息的默认范围解析器。
///
/// This resolver looks for a special authority with the prefix
/// `DATASCOPE:` that encodes the scope type and department IDs.
///
/// 此解析器查找具有前缀 `DATASCOPE:` 的特殊权限，
/// 该权限编码了范围类型和部门 ID。
///
/// # Authority format / 权限格式
///
/// ```text
/// DATASCOPE:<type>:<user_id>:<dept_id>:<dept_ids_csv>
/// ```
///
/// Example: `DATASCOPE:DEPT_AND_SUB:1001:100:100,101,102`
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_security::data_scope::default_scope_resolver;
/// use hiver_security::data_scope::DataScopeMiddleware;
///
/// let middleware = DataScopeMiddleware::new(default_scope_resolver());
/// ```
pub fn default_scope_resolver()
-> impl Fn(&crate::Authentication) -> Option<(DataScope, u64, u64)> + Send + Sync {
    move |auth: &crate::Authentication| {
        for authority in &auth.authorities {
            if let crate::Authority::Permission(perm) = authority {
                if let Some(rest) = perm.strip_prefix("DATASCOPE:") {
                    return parse_data_scope_authority(rest);
                }
            }
        }
        None
    }
}

/// Parse a `DATASCOPE:` authority value into a `(DataScope, user_id, dept_id)`.
/// 将 `DATASCOPE:` 权限值解析为 `(DataScope, user_id, dept_id)`。
///
/// Expected format: `<TYPE>:<user_id>:<dept_id>:<dept_ids_csv>`
/// 预期格式：`<TYPE>:<user_id>:<dept_id>:<dept_ids_csv>`
fn parse_data_scope_authority(value: &str) -> Option<(DataScope, u64, u64)> {
    let parts: Vec<&str> = value.splitn(4, ':').collect();
    if parts.len() < 3 {
        return None;
    }

    let scope_type = match parts[0] {
        "ALL" => DataScopeType::All,
        "CUSTOM" => DataScopeType::Custom,
        "DEPARTMENT" => DataScopeType::Department,
        "DEPT_AND_SUB" => DataScopeType::DeptAndSub,
        "SELF_ONLY" => DataScopeType::SelfOnly,
        _ => return None,
    };

    let user_id: u64 = parts[1].parse().ok()?;
    let dept_id: u64 = parts[2].parse().ok()?;

    let mut scope = DataScope::new(scope_type);

    // Parse department IDs if present (4th field).
    // 如果存在（第 4 个字段），则解析部门 ID。
    if parts.len() == 4 && !parts[3].is_empty() {
        let dept_ids: Vec<u64> = parts[3].split(',').filter_map(|s| s.parse().ok()).collect();
        scope.dept_ids = dept_ids;
    }

    Some((scope, user_id, dept_id))
}

/// Helper: build a `DATASCOPE:` authority string from components.
/// 辅助函数：从组件构建 `DATASCOPE:` 权限字符串。
///
/// This can be used when constructing `Authentication` objects to
/// embed data scope information in the user's authorities.
///
/// 可用于在构建 `Authentication` 对象时，
/// 将数据范围信息嵌入到用户的权限中。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_security::data_scope::build_data_scope_authority;
/// use hiver_security::data_scope::DataScopeType;
///
/// let auth_str = build_data_scope_authority(
///     &DataScopeType::DeptAndSub,
///     1001,
///     100,
///     &[100, 101, 102],
/// );
/// assert_eq!(auth_str, "DATASCOPE:DEPT_AND_SUB:1001:100:100,101,102");
/// ```
pub fn build_data_scope_authority(
    scope_type: &DataScopeType,
    user_id: u64,
    dept_id: u64,
    dept_ids: &[u64],
) -> String {
    let type_str = scope_type.to_string();
    let dept_csv: String = dept_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("DATASCOPE:{}:{}:{}:{}", type_str, user_id, dept_id, dept_csv)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- DataScopeType --

    #[test]
    fn test_data_scope_type_requires_filtering() {
        assert!(!DataScopeType::All.requires_filtering());
        assert!(DataScopeType::Custom.requires_filtering());
        assert!(DataScopeType::Department.requires_filtering());
        assert!(DataScopeType::DeptAndSub.requires_filtering());
        assert!(DataScopeType::SelfOnly.requires_filtering());
    }

    #[test]
    fn test_data_scope_type_display() {
        assert_eq!(DataScopeType::All.to_string(), "ALL");
        assert_eq!(DataScopeType::Custom.to_string(), "CUSTOM");
        assert_eq!(DataScopeType::Department.to_string(), "DEPARTMENT");
        assert_eq!(DataScopeType::DeptAndSub.to_string(), "DEPT_AND_SUB");
        assert_eq!(DataScopeType::SelfOnly.to_string(), "SELF_ONLY");
    }

    // -- DataScope --

    #[test]
    fn test_data_scope_constructors() {
        let all = DataScope::all();
        assert_eq!(all.scope_type, DataScopeType::All);
        assert!(all.dept_ids.is_empty());

        let dept = DataScope::department(100);
        assert_eq!(dept.scope_type, DataScopeType::Department);
        assert_eq!(dept.dept_ids, vec![100]);

        let sub = DataScope::dept_and_sub(vec![100, 101, 102]);
        assert_eq!(sub.scope_type, DataScopeType::DeptAndSub);
        assert_eq!(sub.dept_ids, vec![100, 101, 102]);

        let self_only = DataScope::self_only();
        assert_eq!(self_only.scope_type, DataScopeType::SelfOnly);

        let custom = DataScope::custom(vec!["status = 1".to_string()]);
        assert_eq!(custom.scope_type, DataScopeType::Custom);
        assert_eq!(custom.custom_conditions, vec!["status = 1"]);
    }

    #[test]
    fn test_data_scope_builder() {
        let scope = DataScope::new(DataScopeType::DeptAndSub)
            .add_dept_id(100)
            .add_dept_id(101);
        assert_eq!(scope.dept_ids, vec![100, 101]);
    }

    #[test]
    fn test_data_scope_default() {
        let scope = DataScope::default();
        assert!(!scope.requires_filtering());
    }

    // -- DataScopeRule & DataScopeApply --

    #[test]
    fn test_rule_all_scope() {
        let rule = DataScopeRule::new("d", "dept_id", "create_by");
        let scope = DataScope::all();
        let sql = rule.apply_scope(&scope, 1001, 100);
        assert!(sql.is_empty());
    }

    #[test]
    fn test_rule_custom_scope() {
        let rule = DataScopeRule::new("d", "dept_id", "create_by");
        let scope = DataScope::custom(vec!["status = 1".to_string(), "is_deleted = 0".to_string()]);
        let sql = rule.apply_scope(&scope, 1001, 100);
        assert_eq!(sql, "(status = 1 AND is_deleted = 0)");
    }

    #[test]
    fn test_rule_custom_scope_empty() {
        let rule = DataScopeRule::new("d", "dept_id", "create_by");
        let scope = DataScope::custom(vec![]);
        let sql = rule.apply_scope(&scope, 1001, 100);
        assert!(sql.is_empty());
    }

    #[test]
    fn test_rule_department_scope() {
        let rule = DataScopeRule::new("d", "dept_id", "create_by");
        let scope = DataScope::department(100);
        let sql = rule.apply_scope(&scope, 1001, 100);
        assert_eq!(sql, "d.dept_id = 100");
    }

    #[test]
    fn test_rule_dept_and_sub_scope() {
        let rule = DataScopeRule::new("d", "dept_id", "create_by");
        let scope = DataScope::dept_and_sub(vec![100, 101, 102]);
        let sql = rule.apply_scope(&scope, 1001, 100);
        assert_eq!(sql, "d.dept_id IN (100, 101, 102)");
    }

    #[test]
    fn test_rule_dept_and_sub_scope_empty_ids() {
        let rule = DataScopeRule::new("d", "dept_id", "create_by");
        let scope = DataScope::dept_and_sub(vec![]);
        let sql = rule.apply_scope(&scope, 1001, 100);
        // Falls back to own department.
        assert_eq!(sql, "d.dept_id = 100");
    }

    #[test]
    fn test_rule_self_only_scope() {
        let rule = DataScopeRule::new("d", "dept_id", "create_by");
        let scope = DataScope::self_only();
        let sql = rule.apply_scope(&scope, 1001, 100);
        assert_eq!(sql, "d.create_by = 1001");
    }

    // -- DataScopeContext --

    #[test]
    fn test_data_scope_context() {
        let scope = DataScope::dept_and_sub(vec![100, 101]);
        let ctx = DataScopeContext::new(scope, 1001, 100);
        assert_eq!(ctx.user_id(), 1001);
        assert_eq!(ctx.dept_id(), 100);

        let rule = DataScopeRule::new("t", "dept_id", "user_id");
        let sql = ctx.apply_rule(&rule);
        assert_eq!(sql, "t.dept_id IN (100, 101)");
    }

    // -- Task-local context --

    #[test]
    fn test_with_data_scope() {
        let scope = DataScope::self_only();
        let ctx = DataScopeContext::new(scope, 42, 10);

        let result = with_data_scope(ctx, || {
            // We cannot use `get_data_scope()` here because it is async
            // and we are in a sync closure. We just verify the function runs.
            99
        });

        assert_eq!(result, 99);
    }

    #[tokio::test]
    async fn test_with_data_scope_async() {
        let scope = DataScope::department(200);
        let ctx = DataScopeContext::new(scope, 42, 200);

        let user_id = with_data_scope_async(ctx, || async {
            tokio::task::yield_now().await;
            let current = get_data_scope().await.expect("should have context");
            current.user_id()
        })
        .await;

        assert_eq!(user_id, 42);
    }

    #[tokio::test]
    async fn test_get_data_scope_none_when_unset() {
        assert!(get_data_scope().await.is_none());
    }

    #[tokio::test]
    async fn test_apply_data_scope() {
        let scope = DataScope::self_only();
        let ctx = DataScopeContext::new(scope, 42, 10);

        let result = with_data_scope_async(ctx, || async {
            let rule = DataScopeRule::new("u", "dept_id", "owner_id");
            apply_data_scope(&rule).await
        })
        .await;

        assert_eq!(result, "u.owner_id = 42");
    }

    // -- Authority parsing/building --

    #[test]
    fn test_build_and_parse_authority_all() {
        let auth_str = build_data_scope_authority(&DataScopeType::All, 1, 10, &[]);
        assert_eq!(auth_str, "DATASCOPE:ALL:1:10:");

        let parsed = parse_data_scope_authority("ALL:1:10:").unwrap();
        assert_eq!(parsed.0.scope_type, DataScopeType::All);
        assert_eq!(parsed.1, 1);
        assert_eq!(parsed.2, 10);
    }

    #[test]
    fn test_build_and_parse_authority_dept_and_sub() {
        let auth_str =
            build_data_scope_authority(&DataScopeType::DeptAndSub, 1001, 100, &[100, 101, 102]);
        assert_eq!(auth_str, "DATASCOPE:DEPT_AND_SUB:1001:100:100,101,102");

        let parsed = parse_data_scope_authority("DEPT_AND_SUB:1001:100:100,101,102").unwrap();
        assert_eq!(parsed.0.scope_type, DataScopeType::DeptAndSub);
        assert_eq!(parsed.1, 1001);
        assert_eq!(parsed.2, 100);
        assert_eq!(parsed.0.dept_ids, vec![100, 101, 102]);
    }

    #[test]
    fn test_parse_authority_self_only() {
        let parsed = parse_data_scope_authority("SELF_ONLY:42:5:").unwrap();
        assert_eq!(parsed.0.scope_type, DataScopeType::SelfOnly);
        assert_eq!(parsed.1, 42);
        assert_eq!(parsed.2, 5);
    }

    #[test]
    fn test_parse_authority_invalid() {
        assert!(parse_data_scope_authority("").is_none());
        assert!(parse_data_scope_authority("INVALID:1:2:").is_none());
        assert!(parse_data_scope_authority("ALL").is_none());
        assert!(parse_data_scope_authority("ALL:abc:2:").is_none());
    }

    // -- default_scope_resolver --

    #[test]
    fn test_default_scope_resolver() {
        use crate::{Authentication, Authority};

        let resolver = default_scope_resolver();

        // No data scope authority → None.
        // 无数据范围权限 → None。
        let auth = Authentication {
            principal: "test".to_string(),
            credentials: None,
            authorities: vec![Authority::Role(crate::Role::Admin)],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        };
        assert!(resolver(&auth).is_none());

        // With data scope authority → Some.
        // 带有数据范围权限 → Some。
        let auth_str =
            build_data_scope_authority(&DataScopeType::DeptAndSub, 1001, 100, &[100, 101]);
        let auth = Authentication {
            principal: "test".to_string(),
            credentials: None,
            authorities: vec![Authority::Permission(auth_str)],
            authenticated: true,
            details: None,
            login_time: chrono::Utc::now(),
        };
        let result = resolver(&auth).unwrap();
        assert_eq!(result.0.scope_type, DataScopeType::DeptAndSub);
        assert_eq!(result.1, 1001);
        assert_eq!(result.2, 100);
    }

    // -- DataScopeRule column refs --

    #[test]
    fn test_rule_column_refs() {
        let rule = DataScopeRule::new("dept", "dept_id", "create_by");
        assert_eq!(rule.dept_ref(), "dept.dept_id");
        assert_eq!(rule.user_ref(), "dept.create_by");
    }
}
