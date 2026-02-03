//! Validation groups support / 验证分组支持
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! // Define validation groups
//! public interface CreateGroup {}
//! public interface UpdateGroup {}
//!
//! @NotNull(groups = CreateGroup.class)
//! private String username;
//!
//! @PostMapping("/users")
//! public ResponseEntity<User> createUser(
//!     @Validated(CreateGroup.class) @RequestBody UserRequest request
//! ) {
//!     // Only validates fields marked with CreateGroup
//! }
//! ```
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use nexus_validation::Validated;
//!
//! #[derive(Debug, Default)]
//! struct CreateGroup;
//!
//! #[derive(Debug, Default)]
//! struct UpdateGroup;
//!
//! #[derive(Debug, Deserialize)]
//! struct UserRequest {
//!     #[NotNull(group = "CreateGroup")]
//!     username: String,
//!
//!     #[NotNull(group = "UpdateGroup")]
//!     id: u64,
//! }
//!
//! // Only validates fields with CreateGroup
//! #[nexus_macros::post("/users")]
//! async fn create_user(
//!     #[Validated(CreateGroup)] request: UserRequest,
//! ) -> Result<Json<User>, Error> {
//!     Ok(Json(user))
//! }
//! ```

use crate::ValidationError;
use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::fmt;

/// Validation group trait / 验证分组trait
///
/// Marker trait for validation groups.
/// 验证分组的标记trait。
pub trait ValidationGroup: Any + Send + Sync + 'static {
    /// Get the group name / 获取分组名称
    fn name(&self) -> &'static str {
        let type_name = std::any::type_name::<Self>();
        // Extract just the type name from the full path
        // 从完整路径中提取类型名称
        type_name
            .rsplit("::")
            .next()
            .unwrap_or(type_name)
    }

    /// Get the group TypeId / 获取分组TypeId
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Default validation group / 默认验证分组
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultGroup;

impl ValidationGroup for DefaultGroup {}

/// Validation group set / 验证分组集合
///
/// Represents a set of active validation groups.
/// 表示一组活跃的验证分组。
#[derive(Clone)]
pub struct GroupSet {
    /// Active group type IDs / 活跃分组TypeId
    groups: HashSet<TypeId>,
    /// Group names for display / 用于显示的分组名称
    names: HashSet<String>,
}

impl GroupSet {
    /// Create a new empty group set / 创建新的空分组集合
    pub fn new() -> Self {
        Self {
            groups: HashSet::new(),
            names: HashSet::new(),
        }
    }

    /// Create with default group / 使用默认分组创建
    pub fn default_group() -> Self {
        Self::with_group(DefaultGroup)
    }

    /// Create with a single group / 使用单个分组创建
    pub fn with_group<G: ValidationGroup>(group: G) -> Self {
        let mut set = Self::new();
        set.add(group);
        set
    }

    /// Create with multiple groups / 使用多个分组创建
    pub fn with_groups<G: ValidationGroup>(groups: impl IntoIterator<Item = G>) -> Self {
        let mut set = Self::new();
        for group in groups {
            set.add(group);
        }
        set
    }

    /// Add a group / 添加分组
    pub fn add<G: ValidationGroup>(&mut self, group: G) {
        self.groups.insert(TypeId::of::<G>());
        self.names.insert(group.name().to_string());
    }

    /// Check if a group is active / 检查分组是否活跃
    pub fn contains<G: ValidationGroup>(&self) -> bool {
        self.groups.contains(&TypeId::of::<G>())
    }

    /// Check if default group is active / 检查默认分组是否活跃
    pub fn contains_default(&self) -> bool {
        self.contains::<DefaultGroup>()
    }

    /// Check if the set is empty / 检查集合是否为空
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Get the number of groups / 获取分组数量
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    /// Get all group names / 获取所有分组名称
    pub fn names(&self) -> &HashSet<String> {
        &self.names
    }
}

impl Default for GroupSet {
    fn default() -> Self {
        Self::default_group()
    }
}

impl fmt::Debug for GroupSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set()
            .entries(self.names.iter())
            .finish()
    }
}

/// Validated wrapper for group-based validation
/// 用于基于分组验证的Validated包装器
///
/// Extracts and validates request data using specific validation groups.
/// 使用特定验证分组提取和验证请求数据。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @PostMapping("/users")
/// public ResponseEntity<User> createUser(
///     @Validated(CreateGroup.class) @RequestBody UserRequest request
/// ) { }
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_validation::Validated;
///
/// #[derive(Debug, Default)]
/// struct CreateGroup;
///
/// #[nexus_macros::post("/users")]
/// async fn create_user(
///     #[Validated(CreateGroup)] request: CreateUserRequest,
/// ) -> Result<Json<User>, Error> {
///     // Only validates fields marked with CreateGroup
/// }
/// ```
pub struct Validated<T, G = DefaultGroup> {
    /// The validated value / 已验证的值
    pub value: T,
    /// The validation group / 验证分组
    pub group: G,
}

impl<T, G> Validated<T, G> {
    /// Create a new Validated wrapper / 创建新的Validated包装器
    pub fn new(value: T, group: G) -> Self {
        Self { value, group }
    }

    /// Consume and return the inner value / 消耗并返回内部值
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get reference to the inner value / 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T> Validated<T, DefaultGroup> {
    /// Create with default group / 使用默认分组创建
    pub fn with_default(value: T) -> Self {
        Self {
            value,
            group: DefaultGroup,
        }
    }
}

impl<T, G: ValidationGroup> Validated<T, G> {
    /// Get the group name / 获取分组名称
    pub fn group_name(&self) -> &'static str {
        self.group.name()
    }

    /// Get the group TypeId / 获取分组TypeId
    pub fn group_type_id(&self) -> TypeId {
        ValidationGroup::type_id(&self.group)
    }
}

impl<T: fmt::Debug, G> fmt::Debug for Validated<T, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Validated")
            .field("value", &self.value)
            .field("group", &std::any::type_name::<G>())
            .finish()
    }
}

impl<T: Clone, G: Clone> Clone for Validated<T, G> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            group: self.group.clone(),
        }
    }
}

/// Validated extractor implementation
/// Validated提取器实现
impl<T, G> Validated<T, G>
where
    T: serde::de::DeserializeOwned + crate::Validate,
    G: ValidationGroup,
{
    /// Validate with group / 使用分组验证
    pub fn validate_with_group(data: T, _group: G) -> Result<Self, ValidationError> {
        // For now, just validate using the Validate trait
        // The group-based validation will be enhanced with macro support
        // 目前，仅使用Validate trait进行验证
        // 基于分组的验证将通过宏支持增强
        data.validate()
            .map_err(|e| ValidationError::from(e))?;
        Ok(Self { value: data, group: _group })
    }
}

/// Macro to create validation groups / 创建验证分组的宏
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_validation::validation_groups;
///
/// validation_groups! {
///     CreateGroup,
///     UpdateGroup,
///     DeleteGroup,
/// }
/// ```
#[macro_export]
macro_rules! validation_groups {
    (
        $(
            $(#[$meta:meta])*
            $group:ident
        ),* $(,)?
    ) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct $group;
            impl $crate::groups::ValidationGroup for $group {}
        )*
    };
}

/// Common validation groups / 常用验证分组
pub mod common {
    use super::ValidationGroup;

    /// Create operation group / 创建操作分组
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct CreateGroup;
    impl ValidationGroup for CreateGroup {}

    /// Update operation group / 更新操作分组
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct UpdateGroup;
    impl ValidationGroup for UpdateGroup {}

    /// Delete operation group / 删除操作分组
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct DeleteGroup;
    impl ValidationGroup for DeleteGroup {}

    /// Login operation group / 登录操作分组
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct LoginGroup;
    impl ValidationGroup for LoginGroup {}

    /// Register operation group / 注册操作分组
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct RegisterGroup;
    impl ValidationGroup for RegisterGroup {}
}

// Re-export common groups
pub use common::{CreateGroup, DeleteGroup, LoginGroup, RegisterGroup, UpdateGroup};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_set() {
        let set = GroupSet::default_group();
        assert!(set.contains::<DefaultGroup>());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_group_set_with_multiple() {
        // Test with separate add calls since different group types can't be in the same vec
        let mut set = GroupSet::new();
        set.add(CreateGroup);
        set.add(UpdateGroup);
        assert!(set.contains::<CreateGroup>());
        assert!(set.contains::<UpdateGroup>());
        assert!(!set.contains::<DefaultGroup>());
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_group_names() {
        assert_eq!(DefaultGroup.name(), "DefaultGroup");
        assert_eq!(CreateGroup.name(), "CreateGroup");
        assert_eq!(UpdateGroup.name(), "UpdateGroup");
    }

    #[test]
    fn test_validated_wrapper() {
        let validated = Validated::with_default("test value");
        assert_eq!(validated.get(), &"test value");
        assert_eq!(validated.group_name(), "DefaultGroup");
        assert_eq!(validated.into_inner(), "test value");
    }

    #[test]
    fn test_validation_groups_macro() {
        // The macro should generate the group types
        // 宏应该生成分组类型
        validation_groups! {
            TestGroup1,
            TestGroup2,
        }

        // Verify they implement ValidationGroup
        // 验证它们实现了ValidationGroup
        let _set1 = GroupSet::with_group(TestGroup1);
        let _set2 = GroupSet::with_group(TestGroup2);
    }
}
