//! Hiver DevTools — Spring Boot DevTools equivalent.
//! Hiver 开发工具 — Spring Boot DevTools 等价功能。
//!
//! # Features
//! - **Config hot-reload**: watch config files, reload without restart
//! - **Auto-rebuild**: watch source files, trigger cargo build, signal restart
//! - **LiveReload**: WebSocket server for browser auto-refresh
//! - **Dev profile**: compile-time dev mode detection and configuration
//! - **Build info**: compile-time build metadata
//!
//! # Spring Equivalent / Spring等价物
//!
//! | Hiver | Spring Boot DevTools |
//! |-------|---------------------|
//! | `ConfigReloader` | Automatic config restart |
//! | `AutoRebuilder` | Automatic restart (ClassLoader) |
//! | `LiveReloadServer` | LiveReload (port 35729) |
//! | `Profile` | `@Profile("dev")` |
//! | `BuildInfo` | `BuildProperties` |
//!
//! # Rust Advantage / Rust优势
//!
//! - `notify` crate uses native OS APIs (inotify/kqueue) — zero overhead vs Spring's polling
//! - Type-safe config reload — no reflection
//! - Compile-time profile detection via `cfg(debug_assertions)` — no runtime lookup
//! - Feature-gated: only compile what you need

#![warn(missing_docs)]
#![allow(unreachable_pub)]

pub mod error;
pub mod livereload;
pub mod profile;
pub mod rebuilder;
pub mod reloader;

pub use error::{DevResult, DevToolsError};
pub use livereload::LiveReloadServer;
pub use profile::{BuildInfo, Profile};
pub use rebuilder::{AutoRebuilder, BuildStatus, RebuilderConfig};
pub use reloader::{ConfigEntry, ConfigReloader};

/// Version of the devtools module.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
