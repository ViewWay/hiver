//! Nexus Router - HTTP request router
//! Nexus路由器 - HTTP请求路由器
//!
//! # Overview / 概述
//!
//! `nexus-router` provides efficient HTTP request routing with path parameters
//! and middleware support.
//!
//! `nexus-router` 提供高效的HTTP请求路由，支持路径参数和中间件。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - @`RequestMapping`, @`GetMapping`, @`PostMapping`, etc.
//! - `PathVariable` extraction
//! - `RequestParam` extraction
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_router::Router;
//! use nexus_http::{Method, Response, StatusCode};
//!
//! let router = Router::new()
//!     .get("/users/:id", get_user)
//!     .post("/users", create_user)
//!     .route("/users/:id/posts", Method::GET, list_user_posts);
//!
//! async fn get_user(id: u64) -> Response {
//!     Response::builder()
//!         .status(StatusCode::OK)
//!         .body(format!("User {}", id))
//!         .unwrap()
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod params;
pub mod route;
pub mod router;
pub mod trie;

pub use params::Path;
pub use route::{AsyncHandlerFn, BoxedAsyncHandler, Handler as RouteHandler, Route};
pub use router::{Handler, Middleware, Next, Router, Stateful};
pub use trie::TrieRouter;

// Re-export from nexus-http
pub use nexus_http::Method;
