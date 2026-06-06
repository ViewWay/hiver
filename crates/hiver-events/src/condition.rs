//! Condition expression evaluator for event filtering
//! 用于事件过滤的条件表达式求值器
//!
//! Provides a simple (non-SpEL) expression language for filtering events
//! before they reach listeners. Supports property-based comparisons with
//! logical combinators (And, Or, Not).
//!
//! 提供简单的（非SpEL）表达式语言，用于在事件到达监听器之前进行过滤。
//! 支持基于属性的比较和逻辑组合器（And、Or、Not）。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `EventCondition` | `@EventListener(condition = "...")` |
//! | `PropertyCondition` | SpEL `#event.property` |
//! | `CompareOp` | SpEL operators (`==`, `!=`, `>`, `<`) |
//! | `ConditionParser` | SpEL expression parser |
//!
//! # Supported Syntax / 支持的语法
//!
//! ```text
//! expression    := or_expr
//! or_expr       := and_expr ("or" and_expr)*
//! and_expr      := not_expr ("and" not_expr)*
//! not_expr      := "not" not_expr | primary
//! primary       := property_comparison | "(" expression ")"
//! property_comparison := path operator value
//! path          := identifier ("." identifier)*
//! operator      := "==" | "!=" | ">" | "<" | "contains"
//! value         := string_literal | number_literal
//! string_literal := "'" [^']* "'"
//! number_literal := [0-9]+ ("." [0-9]+)?
//! ```

use std::{any::Any, fmt};

// ---------------------------------------------------------------------------
// EventCondition trait
// ---------------------------------------------------------------------------

/// Trait for event condition evaluation
/// 事件条件求值的trait
///
/// Implementations decide whether a given event should be processed by a listener.
/// 实现决定给定的事件是否应该由监听器处理。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @EventListener(condition = "#event.status == 'ACTIVE'")
/// public void handleActiveEvent(MyEvent event) {
///     // Only receives events where status == 'ACTIVE'
/// }
/// ```
pub trait EventCondition: Send + Sync + Any
{
    /// Evaluate whether the event matches this condition
    /// 评估事件是否匹配此条件
    ///
    /// Returns `true` if the listener should process the event.
    /// 如果监听器应该处理该事件，则返回 `true`。
    fn matches(&self, event: &dyn Any) -> bool;
}

/// A condition that always matches (passthrough)
/// 始终匹配的条件（直通）
#[derive(Debug, Clone, Copy, Default)]
pub struct AlwaysMatchCondition;

impl EventCondition for AlwaysMatchCondition
{
    fn matches(&self, _event: &dyn Any) -> bool
    {
        true
    }
}

/// A condition that never matches
/// 从不匹配的条件
#[derive(Debug, Clone, Copy, Default)]
pub struct NeverMatchCondition;

impl EventCondition for NeverMatchCondition
{
    fn matches(&self, _event: &dyn Any) -> bool
    {
        false
    }
}

// ---------------------------------------------------------------------------
// CompareOp
// ---------------------------------------------------------------------------

/// Comparison operators for property conditions
/// 属性条件的比较运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp
{
    /// Equal to / 等于
    Eq,
    /// Not equal to / 不等于
    Ne,
    /// Greater than / 大于
    Gt,
    /// Less than / 小于
    Lt,
    /// String contains / 字符串包含
    Contains,
}

impl CompareOp
{
    /// Try to parse a comparison operator from a token string
    /// 尝试从标记字符串解析比较运算符
    pub fn from_token(token: &str) -> Option<Self>
    {
        match token.trim()
        {
            "==" => Some(Self::Eq),
            "!=" => Some(Self::Ne),
            ">" => Some(Self::Gt),
            "<" => Some(Self::Lt),
            "contains" => Some(Self::Contains),
            _ => None,
        }
    }

    /// Get the symbolic representation of this operator
    /// 获取此运算符的符号表示
    pub fn as_symbol(&self) -> &'static str
    {
        match self
        {
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Gt => ">",
            Self::Lt => "<",
            Self::Contains => "contains",
        }
    }
}

impl fmt::Display for CompareOp
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.write_str(self.as_symbol())
    }
}

// ---------------------------------------------------------------------------
// PropertyCondition
// ---------------------------------------------------------------------------

/// A condition that compares a named property on the event to a literal value
/// 比较事件上命名属性与字面值的条件
///
/// Uses `std::any::Any` downcasting for property access. For full property-path
/// resolution, events should implement the `ConditionPropertyProvider` trait.
///
/// 使用 `std::any::Any` 向下转换进行属性访问。完整的属性路径解析需要事件实现
/// `ConditionPropertyProvider` trait。
///
/// # Limitations / 限制
///
/// This condition requires the event to implement `ConditionPropertyProvider`
/// to access properties by path. Without it, only top-level type matching works.
///
/// 此条件需要事件实现 `ConditionPropertyProvider` 才能按路径访问属性。
/// 没有该实现，只能进行顶层类型匹配。
#[derive(Debug, Clone)]
pub struct PropertyCondition
{
    /// The property path (e.g., "status", "user.name")
    /// 属性路径（例如 "status"、"user.name"）
    pub property_path: String,

    /// The comparison operator
    /// 比较运算符
    pub operator: CompareOp,

    /// The expected value as a string
    /// 预期值（字符串形式）
    pub value: String,
}

impl PropertyCondition
{
    /// Create a new property condition
    /// 创建新的属性条件
    pub fn new(
        property_path: impl Into<String>,
        operator: CompareOp,
        value: impl Into<String>,
    ) -> Self
    {
        Self {
            property_path: property_path.into(),
            operator,
            value: value.into(),
        }
    }

    /// Compare two string values using the configured operator
    /// 使用配置的运算符比较两个字符串值
    ///
    /// For `Gt` and `Lt`, attempts numeric comparison first (if both values
    /// parse as `f64`), falling back to lexicographic string comparison.
    /// 对于 `Gt` 和 `Lt`，先尝试数值比较（如果两个值都能解析为 `f64`），
    /// 然后回退到字典序字符串比较。
    fn compare_strings(&self, actual: &str) -> bool
    {
        match self.operator
        {
            CompareOp::Eq => actual == self.value,
            CompareOp::Ne => actual != self.value,
            CompareOp::Gt =>
            {
                // Try numeric comparison first
                if let (Ok(a_num), Ok(b_num)) = (actual.parse::<f64>(), self.value.parse::<f64>())
                {
                    a_num > b_num
                }
                else
                {
                    actual > self.value.as_str()
                }
            },
            CompareOp::Lt =>
            {
                // Try numeric comparison first
                if let (Ok(a_num), Ok(b_num)) = (actual.parse::<f64>(), self.value.parse::<f64>())
                {
                    a_num < b_num
                }
                else
                {
                    actual < self.value.as_str()
                }
            },
            CompareOp::Contains => actual.contains(&self.value),
        }
    }
}

impl EventCondition for PropertyCondition
{
    fn matches(&self, _event: &dyn Any) -> bool
    {
        // We cannot directly downcast &dyn Any to dyn ConditionPropertyProvider.
        // Use `evaluate_condition` for typed property-based evaluation.
        // For untyped (dyn Any) matching, this always returns false.
        false
    }
}

impl PropertyCondition
{
    /// Evaluate this condition against a typed event that implements
    /// `ConditionPropertyProvider`.
    /// 对实现了 `ConditionPropertyProvider` 的类型化事件求值此条件。
    ///
    /// This is the primary evaluation path for property conditions.
    /// 这是属性条件的主要求值路径。
    pub fn matches_provider<E: ConditionPropertyProvider>(&self, event: &E) -> bool
    {
        if let Some(property_value) = event.get_property(&self.property_path)
        {
            return self.compare_strings(&property_value);
        }
        false
    }
}

/// Evaluate a condition against a typed event that implements
/// `ConditionPropertyProvider`.
/// 对实现了 `ConditionPropertyProvider` 的类型化事件求值条件。
///
/// For `PropertyCondition` and `CompositeCondition` containing property
/// conditions, this delegates to `matches_provider`. For other condition
/// types, it falls back to `EventCondition::matches`.
///
/// 对于 `PropertyCondition` 和包含属性条件的 `CompositeCondition`，
/// 此函数委托给 `matches_provider`。对于其他条件类型，回退到
/// `EventCondition::matches`。
///
/// # Examples / 示例
///
/// ```rust,ignore
/// let condition = ConditionParser::parse("status == 'active'").unwrap();
/// let event = MyEvent { status: "active".to_string() };
/// assert!(evaluate_condition(&*condition, &event));
/// ```
pub fn evaluate_condition<E: ConditionPropertyProvider + Any>(
    condition: &dyn EventCondition,
    event: &E,
) -> bool
{
    // Try PropertyCondition first (via Any downcast)
    if let Some(pc) = (condition as &dyn Any).downcast_ref::<PropertyCondition>()
    {
        return pc.matches_provider(event);
    }

    // Try CompositeCondition
    if let Some(cc) = (condition as &dyn Any).downcast_ref::<CompositeCondition>()
    {
        return match cc
        {
            CompositeCondition::And(conditions) => conditions
                .iter()
                .all(|c| evaluate_condition(c.as_ref(), event)),
            CompositeCondition::Or(conditions) => conditions
                .iter()
                .any(|c| evaluate_condition(c.as_ref(), event)),
            CompositeCondition::Not(inner) => !evaluate_condition(inner.as_ref(), event),
        };
    }

    // Fallback to untyped match
    condition.matches(event as &dyn Any)
}

// ---------------------------------------------------------------------------
// ConditionPropertyProvider
// ---------------------------------------------------------------------------

/// Trait for events that can provide property values by path
/// 可以按路径提供属性值的事件trait
///
/// Events implementing this trait can be filtered using `PropertyCondition`.
/// The property path uses dot notation (e.g., "user.name").
///
/// 实现此trait的事件可以使用 `PropertyCondition` 进行过滤。
/// 属性路径使用点号表示法（例如 "user.name"）。
pub trait ConditionPropertyProvider
{
    /// Get a property value by its path
    /// 按路径获取属性值
    ///
    /// Returns `Some(value)` if the property exists, `None` otherwise.
    /// 如果属性存在则返回 `Some(value)`，否则返回 `None`。
    fn get_property(&self, path: &str) -> Option<String>;
}

// ---------------------------------------------------------------------------
// CompositeCondition
// ---------------------------------------------------------------------------

/// Composite condition combining multiple conditions with logical operators
/// 使用逻辑运算符组合多个条件的组合条件
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @EventListener(condition = "#event.status == 'ACTIVE' and #event.priority > 5")
/// ```
pub enum CompositeCondition
{
    /// All conditions must match / 所有条件必须匹配
    And(Vec<Box<dyn EventCondition>>),

    /// At least one condition must match / 至少一个条件必须匹配
    Or(Vec<Box<dyn EventCondition>>),

    /// The inner condition must NOT match / 内部条件不得匹配
    Not(Box<dyn EventCondition>),
}

impl fmt::Debug for CompositeCondition
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::And(conditions) =>
            {
                write!(f, "CompositeCondition::And({} conditions)", conditions.len())
            },
            Self::Or(conditions) =>
            {
                write!(f, "CompositeCondition::Or({} conditions)", conditions.len())
            },
            Self::Not(_) => write!(f, "CompositeCondition::Not(..)"),
        }
    }
}

impl EventCondition for CompositeCondition
{
    fn matches(&self, event: &dyn Any) -> bool
    {
        match self
        {
            Self::And(conditions) => conditions.iter().all(|c| c.matches(event)),
            Self::Or(conditions) => conditions.iter().any(|c| c.matches(event)),
            Self::Not(condition) => !condition.matches(event),
        }
    }
}

// ---------------------------------------------------------------------------
// ConditionParser
// ---------------------------------------------------------------------------

/// Simple condition expression parser
/// 简单条件表达式解析器
///
/// Parses expressions like:
///
/// 解析以下表达式：
///
/// - `status == 'active'`
/// - `priority > 5`
/// - `name contains 'test'`
/// - `status == 'active' and priority > 5`
/// - `status == 'active' or status == 'pending'`
/// - `not status == 'deleted'`
///
/// # Grammar / 语法
///
/// ```text
/// expression    := or_expr
/// or_expr       := and_expr ("or" and_expr)*
/// and_expr      := not_expr ("and" not_expr)*
/// not_expr      := "not" not_expr | primary
/// primary       := property_comparison | "(" expression ")"
/// ```
pub struct ConditionParser;

impl ConditionParser
{
    /// Parse a condition expression string into an `EventCondition`
    /// 将条件表达式字符串解析为 `EventCondition`
    ///
    /// # Errors / 错误
    ///
    /// Returns `Err` if the expression is syntactically invalid.
    /// 如果表达式语法无效，则返回 `Err`。
    ///
    /// # Examples / 示例
    ///
    /// ```
    /// use std::any::Any;
    ///
    /// use hiver_events::condition::{ConditionParser, EventCondition};
    ///
    /// let condition = ConditionParser::parse("status == 'active'").unwrap();
    /// // PropertyCondition returns false for untyped dyn Any matching;
    /// // use evaluate_condition() for typed property-based evaluation.
    /// assert_eq!(condition.matches(&42 as &dyn Any), false);
    /// ```
    pub fn parse(input: &str) -> Result<Box<dyn EventCondition>, ConditionParseError>
    {
        let tokens = tokenize(input)?;
        let mut parser = ParserState::new(&tokens);
        let condition = parser.parse_or_expr()?;
        if parser.has_more()
        {
            return Err(ConditionParseError::UnexpectedToken(
                parser.peek().unwrap_or("").to_string(),
            ));
        }
        Ok(condition)
    }
}

/// Error type for condition parsing failures
/// 条件解析失败的错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionParseError
{
    /// Unexpected token encountered
    /// 遇到意外的标记
    UnexpectedToken(String),

    /// Expected a specific token type
    /// 期望特定标记类型
    ExpectedToken(String),

    /// Empty expression
    /// 空表达式
    EmptyExpression,

    /// Unclosed parenthesis
    /// 未闭合的括号
    UnclosedParenthesis,

    /// Unexpected end of expression
    /// 表达式意外结束
    UnexpectedEnd,
}

impl fmt::Display for ConditionParseError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::UnexpectedToken(token) => write!(f, "Unexpected token: '{}'", token),
            Self::ExpectedToken(expected) => write!(f, "Expected: '{}'", expected),
            Self::EmptyExpression => write!(f, "Empty expression"),
            Self::UnclosedParenthesis => write!(f, "Unclosed parenthesis"),
            Self::UnexpectedEnd => write!(f, "Unexpected end of expression"),
        }
    }
}

impl std::error::Error for ConditionParseError {}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

/// Tokenize an expression string into a list of string tokens
/// 将表达式字符串标记化为字符串标记列表
fn tokenize(input: &str) -> Result<Vec<String>, ConditionParseError>
{
    let trimmed = input.trim();
    if trimmed.is_empty()
    {
        return Err(ConditionParseError::EmptyExpression);
    }

    let mut tokens = Vec::new();
    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 0;

    while i < chars.len()
    {
        let ch = chars[i];

        // Skip whitespace
        // 跳过空白字符
        if ch.is_whitespace()
        {
            i += 1;
            continue;
        }

        // Parentheses
        // 括号
        if ch == '(' || ch == ')'
        {
            tokens.push(ch.to_string());
            i += 1;
            continue;
        }

        // Comparison operators: ==, !=, >, <
        // 比较运算符：==、!=、>、<
        if ch == '=' && i + 1 < chars.len() && chars[i + 1] == '='
        {
            tokens.push("==".to_string());
            i += 2;
            continue;
        }
        if ch == '!' && i + 1 < chars.len() && chars[i + 1] == '='
        {
            tokens.push("!=".to_string());
            i += 2;
            continue;
        }
        if ch == '>'
        {
            tokens.push(">".to_string());
            i += 1;
            continue;
        }
        if ch == '<'
        {
            tokens.push("<".to_string());
            i += 1;
            continue;
        }

        // String literal: 'value'
        // 字符串字面量：'value'
        if ch == '\''
        {
            let start = i + 1;
            i += 1;
            while i < chars.len() && chars[i] != '\''
            {
                i += 1;
            }
            if i >= chars.len()
            {
                return Err(ConditionParseError::UnclosedParenthesis);
            }
            let value: String = chars[start..i].iter().collect();
            tokens.push(format!("'{}'", value));
            i += 1; // skip closing quote
            continue;
        }

        // Number or identifier
        // 数字或标识符
        if ch.is_alphanumeric() || ch == '_' || ch == '.'
        {
            let start = i;
            while i < chars.len()
                && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.')
            {
                i += 1;
            }
            let token: String = chars[start..i].iter().collect();
            tokens.push(token);
            continue;
        }

        // Unknown character
        // 未知字符
        return Err(ConditionParseError::UnexpectedToken(ch.to_string()));
    }

    Ok(tokens)
}

// ---------------------------------------------------------------------------
// Recursive descent parser state
// ---------------------------------------------------------------------------

/// Internal parser state for recursive descent parsing
/// 用于递归下降解析的内部解析器状态
struct ParserState<'a>
{
    tokens: &'a [String],
    position: usize,
}

impl<'a> ParserState<'a>
{
    fn new(tokens: &'a [String]) -> Self
    {
        Self {
            tokens,
            position: 0,
        }
    }

    fn has_more(&self) -> bool
    {
        self.position < self.tokens.len()
    }

    fn peek(&self) -> Option<&'a str>
    {
        self.tokens.get(self.position).map(String::as_str)
    }

    fn advance(&mut self) -> Option<&'a str>
    {
        let token = self.tokens.get(self.position).map(String::as_str);
        self.position += 1;
        token
    }

    fn expect(&mut self, expected: &str) -> Result<(), ConditionParseError>
    {
        match self.advance()
        {
            Some(token) if token == expected => Ok(()),
            Some(token) =>
            {
                Err(ConditionParseError::ExpectedToken(format!("'{}', got '{}'", expected, token)))
            },
            None => Err(ConditionParseError::UnexpectedEnd),
        }
    }

    /// or_expr := and_expr ("or" and_expr)*
    fn parse_or_expr(&mut self) -> Result<Box<dyn EventCondition>, ConditionParseError>
    {
        let mut conditions = vec![self.parse_and_expr()?];

        while self.peek() == Some("or")
        {
            self.advance(); // consume "or"
            conditions.push(self.parse_and_expr()?);
        }

        if conditions.len() == 1
        {
            Ok(conditions.pop().unwrap())
        }
        else
        {
            Ok(Box::new(CompositeCondition::Or(conditions)))
        }
    }

    /// and_expr := not_expr ("and" not_expr)*
    fn parse_and_expr(&mut self) -> Result<Box<dyn EventCondition>, ConditionParseError>
    {
        let mut conditions = vec![self.parse_not_expr()?];

        while self.peek() == Some("and")
        {
            self.advance(); // consume "and"
            conditions.push(self.parse_not_expr()?);
        }

        if conditions.len() == 1
        {
            Ok(conditions.pop().unwrap())
        }
        else
        {
            Ok(Box::new(CompositeCondition::And(conditions)))
        }
    }

    /// not_expr := "not" not_expr | primary
    fn parse_not_expr(&mut self) -> Result<Box<dyn EventCondition>, ConditionParseError>
    {
        if self.peek() == Some("not")
        {
            self.advance(); // consume "not"
            let inner = self.parse_not_expr()?;
            Ok(Box::new(CompositeCondition::Not(inner)))
        }
        else
        {
            self.parse_primary()
        }
    }

    /// primary := property_comparison | "(" expression ")"
    fn parse_primary(&mut self) -> Result<Box<dyn EventCondition>, ConditionParseError>
    {
        if self.peek() == Some("(")
        {
            self.advance(); // consume "("
            let expr = self.parse_or_expr()?;
            self.expect(")")?;
            return Ok(expr);
        }

        // property_comparison := path operator value
        let path = self
            .advance()
            .ok_or(ConditionParseError::UnexpectedEnd)?
            .to_string();

        let op_token = self.advance().ok_or(ConditionParseError::UnexpectedEnd)?;

        let operator = CompareOp::from_token(op_token).ok_or_else(|| {
            ConditionParseError::ExpectedToken(format!("comparison operator, got '{}'", op_token))
        })?;

        let value = self
            .advance()
            .ok_or(ConditionParseError::UnexpectedEnd)?
            .to_string();

        // Strip surrounding quotes from string literals
        // 去除字符串字面量的外围引号
        let clean_value = if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2
        {
            value[1..value.len() - 1].to_string()
        }
        else
        {
            value
        };

        Ok(Box::new(PropertyCondition::new(path, operator, clean_value)))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests
{
    use super::*;

    // --- Test event that implements ConditionPropertyProvider ---

    #[derive(Debug, Clone)]
    struct UserCreatedEvent
    {
        username: String,
        status: String,
        priority: i32,
    }

    impl UserCreatedEvent
    {
        fn new(username: &str, status: &str, priority: i32) -> Self
        {
            Self {
                username: username.to_string(),
                status: status.to_string(),
                priority,
            }
        }
    }

    impl ConditionPropertyProvider for UserCreatedEvent
    {
        fn get_property(&self, path: &str) -> Option<String>
        {
            match path
            {
                "username" => Some(self.username.clone()),
                "status" => Some(self.status.clone()),
                "priority" => Some(self.priority.to_string()),
                _ => None,
            }
        }
    }

    // --- AlwaysMatch / NeverMatch Tests ---

    #[test]
    fn test_always_match()
    {
        let condition = AlwaysMatchCondition;
        assert!(condition.matches(&42 as &dyn Any));
    }

    #[test]
    fn test_never_match()
    {
        let condition = NeverMatchCondition;
        assert!(!condition.matches(&42 as &dyn Any));
    }

    // --- CompareOp Tests ---

    #[test]
    fn test_compare_op_from_token()
    {
        assert_eq!(CompareOp::from_token("=="), Some(CompareOp::Eq));
        assert_eq!(CompareOp::from_token("!="), Some(CompareOp::Ne));
        assert_eq!(CompareOp::from_token(">"), Some(CompareOp::Gt));
        assert_eq!(CompareOp::from_token("<"), Some(CompareOp::Lt));
        assert_eq!(CompareOp::from_token("contains"), Some(CompareOp::Contains));
        assert_eq!(CompareOp::from_token("invalid"), None);
    }

    #[test]
    fn test_compare_op_display()
    {
        assert_eq!(CompareOp::Eq.to_string(), "==");
        assert_eq!(CompareOp::Contains.to_string(), "contains");
    }

    // --- PropertyCondition Tests ---

    #[test]
    fn test_property_condition_eq_match()
    {
        let event = UserCreatedEvent::new("alice", "active", 5);
        let condition = PropertyCondition::new("status", CompareOp::Eq, "active");
        assert!(condition.matches_provider(&event));
    }

    #[test]
    fn test_property_condition_eq_no_match()
    {
        let event = UserCreatedEvent::new("alice", "inactive", 5);
        let condition = PropertyCondition::new("status", CompareOp::Eq, "active");
        assert!(!condition.matches_provider(&event));
    }

    #[test]
    fn test_property_condition_ne()
    {
        let event = UserCreatedEvent::new("bob", "pending", 3);
        let condition = PropertyCondition::new("status", CompareOp::Ne, "deleted");
        assert!(condition.matches_provider(&event));
    }

    #[test]
    fn test_property_condition_gt()
    {
        let event = UserCreatedEvent::new("carol", "active", 10);
        let condition = PropertyCondition::new("priority", CompareOp::Gt, "5");
        assert!(condition.matches_provider(&event));
    }

    #[test]
    fn test_property_condition_lt()
    {
        let event = UserCreatedEvent::new("dave", "active", 3);
        let condition = PropertyCondition::new("priority", CompareOp::Lt, "5");
        assert!(condition.matches_provider(&event));
    }

    #[test]
    fn test_property_condition_contains()
    {
        let event = UserCreatedEvent::new("admin_user", "active", 1);
        let condition = PropertyCondition::new("username", CompareOp::Contains, "admin");
        assert!(condition.matches_provider(&event));
    }

    #[test]
    fn test_property_condition_unknown_property()
    {
        let event = UserCreatedEvent::new("eve", "active", 1);
        let condition = PropertyCondition::new("unknown_field", CompareOp::Eq, "value");
        assert!(!condition.matches_provider(&event));
    }

    #[test]
    fn test_property_condition_no_provider()
    {
        let condition = PropertyCondition::new("status", CompareOp::Eq, "active");
        // A plain i32 does not implement ConditionPropertyProvider,
        // so matches() on dyn Any returns false
        assert!(!condition.matches(&42 as &dyn Any));
    }

    // --- CompositeCondition Tests ---

    #[test]
    fn test_composite_and_both_match()
    {
        let event = UserCreatedEvent::new("alice", "active", 10);
        let condition = CompositeCondition::And(vec![
            Box::new(PropertyCondition::new("status", CompareOp::Eq, "active")),
            Box::new(PropertyCondition::new("priority", CompareOp::Gt, "5")),
        ]);
        assert!(evaluate_condition(&condition, &event));
    }

    #[test]
    fn test_composite_and_one_fails()
    {
        let event = UserCreatedEvent::new("bob", "inactive", 10);
        let condition = CompositeCondition::And(vec![
            Box::new(PropertyCondition::new("status", CompareOp::Eq, "active")),
            Box::new(PropertyCondition::new("priority", CompareOp::Gt, "5")),
        ]);
        assert!(!evaluate_condition(&condition, &event));
    }

    #[test]
    fn test_composite_or_first_matches()
    {
        let event = UserCreatedEvent::new("carol", "active", 1);
        let condition = CompositeCondition::Or(vec![
            Box::new(PropertyCondition::new("status", CompareOp::Eq, "active")),
            Box::new(PropertyCondition::new("priority", CompareOp::Gt, "100")),
        ]);
        assert!(evaluate_condition(&condition, &event));
    }

    #[test]
    fn test_composite_or_second_matches()
    {
        let event = UserCreatedEvent::new("dave", "inactive", 200);
        let condition = CompositeCondition::Or(vec![
            Box::new(PropertyCondition::new("status", CompareOp::Eq, "active")),
            Box::new(PropertyCondition::new("priority", CompareOp::Gt, "100")),
        ]);
        assert!(evaluate_condition(&condition, &event));
    }

    #[test]
    fn test_composite_or_none_match()
    {
        let event = UserCreatedEvent::new("eve", "inactive", 1);
        let condition = CompositeCondition::Or(vec![
            Box::new(PropertyCondition::new("status", CompareOp::Eq, "active")),
            Box::new(PropertyCondition::new("priority", CompareOp::Gt, "100")),
        ]);
        assert!(!evaluate_condition(&condition, &event));
    }

    #[test]
    fn test_composite_not()
    {
        let event = UserCreatedEvent::new("mallory", "deleted", 1);
        let condition = CompositeCondition::Not(Box::new(PropertyCondition::new(
            "status",
            CompareOp::Eq,
            "deleted",
        )));
        assert!(!evaluate_condition(&condition, &event));
    }

    #[test]
    fn test_composite_not_inverted()
    {
        let event = UserCreatedEvent::new("mallory", "active", 1);
        let condition = CompositeCondition::Not(Box::new(PropertyCondition::new(
            "status",
            CompareOp::Eq,
            "deleted",
        )));
        assert!(evaluate_condition(&condition, &event));
    }

    // --- Tokenizer Tests ---

    #[test]
    fn test_tokenize_simple()
    {
        let tokens = tokenize("status == 'active'").unwrap();
        assert_eq!(tokens, vec!["status", "==", "'active'"]);
    }

    #[test]
    fn test_tokenize_number()
    {
        let tokens = tokenize("priority > 5").unwrap();
        assert_eq!(tokens, vec!["priority", ">", "5"]);
    }

    #[test]
    fn test_tokenize_contains()
    {
        let tokens = tokenize("name contains 'test'").unwrap();
        assert_eq!(tokens, vec!["name", "contains", "'test'"]);
    }

    #[test]
    fn test_tokenize_and_or()
    {
        let tokens = tokenize("status == 'active' and priority > 5").unwrap();
        assert_eq!(tokens, vec!["status", "==", "'active'", "and", "priority", ">", "5"]);
    }

    #[test]
    fn test_tokenize_not()
    {
        let tokens = tokenize("not status == 'deleted'").unwrap();
        assert_eq!(tokens, vec!["not", "status", "==", "'deleted'"]);
    }

    #[test]
    fn test_tokenize_parentheses()
    {
        let tokens = tokenize("( status == 'active' )").unwrap();
        assert_eq!(tokens, vec!["(", "status", "==", "'active'", ")"]);
    }

    #[test]
    fn test_tokenize_ne()
    {
        let tokens = tokenize("status != 'deleted'").unwrap();
        assert_eq!(tokens, vec!["status", "!=", "'deleted'"]);
    }

    #[test]
    fn test_tokenize_empty()
    {
        assert!(tokenize("").is_err());
        assert!(tokenize("   ").is_err());
    }

    // --- ConditionParser Tests ---

    #[test]
    fn test_parser_simple_eq()
    {
        let condition = ConditionParser::parse("status == 'active'").unwrap();
        let event = UserCreatedEvent::new("alice", "active", 5);
        assert!(evaluate_condition(condition.as_ref(), &event));
    }

    #[test]
    fn test_parser_simple_gt()
    {
        let condition = ConditionParser::parse("priority > 5").unwrap();
        let event = UserCreatedEvent::new("bob", "active", 10);
        assert!(evaluate_condition(condition.as_ref(), &event));

        let event2 = UserCreatedEvent::new("carol", "active", 3);
        assert!(!evaluate_condition(condition.as_ref(), &event2));
    }

    #[test]
    fn test_parser_simple_lt()
    {
        let condition = ConditionParser::parse("priority < 5").unwrap();
        let event = UserCreatedEvent::new("dave", "active", 3);
        assert!(evaluate_condition(condition.as_ref(), &event));
    }

    #[test]
    fn test_parser_contains()
    {
        let condition = ConditionParser::parse("username contains 'admin'").unwrap();
        let event = UserCreatedEvent::new("admin_user", "active", 1);
        assert!(evaluate_condition(condition.as_ref(), &event));
    }

    #[test]
    fn test_parser_ne()
    {
        let condition = ConditionParser::parse("status != 'deleted'").unwrap();
        let event = UserCreatedEvent::new("eve", "active", 1);
        assert!(evaluate_condition(condition.as_ref(), &event));

        let event2 = UserCreatedEvent::new("eve", "deleted", 1);
        assert!(!evaluate_condition(condition.as_ref(), &event2));
    }

    #[test]
    fn test_parser_and()
    {
        let condition = ConditionParser::parse("status == 'active' and priority > 5").unwrap();
        let event = UserCreatedEvent::new("alice", "active", 10);
        assert!(evaluate_condition(condition.as_ref(), &event));

        let event2 = UserCreatedEvent::new("bob", "inactive", 10);
        assert!(!evaluate_condition(condition.as_ref(), &event2));
    }

    #[test]
    fn test_parser_or()
    {
        let condition =
            ConditionParser::parse("status == 'active' or status == 'pending'").unwrap();
        let event = UserCreatedEvent::new("alice", "active", 1);
        assert!(evaluate_condition(condition.as_ref(), &event));

        let event2 = UserCreatedEvent::new("bob", "pending", 1);
        assert!(evaluate_condition(condition.as_ref(), &event2));

        let event3 = UserCreatedEvent::new("carol", "deleted", 1);
        assert!(!evaluate_condition(condition.as_ref(), &event3));
    }

    #[test]
    fn test_parser_not()
    {
        let condition = ConditionParser::parse("not status == 'deleted'").unwrap();
        let event = UserCreatedEvent::new("alice", "active", 1);
        assert!(evaluate_condition(condition.as_ref(), &event));

        let event2 = UserCreatedEvent::new("bob", "deleted", 1);
        assert!(!evaluate_condition(condition.as_ref(), &event2));
    }

    #[test]
    fn test_parser_parenthesized()
    {
        let condition = ConditionParser::parse("( status == 'active' )").unwrap();
        let event = UserCreatedEvent::new("alice", "active", 1);
        assert!(evaluate_condition(condition.as_ref(), &event));
    }

    #[test]
    fn test_parser_complex()
    {
        let condition = ConditionParser::parse(
            "status == 'active' and ( priority > 5 or username contains 'admin' )",
        )
        .unwrap();

        // Matches: active + high priority
        let event1 = UserCreatedEvent::new("user1", "active", 10);
        assert!(evaluate_condition(condition.as_ref(), &event1));

        // Matches: active + admin username
        let event2 = UserCreatedEvent::new("admin_user", "active", 1);
        assert!(evaluate_condition(condition.as_ref(), &event2));

        // Does not match: active but low priority and no admin
        let event3 = UserCreatedEvent::new("regular", "active", 1);
        assert!(!evaluate_condition(condition.as_ref(), &event3));

        // Does not match: inactive
        let event4 = UserCreatedEvent::new("admin_user", "inactive", 100);
        assert!(!evaluate_condition(condition.as_ref(), &event4));
    }

    #[test]
    fn test_parser_error_empty()
    {
        assert!(ConditionParser::parse("").is_err());
        assert!(ConditionParser::parse("   ").is_err());
    }

    #[test]
    fn test_parser_error_unexpected_token()
    {
        assert!(ConditionParser::parse("status active").is_err());
    }

    #[test]
    fn test_parser_error_unclosed_paren()
    {
        assert!(ConditionParser::parse("( status == 'active'").is_err());
    }
}
