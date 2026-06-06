//! Validation annotations / 验证注解
//!
//! Field-level validation attributes equivalent to Spring's validation annotations.
//! 等价于Spring验证注解的属性级验证。
//!
//! # Available Annotations / 可用注解
//!
//! - `#[NotNull]` - Field cannot be null / 字段不能为空
//! - `#[NotNull(group = "CreateGroup")]` - Validation with groups / 分组验证
//! - `#[NotEmpty]` - Collection/string cannot be empty / 集合/字符串不能为空
//! - `#[NotBlank]` - String cannot be blank (only whitespace) / 字符串不能为空白
//! - `#[Size(min = 3, max = 20)]` - Size bounds / 大小边界
//! - `#[Min(18)]` - Minimum value / 最小值
//! - `#[Max(120)]` - Maximum value / 最大值
//! - `#[Email]` - Email validation / 邮箱验证
//! - `#[Pattern(regex = "^[a-z]+$")]` - Regex pattern / 正则表达式
//! - `#[Negative]` - Must be negative / 必须为负数
//! - `[Positive]` - Must be positive / 必须为正数
//! - `#[Past]` - Date must be in the past / 日期必须在过去
//! - `#[Future]` - Date must be in the future / 日期必须在未来
//! - `#[Url]` - URL validation / URL验证
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_validation::{NotNull, Size, Email, Min, Max};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CreateUserRequest {
//!     #[NotNull]
//!     #[Size(min = 3, max = 20)]
//!     username: String,
//!
//!     #[Email]
//!     email: String,
//!
//!     #[Min(18)]
//!     #[Max(120)]
//!     age: u32,
//! }
//! ```

use std::{collections::HashSet, fmt};

use crate::ValidationError;

/// Validation group marker / 验证分组标记
///
/// Groups allow selective validation based on context.
/// 分组允许根据上下文选择性验证。
///
/// # Spring Equivalent / Spring等价物
///
/// The following Java code shows the equivalent Spring validation groups:
/// `以下Java代码显示了等效的Spring验证分组`:
///
/// ```java
/// public interface CreateGroup {}
/// public interface UpdateGroup {}
///
/// @NotNull(groups = CreateGroup.class)
/// private String username;
/// ```
pub trait ValidationGroup: std::any::Any + Send + Sync
{
    /// Get the group name / 获取分组名称
    fn group_name() -> &'static str
    where
        Self: Sized;
}

/// Default validation group / 默认验证分组
///
/// Used when no group is specified.
/// 当没有指定分组时使用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultGroup;

impl ValidationGroup for DefaultGroup
{
    fn group_name() -> &'static str
    {
        "Default"
    }
}

/// Validation metadata for a field / 字段验证元数据
#[derive(Clone)]
pub struct FieldValidation
{
    /// Field name / 字段名
    pub field: String,
    /// Validation rules / 验证规则
    pub rules: Vec<ValidationRule>,
    /// Active groups / 活跃分组
    pub groups: HashSet<String>,
}

impl FieldValidation
{
    /// Create new field validation / 创建新的字段验证
    pub fn new(field: impl Into<String>) -> Self
    {
        Self {
            field: field.into(),
            rules: Vec::new(),
            groups: HashSet::new(),
        }
    }

    /// Add validation rule / 添加验证规则
    pub fn with_rule(mut self, rule: ValidationRule) -> Self
    {
        self.rules.push(rule);
        self
    }

    /// Add group / 添加分组
    pub fn with_group(mut self, group: impl Into<String>) -> Self
    {
        self.groups.insert(group.into());
        self
    }

    /// Validate a value / 验证值
    pub fn validate(
        &self,
        value: &str,
        active_groups: &HashSet<String>,
    ) -> Result<(), ValidationError>
    {
        // Check if this field should be validated based on groups
        // 检查是否应根据分组验证此字段
        if !self.groups.is_empty() && !active_groups.is_empty()
        {
            // Field has group restrictions - check if any active group matches
            // 字段有分组限制 - 检查是否有活跃分组匹配
            let should_validate = self.groups.intersection(active_groups).next().is_some();
            if !should_validate
            {
                return Ok(());
            }
        }

        // Apply all rules / 应用所有规则
        for rule in &self.rules
        {
            rule.validate(&self.field, value)?;
        }

        Ok(())
    }
}

/// Validation rule / 验证规则
#[derive(Clone)]
pub enum ValidationRule
{
    /// Not null - value cannot be empty/null
    /// 非空 - 值不能为空/null
    NotNull,

    /// Not empty - collections/strings must have elements
    /// 非空 - 集合/字符串必须有元素
    NotEmpty,

    /// Not blank - strings must have non-whitespace content
    /// 非空白 - 字符串必须有非空白内容
    NotBlank,

    /// Size validation / 大小验证
    Size
    {
        /// Minimum size / 最小大小
        min: Option<usize>,
        /// Maximum size / 最大大小
        max: Option<usize>,
    },

    /// Minimum value / 最小值
    Min(i64),

    /// Maximum value / 最大值
    Max(i64),

    /// Decimal minimum / 小数最小值
    DecimalMin(f64),

    /// Decimal maximum / 小数最大值
    DecimalMax(f64),

    /// Digits validation / 数字位数验证
    Digits
    {
        /// Maximum integer digits / 最大整数位数
        integer: usize,
        /// Maximum fraction digits / 最大小数位数
        fraction: usize,
    },

    /// Email validation / 邮箱验证
    Email,

    /// URL validation / URL验证
    Url,

    /// Pattern validation / 模式验证
    Pattern
    {
        /// Regular expression pattern / 正则表达式模式
        regex: String,
    },

    /// Negative number / 负数
    Negative,

    /// Positive number / 正数
    Positive,

    /// Negative or zero / 负数或零
    NegativeOrZero,

    /// Positive or zero / 正数或零
    PositiveOrZero,

    /// Past date / 过去日期
    Past,

    /// Future date / 未来日期
    Future,

    /// Past or present / 过去或现在
    PastOrPresent,

    /// Future or present / 未来或现在
    FutureOrPresent,

    /// Assert true / 必须为真
    AssertTrue,

    /// Assert false / 必须为假
    AssertFalse,

    /// Credit card number / 信用卡号
    CreditCardNumber,
}

impl ValidationRule
{
    /// Validate a value / 验证值
    #[allow(clippy::indexing_slicing)]
    pub fn validate(&self, field: &str, value: &str) -> Result<(), ValidationError>
    {
        match self
        {
            ValidationRule::NotNull =>
            {
                if value.is_empty()
                {
                    return Err(ValidationError::new(field, "Cannot be null"));
                }
            },
            ValidationRule::NotEmpty =>
            {
                if value.is_empty()
                {
                    return Err(ValidationError::new(field, "Cannot be empty"));
                }
            },
            ValidationRule::NotBlank =>
            {
                if value.trim().is_empty()
                {
                    return Err(ValidationError::new(field, "Cannot be blank"));
                }
            },
            ValidationRule::Size { min, max } =>
            {
                let len = value.len();
                if let Some(min_val) = min
                    && len < *min_val
                {
                    return Err(ValidationError::new(
                        field,
                        format!("Size must be at least {}", min_val),
                    ));
                }
                if let Some(max_val) = max
                    && len > *max_val
                {
                    return Err(ValidationError::new(
                        field,
                        format!("Size must be at most {}", max_val),
                    ));
                }
            },
            ValidationRule::Min(min) =>
            {
                if let Ok(num) = value.parse::<i64>()
                {
                    if num < *min
                    {
                        return Err(ValidationError::new(
                            field,
                            format!("Must be at least {}", min),
                        ));
                    }
                }
                else if let Ok(num) = value.parse::<f64>()
                {
                    if (num as i64) < *min
                    {
                        return Err(ValidationError::new(
                            field,
                            format!("Must be at least {}", min),
                        ));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Must be a number"));
                }
            },
            ValidationRule::Max(max) =>
            {
                if let Ok(num) = value.parse::<i64>()
                {
                    if num > *max
                    {
                        return Err(ValidationError::new(
                            field,
                            format!("Must be at most {}", max),
                        ));
                    }
                }
                else if let Ok(num) = value.parse::<f64>()
                {
                    if (num as i64) > *max
                    {
                        return Err(ValidationError::new(
                            field,
                            format!("Must be at most {}", max),
                        ));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Must be a number"));
                }
            },
            ValidationRule::DecimalMin(min) =>
            {
                if let Ok(num) = value.parse::<f64>()
                {
                    if num < *min
                    {
                        return Err(ValidationError::new(
                            field,
                            format!("Must be at least {}", min),
                        ));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Must be a number"));
                }
            },
            ValidationRule::DecimalMax(max) =>
            {
                if let Ok(num) = value.parse::<f64>()
                {
                    if num > *max
                    {
                        return Err(ValidationError::new(
                            field,
                            format!("Must be at most {}", max),
                        ));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Must be a number"));
                }
            },
            ValidationRule::Digits { integer, fraction } =>
            {
                let parts: Vec<&str> = value.split('.').collect();
                match parts.len()
                {
                    1 =>
                    {
                        if parts[0].len() > *integer
                        {
                            return Err(ValidationError::new(
                                field,
                                format!("Integer part must have at most {} digits", integer),
                            ));
                        }
                    },
                    2 =>
                    {
                        if parts[0].len() > *integer
                        {
                            return Err(ValidationError::new(
                                field,
                                format!("Integer part must have at most {} digits", integer),
                            ));
                        }
                        if parts[1].len() > *fraction
                        {
                            return Err(ValidationError::new(
                                field,
                                format!("Fraction part must have at most {} digits", fraction),
                            ));
                        }
                    },
                    _ =>
                    {
                        return Err(ValidationError::new(field, "Invalid number format"));
                    },
                }
            },
            ValidationRule::Email =>
            {
                use crate::validators::EMAIL_REGEX;
                if !EMAIL_REGEX.is_match(value)
                {
                    return Err(ValidationError::invalid_email(field));
                }
            },
            ValidationRule::Url =>
            {
                use crate::validators::URL_REGEX;
                if !URL_REGEX.is_match(value)
                {
                    return Err(ValidationError::new(field, "Invalid URL format"));
                }
            },
            ValidationRule::Pattern { regex } =>
            {
                use regex::Regex;
                let re = Regex::new(regex)
                    .map_err(|_| ValidationError::new(field, "Invalid regex pattern"))?;
                if !re.is_match(value)
                {
                    return Err(ValidationError::pattern_mismatch(field, regex));
                }
            },
            ValidationRule::Negative =>
            {
                if let Ok(num) = value.parse::<i64>()
                {
                    if num >= 0
                    {
                        return Err(ValidationError::new(field, "Must be negative"));
                    }
                }
                else if let Ok(num) = value.parse::<f64>()
                    && num >= 0.0
                {
                    return Err(ValidationError::new(field, "Must be negative"));
                }
            },
            ValidationRule::Positive =>
            {
                if let Ok(num) = value.parse::<i64>()
                {
                    if num <= 0
                    {
                        return Err(ValidationError::new(field, "Must be positive"));
                    }
                }
                else if let Ok(num) = value.parse::<f64>()
                    && num <= 0.0
                {
                    return Err(ValidationError::new(field, "Must be positive"));
                }
            },
            ValidationRule::NegativeOrZero =>
            {
                if let Ok(num) = value.parse::<i64>()
                {
                    if num > 0
                    {
                        return Err(ValidationError::new(field, "Must be negative or zero"));
                    }
                }
                else if let Ok(num) = value.parse::<f64>()
                    && num > 0.0
                {
                    return Err(ValidationError::new(field, "Must be negative or zero"));
                }
            },
            ValidationRule::PositiveOrZero =>
            {
                if let Ok(num) = value.parse::<i64>()
                {
                    if num < 0
                    {
                        return Err(ValidationError::new(field, "Must be positive or zero"));
                    }
                }
                else if let Ok(num) = value.parse::<f64>()
                    && num < 0.0
                {
                    return Err(ValidationError::new(field, "Must be positive or zero"));
                }
            },
            ValidationRule::Past =>
            {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(value)
                {
                    if dt > chrono::Utc::now()
                    {
                        return Err(ValidationError::new(field, "Must be in the past"));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Invalid date format"));
                }
            },
            ValidationRule::Future =>
            {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(value)
                {
                    if dt < chrono::Utc::now()
                    {
                        return Err(ValidationError::new(field, "Must be in the future"));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Invalid date format"));
                }
            },
            ValidationRule::PastOrPresent =>
            {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(value)
                {
                    if dt > chrono::Utc::now()
                    {
                        return Err(ValidationError::new(field, "Must be in the past or present"));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Invalid date format"));
                }
            },
            ValidationRule::FutureOrPresent =>
            {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(value)
                {
                    if dt < chrono::Utc::now()
                    {
                        return Err(ValidationError::new(
                            field,
                            "Must be in the future or present",
                        ));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Invalid date format"));
                }
            },
            ValidationRule::AssertTrue =>
            {
                if let Ok(b) = value.parse::<bool>()
                {
                    if !b
                    {
                        return Err(ValidationError::new(field, "Must be true"));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Must be a boolean"));
                }
            },
            ValidationRule::AssertFalse =>
            {
                if let Ok(b) = value.parse::<bool>()
                {
                    if b
                    {
                        return Err(ValidationError::new(field, "Must be false"));
                    }
                }
                else
                {
                    return Err(ValidationError::new(field, "Must be a boolean"));
                }
            },
            ValidationRule::CreditCardNumber =>
            {
                // Basic Luhn algorithm for credit card validation
                // 基本的Luhn算法用于信用卡验证
                if !is_valid_credit_card(value)
                {
                    return Err(ValidationError::new(field, "Invalid credit card number"));
                }
            },
        }
        Ok(())
    }
}

impl fmt::Debug for ValidationRule
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            ValidationRule::NotNull => write!(f, "NotNull"),
            ValidationRule::NotEmpty => write!(f, "NotEmpty"),
            ValidationRule::NotBlank => write!(f, "NotBlank"),
            ValidationRule::Size { min, max } => write!(f, "Size(min={:?}, max={:?})", min, max),
            ValidationRule::Min(n) => write!(f, "Min({})", n),
            ValidationRule::Max(n) => write!(f, "Max({})", n),
            ValidationRule::DecimalMin(n) => write!(f, "DecimalMin({})", n),
            ValidationRule::DecimalMax(n) => write!(f, "DecimalMax({})", n),
            ValidationRule::Digits { integer, fraction } =>
            {
                write!(f, "Digits(integer={}, fraction={})", integer, fraction)
            },
            ValidationRule::Email => write!(f, "Email"),
            ValidationRule::Url => write!(f, "Url"),
            ValidationRule::Pattern { regex } => write!(f, "Pattern({})", regex),
            ValidationRule::Negative => write!(f, "Negative"),
            ValidationRule::Positive => write!(f, "Positive"),
            ValidationRule::NegativeOrZero => write!(f, "NegativeOrZero"),
            ValidationRule::PositiveOrZero => write!(f, "PositiveOrZero"),
            ValidationRule::Past => write!(f, "Past"),
            ValidationRule::Future => write!(f, "Future"),
            ValidationRule::PastOrPresent => write!(f, "PastOrPresent"),
            ValidationRule::FutureOrPresent => write!(f, "FutureOrPresent"),
            ValidationRule::AssertTrue => write!(f, "AssertTrue"),
            ValidationRule::AssertFalse => write!(f, "AssertFalse"),
            ValidationRule::CreditCardNumber => write!(f, "CreditCardNumber"),
        }
    }
}

/// Check if a credit card number is valid using Luhn algorithm
/// 使用Luhn算法检查信用卡号是否有效
fn is_valid_credit_card(number: &str) -> bool
{
    let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();

    if digits.len() < 13 || digits.len() > 19
    {
        return false;
    }

    let mut sum = 0u32;
    let mut double = false; // Start with false so rightmost digit is NOT doubled

    for digit in digits.iter().rev()
    {
        let mut d = *digit;
        if double
        {
            d *= 2;
            if d > 9
            {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
    }

    sum.is_multiple_of(10)
}

// ============================================================================
// Annotation marker structs - used as attribute markers
// 注解标记结构体 - 用作属性标记
// ============================================================================

/// `NotNull` annotation marker / `NotNull注解标记`
///
/// Validates that a value is not null/empty.
/// 验证值不为null/空。
///
/// # Spring Equivalent / Spring等价物
///
/// The following Java code shows the equivalent:
///
/// ```java
/// @NotNull
/// private String username;
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct User {
///     #[NotNull]
///     username: String,
/// }
/// ```
pub struct NotNull;

/// `NotEmpty` annotation marker / `NotEmpty注解标记`
///
/// Validates that a string, collection, or array is not empty.
/// 验证字符串、集合或数组不为空。
pub struct NotEmpty;

/// `NotBlank` annotation marker / `NotBlank注解标记`
///
/// Validates that a string is not empty and contains non-whitespace characters.
/// 验证字符串不为空且包含非空白字符。
pub struct NotBlank;

/// Size annotation marker / Size注解标记
///
/// Validates that a value's size is between min and max.
/// 验证值的大小在最小值和最大值之间。
///
/// # Example / 示例
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct User {
///     #[Size(min = 3, max = 20)]
///     username: String,
/// }
/// ```
pub struct Size
{
    /// Minimum size / 最小大小
    pub min: Option<usize>,
    /// Maximum size / 最大大小
    pub max: Option<usize>,
}

/// Min annotation marker / Min注解标记
///
/// Validates that a number is at least the given minimum.
/// 验证数字至少为给定的最小值。
pub struct Min(pub i64);

/// Max annotation marker / Max注解标记
///
/// Validates that a number is at most the given maximum.
/// 验证数字最多为给定的最大值。
pub struct Max(pub i64);

/// `DecimalMin` annotation marker / `DecimalMin注解标记`
///
/// Validates that a decimal number is at least the given minimum.
/// 验证小数至少为给定的最小值。
pub struct DecimalMin(pub f64);

/// `DecimalMax` annotation marker / `DecimalMax注解标记`
///
/// Validates that a decimal number is at most the given maximum.
/// 验证小数最多为给定的最大值。
pub struct DecimalMax(pub f64);

/// Digits annotation marker / Digits注解标记
///
/// Validates the number of integer and fraction digits.
/// 验证整数和小数部分的位数。
pub struct Digits
{
    /// Maximum integer digits / 最大整数位数
    pub integer: usize,
    /// Maximum fraction digits / 最大小数位数
    pub fraction: usize,
}

/// Email annotation marker / Email注解标记
///
/// Validates that a string is a valid email address.
/// 验证字符串是有效的电子邮件地址。
pub struct Email;

/// Url annotation marker / Url注解标记
///
/// Validates that a string is a valid URL.
/// 验证字符串是有效的URL。
pub struct Url;

/// Pattern annotation marker / Pattern注解标记
///
/// Validates that a string matches the given regex pattern.
/// 验证字符串匹配给定的正则表达式模式。
///
/// # Example / 示例
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct User {
///     #[Pattern(regex = "^[a-zA-Z0-9]+$")]
///     username: String,
/// }
/// ```
pub struct Pattern
{
    /// Regex pattern / 正则表达式模式
    pub regex: &'static str,
}

/// Negative annotation marker / Negative注解标记
///
/// Validates that a number is negative.
/// 验证数字为负数。
pub struct Negative;

/// Positive annotation marker / Positive注解标记
///
/// Validates that a number is positive.
/// 验证数字为正数。
pub struct Positive;

/// `NegativeOrZero` annotation marker / `NegativeOrZero注解标记`
///
/// Validates that a number is negative or zero.
/// 验证数字为负数或零。
pub struct NegativeOrZero;

/// `PositiveOrZero` annotation marker / `PositiveOrZero注解标记`
///
/// Validates that a number is positive or zero.
/// 验证数字为正数或零。
pub struct PositiveOrZero;

/// Past annotation marker / Past注解标记
///
/// Validates that a date is in the past.
/// 验证日期在过去。
pub struct Past;

/// Future annotation marker / Future注解标记
///
/// Validates that a date is in the future.
/// 验证日期在未来。
pub struct Future;

/// `PastOrPresent` annotation marker / `PastOrPresent注解标记`
///
/// Validates that a date is in the past or present.
/// 验证日期在过去或现在。
pub struct PastOrPresent;

/// `FutureOrPresent` annotation marker / `FutureOrPresent注解标记`
///
/// Validates that a date is in the future or present.
/// 验证日期在未来或现在。
pub struct FutureOrPresent;

/// `AssertTrue` annotation marker / `AssertTrue注解标记`
///
/// Validates that a boolean is true.
/// 验证布尔值为true。
pub struct AssertTrue;

/// `AssertFalse` annotation marker / `AssertFalse注解标记`
///
/// Validates that a boolean is false.
/// 验证布尔值为false。
pub struct AssertFalse;

/// `CreditCardNumber` annotation marker / `CreditCardNumber注解标记`
///
/// Validates that a string is a valid credit card number.
/// 验证字符串是有效的信用卡号。
pub struct CreditCardNumber;

/// Length annotation marker (alias for Size) / Length注解标记（Size的别名）
///
/// Validates the length of a string.
/// 验证字符串的长度。
pub struct Length
{
    /// Minimum length / 最小长度
    pub min: Option<usize>,
    /// Maximum length / 最大长度
    pub max: Option<usize>,
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_not_null_validation()
    {
        let rule = ValidationRule::NotNull;
        assert!(rule.validate("field", "value").is_ok());
        assert!(rule.validate("field", "").is_err());
    }

    #[test]
    fn test_not_blank_validation()
    {
        let rule = ValidationRule::NotBlank;
        assert!(rule.validate("field", "value").is_ok());
        assert!(rule.validate("field", "").is_err());
        assert!(rule.validate("field", "   ").is_err());
    }

    #[test]
    fn test_size_validation()
    {
        let rule = ValidationRule::Size {
            min: Some(3),
            max: Some(10),
        };
        assert!(rule.validate("field", "test").is_ok());
        assert!(rule.validate("field", "ab").is_err()); // Too short
        assert!(rule.validate("field", "12345678901").is_err()); // Too long
    }

    #[test]
    fn test_min_max_validation()
    {
        let min_rule = ValidationRule::Min(18);
        let max_rule = ValidationRule::Max(120);

        assert!(min_rule.validate("age", "18").is_ok());
        assert!(min_rule.validate("age", "17").is_err());

        assert!(max_rule.validate("age", "120").is_ok());
        assert!(max_rule.validate("age", "121").is_err());
    }

    #[test]
    fn test_email_validation()
    {
        let rule = ValidationRule::Email;
        assert!(rule.validate("email", "test@example.com").is_ok());
        assert!(rule.validate("email", "invalid").is_err());
    }

    #[test]
    fn test_pattern_validation()
    {
        let rule = ValidationRule::Pattern {
            regex: "^[a-z]+$".to_string(),
        };
        assert!(rule.validate("field", "abc").is_ok());
        assert!(rule.validate("field", "abc123").is_err());
        assert!(rule.validate("field", "ABC").is_err());
    }

    #[test]
    fn test_positive_negative_validation()
    {
        let pos_rule = ValidationRule::Positive;
        let neg_rule = ValidationRule::Negative;

        assert!(pos_rule.validate("value", "1").is_ok());
        assert!(pos_rule.validate("value", "0").is_err());
        assert!(pos_rule.validate("value", "-1").is_err());

        assert!(neg_rule.validate("value", "-1").is_ok());
        assert!(neg_rule.validate("value", "0").is_err());
        assert!(neg_rule.validate("value", "1").is_err());
    }

    #[test]
    fn test_credit_card_validation()
    {
        let rule = ValidationRule::CreditCardNumber;
        // Valid test card numbers (using Luhn algorithm)
        // 有效的测试卡号（使用Luhn算法）
        // 4111111111111111 is a well-known Visa test number that passes Luhn
        assert!(rule.validate("card", "4111111111111111").is_ok()); // Visa test
        assert!(rule.validate("card", "4242424242424242").is_ok()); // Another Visa test
        assert!(rule.validate("card", "123").is_err()); // Too short
        assert!(rule.validate("card", "1234567890123456").is_err()); // Invalid Luhn
    }

    #[test]
    fn test_group_name()
    {
        assert_eq!(DefaultGroup::group_name(), "Default");
    }
}
