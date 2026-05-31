//! 条件注解系统 / Conditional Annotation System
//!
//! 实现类似 Spring Boot 的条件装配功能。
//! Implements conditional assembly similar to Spring Boot.
//!
//! 参考 Spring Boot 的 @Conditional* 注解：
//! - @`ConditionalOnClass` → `ConditionalOnFeature`
//! - @`ConditionalOnProperty` → `ConditionalOnProperty`
//! - @`ConditionalOnMissingBean` → `ConditionalOnMissingBean`

use std::any::TypeId;
use std::fmt::Debug;

use super::container::ApplicationContext;

// ============================================================================
// Conditional Trait / 条件 Trait
// ============================================================================

/// 条件 trait
/// Condition trait
///
/// 用于判断是否应该应用某个配置。
/// Used to determine whether a configuration should be applied.
pub trait Conditional: Send + Sync {
    /// 检查条件是否满足
    /// Check if the condition is met
    fn matches(&self, ctx: &ApplicationContext) -> bool;
}

// ============================================================================
// 内置条件实现 / Built-in Conditions
// ============================================================================

/// 属性条件
/// Property condition
///
/// 检查配置属性是否存在或有特定值。
/// Checks if a configuration property exists or has a specific value.
///
/// 等价于 Spring Boot 的 `@ConditionalOnProperty`。
/// Equivalent to Spring Boot's `@ConditionalOnProperty`.
#[derive(Debug, Clone)]
pub struct ConditionalOnProperty {
    /// 属性键
    pub key: String,

    /// 期望的值（None 表示只需存在）
    pub value: Option<String>,

    /// 是否要求值不为空
    pub match_if_empty: bool,
}

impl ConditionalOnProperty {
    /// 创建新的属性条件
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
            match_if_empty: false,
        }
    }

    /// 设置期望的值
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// 设置 `match_if_empty`
    pub fn match_if_empty(mut self, match_if_empty: bool) -> Self {
        self.match_if_empty = match_if_empty;
        self
    }
}

impl Conditional for ConditionalOnProperty {
    fn matches(&self, ctx: &ApplicationContext) -> bool {
        if let Some(actual_value) = ctx.get_property(&self.key) {
            if let Some(ref expected_value) = self.value {
                actual_value == *expected_value
            } else {
                self.match_if_empty || !actual_value.is_empty()
            }
        } else {
            false
        }
    }
}

// ============================================================================
// Bean 缺失条件 / Bean Missing Condition
// ============================================================================

/// Bean 缺失条件
/// Bean missing condition
///
/// 检查容器中是否不存在指定类型的 Bean。
/// Checks if a bean of the specified type doesn't exist in the container.
///
/// 等价于 Spring Boot 的 `@ConditionalOnMissingBean`。
/// Equivalent to Spring Boot's `@ConditionalOnMissingBean`.
#[derive(Debug, Clone)]
pub struct ConditionalOnMissingBean {
    /// Bean 类型 ID
    pub type_id: TypeId,

    /// Bean 类型名称
    pub type_name: &'static str,
}

impl ConditionalOnMissingBean {
    /// 创建新的 Bean 缺失条件
    pub fn new<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl Conditional for ConditionalOnMissingBean {
    fn matches(&self, ctx: &ApplicationContext) -> bool {
        !ctx.contains_bean_by_id(self.type_id)
    }
}

// ============================================================================
// Bean 存在条件 / Bean Present Condition
// ============================================================================

/// Bean 存在条件
/// Bean present condition
///
/// 检查容器中是否存在指定类型的 Bean。
/// Checks if a bean of the specified type exists in the container.
///
/// 等价于 Spring Boot 的 `@ConditionalOnBean`。
/// Equivalent to Spring Boot's `@ConditionalOnBean`.
#[derive(Debug, Clone)]
pub struct ConditionalOnBean {
    /// Bean 类型 ID
    pub type_id: TypeId,

    /// Bean 类型名称
    pub type_name: &'static str,
}

impl ConditionalOnBean {
    /// 创建新的 Bean 存在条件
    pub fn new<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl Conditional for ConditionalOnBean {
    fn matches(&self, ctx: &ApplicationContext) -> bool {
        ctx.contains_bean_by_id(self.type_id)
    }
}

// ============================================================================
// Feature 条件 / Feature Condition
// ============================================================================

/// Feature 条件
/// Feature condition
///
/// 检查某个 feature 是否启用。
/// Checks if a feature is enabled.
///
/// 等价于 Spring Boot 的 `@ConditionalOnClass`。
/// Equivalent to Spring Boot's `@ConditionalOnClass`.
#[derive(Debug, Clone)]
pub struct ConditionalOnFeature {
    /// Feature 名称
    pub feature: String,
}

impl ConditionalOnFeature {
    /// 创建新的 Feature 条件
    pub fn new(feature: impl Into<String>) -> Self {
        Self {
            feature: feature.into(),
        }
    }
}

impl Conditional for ConditionalOnFeature {
    fn matches(&self, _ctx: &ApplicationContext) -> bool {
        // 检查 feature 是否启用（通过环境变量）
        std::env::var("CARGO_FEATURE_")
            .or_else(|_| std::env::var(format!("NEXUS_FEATURE_{}", self.feature.to_uppercase())))
            .is_ok()
    }
}

// ============================================================================
// 表达式条件 / Expression Condition
// ============================================================================

/// 表达式条件
/// Expression condition
///
/// 支持类似 `SpEL` 的简单表达式。
/// Supports simple expressions similar to `SpEL`.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// // "hasRole('ADMIN') and !disabled"
/// // "cache.enabled == true"
/// ```
#[derive(Debug, Clone)]
pub struct ConditionalOnExpression {
    /// 表达式字符串
    pub expression: String,
}

impl ConditionalOnExpression {
    /// 创建新的表达式条件
    pub fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
        }
    }
}

impl Conditional for ConditionalOnExpression {
    fn matches(&self, ctx: &ApplicationContext) -> bool {
        // 表达式解析器 - 支持逻辑和比较运算符
        // Expression parser - supports logical and comparison operators
        let mut parser = ExpressionParser::new(&self.expression, ctx);
        parser.parse().unwrap_or(false)
    }
}

// ============================================================================
// 表达式解析器 / Expression Parser
// ============================================================================

/// 表达式解析器
/// Expression parser
///
/// 支持的语法 / Supported syntax:
/// - 逻辑运算: `&&`, `||`, `!` (以及 `and`, `or`, `not`)
/// - 比较运算: `==`, `!=`, `<`, `>`, `<=`, `>=`
/// - 分组: `(expr)`
/// - 布尔值: `true`, `false`
/// - 属性值: `key.name` (解析为字符串)
/// - 字符串: `"text"` 或 `'text'`
/// - 数字: `123`, `3.14`
struct ExpressionParser<'a> {
    input: &'a str,
    ctx: &'a ApplicationContext,
    pos: usize,
}

impl<'a> ExpressionParser<'a> {
    /// 创建新的表达式解析器
    fn new(input: &'a str, ctx: &'a ApplicationContext) -> Self {
        Self {
            input: input.trim(),
            ctx,
            pos: 0,
        }
    }

    /// 解析表达式
    fn parse(&mut self) -> Option<bool> {
        self.parse_or()
    }

    /// 解析 OR 表达式 (||, or)
    fn parse_or(&mut self) -> Option<bool> {
        let mut left = self.parse_and()?;

        loop {
            self.skip_whitespace();
            if self.consume("||") || self.consume("or") {
                let right = self.parse_and()?;
                left = left || right;
            } else {
                break;
            }
        }

        Some(left)
    }

    /// 解析 AND 表达式 (&&, and)
    fn parse_and(&mut self) -> Option<bool> {
        let mut left = self.parse_not()?;

        loop {
            self.skip_whitespace();
            if self.consume("&&") || self.consume("and") {
                let right = self.parse_not()?;
                left = left && right;
            } else {
                break;
            }
        }

        Some(left)
    }

    /// 解析 NOT 表达式 (!, not)
    fn parse_not(&mut self) -> Option<bool> {
        self.skip_whitespace();
        if self.consume("!") || self.consume("not") {
            Some(!self.parse_primary()?)
        } else {
            self.parse_primary()
        }
    }

    /// 解析基本表达式（比较、值、分组）
    fn parse_primary(&mut self) -> Option<bool> {
        self.skip_whitespace();

        // 处理括号分组
        if self.consume("(") {
            let result = self.parse_or()?;
            self.skip_whitespace();
            if !self.consume(")") {
                return None;
            }
            return Some(result);
        }

        // 读取左边的值
        let left = self.parse_value()?;
        self.skip_whitespace();

        // 检查是否有比较运算符
        if let Some(op) = self.try_consume_comparison_op() {
            let right = self.parse_value()?;
            Some(self.evaluate_comparison(&left, &op, &right))
        } else {
            // 没有比较运算符，将值转换为布尔值
            Some(self.value_to_bool(&left))
        }
    }

    /// 尝试消费比较运算符
    fn try_consume_comparison_op(&mut self) -> Option<String> {
        let ops = ["==", "!=", "<=", ">=", "<", ">"];
        for op in &ops {
            if self.consume(op) {
                return Some((*op).to_string());
            }
        }
        None
    }

    /// 评估比较运算
    fn evaluate_comparison(&self, left: &Value, op: &str, right: &Value) -> bool {
        match op {
            "==" => left.equals(right),
            "!=" => !left.equals(right),
            "<" => left.less_than(right),
            ">" => left.greater_than(right),
            "<=" => left.less_than_or_equal(right),
            ">=" => left.greater_than_or_equal(right),
            _ => false,
        }
    }

    /// 解析值
    fn parse_value(&mut self) -> Option<Value> {
        self.skip_whitespace();

        // 布尔值 true
        if self.consume("true") {
            return Some(Value::Boolean(true));
        }

        // 布尔值 false
        if self.consume("false") {
            return Some(Value::Boolean(false));
        }

        // 字符串值（双引号）
        if self.peek() == Some('"') {
            return self.parse_string('"');
        }

        // 字符串值（单引号）
        if self.peek() == Some('\'') {
            return self.parse_string('\'');
        }

        // 数字值
        if let Some(num) = self.parse_number() {
            return Some(num);
        }

        // 属性引用（标识符）
        if let Some(ident) = self.parse_identifier() {
            return Some(ctx_get_property(self.ctx, &ident));
        }

        None
    }

    /// 解析数字
    fn parse_number(&mut self) -> Option<Value> {
        let start = self.pos;
        let mut has_dot = false;

        while let Some(c) = self.peek() {
            if c.is_numeric() {
                self.pos += 1;
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.pos += 1;
            } else {
                break;
            }
        }

        if self.pos > start {
            let num_str = &self.input[start..self.pos];
            num_str.parse::<f64>()
                .ok()
                .map(Value::Number)
        } else {
            None
        }
    }

    /// 解析字符串
    fn parse_string(&mut self, quote: char) -> Option<Value> {
        self.consume(&quote.to_string());
        let start = self.pos;
        let mut escaped = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos)?;
            self.pos += 1;

            if escaped {
                escaped = false;
                continue;
            }

            if c == '\\' {
                escaped = true;
                continue;
            }

            if c == quote {
                let s = &self.input[start..self.pos - 1];
                return Some(Value::String(s.to_string()));
            }
        }

        None
    }

    /// 解析标识符（属性名）
    fn parse_identifier(&mut self) -> Option<String> {
        let start = self.pos;

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' {
                self.pos += 1;
            } else {
                break;
            }
        }

        if self.pos > start {
            Some(self.input[start..self.pos].to_string())
        } else {
            None
        }
    }

    /// 跳过空白字符
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// 查看当前字符
    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    /// 消费指定的字符串
    fn consume(&mut self, s: &str) -> bool {
        if self.input[self.pos..].starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    /// 将值转换为布尔值
    fn value_to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Number(n) => *n != 0.0,
            Value::Null => false,
        }
    }
}

/// 从上下文获取属性值
fn ctx_get_property(ctx: &ApplicationContext, key: &str) -> Value {
    ctx.get_property(key)
        .map_or(Value::Null, |v| parse_value_string(&v))
}

/// 解析值字符串
fn parse_value_string(s: &str) -> Value {
    let s = s.trim();

    // 尝试解析为布尔值
    if s.eq_ignore_ascii_case("true") {
        return Value::Boolean(true);
    }
    if s.eq_ignore_ascii_case("false") {
        return Value::Boolean(false);
    }

    // 尝试解析为数字
    if let Ok(n) = s.parse::<f64>() {
        return Value::Number(n);
    }

    // 默认为字符串
    Value::String(s.to_string())
}

/// 表达式值
#[derive(Debug, Clone)]
enum Value {
    Boolean(bool),
    String(String),
    Number(f64),
    Null,
}

impl Value {
    fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Null, Value::Null) => true,
            (Value::String(a), Value::Number(b)) => {
                a.parse::<f64>().is_ok_and(|n| (n - b).abs() < f64::EPSILON)
            }
            (Value::Number(a), Value::String(b)) => {
                b.parse::<f64>().is_ok_and(|n| (a - n).abs() < f64::EPSILON)
            }
            _ => false,
        }
    }

    fn less_than(&self, other: &Value) -> bool {
        self.compare(other).is_some_and(|v| v < 0)
    }

    fn greater_than(&self, other: &Value) -> bool {
        self.compare(other).is_some_and(|v| v > 0)
    }

    fn less_than_or_equal(&self, other: &Value) -> bool {
        self.compare(other).is_some_and(|v| v <= 0)
    }

    fn greater_than_or_equal(&self, other: &Value) -> bool {
        self.compare(other).is_some_and(|v| v >= 0)
    }

    fn compare(&self, other: &Value) -> Option<i32> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if a < b { Some(-1) }
                else if a > b { Some(1) }
                else { Some(0) }
            }
            (Value::String(a), Value::String(b)) => {
                if a < b { Some(-1) }
                else if a > b { Some(1) }
                else { Some(0) }
            }
            (Value::Boolean(a), Value::Boolean(b)) => {
                if !a && *b { Some(-1) }
                else if *a && !b { Some(1) }
                else { Some(0) }
            }
            _ => None,
        }
    }
}

// ============================================================================
// 条件组合 / Condition Composition
// ============================================================================

/// 所有条件都必须满足（AND）
/// All conditions must be met (AND)
pub struct AllConditions {
    conditions: Vec<Box<dyn Conditional>>,
}

impl AllConditions {
    /// 创建新的 AND 条件组合
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    /// 添加条件
    pub fn add(mut self, condition: Box<dyn Conditional>) -> Self {
        self.conditions.push(condition);
        self
    }
}

impl Default for AllConditions {
    fn default() -> Self {
        Self::new()
    }
}

impl Conditional for AllConditions {
    fn matches(&self, ctx: &ApplicationContext) -> bool {
        self.conditions.iter().all(|c| c.matches(ctx))
    }
}

/// 任一条件满足即可（OR）
/// Any condition can be met (OR)
pub struct AnyConditions {
    conditions: Vec<Box<dyn Conditional>>,
}

impl AnyConditions {
    /// 创建新的 OR 条件组合
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    /// 添加条件
    pub fn add(mut self, condition: Box<dyn Conditional>) -> Self {
        self.conditions.push(condition);
        self
    }
}

impl Default for AnyConditions {
    fn default() -> Self {
        Self::new()
    }
}

impl Conditional for AnyConditions {
    fn matches(&self, ctx: &ApplicationContext) -> bool {
        self.conditions.iter().any(|c| c.matches(ctx))
    }
}

/// 条件不满足（NOT）
/// Condition is not met (NOT)
pub struct NotCondition {
    condition: Box<dyn Conditional>,
}

impl NotCondition {
    /// 创建新的 NOT 条件
    pub fn new(condition: Box<dyn Conditional>) -> Self {
        Self { condition }
    }
}

impl Conditional for NotCondition {
    fn matches(&self, ctx: &ApplicationContext) -> bool {
        !self.condition.matches(ctx)
    }
}

// ============================================================================
// 辅助宏 / Helper Macros
// ============================================================================

/// 创建属性条件的宏
/// Macro to create property condition
#[macro_export]
macro_rules! cond_on_property {
    ($key:expr) => {
        $crate::core::condition::ConditionalOnProperty::new($key)
    };
    ($key:expr, $value:expr) => {
        $crate::core::condition::ConditionalOnProperty::new($key).value($value)
    };
}

/// 创建 Bean 缺失条件的宏
/// Macro to create bean missing condition
#[macro_export]
macro_rules! cond_on_missing_bean {
    ($type:ty) => {
        $crate::core::condition::ConditionalOnMissingBean::new::<$type>()
    };
}

/// 创建组合条件的宏
/// Macro to create composite conditions
#[macro_export]
macro_rules! all_conditions {
    ($($condition:expr),* $(,)?) => {{
        let mut all = $crate::core::condition::AllConditions::new();
        $(
            all = all.add(Box::new($condition));
        )*
        all
    }};
}

/// Macro to create an `AnyConditions` collection from multiple conditions
/// 从多个条件创建 `AnyConditions` 集合的宏
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_starter::core::condition::any_conditions;
///
/// let condition = any_conditions!(
///     ConditionalOnProperty::new("feature.enabled", true),
///     ConditionalOnProperty::new("fallback.enabled", true),
/// );
/// ```
#[macro_export]
macro_rules! any_conditions {
    ($($condition:expr),* $(,)?) => {{
        let mut any = $crate::core::condition::AnyConditions::new();
        $(
            any = any.add(Box::new($condition));
        )*
        any
    }};
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::config::loader::ConfigurationLoader;

    #[test]
    fn test_conditional_on_property() {
        let ctx = ApplicationContext::new();
        let cond = ConditionalOnProperty::new("test.key");

        // 属性不存在，应该返回 false
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn test_conditional_on_expression() {
        let ctx = ApplicationContext::new();
        ctx.register_bean("test_value".to_string());

        // 测试简单表达式
        let cond = ConditionalOnExpression::new("test_key");
        // 由于没有这个属性，应该返回 false
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn test_expression_comparison() {
        let mut loader = ConfigurationLoader::new();
        loader.set("port".to_string(), "8080".to_string());
        loader.set("count".to_string(), "5".to_string());
        loader.set("enabled".to_string(), "true".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));

        // 相等测试
        let cond = ConditionalOnExpression::new("port == 8080");
        assert!(cond.matches(&ctx));

        // 不等测试
        let cond = ConditionalOnExpression::new("port != 9090");
        assert!(cond.matches(&ctx));

        // 大于测试
        let cond = ConditionalOnExpression::new("count > 3");
        assert!(cond.matches(&ctx));

        // 小于测试
        let cond = ConditionalOnExpression::new("count < 10");
        assert!(cond.matches(&ctx));

        // 布尔值测试
        let cond = ConditionalOnExpression::new("enabled == true");
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn test_expression_logical() {
        let mut loader = ConfigurationLoader::new();
        loader.set("a".to_string(), "true".to_string());
        loader.set("b".to_string(), "false".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));

        // AND 测试
        let cond = ConditionalOnExpression::new("a == true && b == false");
        assert!(cond.matches(&ctx));

        let cond = ConditionalOnExpression::new("a == true && b == true");
        assert!(!cond.matches(&ctx));

        // OR 测试
        let cond = ConditionalOnExpression::new("a == true || b == true");
        assert!(cond.matches(&ctx));

        // NOT 测试
        let cond = ConditionalOnExpression::new("!b");
        assert!(cond.matches(&ctx));

        let cond = ConditionalOnExpression::new("not a");
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn test_expression_grouping() {
        let mut loader = ConfigurationLoader::new();
        loader.set("a".to_string(), "true".to_string());
        loader.set("b".to_string(), "true".to_string());
        loader.set("c".to_string(), "false".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));

        // (a && b) || c
        let cond = ConditionalOnExpression::new("(a == true && b == true) || c == true");
        assert!(cond.matches(&ctx));

        // !(a && c)
        let cond = ConditionalOnExpression::new("!(a == true && c == true)");
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn test_expression_strings() {
        let mut loader = ConfigurationLoader::new();
        loader.set("env".to_string(), "production".to_string());
        loader.set("mode".to_string(), "dev".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));

        // 字符串相等
        let cond = ConditionalOnExpression::new(r#"env == "production""#);
        assert!(cond.matches(&ctx));

        // 字符串不等
        let cond = ConditionalOnExpression::new(r#"mode != "production""#);
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn test_expression_complex() {
        let mut loader = ConfigurationLoader::new();
        loader.set("env".to_string(), "production".to_string());
        loader.set("port".to_string(), "8080".to_string());
        loader.set("enabled".to_string(), "true".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));

        // 复杂表达式: env == "production" && (port >= 8080 || !enabled)
        let cond = ConditionalOnExpression::new(r#"env == "production" && (port >= 8080 || !enabled)"#);
        assert!(cond.matches(&ctx));

        // 复杂表达式: env == "dev" || port < 8000
        let cond = ConditionalOnExpression::new(r#"env == "dev" || port < 8000"#);
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn test_expression_debug() {
        let mut loader = ConfigurationLoader::new();
        loader.set("port".to_string(), "8080".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));

        // Debug: test simple comparison
        let cond = ConditionalOnExpression::new("port == 8080");
        println!("port == 8080: {}", cond.matches(&ctx));

        let cond = ConditionalOnExpression::new("port != 9090");
        println!("port != 9090: {}", cond.matches(&ctx));

        let cond = ConditionalOnExpression::new("port != 8080");
        println!("port != 8080: {}", cond.matches(&ctx));
    }

    #[test]
    fn test_all_conditions() {
        let ctx = ApplicationContext::new();
        let cond1 = ConditionalOnProperty::new("key1");
        let cond2 = ConditionalOnProperty::new("key2");
        let all = all_conditions!(cond1, cond2);

        // 两个条件都不满足
        assert!(!all.matches(&ctx));
    }

    #[test]
    fn test_any_conditions() {
        let ctx = ApplicationContext::new();
        let cond1 = ConditionalOnProperty::new("key1");
        let cond2 = ConditionalOnProperty::new("key2");
        let any = any_conditions!(cond1, cond2);

        // 两个条件都不满足
        assert!(!any.matches(&ctx));
    }
}
