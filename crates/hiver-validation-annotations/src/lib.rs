//! # Hiver Validation Annotations
//!
//! Bean Validation style annotations for Hiver framework
//! Hiver 框架的 Bean Validation 风格注解
//!
//! ## Features / 功能
//!
//! - **`#[Valid]`** - Trigger validation for request parameters
//! - **`@NotNull`** - Validates field is not null
//! - **`@Email`** - Validates email format
//! - **`@Size`** - Validates string length
//! - **`@Min`** / **`@Max`** - Validates numeric ranges
//! - **`@Pattern`** - Validates with regex pattern
//!
//! ## Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_validation_annotations::{Valid, NotNull, Email, Size};
//! use hiver_http::Json;
//!
//! #[derive(Valid)]
//! struct CreateUserRequest {
//!     #[validate(email)]
//!     pub email: String,
//!
//!     #[validate(length(min = 3))]
//!     pub username: String,
//! }
//!
//! #[post("/users")]
//! async fn create_user(
//!     #[Valid] req: Json<CreateUserRequest>,
//! ) -> Result<Json<User>, Error> {
//!     // req is automatically validated
//!     // req 会被自动验证
//!     let user = service.create(req.into_inner()).await?;
//!     Ok(Json(user))
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

// ========================================================================
// @Valid Attribute / @Valid 属性
// ========================================================================

/// Marks a parameter to be validated
/// 标记参数以进行验证
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation_annotations::Valid;
///
/// #[post("/users")]
/// async fn create_user(
///     #[Valid] req: Json<CreateUserRequest>,
/// ) -> Result<Json<User>, Error> {
///     // req is automatically validated before this function runs
///     // req 会在函数运行前自动验证
///     Ok(Json(service.create(req.into_inner()).await?))
/// }
/// ```
#[proc_macro_attribute]
pub fn valid(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Currently just a marker, validation happens at runtime via extractor
    // 目前只是一个标记，验证通过提取器在运行时发生
    item
}

// ========================================================================
// Validation Derive Macros / 验证派生宏
// ========================================================================

/// Derive macro for NotNull validation
/// NotNull 验证的派生宏
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation_annotations::NotNull;
///
/// #[derive(NotNull)]
/// struct User {
///     #[not_null]
///     username: String,
/// }
/// ```
#[proc_macro_derive(NotNull)]
pub fn derive_not_null(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    // Extract fields and generate validation code
    // 提取字段并生成验证代码
    let fields = extract_fields_with_validation(&input);

    let validation_methods = fields.iter().map(|(field_name, field_type)| {
        // Generate function name using format_ident for stable Rust compatibility
        // 使用 format_ident 生成函数名以确保稳定版 Rust 兼容性
        let validate_fn_name = format_ident!("validate_{}", field_name);
        quote! {
            pub fn #validate_fn_name(&self) -> Result<(), String> {
                if self.#field_name.is_empty() {
                    Err(concat!(stringify!(#field_name), " cannot be null"))
                } else {
                    Ok(())
                }
            }
        }
    });

    let expanded = quote! {
        #input

        impl #name {
            #(#validation_methods)*

            pub fn validate(&self) -> Result<(), String> {
                #(#validation_methods.map(|m| m(&self)))*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Email validation
/// Email 验证的派生宏
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation_annotations::Email;
///
/// #[derive(Email)]
/// struct User {
///     #[email]
/// pub email: String,
/// }
/// ```
#[proc_macro_derive(Email)]
pub fn derive_email(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    // Extract fields with #[email] attribute
    // 提取带有 #[email] 属性的字段
    let fields = extract_email_fields(&input);

    let validation_methods = fields.iter().map(|(field_name, _)| {
        let validate_fn_name = format_ident!("validate_{}", field_name);
        quote! {
            pub fn #validate_fn_name(&self) -> Result<(), String> {
                // Simple email validation regex
                // 简单的 email 验证正则
                let email_regex = regex::Regex::new(
                    r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
                ).unwrap();

                if !email_regex.is_match(&self.#field_name) {
                    Err(concat!(stringify!(#field_name), " is not a valid email"))
                } else {
                    Ok(())
                }
            }
        }
    });

    let expanded = quote! {
        #input

        impl #name {
            #(#validation_methods)*

            pub fn validate(&self) -> Result<(), String> {
                #(#validation_methods.map(|m| m(&self)))*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Size validation
/// Size 验证的派生宏
///
/// # Attributes / 属性
///
/// - `min` - Minimum length
///   最小长度
/// - `max` - Maximum length
///   最大长度
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation_annotations::Size;
///
/// #[derive(Size)]
/// struct User {
///     #[size(min = 3, max = 20)]
///     pub username: String,
/// }
/// ```
#[proc_macro_derive(Size)]
pub fn derive_size(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    // Extract fields with size validation
    // 提取带有 size 验证的字段
    let fields_with_size = extract_fields_with_size(&input);

    let validation_methods = fields_with_size.iter().map(|(field_name, min, max)| {
        let validate_fn_name = format_ident!("validate_{}", field_name);
        quote! {
            pub fn #validate_fn_name(&self) -> Result<(), String> {
                let len = self.#field_name.len();
                let min = #min;
                let max = #max;

                if len < min {
                    Err(format!(
                        "{} length must be at least {} characters, but got {}",
                        stringify!(#field_name), min, len
                    ))
                } else if len > max {
                    Err(format!(
                        "{} length must be at most {} characters, but got {}",
                        stringify!(#field_name), max, len
                    ))
                } else {
                    Ok(())
                }
            }
        }
    });

    let expanded = quote! {
        #input

        impl #name {
            #(#validation_methods)*

            pub fn validate(&self) -> Result<(), String> {
                #(#validation_methods.map(|m| m(&self)))*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Min validation
/// Min 验证的派生宏
///
/// # Attributes / 属性
///
/// - `value` - Minimum value
///   最小值
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation_annotations::Min;
///
/// #[derive(Min)]
/// struct Order {
///     #[min(value = 1)]
///     pub quantity: i32,
/// }
/// ```
#[proc_macro_derive(Min, attributes(min))]
pub fn derive_min(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    // Extract fields with #[min] attribute
    // 提取带有 #[min] 属性的字段
    let fields_with_min = extract_fields_with_min(&input);

    let validation_methods = fields_with_min.iter().map(|(field_name, min_value)| {
        let validate_fn_name = format_ident!("validate_{}", field_name);
        quote! {
            pub fn #validate_fn_name(&self) -> Result<(), String> {
                if self.#field_name < #min_value {
                    Err(format!(
                        "{} must be at least {}, but got {}",
                        stringify!(#field_name), #min_value, self.#field_name
                    ))
                } else {
                    Ok(())
                }
            }
        }
    });

    let expanded = quote! {
        #input

        impl #name {
            #(#validation_methods)*

            pub fn validate(&self) -> Result<(), String> {
                #(#validation_methods.map(|m| m(&self)))*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Max validation
/// Max 验证的派生宏
///
/// # Attributes / 属性
///
/// - `value` - Maximum value
///   最大值
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation_annotations::Max;
///
/// #[derive(Max)]
/// struct Order {
///     #[max(value = 100)]
///     pub quantity: i32,
/// }
/// ```
#[proc_macro_derive(Max, attributes(max))]
pub fn derive_max(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    // Extract fields with #[max] attribute
    // 提取带有 #[max] 属性的字段
    let fields_with_max = extract_fields_with_max(&input);

    let validation_methods = fields_with_max.iter().map(|(field_name, max_value)| {
        let validate_fn_name = format_ident!("validate_{}", field_name);
        quote! {
            pub fn #validate_fn_name(&self) -> Result<(), String> {
                if self.#field_name > #max_value {
                    Err(format!(
                        "{} must be at most {}, but got {}",
                        stringify!(#field_name), #max_value, self.#field_name
                    ))
                } else {
                    Ok(())
                }
            }
        }
    });

    let expanded = quote! {
        #input

        impl #name {
            #(#validation_methods)*

            pub fn validate(&self) -> Result<(), String> {
                #(#validation_methods.map(|m| m(&self)))*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Pattern validation
/// Pattern 验证的派生宏
///
/// # Attributes / 属性
///
/// - `regex` - Regular expression pattern
///   正则表达式模式
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation_annotations::Pattern;
///
/// #[derive(Pattern)]
/// struct User {
///     #[pattern(regex = "^[a-zA-Z0-9]+$")]
///     pub username: String,
/// }
/// ```
#[proc_macro_derive(Pattern, attributes(pattern))]
pub fn derive_pattern(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    // Extract fields with #[pattern] attribute
    // 提取带有 #[pattern] 属性的字段
    let fields_with_pattern = extract_fields_with_pattern(&input);

    let validation_methods = fields_with_pattern.iter().map(|(field_name, regex_pattern)| {
        let validate_fn_name = format_ident!("validate_{}", field_name);
        quote! {
            pub fn #validate_fn_name(&self) -> Result<(), String> {
                use regex::Regex;
                let re = Regex::new(#regex_pattern).unwrap();
                if !re.is_match(&self.#field_name) {
                    Err(format!(
                        "{} does not match pattern {}",
                        stringify!(#field_name), #regex_pattern
                    ))
                } else {
                    Ok(())
                }
            }
        }
    });

    let expanded = quote! {
        #input

        impl #name {
            #(#validation_methods)*

            pub fn validate(&self) -> Result<(), String> {
                #(#validation_methods.map(|m| m(&self)))*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Length validation
/// Length 验证的派生宏
///
/// # Attributes / 属性
///
/// - `min` - Minimum length
///   最小长度
/// - `max` - Maximum length
///   最大长度
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_validation_annotations::Length;
///
/// #[derive(Length)]
/// struct User {
///     #[length(min = 3, max = 20)]
///     pub username: String,
/// }
/// ```
#[proc_macro_derive(Length, attributes(length))]
pub fn derive_length(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    // Extract fields with #[length] attribute
    // 提取带有 #[length] 属性的字段
    let fields_with_length = extract_fields_with_length(&input);

    let validation_methods = fields_with_length.iter().map(|(field_name, min, max)| {
        let validate_fn_name = format_ident!("validate_{}", field_name);
        quote! {
            pub fn #validate_fn_name(&self) -> Result<(), String> {
                let len = self.#field_name.len();
                let min = #min;
                let max = #max;

                if len < min {
                    Err(format!(
                        "{} length must be at least {}, but got {}",
                        stringify!(#field_name), min, len
                    ))
                } else if len > max {
                    Err(format!(
                        "{} length must be at most {}, but got {}",
                        stringify!(#field_name), max, len
                    ))
                } else {
                    Ok(())
                }
            }
        }
    });

    let expanded = quote! {
        #input

        impl #name {
            #(#validation_methods)*

            pub fn validate(&self) -> Result<(), String> {
                #(#validation_methods.map(|m| m(&self)))*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

// ========================================================================
// Helper Functions / 辅助函数
// ========================================================================

use syn::{Data, DataStruct, Fields, Attribute};

/// Extract fields with validation attributes
/// 提取带有验证属性的字段
fn extract_fields_with_validation(input: &syn::DeriveInput) -> Vec<(proc_macro2::Ident, &syn::Type)> {
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => return Vec::new(),
    };

    fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|id| (id, &f.ty)))
        .collect()
}

/// Extract fields with #[email] attribute
/// 提取带有 #[email] 属性的字段
fn extract_email_fields(input: &syn::DeriveInput) -> Vec<(proc_macro2::Ident, &syn::Type)> {
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => return Vec::new(),
    };

    fields
        .iter()
        .filter_map(|f| {
            let has_email_attr = f.attrs.iter().any(|attr| {
                attr.path()
                    .segments
                    .last()
                    .map(|s| s.ident == "email")
                    .unwrap_or(false)
            });

            f.ident.as_ref().map(|id| (id, &f.ty)).filter(|_| has_email_attr)
        })
        .collect()
}

/// Extract fields with size attributes
/// 提取带有 size 属性的字段
fn extract_fields_with_size(input: &syn::DeriveInput) -> Vec<(proc_macro2::Ident, u32, u32)> {
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => return Vec::new(),
    };

    fields
        .iter()
        .filter_map(|f| {
            let size_attr = f.attrs.iter().find(|attr| {
                attr.path()
                    .segments
                    .last()
                    .map(|s| s.ident == "size")
                    .unwrap_or(false)
            });

            f.ident.as_ref().and_then(|id| {
                size_attr.and_then(|attr| {
                    parse_min_max_attr(attr).map(|(min, max)| (id.clone(), min, max))
                })
            })
        })
        .collect()
}

/// Parse a single numeric value from an attribute like `#[name(5)]` or `#[name(value = 5)]`.
/// 从属性中解析单个数值，支持 `#[name(5)]` 和 `#[name(value = 5)]` 两种写法。
///
/// For bare value form `#[min(5)]`, reads the literal directly.
/// For named form `#[min(value = 5)]`, looks up the `key_name` parameter.
/// 对于裸值形式 `#[min(5)]`，直接读取字面量。
/// 对于命名形式 `#[min(value = 5)]`，查找 `key_name` 参数。
fn parse_single_numeric_attr(attr: &Attribute, key_name: &str) -> Option<u32> {
    let mut result: Option<u32> = None;

    // Try named form first: `#[min(value = 5)]`
    // 先尝试命名形式：`#[min(value = 5)]`
    let parsed = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident(key_name) {
            let value: syn::LitInt = meta.value()?.parse()?;
            result = value.base10_parse().ok();
        }
        Ok(())
    });

    if parsed.is_ok() && result.is_some() {
        return result;
    }

    // Fallback: bare literal form `#[min(5)]`
    // 回退：裸字面量形式 `#[min(5)]`
    if let Ok(lit) = attr.parse_args::<syn::LitInt>() {
        return lit.base10_parse().ok();
    }

    None
}

/// Parse a single string value from an attribute like `#[name("...")]` or `#[name(key = "...")]`.
/// 从属性中解析单个字符串值，支持 `#[name("...")]` 和 `#[name(key = "...")]` 两种写法。
fn parse_single_string_attr(attr: &Attribute, key_name: &str) -> Option<String> {
    let mut result: Option<String> = None;

    // Try named form first: `#[pattern(regex = "...")]`
    // 先尝试命名形式：`#[pattern(regex = "...")]`
    let parsed = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident(key_name) {
            let value: syn::LitStr = meta.value()?.parse()?;
            result = Some(value.value());
        }
        Ok(())
    });

    if parsed.is_ok() && result.is_some() {
        return result;
    }

    // Fallback: bare literal form `#[pattern("...")]`
    // 回退：裸字面量形式 `#[pattern("...")]`
    if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
        return Some(lit.value());
    }

    None
}

/// Parse min/max pair from an attribute like `#[length(min = 3, max = 20)]`.
/// 从属性中解析 min/max 对，例如 `#[length(min = 3, max = 20)]`。
///
/// Both min and max are optional; defaults are 0 and `u32::MAX` respectively.
/// min 和 max 均为可选，默认值分别为 0 和 `u32::MAX`。
fn parse_min_max_attr(attr: &Attribute) -> Option<(u32, u32)> {
    let mut min: u32 = 0;
    let mut max: u32 = u32::MAX;

    let parsed = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("min") {
            let value: syn::LitInt = meta.value()?.parse()?;
            if let Ok(v) = value.base10_parse::<u32>() {
                min = v;
            }
        } else if meta.path.is_ident("max") {
            let value: syn::LitInt = meta.value()?.parse()?;
            if let Ok(v) = value.base10_parse::<u32>() {
                max = v;
            }
        }
        Ok(())
    });

    if parsed.is_ok() {
        return Some((min, max));
    }

    None
}

/// Extract fields with #[min] attribute
/// 提取带有 #[min] 属性的字段
///
/// Supports both `#[min(5)]` and `#[min(value = 5)]` forms.
/// 同时支持 `#[min(5)]` 和 `#[min(value = 5)]` 两种写法。
fn extract_fields_with_min(input: &syn::DeriveInput) -> Vec<(proc_macro2::Ident, u32)> {
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => return Vec::new(),
    };

    fields
        .iter()
        .filter_map(|f| {
            let min_attr = f.attrs.iter().find(|attr| {
                attr.path()
                    .segments
                    .last()
                    .map(|s| s.ident == "min")
                    .unwrap_or(false)
            });

            f.ident.as_ref().and_then(|id| {
                min_attr.and_then(|attr| {
                    parse_single_numeric_attr(attr, "value").map(|val| (id.clone(), val))
                })
            })
        })
        .collect()
}

/// Extract fields with #[max] attribute
/// 提取带有 #[max] 属性的字段
///
/// Supports both `#[max(100)]` and `#[max(value = 100)]` forms.
/// 同时支持 `#[max(100)]` 和 `#[max(value = 100)` 两种写法。
fn extract_fields_with_max(input: &syn::DeriveInput) -> Vec<(proc_macro2::Ident, u32)> {
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => return Vec::new(),
    };

    fields
        .iter()
        .filter_map(|f| {
            let max_attr = f.attrs.iter().find(|attr| {
                attr.path()
                    .segments
                    .last()
                    .map(|s| s.ident == "max")
                    .unwrap_or(false)
            });

            f.ident.as_ref().and_then(|id| {
                max_attr.and_then(|attr| {
                    parse_single_numeric_attr(attr, "value").map(|val| (id.clone(), val))
                })
            })
        })
        .collect()
}

/// Extract fields with #[pattern] attribute
/// 提取带有 #[pattern] 属性的字段
///
/// Supports `#[pattern("^[a-z]+$")]` and `#[pattern(regex = "^[a-z]+$")]`.
/// 支持 `#[pattern("^[a-z]+$")]` 和 `#[pattern(regex = "^[a-z]+$")]` 两种写法。
fn extract_fields_with_pattern(input: &syn::DeriveInput) -> Vec<(proc_macro2::Ident, String)> {
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => return Vec::new(),
    };

    fields
        .iter()
        .filter_map(|f| {
            let pattern_attr = f.attrs.iter().find(|attr| {
                attr.path()
                    .segments
                    .last()
                    .map(|s| s.ident == "pattern")
                    .unwrap_or(false)
            });

            f.ident.as_ref().and_then(|id| {
                pattern_attr.and_then(|attr| {
                    parse_single_string_attr(attr, "regex").map(|val| (id.clone(), val))
                })
            })
        })
        .collect()
}

/// Extract fields with #[length] attribute
/// 提取带有 #[length] 属性的字段
///
/// Supports `#[length(min = 3, max = 20)]` with optional min/max.
/// 支持 `#[length(min = 3, max = 20)]`，min 和 max 均为可选。
fn extract_fields_with_length(input: &syn::DeriveInput) -> Vec<(proc_macro2::Ident, u32, u32)> {
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => return Vec::new(),
    };

    fields
        .iter()
        .filter_map(|f| {
            let length_attr = f.attrs.iter().find(|attr| {
                attr.path()
                    .segments
                    .last()
                    .map(|s| s.ident == "length")
                    .unwrap_or(false)
            });

            f.ident.as_ref().and_then(|id| {
                length_attr.and_then(|attr| {
                    parse_min_max_attr(attr).map(|(min, max)| (id.clone(), min, max))
                })
            })
        })
        .collect()
}

/// Concat strings at compile time
/// 在编译时连接字符串
macro_rules! concat {
    ($($str:expr),*) => {
        #[allow(unused_imports)]
        use proc_macro2::Ident;
        Ident::new($str).to_string()
    }
}


#[cfg(test)]
mod tests;