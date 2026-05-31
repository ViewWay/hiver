//! Validation trait
//! 验证 trait

use crate::ValidationError;

/// Validate trait
/// 验证 trait
///
/// Equivalent to Spring's `@Validated` annotation.
/// 等价于Spring的`@Validated`注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Validated
/// public class UserRequest {
///     @NotNull
///     @Size(min = 3, max = 50)
///     private String username;
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation::Validate;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, validator::Validate)]
/// struct CreateUser {
///     #[validate(length(min = 3, max = 50))]
///     username: String,
///
///     #[validate(email)]
///     email: String,
/// }
///
/// // Manual validation / 手动验证
/// impl Validate for CreateUser {
///     fn validate(&self) -> Result<(), ValidationError> {
///         validator::validate(self)
///             .map(|_| ())
///             .map_err(ValidationError::from)
///     }
/// }
/// ```
pub trait Validate {
    /// Validate the struct
    /// 验证结构体
    fn validate(&self) -> Result<(), ValidationError>;
}

/// Re-export of validator crate for derive macro
/// validator crate 的重新导出，用于 derive 宏
pub use validator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_trait_exists() {
        // Verify the trait can be used as a bound
        fn check<T: Validate>(_item: &T) -> bool {
            true
        }

        // Create a simple test type
        struct TestType;
        impl Validate for TestType {
            fn validate(&self) -> Result<(), ValidationError> {
                Ok(())
            }
        }

        // Just check that the trait exists
        assert!(check(&TestType));
    }
}
