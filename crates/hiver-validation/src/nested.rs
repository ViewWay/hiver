//! Nested validation support / 嵌套验证支持
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! @Valid
//! private Address address; // Validates nested object
//!
//! @Valid
//! private List<Phone> phones; // Validates each element
//! ```
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_validation::{ValidateNested, Valid};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Address {
//!     #[NotNull]
//!     street: String,
//!
//!     #[NotNull]
//!     city: String,
//! }
//!
//! #[derive(Deserialize)]
//! struct CreateUserRequest {
//!     username: String,
//!
//!     #[ValidateNested]
//!     address: Address,  // Address will be validated
//! }
//! ```

use std::fmt;

use crate::{Validate, ValidationError, ValidationErrors};

/// Marker trait for nested validation
/// 嵌套验证的标记trait
///
/// Types implementing this trait can be validated as part of a parent object.
/// 实现此trait的类型可以作为父对象的一部分进行验证。
pub trait ValidateNested: Validate + Send + Sync
{
    /// Validate nested fields / 验证嵌套字段
    fn validate_nested(&self) -> Result<(), ValidationErrors>
    {
        self.validate()
    }
}

/// Validate nested annotation marker
/// 验证嵌套注解标记
///
/// Applied to fields to indicate they should be recursively validated.
/// 应用于字段以指示应递归验证它们。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Valid
/// private Address address;
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct UserRequest {
///     username: String,
///
///     #[Nested]
///     address: Address,
/// }
/// ```
pub struct Nested;

/// Collection validation marker
/// 集合验证标记
///
/// Applied to collections to validate each element.
/// 应用于集合以验证每个元素。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Valid
/// private List<Phone> phones;
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct UserRequest {
///     #[ValidateEach]
///     phones: Vec<Phone>,
/// }
/// ```
pub struct ValidateEach;

/// Validate nested object
/// 验证嵌套对象
pub fn validate_nested<T>(value: &T) -> Result<(), ValidationErrors>
where
    T: ValidateNested,
{
    value.validate_nested()
}

/// Validate collection of items
/// 验证项集合
pub fn validate_collection<T, I>(items: I) -> Result<(), ValidationErrors>
where
    I: IntoIterator<Item = T>,
    T: Validate,
{
    let mut errors = ValidationErrors::new();

    for (index, item) in items.into_iter().enumerate()
    {
        if let Err(e) = item.validate()
        {
            // Add index information to errors
            // 将索引信息添加到错误
            for (field, field_errors) in e.errors
            {
                for field_error in field_errors
                {
                    let nested_field = format!("{}[{}]", field, index);
                    errors.add_error(ValidationError {
                        field: nested_field,
                        message: field_error.message,
                        code: field_error.code,
                        value: field_error.value,
                        field_path: field_error.field_path,
                        rejected_value: field_error.rejected_value,
                        constraint_name: field_error.constraint_name,
                    });
                }
            }
        }
    }

    if errors.has_errors()
    {
        Err(errors)
    }
    else
    {
        Ok(())
    }
}

/// Validate nested option
/// 验证嵌套选项
pub fn validate_nested_option<T>(value: &Option<T>) -> Result<(), ValidationErrors>
where
    T: ValidateNested,
{
    if let Some(inner) = value
    {
        inner.validate_nested()
    }
    else
    {
        Ok(())
    }
}

/// Wrapper for validating nested objects
/// 用于验证嵌套对象的包装器
#[derive(Clone)]
pub struct NestedValidator<T>
{
    /// The wrapped value / 包装的值
    pub inner: T,
}

impl<T> NestedValidator<T>
{
    /// Create a new nested validator / 创建新的嵌套验证器
    pub fn new(inner: T) -> Self
    {
        Self { inner }
    }

    /// Consume and return the inner value / 消耗并返回内部值
    pub fn into_inner(self) -> T
    {
        self.inner
    }
}

impl<T: fmt::Debug> fmt::Debug for NestedValidator<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_tuple("NestedValidator").field(&self.inner).finish()
    }
}

impl<T> Validate for NestedValidator<T>
where
    T: ValidateNested + fmt::Debug,
{
    fn validate(&self) -> Result<(), ValidationErrors>
    {
        self.inner.validate_nested()
    }
}

/// Helper macro to implement `ValidateNested` for a type
/// `帮助宏为类型实现ValidateNested`
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_validation::validate_nested_impl;
///
/// #[derive(Deserialize)]
/// struct Address {
///     street: String,
///     city: String,
/// }
///
/// validate_nested_impl!(Address);
/// ```
#[macro_export]
macro_rules! validate_nested_impl {
    ($type:ty) => {
        impl $crate::nested::ValidateNested for $type {}
    };
    ($type:ty, $($life:lifetime),+) => {
        impl<$($life),+> $crate::nested::ValidateNested for $type {}
    };
}

/// Collection validator wrapper
/// 集合验证器包装器
#[derive(Clone)]
pub struct CollectionValidator<T>
{
    /// The wrapped collection / 包装的集合
    pub items: Vec<T>,
}

impl<T> CollectionValidator<T>
{
    /// Create a new collection validator / 创建新的集合验证器
    pub fn new(items: Vec<T>) -> Self
    {
        Self { items }
    }

    /// Create from iterator / 从迭代器创建
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            items: iter.into_iter().collect(),
        }
    }

    /// Consume and return the inner collection / 消耗并返回内部集合
    pub fn into_inner(self) -> Vec<T>
    {
        self.items
    }
}

impl<T: fmt::Debug> fmt::Debug for CollectionValidator<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_tuple("CollectionValidator")
            .field(&self.items)
            .finish()
    }
}

impl<T> Validate for CollectionValidator<T>
where
    T: Validate + Clone,
{
    fn validate(&self) -> Result<(), ValidationErrors>
    {
        validate_collection(self.items.clone())
    }
}

/// Map validator wrapper
/// Map验证器包装器
#[derive(Clone)]
pub struct MapValidator<K, V>
{
    /// The wrapped map / 包装的map
    pub map: Vec<(K, V)>,
}

impl<K, V> MapValidator<K, V>
{
    /// Create a new map validator / 创建新的map验证器
    pub fn new(map: Vec<(K, V)>) -> Self
    {
        Self { map }
    }

    /// Create from iterator / 从迭代器创建
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        Self {
            map: iter.into_iter().collect(),
        }
    }

    /// Consume and return the inner map / 消耗并返回内部map
    pub fn into_inner(self) -> Vec<(K, V)>
    {
        self.map
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for MapValidator<K, V>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_tuple("MapValidator").field(&self.map).finish()
    }
}

impl<K, V> Validate for MapValidator<K, V>
where
    K: fmt::Debug,
    V: Validate,
{
    fn validate(&self) -> Result<(), ValidationErrors>
    {
        let mut errors = ValidationErrors::new();

        for (key, value) in &self.map
        {
            if let Err(e) = value.validate()
            {
                // Add key information to errors
                // 将键信息添加到错误
                for (field, field_errors) in e.errors
                {
                    for field_error in field_errors
                    {
                        let nested_field = format!("{:?}", key);
                        let full_field = if field.is_empty()
                        {
                            nested_field
                        }
                        else
                        {
                            format!("{}.{}", nested_field, field)
                        };
                        errors.add_error(ValidationError {
                            field: full_field,
                            message: field_error.message,
                            code: field_error.code,
                            value: field_error.value,
                            field_path: field_error.field_path,
                            rejected_value: field_error.rejected_value,
                            constraint_name: field_error.constraint_name,
                        });
                    }
                }
            }
        }

        if errors.has_errors()
        {
            Err(errors)
        }
        else
        {
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize, Clone)]
    struct Address
    {
        street: String,
        city: String,
    }

    impl Validate for Address
    {
        fn validate(&self) -> Result<(), ValidationErrors>
        {
            let mut errors = ValidationErrors::new();

            if self.street.is_empty()
            {
                errors.add("street", "Street is required");
            }
            if self.city.is_empty()
            {
                errors.add("city", "City is required");
            }

            if errors.has_errors()
            {
                return Err(errors);
            }

            Ok(())
        }
    }

    validate_nested_impl!(Address);

    #[test]
    fn test_validate_nested()
    {
        let valid_address = Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
        };
        assert!(validate_nested(&valid_address).is_ok());

        let invalid_address = Address {
            street: String::new(),
            city: String::new(),
        };
        assert!(validate_nested(&invalid_address).is_err());
    }

    #[test]
    fn test_validate_nested_option()
    {
        let some_address: Option<Address> = Some(Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
        });
        assert!(validate_nested_option(&some_address).is_ok());

        let none_address: Option<Address> = None;
        assert!(validate_nested_option(&none_address).is_ok());

        let invalid_address: Option<Address> = Some(Address {
            street: String::new(),
            city: String::new(),
        });
        assert!(validate_nested_option(&invalid_address).is_err());
    }

    #[test]
    fn test_validate_collection()
    {
        let valid_items = vec![
            Address {
                street: "123 Main St".to_string(),
                city: "Springfield".to_string(),
            },
            Address {
                street: "456 Oak Ave".to_string(),
                city: "Shelbyville".to_string(),
            },
        ];
        assert!(validate_collection(valid_items).is_ok());

        let invalid_items = vec![
            Address {
                street: "123 Main St".to_string(),
                city: "Springfield".to_string(),
            },
            Address {
                street: String::new(),
                city: String::new(),
            },
        ];
        assert!(validate_collection(invalid_items).is_err());
    }

    #[test]
    fn test_nested_validator()
    {
        let validator = NestedValidator::new(Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
        });
        assert!(validator.validate().is_ok());

        let validator = NestedValidator::new(Address {
            street: String::new(),
            city: String::new(),
        });
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_collection_validator()
    {
        let validator = CollectionValidator::new(vec![
            Address {
                street: "123 Main St".to_string(),
                city: "Springfield".to_string(),
            },
            Address {
                street: "456 Oak Ave".to_string(),
                city: "Shelbyville".to_string(),
            },
        ]);
        assert!(validator.validate().is_ok());

        let validator = CollectionValidator::new(vec![
            Address {
                street: "123 Main St".to_string(),
                city: "Springfield".to_string(),
            },
            Address {
                street: String::new(),
                city: String::new(),
            },
        ]);
        assert!(validator.validate().is_err());
    }
}
