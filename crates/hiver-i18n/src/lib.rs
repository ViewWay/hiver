//! Internationalization (i18n) support
//! 国际化支持
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `MessageSource` | `MessageSource` |
//! | `ResourceBundleMessageSource` | `ResourceBundleMessageSource` |
//! | `ReloadableResourceBundleMessageSource` | `ReloadableResourceBundleMessageSource` |
//! | `MessageSourceAware` | `MessageSourceAware` |
//! | `LocaleContextHolder` | `LocaleContextHolder` |
//!
//! # Examples / 示例
//!
//! ```rust,ignore
//! use hiver_i18n::{MessageSource, ResourceBundleMessageSource};
//!
//! // Create message source
//! let message_source = ResourceBundleMessageSource::new()
//!     .with_basenames(&["messages", "errors"])
//!     .with_default_locale("zh_CN");
//!
//! // Resolve message
//! let message = message_source.get_message("welcome", &[], "en_US").await?;
//! // Or with arguments
//! let message = message_source.get_message("greeting", &["Alice"], "en_US").await?;
//! ```
//!
//! # Message Properties Format / 消息属性文件格式
//!
//! ## messages_en.properties
//! ```properties
//! welcome=Welcome to our application!
//! greeting=Hello, {0}!
//! error.not.found=Resource not found: {0}
//! ```
//!
//! ## messages_zh_CN.properties
//! ```properties
//! welcome=\u6b22\u8fce\u4f7f\u7528\u6211\u4eec\u7684\u5e94\u7528\uff01
//! greeting=\u4f60\u597d\uff0c{0}\uff01
//! error.not.found=\u672a\u627e\u5230\u8d44\u6e90\uff1a{0}
//! ```

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;

mod error;
mod locale;
mod message_source;
mod resolver;
mod resource_bundle;

pub use error::{I18nError, I18nResult};
pub use locale::{Locale, LocaleContextHolder, LocaleResolver};
pub use message_source::{MessageSource, MessageSourceResolvable};
pub use resolver::{
    AcceptHeaderLocaleResolver, CookieLocaleResolver, FixedLocaleResolver, SessionLocaleResolver,
};
pub use resource_bundle::{ResourceBundleMessageSource, ResourceBundleSource};
