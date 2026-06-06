//! Validation extractor
//! 验证提取器

use std::fmt;

use serde::de::DeserializeOwned;

// Use the Validate trait from the crate root (re-exported from traits module)
// 使用从 crate 根目录导出的 Validate trait（从 traits 模块重新导出）
use crate::Validate as HiverValidate;
use crate::ValidationError;

/// Valid extractor wrapper
/// 有效提取器包装器
///
/// Extracts and validates the request data.
/// Equivalent to Spring's `@Valid` and `@Validated`.
///
/// 提取并验证请求数据。
/// 等价于Spring的`@Valid`和`@Validated`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @PostMapping("/users")
/// public ResponseEntity<User> createUser(@Valid @RequestBody CreateUserRequest request) {
///     // Request is automatically validated
///     return ResponseEntity.ok(userService.create(request));
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation::Valid;
/// use serde::Deserialize;
/// use validator::Validate;
///
/// #[derive(Deserialize, Validate)]
/// struct CreateUser {
///     #[validate(length(min = 3))]
///     username: String,
/// }
///
/// async fn create_user(Valid(user): Valid<CreateUser>) -> &'static str {
///     "User created!"
/// }
/// ```
pub struct Valid<T>(pub T);

impl<T> Valid<T>
{
    /// Consumes the Valid wrapper and returns the inner value
    /// 消耗 Valid 包装器并返回内部值
    pub fn into_inner(self) -> T
    {
        self.0
    }

    /// Gets a reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T
    {
        &self.0
    }
}

impl<T> fmt::Debug for Valid<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_tuple("Valid").field(&self.0).finish()
    }
}

impl<T> Clone for Valid<T>
where
    T: Clone,
{
    fn clone(&self) -> Self
    {
        Self(self.0.clone())
    }
}

/// Valid extractor implementation
/// Valid 提取器实现
///
/// This extractor validates the input data using the validator crate.
/// 此提取器使用 validator crate 验证输入数据。
impl<T> Valid<T>
where
    T: DeserializeOwned + HiverValidate,
{
    /// Validate the given data
    /// 验证给定数据
    pub fn validate(data: T) -> Result<Self, ValidationError>
    {
        data.validate()?;
        Ok(Valid(data))
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use serde::Deserialize;

    use super::*;
    use crate::ValidationErrors;

    #[derive(Debug, Clone, Deserialize)]
    struct TestUser
    {
        username: String,
        email: String,
    }

    // Implement our Validate trait for tests
    // 为测试实现我们的 Validate trait
    impl HiverValidate for TestUser
    {
        fn validate(&self) -> Result<(), ValidationErrors>
        {
            if self.username.len() < 3
            {
                let mut errors = ValidationErrors::new();
                errors.add("username", "Username must be at least 3 characters");
                return Err(errors);
            }
            Ok(())
        }
    }

    #[test]
    fn test_valid_success()
    {
        let user = TestUser {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let result = Valid::validate(user);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_failure()
    {
        let user = TestUser {
            username: "ab".to_string(), // Too short
            email: "test@example.com".to_string(),
        };

        let result = Valid::validate(user);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_into_inner()
    {
        let user = TestUser {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let valid = Valid::validate(user).unwrap();
        let inner = valid.into_inner();
        assert_eq!(inner.username, "alice");
    }
}
