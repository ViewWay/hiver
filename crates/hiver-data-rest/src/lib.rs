//! Hiver Data REST — Spring Data REST equivalent.
//! Hiver Data REST — Spring Data REST 等价功能。
//!
//! Auto-generates type-safe REST CRUD handlers from `PagingAndSortingRepository` traits.
//! No reflection, no runtime scanning — pure Rust generics.
//!
//! 从 `PagingAndSortingRepository` trait 自动生成类型安全的 REST CRUD 处理器。
//! 无反射，无运行时扫描 — 纯 Rust 泛型。
//!
//! # Spring Equivalent / Spring等价物
//!
//! | Hiver | Spring Data REST |
//! |-------|-----------------|
//! | `RestResource<R, T, ID>` | `@RepositoryRestResource` |
//! | `.list(page, size)` | `GET /resource` (collection) |
//! | `.get(id)` | `GET /resource/{id}` (item) |
//! | `.create(entity)` | `POST /resource` |
//! | `.update(entity)` | `PUT /resource/{id}` |
//! | `.delete(id)` | `DELETE /resource/{id}` |
//! | `RestResponse<T>` | HAL JSON response |
//!
//! # Rust Advantage / Rust优势
//!
//! - **Compile-time**: wrong type = compile error (Spring: runtime 500)
//! - **No reflection**: generic dispatch, zero overhead
//! - **Type-safe IDs**: `ID: FromStr + Display`, not raw strings

#![warn(missing_docs)]
#![allow(unreachable_pub)]

mod resource;

pub use resource::{PageMeta, RestResource, RestResourceConfig, RestResponse};

/// Re-exports of commonly used types.
/// 常用类型的重新导出。
pub mod prelude
{
    pub use crate::{PageMeta, RestResource, RestResourceConfig, RestResponse};
}

/// Version of the data-rest module.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
