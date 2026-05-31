//! Hiver Security - Security framework module
//! Hiver安全 - 安全框架模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@PreAuthorize` - `PreAuthorize`
//! - `@Secured` - Secured
//! - `@RolesAllowed` - `RolesAllowed`
//! - `UserDetails` - User
//! - `GrantedAuthority` - Permission/Role
//! - `Authentication` - Auth
//! - `SecurityContext` - `SecurityContext`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_security::{PreAuthorize, Secured, User, Role};
//!
//! struct UserService {
//!     // ... fields
//! }
//!
//! impl UserService {
//!     #[pre_authorize("hasRole('ADMIN')")]
//!     async fn delete_user(&self, id: u64) -> Result<(), Error> {
//!         // Only accessible by users with ADMIN role
//!         Ok(())
//!     }
//!
//!     #[secured("ROLE_USER")]
//!     async fn get_profile(&self) -> Result<Profile, Error> {
//!         // Only accessible by authenticated users
//!         Ok(Profile::default())
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow dead_code: This is a framework library with many public APIs that are
// provided for users but not used internally. This is expected and intentional.
// 允许 dead_code：这是一个框架库，包含许多公共 API 供用户使用但内部未使用。
// 这是预期且有意的设计。
#![allow(dead_code)]
// Allow unreachable_pub: Some public items are exported for feature flag compatibility
// 允许 unreachable_pub：某些公共项目导出用于功能标志兼容性
#![allow(unreachable_pub)]
// Allow expect_used/unwrap_used on RwLock/Mutex guards: lock poisoning is
// intentionally unrecoverable — panicking is the standard Rust idiom.
// 允许在 RwLock/Mutex 守卫上使用 expect/unwrap：锁中毒是有意不可恢复的——恐慌是标准 Rust 惯用法。
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
// Allow indexing_slicing: many internal parsers use known-length vectors.
// 允许索引/切片：许多内部解析器使用已知长度的向量。
#![allow(clippy::indexing_slicing)]
// Allow new_ret_no_self: AnonymousAuthentication::new returns Authentication,
// which is the intentional factory pattern matching Spring Security's API.
// 允许 new_ret_no_self：AnonymousAuthentication::new 返回 Authentication，
// 这是有意的工厂模式，匹配 Spring Security 的 API。
#![allow(clippy::new_ret_no_self)]
// Allow collapsible_if: chained let-conditions reduce readability in
// middleware-style code with early returns.
// 允许 collapsible_if：链式 let 条件在带有提前返回的中间件风格代码中降低可读性。
#![allow(clippy::collapsible_if)]
// Allow format_push_string: intentional for email payload building where
// clarity of format strings outweighs the minor allocation cost.
// 允许 format_push_string：对于电子邮件载荷构建，格式字符串的清晰性超过了少量分配成本。
#![allow(clippy::format_push_string)]
// Allow items_after_statements: use statements for crypto traits placed
// near their usage for self-contained method impls.
// 允许 items_after_statements：用于自包含方法实现的加密 trait 的 use 语句放置在用法附近。
#![allow(clippy::items_after_statements)]
// Allow manual_let_else: match-based error handling is clearer for
// hex decode operations with multiple failure modes.
// 允许 manual_let_else：基于 match 的错误处理对于具有多种失败模式的十六进制解码操作更清晰。
#![allow(clippy::manual_let_else)]
// Allow redundant_closure: method resolution trait compatibility.
// 允许冗余闭包：方法解析 trait 兼容性。
#![allow(clippy::redundant_closure_for_method_calls)]
// Allow manual_range_contains: explicit range checks for clarity.
// 允许 manual_range_contains：显式范围检查以增加清晰度。
#![allow(clippy::manual_range_contains)]
// Allow unused_async: public API compatibility with async trait methods.
// 允许 unused_async：与异步 trait 方法的公共 API 兼容性。
#![allow(clippy::unused_async)]

#[cfg(test)]
mod tests;

mod auth;
pub mod acl;
pub mod authorization_server;
mod authority;
mod context;
pub mod data_scope;
mod csrf;
pub mod email;
mod encoder;
mod error;
mod jwt;
mod oauth2;
pub mod permission;
mod pre_authorize;
mod post_authorize;
mod rbac;
mod rememberme;
mod request_ext;
mod role;
mod secured;
mod user;

pub use auth::{Authentication, AuthenticationManager};
pub use authority::{Authority, GrantedAuthority};
pub use context::{SecurityContext, SecurityContextGuard};
pub use data_scope::{
    DataScope, DataScopeApply, DataScopeContext, DataScopeMiddleware, DataScopeRule,
    DataScopeType,
};
pub use csrf::{CsrfProtectionConfig, CsrfToken, CsrfTokenRepository, InMemoryCsrfTokenRepository};
pub use encoder::{BcryptPasswordEncoder, NoOpPasswordEncoder, PasswordEncoder, Pbkdf2PasswordEncoder, StandardPasswordEncoder};
pub use error::{SecurityError, SecurityResult};
pub use jwt::{JwtAlgorithm, JwtAuthentication, JwtClaims, JwtClaimsBuilder, JwtTokenProvider, JwtUtil};
pub use oauth2::{
    IntrospectionResponse, OIDCDiscovery, OIDCDiscoveryDocument, OAuth2Client, OAuth2Config,
    PkceParams, StateManager, TokenEndpointAuthMethod, TokenResponse, TokenResponseWithTimestamp,
    UserInfo,
};
pub use pre_authorize::{PreAuthorize, SecurityExpression};
pub use post_authorize::{PostAuthorize, PostAuthorizeOptions};
pub use rbac::{
    AuditLog, AuditLogger, ConsoleAuditLogger, PermissionEntry, RbacConfig, RbacManager,
    RolePermission, UserRole,
};
pub use request_ext::{SecurityContextExt, get_authentication_from_request};
pub use role::{Permission, Role, Role as RoleEnum, Roles};
pub use secured::{Secured, SecuredHelper, SecurityMetadata};
pub use user::{InMemoryUserService, User, UserDetails, UserService};

pub use authorization_server::{
    AuthorizationServer, AuthorizationServerBuilder, DeviceAuthorizationResponse,
    DeviceCodeStatus, GrantType, IntrospectionResult, IssuedTokenResponse, RegisteredClient,
};
pub use email::{
    Attachment, EmailConfig, EmailError, EmailMessage, EmailQueue, EmailResult, EmailSender,
    EmailTemplate, SmtpEmailSender,
};
pub use permission::{
    InMemoryPermissionAuditLogger, PermissionAuditEntry, PermissionAuditLog, PermissionAuditLogger,
    PermissionDef, PermissionEvaluator, PermissionRegistry,
};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        AuditLogger, Authentication, AuthenticationManager, Authority, ConsoleAuditLogger,
        DataScope, DataScopeApply, DataScopeContext, DataScopeMiddleware, DataScopeRule,
        DataScopeType, GrantedAuthority, JwtAuthentication, JwtClaims, JwtTokenProvider, JwtUtil,
        PasswordEncoder, Permission, PermissionAuditEntry, PermissionAuditLogger,
        PermissionAuditLog, PermissionDef, PermissionEvaluator, PermissionEntry,
        PermissionRegistry, PreAuthorize, RbacConfig, RbacManager, RoleEnum, RolePermission,
        Roles, Secured, SecurityContext, SecurityContextGuard, SecurityExpression, User,
        UserDetails, UserRole, UserService,
    };

    // CSRF re-exports / CSRF重新导出
    pub use super::csrf::{
        CookieCsrfTokenRepository, CsrfProtectionConfig, CsrfToken, CsrfTokenRepository,
        CsrfValidator, InMemoryCsrfTokenRepository,
    };
}

/// Version of the security module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default role prefix
/// 默认角色前缀
pub const DEFAULT_ROLE_PREFIX: &str = "ROLE_";

/// Anonymous user principal
/// 匿名用户主体
pub const ANONYMOUS_USER: &str = "anonymousUser";

/// Remember me key
/// 记住我密钥
pub const REMEMBER_ME_KEY: &str = "remember_me";
