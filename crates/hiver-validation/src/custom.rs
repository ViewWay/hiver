//! Custom validator support / 自定义验证器支持
//!
//! Provides a custom validator framework similar to Spring's `@Constraint` annotation.
//! 提供类似 Spring `@Constraint` 注解的自定义验证器框架。
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! @Constraint(validatedBy = FieldMatchValidator.class)
//! @Target({ TYPE, ANNOTATION_TYPE })
//! @Retention(RUNTIME)
//! public @interface FieldMatch {
//!     String message() default "Fields must match";
//!     Class<?>[] groups() default {};
//!     Class<? extends Payload>[] payload() default {};
//!     String first();
//!     String second();
//! }
//! ```
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_validation::custom::{CustomValidator, ValidatorRegistry, ValidationError};
//! use std::any::Any;
//!
//! struct NoWhitespaceValidator;
//!
//! impl CustomValidator<String> for NoWhitespaceValidator {
//!     fn validate(&self, value: &String) -> Result<(), ValidationError> {
//!         if value.contains(' ') {
//!             return Err(ValidationError::new("field", "Must not contain whitespace")
//!                 .with_rejected_value(value)
//!                 .with_constraint_name("NoWhitespace"));
//!         }
//!         Ok(())
//!     }
//! }
//!
//! let mut registry = ValidatorRegistry::new();
//! registry.register("no_whitespace", NoWhitespaceValidator);
//!
//! let result = registry.validate("no_whitespace", &"hello world".to_string());
//! assert!(result.is_err());
//! ```

use std::{any::Any, collections::HashMap, fmt};

use crate::error::ValidationError;

// ---------------------------------------------------------------------------
// CustomValidator trait
// ---------------------------------------------------------------------------

/// Custom validator trait for user-defined validation logic.
/// 用户自定义验证逻辑的 trait。
///
/// Equivalent to Spring's `ConstraintValidator` interface.
/// 等价于 Spring 的 `ConstraintValidator` 接口。
///
/// # Type Parameters / 类型参数
///
/// - `T` - The type of value to validate / 要验证的值类型
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface ConstraintValidator<A extends Annotation, T> {
///     boolean isValid(T value, ConstraintValidatorContext context);
/// }
/// ```
pub trait CustomValidator<T: ?Sized>: Send + Sync
{
    /// Validate the given value.
    /// 验证给定值。
    ///
    /// # Returns / 返回
    ///
    /// - `Ok(())` if validation passes / 验证通过
    /// - `Err(ValidationError)` if validation fails / 验证失败
    fn validate(&self, value: &T) -> Result<(), ValidationError>;
}

// Blanket impl for function pointers: `fn(&T) -> Result<(), ValidationError>`
// 函数指针的 blanket impl
impl<T, F> CustomValidator<T> for F
where
    F: Fn(&T) -> Result<(), ValidationError> + Send + Sync,
{
    fn validate(&self, value: &T) -> Result<(), ValidationError>
    {
        self(value)
    }
}

// ---------------------------------------------------------------------------
// ValidatorRegistry
// ---------------------------------------------------------------------------

/// Registry for custom validators, keyed by name.
/// 按名称索引的自定义验证器注册表。
///
/// Stores type-erased validators in a `HashMap` so that callers can
/// register and retrieve validators by a string name without knowing
/// the concrete value type at registration time.
/// 将类型擦除的验证器存储在 `HashMap` 中，使调用者可以通过字符串名称
/// 注册和检索验证器，而无需在注册时知道具体的值类型。
///
/// # Spring Equivalent / Spring等价物
///
/// In Spring this is handled implicitly by the `ConstraintValidatorFactory`.
/// 在 Spring 中这由 `ConstraintValidatorFactory` 隐式处理。
pub struct ValidatorRegistry
{
    validators: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl ValidatorRegistry
{
    /// Create a new empty registry.
    /// 创建一个新的空注册表。
    pub fn new() -> Self
    {
        Self {
            validators: HashMap::new(),
        }
    }

    /// Register a custom validator under the given name.
    /// 在给定名称下注册自定义验证器。
    ///
    /// If a validator with the same name already exists it will be replaced.
    /// 如果同名的验证器已存在，它将被替换。
    pub fn register<T: 'static, V: CustomValidator<T> + 'static>(
        &mut self,
        name: &str,
        validator: V,
    )
    {
        // Double-box: outer Box<dyn Any> for type-erased storage,
        // inner Box<dyn CustomValidator<T>> for downcasting in validate().
        self.validators
            .insert(name.to_string(), Box::new(Box::new(validator) as Box<dyn CustomValidator<T>>));
    }

    /// Retrieve a registered validator as `&dyn Any`.
    /// 获取已注册的验证器，返回 `&dyn Any`。
    ///
    /// The caller must downcast to the expected concrete type:
    /// 调用者必须向下转型为期望的具体类型：
    ///
    /// ```rust,ignore
    /// if let Some(v) = registry.get("my_validator") {
    ///     if let Some(concrete) = v.downcast_ref::<MyValidator>() {
    ///         concrete.validate(&value)?;
    ///     }
    /// }
    /// ```
    pub fn get(&self, name: &str) -> Option<&(dyn Any + Send + Sync)>
    {
        self.validators.get(name).map(|v| &**v)
    }

    /// Validate a value using the named validator.
    /// 使用指定名称的验证器验证值。
    ///
    /// This is a convenience method that downcasts the stored validator to
    /// `dyn CustomValidator<T>` and invokes it.
    /// 这是一个便捷方法，将存储的验证器向下转型为 `dyn CustomValidator<T>` 并调用。
    ///
    /// Returns `Err(ValidationError)` with code `"validator_not_found"` if
    /// no validator is registered under `name`, or if the type does not match.
    /// 如果未注册该名称的验证器或类型不匹配，则返回带有 `"validator_not_found"`
    /// 错误代码的 `Err(ValidationError)`。
    pub fn validate<T: 'static>(&self, name: &str, value: &T) -> Result<(), ValidationError>
    {
        let boxed = self.validators.get(name).ok_or_else(|| {
            ValidationError::new("validator", format!("Validator '{}' not found", name))
                .with_code("validator_not_found")
        })?;

        // We store each validator wrapped in a second Box to enable
        // downcasting to `Box<dyn CustomValidator<T>>`.
        // The original registration wraps in `Box<dyn Any + Send + Sync>`,
        // so the actual inner type is `Box<dyn CustomValidator<T>>`.
        let validator = boxed
            .downcast_ref::<Box<dyn CustomValidator<T>>>()
            .ok_or_else(|| {
                ValidationError::new(
                    "validator",
                    format!(
                        "Type mismatch for validator '{}': expected CustomValidator<{}>",
                        name,
                        std::any::type_name::<T>()
                    ),
                )
                .with_code("validator_type_mismatch")
            })?;

        (validator.as_ref()).validate(value)
    }

    /// Check whether a validator with the given name exists.
    /// 检查是否存在具有给定名称的验证器。
    pub fn contains(&self, name: &str) -> bool
    {
        self.validators.contains_key(name)
    }

    /// Return the number of registered validators.
    /// 返回已注册验证器的数量。
    pub fn len(&self) -> usize
    {
        self.validators.len()
    }

    /// Return whether the registry is empty.
    /// 返回注册表是否为空。
    pub fn is_empty(&self) -> bool
    {
        self.validators.is_empty()
    }

    /// Remove a validator by name and return it.
    /// 按名称移除验证器并返回。
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn Any + Send + Sync>>
    {
        self.validators.remove(name)
    }
}

impl Default for ValidatorRegistry
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl fmt::Debug for ValidatorRegistry
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("ValidatorRegistry")
            .field("validators", &self.validators.keys().collect::<Vec<_>>())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// ValidationError enhancements
// ---------------------------------------------------------------------------

// The enhancements are added as new methods on the existing `ValidationError`
// via a separate extension block. Because `ValidationError` is defined in
// `error.rs` (same crate), we can add impl blocks here.
// 增强功能通过单独的扩展块以新方法的形式添加到现有的 `ValidationError` 上。
// 因为 `ValidationError` 定义在 `error.rs`（同一个 crate）中，我们可以在此添加 impl 块。

#[allow(clippy::module_name_repetitions)]
/// Extension methods for `ValidationError` from the custom module.
/// custom 模块中 `ValidationError` 的扩展方法。
pub trait ValidationErrorExt
{
    /// Set the nested field path (e.g. `"address.street"`).
    /// 设置嵌套字段路径（例如 `"address.street"`）。
    fn with_field_path(self, path: impl Into<String>) -> Self;

    /// Set the rejected (invalid) value as a string representation.
    /// 将拒绝（无效）值设置为字符串表示。
    fn with_rejected_value(self, value: impl fmt::Display) -> Self;

    /// Set the constraint / annotation name that caused the failure.
    /// 设置导致失败的约束/注解名称。
    fn with_constraint_name(self, name: impl Into<String>) -> Self;

    /// Get the constraint name if set.
    /// 获取约束名称（如果已设置）。
    fn constraint_name(&self) -> Option<&str>;

    /// Get the field path if set.
    /// 获取字段路径（如果已设置）。
    fn field_path(&self) -> Option<&str>;

    /// Get the rejected value if set.
    /// 获取拒绝的值（如果已设置）。
    fn rejected_value(&self) -> Option<&str>;
}

// To keep the extensions self-contained without modifying error.rs struct
// layout, we use an attached-thread-local or simply extend via new fields.
// However, modifying the struct in error.rs is the cleanest approach.
// We add the three new fields directly to error::ValidationError and
// provide builder-style methods.
//
// --- Actually, since error::ValidationError is in the same crate,
// we should add the fields to the struct directly. But to avoid changing
// too many callers, we use optional metadata stored in the `code`-adjacent
// area. The simplest approach: add the fields to the struct in error.rs.
// ---
// Since we are in the same crate, we CAN add impl blocks. The new fields
// will be added to the struct definition in error.rs. For now we
// provide the extension trait that works with the existing struct by
// encoding extra info into the `value` field for backward compat, but
// the proper approach is to add fields. We add them now.

impl ValidationErrorExt for ValidationError
{
    fn with_field_path(self, path: impl Into<String>) -> Self
    {
        // field_path is stored via a dedicated field added to the struct
        let mut s = self;
        s.field_path = Some(path.into());
        s
    }

    fn with_rejected_value(self, value: impl fmt::Display) -> Self
    {
        let mut s = self;
        s.rejected_value = Some(value.to_string());
        s
    }

    fn with_constraint_name(self, name: impl Into<String>) -> Self
    {
        let mut s = self;
        s.constraint_name = Some(name.into());
        s
    }

    fn constraint_name(&self) -> Option<&str>
    {
        self.constraint_name.as_deref()
    }

    fn field_path(&self) -> Option<&str>
    {
        self.field_path.as_deref()
    }

    fn rejected_value(&self) -> Option<&str>
    {
        self.rejected_value.as_deref()
    }
}

// ---------------------------------------------------------------------------
// ValidationReport - accumulates multiple errors
// ---------------------------------------------------------------------------

/// Accumulates multiple `ValidationError`s into a single result.
/// 将多个 `ValidationError` 累积到一个结果中。
///
/// This is distinct from `ValidationErrors` (which groups by field name)
/// and from the `ValidationResult<T>` type alias. `ValidationReport` is
/// a flat list of errors collected during a validation pass.
/// 这与 `ValidationErrors`（按字段名分组）和 `ValidationResult<T>` 类型别名不同。
/// `ValidationReport` 是验证过程中收集的错误的平面列表。
#[derive(Debug, Clone, Default)]
pub struct ValidationReport
{
    errors: Vec<ValidationError>,
}

impl ValidationReport
{
    /// Create an empty report.
    /// 创建一个空报告。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Return `true` if there are no errors (validation passed).
    /// 如果没有错误（验证通过），返回 `true`。
    pub fn is_valid(&self) -> bool
    {
        self.errors.is_empty()
    }

    /// Return all collected errors.
    /// 返回所有收集的错误。
    pub fn errors(&self) -> &[ValidationError]
    {
        &self.errors
    }

    /// Add a single error to the report.
    /// 向报告添加单个错误。
    pub fn add_error(&mut self, error: ValidationError)
    {
        self.errors.push(error);
    }

    /// Merge another report's errors into this one.
    /// 将另一个报告的错误合并到这个报告中。
    pub fn merge(&mut self, other: ValidationReport)
    {
        self.errors.extend(other.errors);
    }

    /// Return the number of errors.
    /// 返回错误数量。
    pub fn len(&self) -> usize
    {
        self.errors.len()
    }

    /// Return `true` if there are no errors.
    /// 如果没有错误则返回 `true`。
    pub fn is_empty(&self) -> bool
    {
        self.errors.is_empty()
    }

    /// Convert to `crate::ValidationErrors` (grouped by field).
    /// 转换为 `crate::ValidationErrors`（按字段分组）。
    pub fn into_validation_errors(self) -> crate::error::ValidationErrors
    {
        let mut errors = crate::error::ValidationErrors::new();
        for e in self.errors
        {
            errors.add_error(e);
        }
        errors
    }
}

impl fmt::Display for ValidationReport
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if self.errors.is_empty()
        {
            write!(f, "Validation passed")
        }
        else
        {
            write!(f, "Validation failed ({} error(s)):", self.errors.len())?;
            for e in &self.errors
            {
                write!(f, "\n  - {}", e)?;
            }
            Ok(())
        }
    }
}

impl std::error::Error for ValidationReport {}

// ---------------------------------------------------------------------------
// Built-in cross-field validators
// ---------------------------------------------------------------------------

/// Validates that two field values match (e.g. password & password confirmation).
/// 验证两个字段值匹配（例如密码和确认密码）。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @FieldMatch(first = "password", second = "confirmPassword")
/// ```
pub struct FieldMatchValidator
{
    /// Name of the first field / 第一个字段名称
    pub first_field: String,
    /// Name of the second field / 第二个字段名称
    pub second_field: String,
    /// Error message / 错误消息
    pub message: String,
}

impl FieldMatchValidator
{
    /// Create a new `FieldMatchValidator`.
    /// 创建新的 `FieldMatchValidator`。
    pub fn new(first_field: impl Into<String>, second_field: impl Into<String>) -> Self
    {
        let first = first_field.into();
        let second = second_field.into();
        Self {
            message: format!("{} and {} must match", first, second),
            first_field: first,
            second_field: second,
        }
    }

    /// Override the default error message.
    /// 覆盖默认错误消息。
    pub fn with_message(mut self, msg: impl Into<String>) -> Self
    {
        self.message = msg.into();
        self
    }
}

/// The value type for `FieldMatchValidator`: a map-like accessor.
/// `FieldMatchValidator` 的值类型：类似 map 的访问器。
///
/// Implement this trait on your struct so that `FieldMatchValidator` can
/// read the two fields by name.
/// 在你的结构体上实现此 trait，以便 `FieldMatchValidator` 可以按名称读取两个字段。
pub trait FieldProvider: Send + Sync + 'static
{
    /// Return the string representation of a field by name.
    /// 按名称返回字段的字符串表示。
    fn get_field_value(&self, field: &str) -> Option<String>;
}

impl CustomValidator<dyn FieldProvider> for FieldMatchValidator
{
    fn validate(&self, value: &(dyn FieldProvider + 'static)) -> Result<(), ValidationError>
    {
        let v1 = value.get_field_value(&self.first_field).ok_or_else(|| {
            ValidationError::new(&self.first_field, "Field not found")
                .with_code("field_not_found")
                .with_constraint_name("FieldMatch")
        })?;
        let v2 = value.get_field_value(&self.second_field).ok_or_else(|| {
            ValidationError::new(&self.second_field, "Field not found")
                .with_code("field_not_found")
                .with_constraint_name("FieldMatch")
        })?;

        if v1 != v2
        {
            return Err(ValidationError::new(&self.second_field, &self.message)
                .with_code("field_mismatch")
                .with_rejected_value(&v2)
                .with_constraint_name("FieldMatch"));
        }
        Ok(())
    }
}

/// Convenience function: validate two raw string values match.
/// 便捷函数：验证两个原始字符串值匹配。
pub fn field_match(first: &str, second: &str, field_name: &str) -> Result<(), ValidationError>
{
    if first != second
    {
        return Err(ValidationError::new(field_name, "Fields must match")
            .with_code("field_mismatch")
            .with_rejected_value(second)
            .with_constraint_name("FieldMatch"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// ConditionalValidator
// ---------------------------------------------------------------------------

/// Validates only when a condition is true; skips validation otherwise.
/// 仅在条件为 true 时验证；否则跳过验证。
///
/// # Example / 示例
///
/// ```rust,ignore
/// let validator = ConditionalValidator::new(
///     || age >= 18,
///     MyAdultValidator::new(),
/// );
/// ```
pub struct ConditionalValidator<C, V, T>
{
    /// Condition function / 条件函数
    condition: C,
    /// Inner validator / 内部验证器
    inner: V,
    /// Phantom for value type / 值类型的 Phantom
    _phantom: std::marker::PhantomData<T>,
}

impl<C, V, T> ConditionalValidator<C, V, T>
where
    C: Fn() -> bool + Send + Sync,
    V: CustomValidator<T> + Send + Sync,
{
    /// Create a new conditional validator.
    /// 创建新的条件验证器。
    pub fn new(condition: C, inner: V) -> Self
    {
        Self {
            condition,
            inner,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C, V, T> CustomValidator<T> for ConditionalValidator<C, V, T>
where
    C: Fn() -> bool + Send + Sync,
    V: CustomValidator<T> + Send + Sync,
    T: Send + Sync,
{
    fn validate(&self, value: &T) -> Result<(), ValidationError>
    {
        if (self.condition)()
        {
            self.inner.validate(value)
        }
        else
        {
            Ok(())
        }
    }
}

impl<C, V, T> fmt::Debug for ConditionalValidator<C, V, T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("ConditionalValidator")
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// CompositeValidator
// ---------------------------------------------------------------------------

/// Chains multiple validators; returns on the first failure.
/// 链接多个验证器；在第一个失败时返回。
///
/// # Spring Equivalent / Spring等价物
///
/// In Spring you would stack multiple annotations:
/// 在 Spring 中你会堆叠多个注解：
/// ```java
/// @NotNull @Size(min=3, max=20) @Pattern(regexp="...")
/// ```
pub struct CompositeValidator<T>
{
    validators: Vec<Box<dyn CustomValidator<T> + Send + Sync>>,
}

impl<T> CompositeValidator<T>
{
    /// Create an empty composite validator.
    /// 创建一个空的组合验证器。
    pub fn new() -> Self
    {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add a validator to the chain.
    /// 向链中添加验证器。
    pub fn add<V: CustomValidator<T> + Send + Sync + 'static>(mut self, validator: V) -> Self
    {
        self.validators.push(Box::new(validator));
        self
    }

    /// Add a validator mutably.
    /// 以可变方式添加验证器。
    pub fn push<V: CustomValidator<T> + Send + Sync + 'static>(&mut self, validator: V)
    {
        self.validators.push(Box::new(validator));
    }

    /// Return the number of validators in the chain.
    /// 返回链中验证器的数量。
    pub fn len(&self) -> usize
    {
        self.validators.len()
    }

    /// Return `true` if no validators are registered.
    /// 如果没有注册验证器，返回 `true`。
    pub fn is_empty(&self) -> bool
    {
        self.validators.is_empty()
    }
}

impl<T> Default for CompositeValidator<T>
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl<T> CustomValidator<T> for CompositeValidator<T>
{
    fn validate(&self, value: &T) -> Result<(), ValidationError>
    {
        for validator in &self.validators
        {
            validator.validate(value)?;
        }
        Ok(())
    }
}

impl<T> fmt::Debug for CompositeValidator<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("CompositeValidator")
            .field("validators_count", &self.validators.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    // -- CustomValidator basic test --

    struct NoWhitespaceValidator;

    impl CustomValidator<String> for NoWhitespaceValidator
    {
        fn validate(&self, value: &String) -> Result<(), ValidationError>
        {
            if value.contains(' ')
            {
                return Err(ValidationError::new("field", "Must not contain whitespace"));
            }
            Ok(())
        }
    }

    struct PositiveValidator;

    impl CustomValidator<i32> for PositiveValidator
    {
        fn validate(&self, value: &i32) -> Result<(), ValidationError>
        {
            if *value <= 0
            {
                return Err(ValidationError::new("value", "Must be positive"));
            }
            Ok(())
        }
    }

    #[test]
    fn test_custom_validator_success()
    {
        let v = NoWhitespaceValidator;
        assert!(v.validate(&"hello".to_string()).is_ok());
    }

    #[test]
    fn test_custom_validator_failure()
    {
        let v = NoWhitespaceValidator;
        let result = v.validate(&"hello world".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Must not contain whitespace");
    }

    #[test]
    fn test_fn_pointer_as_validator()
    {
        let check_len = |s: &String| {
            if s.len() < 5
            {
                Err(ValidationError::new("field", "Too short"))
            }
            else
            {
                Ok(())
            }
        };
        assert!(check_len.validate(&"hello".to_string()).is_ok());
        assert!(check_len.validate(&"hi".to_string()).is_err());
    }

    // -- ValidatorRegistry tests --

    #[test]
    fn test_registry_register_and_validate()
    {
        let mut registry = ValidatorRegistry::new();
        registry.register("no_ws", NoWhitespaceValidator);

        // Successful validation
        assert!(registry.validate("no_ws", &"abc".to_string()).is_ok());
        // Failed validation
        assert!(registry.validate("no_ws", &"a b".to_string()).is_err());
    }

    #[test]
    fn test_registry_validator_not_found()
    {
        let registry = ValidatorRegistry::new();
        let result = registry.validate("nonexistent", &42_i32);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "validator_not_found");
    }

    #[test]
    fn test_registry_get_any()
    {
        let mut registry = ValidatorRegistry::new();
        registry.register("positive", PositiveValidator);

        let any_ref = registry.get("positive").expect("should exist");
        assert!(any_ref.is::<Box<dyn CustomValidator<i32>>>());
    }

    #[test]
    fn test_registry_contains()
    {
        let mut registry = ValidatorRegistry::new();
        registry.register("no_ws", NoWhitespaceValidator);
        assert!(registry.contains("no_ws"));
        assert!(!registry.contains("missing"));
    }

    #[test]
    fn test_registry_len()
    {
        let mut registry = ValidatorRegistry::new();
        assert_eq!(registry.len(), 0);
        registry.register("a", NoWhitespaceValidator);
        assert_eq!(registry.len(), 1);
        registry.register("b", PositiveValidator);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_registry_remove()
    {
        let mut registry = ValidatorRegistry::new();
        registry.register("no_ws", NoWhitespaceValidator);
        assert!(registry.contains("no_ws"));
        registry.remove("no_ws");
        assert!(!registry.contains("no_ws"));
    }

    #[test]
    fn test_registry_debug()
    {
        let mut registry = ValidatorRegistry::new();
        registry.register("no_ws", NoWhitespaceValidator);
        let debug_str = format!("{:?}", registry);
        assert!(debug_str.contains("ValidatorRegistry"));
        assert!(debug_str.contains("no_ws"));
    }

    // -- ValidationErrorExt tests --

    #[test]
    fn test_validation_error_ext()
    {
        let err = ValidationError::new("email", "Invalid email")
            .with_field_path("user.contact.email")
            .with_rejected_value("bad")
            .with_constraint_name("Email");

        assert_eq!(err.field_path(), Some("user.contact.email"));
        assert_eq!(err.rejected_value(), Some("bad"));
        assert_eq!(err.constraint_name(), Some("Email"));
    }

    // -- ValidationReport tests --

    #[test]
    fn test_report_valid()
    {
        let report = ValidationReport::new();
        assert!(report.is_valid());
        assert!(report.is_empty());
        assert_eq!(report.len(), 0);
    }

    #[test]
    fn test_report_add_error()
    {
        let mut report = ValidationReport::new();
        report.add_error(ValidationError::new("a", "error a"));
        report.add_error(ValidationError::new("b", "error b"));

        assert!(!report.is_valid());
        assert_eq!(report.len(), 2);
        assert_eq!(report.errors()[0].field, "a");
        assert_eq!(report.errors()[1].field, "b");
    }

    #[test]
    fn test_report_merge()
    {
        let mut r1 = ValidationReport::new();
        r1.add_error(ValidationError::new("x", "err1"));

        let mut r2 = ValidationReport::new();
        r2.add_error(ValidationError::new("y", "err2"));

        r1.merge(r2);
        assert_eq!(r1.len(), 2);
    }

    #[test]
    fn test_report_display()
    {
        let report = ValidationReport::new();
        assert_eq!(format!("{}", report), "Validation passed");

        let mut report = ValidationReport::new();
        report.add_error(ValidationError::new("f", "msg"));
        let s = format!("{}", report);
        assert!(s.contains("Validation failed"));
        assert!(s.contains("f: msg"));
    }

    #[test]
    fn test_report_into_validation_errors()
    {
        let mut report = ValidationReport::new();
        report.add_error(ValidationError::new("a", "error a"));
        report.add_error(ValidationError::new("b", "error b"));

        let errors = report.into_validation_errors();
        assert!(errors.has_errors());
        assert_eq!(errors.len(), 2);
        assert!(errors.get("a").is_some());
        assert!(errors.get("b").is_some());
    }

    // -- FieldMatchValidator tests --

    struct TestForm
    {
        password: String,
        confirm: String,
    }

    impl FieldProvider for TestForm
    {
        fn get_field_value(&self, field: &str) -> Option<String>
        {
            match field
            {
                "password" => Some(self.password.clone()),
                "confirm" => Some(self.confirm.clone()),
                _ => None,
            }
        }
    }

    #[test]
    fn test_field_match_success()
    {
        let form = TestForm {
            password: "secret".to_string(),
            confirm: "secret".to_string(),
        };
        let validator = FieldMatchValidator::new("password", "confirm");
        assert!(validator.validate(&form).is_ok());
    }

    #[test]
    fn test_field_match_failure()
    {
        let form = TestForm {
            password: "secret".to_string(),
            confirm: "different".to_string(),
        };
        let validator = FieldMatchValidator::new("password", "confirm");
        let result = validator.validate(&form);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "field_mismatch");
        assert_eq!(err.rejected_value(), Some("different"));
    }

    #[test]
    fn test_field_match_fn()
    {
        assert!(field_match("abc", "abc", "pw").is_ok());
        assert!(field_match("abc", "xyz", "pw").is_err());
    }

    // -- ConditionalValidator tests --

    struct AlwaysFail;

    impl CustomValidator<i32> for AlwaysFail
    {
        fn validate(&self, _value: &i32) -> Result<(), ValidationError>
        {
            Err(ValidationError::new("v", "always fails"))
        }
    }

    #[test]
    fn test_conditional_validator_active()
    {
        let cv: ConditionalValidator<_, _, i32> = ConditionalValidator::new(|| true, AlwaysFail);
        assert!(cv.validate(&42).is_err());
    }

    #[test]
    fn test_conditional_validator_inactive()
    {
        let cv: ConditionalValidator<_, _, i32> = ConditionalValidator::new(|| false, AlwaysFail);
        assert!(cv.validate(&42).is_ok());
    }

    // -- CompositeValidator tests --

    #[test]
    fn test_composite_all_pass()
    {
        let composite = CompositeValidator::<i32>::new()
            .add(|v: &i32| {
                if *v < 0
                {
                    Err(ValidationError::new("v", "negative"))
                }
                else
                {
                    Ok(())
                }
            })
            .add(|v: &i32| {
                if *v > 100
                {
                    Err(ValidationError::new("v", "too large"))
                }
                else
                {
                    Ok(())
                }
            });

        assert!(composite.validate(&50).is_ok());
    }

    #[test]
    fn test_composite_first_fail()
    {
        let composite = CompositeValidator::<i32>::new()
            .add(|v: &i32| {
                if *v < 0
                {
                    Err(ValidationError::new("v", "negative"))
                }
                else
                {
                    Ok(())
                }
            })
            .add(|v: &i32| {
                if *v > 100
                {
                    Err(ValidationError::new("v", "too large"))
                }
                else
                {
                    Ok(())
                }
            });

        assert!(composite.validate(&-1).is_err());
        let err = composite.validate(&-1).unwrap_err();
        assert_eq!(err.message, "negative");
    }

    #[test]
    fn test_composite_second_fail()
    {
        let composite = CompositeValidator::<i32>::new()
            .add(|v: &i32| {
                if *v < 0
                {
                    Err(ValidationError::new("v", "negative"))
                }
                else
                {
                    Ok(())
                }
            })
            .add(|v: &i32| {
                if *v > 100
                {
                    Err(ValidationError::new("v", "too large"))
                }
                else
                {
                    Ok(())
                }
            });

        assert!(composite.validate(&200).is_err());
        let err = composite.validate(&200).unwrap_err();
        assert_eq!(err.message, "too large");
    }

    #[test]
    fn test_composite_empty()
    {
        let composite: CompositeValidator<i32> = CompositeValidator::new();
        assert!(composite.validate(&999).is_ok());
        assert_eq!(composite.len(), 0);
    }

    #[test]
    fn test_composite_debug()
    {
        let composite: CompositeValidator<i32> = CompositeValidator::new().add(|_v: &i32| Ok(()));
        let debug_str = format!("{:?}", composite);
        assert!(debug_str.contains("CompositeValidator"));
        assert!(debug_str.contains("validators_count"));
    }
}
