//! Locale resolvers
//! 语言环境解析器
//!
//! This module re-exports the locale resolvers defined in the locale module.
//! 此模块重新导出在locale模块中定义的语言环境解析器。

pub use crate::locale::{
    LocaleResolver, FixedLocaleResolver, AcceptHeaderLocaleResolver,
    CookieLocaleResolver, SessionLocaleResolver,
};
