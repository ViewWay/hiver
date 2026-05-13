//! # Nexus Lombok
//!
//! Lombok-style procedural macros for Nexus framework.
//! 为 Nexus 框架提供 Lombok 风格的过程宏。
//!
//! ## Features / 功能
//!
//! - **`#[Data]`** - Generates getters, setters, constructor, and with methods
//!   生成 getters, setters, constructor 和 with 方法
//! - **`#[Getter]`** - Generates getter methods for struct fields
//!   为结构体字段生成 getter 方法
//! - **`#[Setter]`** - Generates setter methods for struct fields
//!   为结构体字段生成 setter 方法
//! - **`#[AllArgsConstructor]`** - Generates constructor with all fields
//!   生成包含所有字段的构造函数
//! - **`#[NoArgsConstructor]`** - Generates default constructor
//!   生成默认构造函数
//! - **`#[Builder]`** - Generates builder pattern
//!   生成 builder 模式
//! - **`#[Value]`** - Generates immutable class with getters
//!   生成带有 getter 的不可变类
//! - **`#[With]`** - Generates with_xxx methods for copying
//!   生成用于拷贝的 with_xxx 方法
//!
//! ## Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_lombok::Data;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Data, Serialize, Deserialize)]
//! pub struct User {
//!     pub id: i64,
//!     pub username: String,
//!     pub email: String,
//! }
//!
//! // Auto-generated / 自动生成:
//! // - User::new(id, username, email)
//! // - user.username(), user.email()
//! // - user.set_username(...), user.set_email(...)
//! // - user.with_id(...), user.with_username(...)
//! ```
//!
//! ## Quick Start / 快速开始
//!
//! Add to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! nexus-lombok = "0.1"
//! ```
//!
//! And use:
//!
//! ```rust,no_run,ignore
//! use nexus_lombok::Data;
//!
//! #[derive(Data)]
//! pub struct User {
//!     pub id: i64,
//!     pub username: String,
//! }
//! ```

#![allow(missing_docs)]
#![allow(unreachable_pub)]
#![allow(clippy::all)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

// ========================================================================
// Public API - Derive Macros / 公共 API - 派生宏
// ========================================================================

/// Generates getter methods for all struct fields
/// 为结构体的所有字段生成 getter 方法
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_lombok::Getter;
///
/// #[derive(Getter)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// // Generates / 生成:
/// // impl User {
/// //     pub fn id(&self) -> i64 { self.id }
/// //     pub fn username(&self) -> &str { &self.username }
/// // }
/// ```
#[proc_macro_derive(Getter)]
pub fn derive_getter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    getter::impl_getter(input).into()
}

/// Generates setter methods for all struct fields
/// 为结构体的所有字段生成 setter 方法
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_lombok::Setter;
///
/// #[derive(Setter)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// // Generates / 生成:
/// // impl User {
/// //     pub fn set_id(&mut self, id: i64) { self.id = id; }
/// //     pub fn set_username(&mut self, username: String) { self.username = username; }
/// // }
/// ```
#[proc_macro_derive(Setter)]
pub fn derive_setter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    setter::impl_setter(input).into()
}

/// Generates constructor with all fields
/// 生成包含所有字段的构造函数
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_lombok::AllArgsConstructor;
///
/// #[derive(AllArgsConstructor)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// // Generates / 生成:
/// // impl User {
/// //     pub fn new(id: i64, username: String) -> Self {
/// //         Self { id, username }
/// //     }
/// // }
/// ```
#[proc_macro_derive(AllArgsConstructor)]
pub fn derive_all_args_constructor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    constructor::impl_all_args(input).into()
}

/// Generates default constructor (no args)
/// 生成默认构造函数（无参数）
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_lombok::NoArgsConstructor;
///
/// #[derive(NoArgsConstructor)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// // Generates / 生成:
/// // impl User {
/// //     pub fn new() -> Self {
/// //         Self {
/// //             id: Default::default(),
/// //             username: Default::default(),
/// //         }
/// //     }
/// // }
///  impl Default for User { ... }
/// ```
#[proc_macro_derive(NoArgsConstructor)]
pub fn derive_no_args_constructor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    constructor::impl_no_args(input).into()
}

/// Generates getters, setters, constructor, and with methods
/// 生成 getters, setters, constructor 和 with 方法
///
/// This is the most commonly used macro, combining functionality from
/// Getter, Setter, AllArgsConstructor, and With.
///
/// 这是最常用的宏，结合了 Getter、Setter、AllArgsConstructor 和 With 的功能。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_lombok::Data;
///
/// #[derive(Data)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
///     pub email: String,
/// }
///
/// // Generates:
/// // - Constructor: User::new(id, username, email)
/// // - Getters: user.id(), user.username(), user.email()
/// // - Setters: user.set_id(...), user.set_username(...), user.set_email(...)
/// // - With methods: user.with_id(...), user.with_username(...), user.with_email(...)
/// ```
#[proc_macro_derive(Data)]
pub fn derive_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    data::impl_data(input).into()
}

/// Generates builder pattern
/// 生成 builder 模式
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_lombok::Builder;
///
/// #[derive(Builder)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// // Usage / 使用:
/// let user = User::builder()
///     .id(1)
///     .username("alice".into())
///     .build()
///     .unwrap();
/// ```
#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    builder::impl_builder(input).into()
}

/// Generates immutable value class with getters
/// 生成带有 getter 的不可变值类
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_lombok::Value;
///
/// #[derive(Value)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// // Generates:
/// // - Constructor: User::new(id, username)
/// // - Getters: user.id(), user.username()
/// // - With methods: user.with_id(...), user.with_username(...)
/// // All fields are immutable (private + read-only)
/// ```
#[proc_macro_derive(Value)]
pub fn derive_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    value::impl_value(input).into()
}

/// Generates with_xxx methods for creating modified copies
/// 生成用于创建修改副本的 with_xxx 方法
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_lombok::With;
///
/// #[derive(With, Clone)]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
///
/// // Generates / 生成:
/// // impl User {
/// //     pub fn with_id(&self, id: i64) -> Self {
/// //         Self { id, ..self.clone() }
/// //     }
/// //     pub fn with_username(&self, username: String) -> Self {
/// //         Self { username, ..self.clone() }
/// //     }
/// // }
/// ```
#[proc_macro_derive(With)]
pub fn derive_with(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    with_method::impl_with(input).into()
}

// ========================================================================
// Internal modules / 内部模块
mod data;
mod getter;
mod setter;
mod constructor;
mod builder;
mod value;
mod with_method;
