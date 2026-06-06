//! Query by Example support
//! Example 查询支持
//!
//! Equivalent to Spring Data's Query by Example (QBE).
//! Provides a way to query entities using a probe entity as a template.
//!
//! 等价于 Spring Data 的 Query by Example (QBE)。
//! 提供使用探针实体作为模板来查询实体的方法。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_orm::example::{Example, ExampleMatcher};
//!
//! let probe = User { name: "Alice".into(), age: 30, ..Default::default() };
//! let example = Example::new(&probe)
//!     .with_matcher(ExampleMatcher::new().ignore_case());
//! let users = repository.find_by_example(&example).await?;
//! ```

/// Match mode for example queries
/// Example 查询的匹配模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StringMatcher
{
    /// Exact match
    /// 精确匹配
    #[default]
    Exact,
    /// Case-insensitive match
    /// 不区分大小写匹配
    IgnoreCase,
    /// Starts with
    /// 前缀匹配
    Starting,
    /// Ends with
    /// 后缀匹配
    Ending,
    /// Contains substring
    /// 包含子串
    Containing,
}

/// Null handling mode
/// NULL 处理模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NullHandling
{
    /// Skip null values in probe
    /// 跳过探针中的 NULL 值
    Ignore,
    /// Include null values as IS NULL conditions
    /// 将 NULL 值作为 IS NULL 条件
    Include,
    /// Use default (same as Ignore)
    /// 使用默认（等同于 Ignore）
    #[default]
    Default,
}

/// Configuration for how example matching behaves
/// Example 匹配行为的配置
#[derive(Debug, Clone)]
pub struct ExampleMatcher
{
    /// String match mode
    /// 字符串匹配模式
    pub string_matcher: StringMatcher,
    /// Null handling mode
    /// NULL 处理模式
    pub null_handling: NullHandling,
    /// Whether to match all properties (AND) or any (OR)
    /// 是否匹配所有属性（AND）或任一（OR）
    pub match_all: bool,
    /// Fields to ignore
    /// 要忽略的字段
    pub ignored_fields: Vec<String>,
    /// Per-field overrides: (field_name, StringMatcher)
    /// 每字段覆盖：(字段名, StringMatcher)
    pub field_matchers: Vec<(String, StringMatcher)>,
}

impl ExampleMatcher
{
    /// Create a new matcher with default settings
    /// 创建使用默认设置的新匹配器
    pub fn new() -> Self
    {
        Self {
            string_matcher: StringMatcher::Exact,
            null_handling: NullHandling::Default,
            match_all: true,
            ignored_fields: Vec::new(),
            field_matchers: Vec::new(),
        }
    }

    /// Use case-insensitive matching
    /// 使用不区分大小写匹配
    pub fn ignore_case(mut self) -> Self
    {
        self.string_matcher = StringMatcher::IgnoreCase;
        self
    }

    /// Use substring containing match
    /// 使用包含子串匹配
    pub fn containing(mut self) -> Self
    {
        self.string_matcher = StringMatcher::Containing;
        self
    }

    /// Use starts-with match
    /// 使用前缀匹配
    pub fn starting(mut self) -> Self
    {
        self.string_matcher = StringMatcher::Starting;
        self
    }

    /// Use ends-with match
    /// 使用后缀匹配
    pub fn ending(mut self) -> Self
    {
        self.string_matcher = StringMatcher::Ending;
        self
    }

    /// Include null values in matching
    /// 在匹配中包含 NULL 值
    pub fn include_nulls(mut self) -> Self
    {
        self.null_handling = NullHandling::Include;
        self
    }

    /// Match any property (OR logic)
    /// 匹配任一属性（OR 逻辑）
    pub fn match_any(mut self) -> Self
    {
        self.match_all = false;
        self
    }

    /// Ignore a field from matching
    /// 在匹配中忽略字段
    pub fn ignore(mut self, field: impl Into<String>) -> Self
    {
        self.ignored_fields.push(field.into());
        self
    }

    /// Set per-field string matcher
    /// 设置每字段的字符串匹配器
    pub fn with_field_matcher(mut self, field: impl Into<String>, matcher: StringMatcher) -> Self
    {
        self.field_matchers.push((field.into(), matcher));
        self
    }
}

impl Default for ExampleMatcher
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// A probe-based query definition
/// 基于探针的查询定义
///
/// Wraps a probe entity and an ExampleMatcher to define
/// how to match entities against the probe.
///
/// 包装探针实体和 ExampleMatcher 来定义如何匹配实体。
#[derive(Debug, Clone)]
pub struct Example<T>
{
    /// The probe entity
    /// 探针实体
    pub probe: T,
    /// The matcher configuration
    /// 匹配器配置
    pub matcher: ExampleMatcher,
}

impl<T> Example<T>
{
    /// Create a new example from a probe entity
    /// 从探针实体创建新 example
    pub fn new(probe: T) -> Self
    {
        Self {
            probe,
            matcher: ExampleMatcher::new(),
        }
    }

    /// Set the matcher
    /// 设置匹配器
    pub fn with_matcher(mut self, matcher: ExampleMatcher) -> Self
    {
        self.matcher = matcher;
        self
    }

    /// Get the probe
    /// 获取探针
    pub fn probe(&self) -> &T
    {
        &self.probe
    }

    /// Get the matcher
    /// 获取匹配器
    pub fn matcher(&self) -> &ExampleMatcher
    {
        &self.matcher
    }
}

/// Trait for repositories that support Query by Example
/// 支持 Query by Example 的 Repository trait
pub trait QueryByExample<T: Send + Sync>: Send + Sync
{
    /// Find all entities matching the example
    /// 查找所有匹配 example 的实体
    fn find_by_example(
        &self,
        example: &Example<T>,
    ) -> impl std::future::Future<Output = Result<Vec<T>, crate::OrmError>> + Send;

    /// Count entities matching the example
    /// 计算匹配 example 的实体数
    fn count_by_example(
        &self,
        example: &Example<T>,
    ) -> impl std::future::Future<Output = Result<i64, crate::OrmError>> + Send;

    /// Check if any entity matches the example
    /// 检查是否有实体匹配 example
    fn exists_by_example(
        &self,
        example: &Example<T>,
    ) -> impl std::future::Future<Output = Result<bool, crate::OrmError>> + Send
    where
        Self: Sync,
    {
        async move { Ok(self.count_by_example(example).await? > 0) }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_example_matcher_default()
    {
        let matcher = ExampleMatcher::new();
        assert_eq!(matcher.string_matcher, StringMatcher::Exact);
        assert!(matcher.match_all);
        assert!(matcher.ignored_fields.is_empty());
    }

    #[test]
    fn test_example_matcher_builder()
    {
        let matcher = ExampleMatcher::new()
            .ignore_case()
            .ignore("id")
            .ignore("created_at")
            .containing();

        assert_eq!(matcher.string_matcher, StringMatcher::Containing);
        assert_eq!(matcher.ignored_fields.len(), 2);
    }

    #[test]
    fn test_example_matcher_field_override()
    {
        let matcher = ExampleMatcher::new()
            .with_field_matcher("name", StringMatcher::Containing)
            .with_field_matcher("email", StringMatcher::Starting);

        assert_eq!(matcher.field_matchers.len(), 2);
    }

    #[test]
    fn test_example_creation()
    {
        #[derive(Debug, Clone)]
        struct User
        {
            name: String,
        }
        let probe = User {
            name: "Alice".into(),
        };
        let example = Example::new(probe);
        assert_eq!(example.probe().name, "Alice");
        assert_eq!(example.matcher().string_matcher, StringMatcher::Exact);
    }

    #[test]
    fn test_example_with_custom_matcher()
    {
        #[derive(Debug, Clone)]
        struct User
        {
            name: String,
        }
        let probe = User {
            name: "alice".into(),
        };
        let example =
            Example::new(probe).with_matcher(ExampleMatcher::new().ignore_case().containing());
        assert_eq!(example.matcher().string_matcher, StringMatcher::Containing);
    }

    #[test]
    fn test_string_matcher_variants()
    {
        assert_ne!(StringMatcher::Exact, StringMatcher::IgnoreCase);
        assert_ne!(StringMatcher::Starting, StringMatcher::Ending);
    }

    #[test]
    fn test_null_handling_variants()
    {
        assert_eq!(NullHandling::Default, NullHandling::Default);
        assert_ne!(NullHandling::Include, NullHandling::Ignore);
    }
}
