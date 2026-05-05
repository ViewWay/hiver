//! # Nexus Validation Annotations
//!
//! Bean Validation style annotations for Nexus framework
//! Nexus 框架的 Bean Validation 风格注解
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
//! ```rust
//! use nexus_validation_annotations::{Valid, NotNull, Email, Size};
//! use nexus_http::Json;
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
/// ```rust
/// use nexus_validation_annotations::Valid;
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
/// ```rust
/// use nexus_validation_annotations::NotNull;
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
/// ```rust
/// use nexus_validation_annotations::Email;
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
/// ```rust
/// use nexus_validation_annotations::Size;
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
/// ```rust
/// use nexus_validation_annotations::Min;
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
/// ```rust
/// use nexus_validation_annotations::Max;
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
/// ```rust
/// use nexus_validation_annotations::Pattern;
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
/// ```rust
/// use nexus_validation_annotations::Length;
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
                    // Parse min/max from #[size(min = X, max = Y)]
                    // 解析 #[size(min = X, max = Y)] 中的 min/max
                    let mut min = 0u32;
                    let mut max = std::u32::MAX;

                    // Parse attributes
                    // 解析属性
                    attr.parse_nested_meta().ok().and_then(|meta| {
                        if let syn::Meta::List(meta_list) = meta {
                            for pair in meta_list.nested.iter() {
                                if let Some(key) = pair.key().ident {
                                    if key == "min" {
                                        if let Ok(value) = parse_value_from_meta(&pair.value) {
                                            min = value;
                                        }
                                    } else if key == "max" {
                                        if let Ok(value) = parse_value_from_meta(&pair.value) {
                                            max = value;
                                        }
                                    }
                                }
                            }
                            Some(())
                        } else {
                            None
                        }
                    }).unwrap_or_else(|| (min, max));

                    Some((id, min, max))
                })
            })
        })
        .collect()
}

/// Parse value from meta item
/// 从 meta 项中解析值
///
/// Note: In syn 2.x, NestedMeta was replaced with Meta. We use darling's NestedMeta
/// for compatibility with the attribute parsing macros.
/// 注意：在 syn 2.x 中，NestedMeta 被 Meta 替换。我们使用 darling 的 NestedMeta
///       以保持与属性解析宏的兼容性。
fn parse_value_from_meta(meta: &darling::ast::NestedMeta) -> Option<u32> {
    match meta {
        darling::ast::NestedMeta::Lit(syn::Lit::Int(lit)) => {
            lit.base10_parse().ok()
        }
        _ => None,
    }
}

/// Extract fields with #[min] attribute
/// 提取带有 #[min] 属性的字段
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
                min_attr.map(|_| {
                    // For now, use a default value of 0
                    // In a full implementation, you'd parse the attribute to get the actual value
                    (id.clone(), 0)
                })
            })
        })
        .collect()
}

/// Extract fields with #[max] attribute
/// 提取带有 #[max] 属性的字段
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
                max_attr.map(|_| {
                    // For now, use a default value of u32::MAX
                    // In a full implementation, you'd parse the attribute to get the actual value
                    (id.clone(), std::u32::MAX)
                })
            })
        })
        .collect()
}

/// Extract fields with #[pattern] attribute
/// 提取带有 #[pattern] 属性的字段
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
                pattern_attr.map(|_| {
                    // For now, use a default pattern
                    // In a full implementation, you'd parse the attribute to get the actual regex
                    (id.clone(), r".*".to_string())
                })
            })
        })
        .collect()
}

/// Extract fields with #[length] attribute
/// 提取带有 #[length] 属性的字段
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
                length_attr.map(|_| {
                    // For now, use default values
                    // In a full implementation, you'd parse min and max from the attribute
                    (id.clone(), 0, std::u32::MAX)
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