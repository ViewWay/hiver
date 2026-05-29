//! Nexus Security - Security framework module
//! Nexus安全 - 安全框架模块
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
//! use nexus_security::{PreAuthorize, Secured, User, Role};
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

#[cfg(test)]
mod tests;

mod auth;
pub mod acl;
pub mod authorization_server;
mod authority;
mod context;
pub mod data_scope;
mod csrf;
mod encoder;
mod error;
mod jwt;
mod oauth2;
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

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        AuditLogger, Authentication, AuthenticationManager, Authority, ConsoleAuditLogger,
        DataScope, DataScopeApply, DataScopeContext, DataScopeMiddleware, DataScopeRule,
        DataScopeType, GrantedAuthority, JwtAuthentication, JwtClaims, JwtTokenProvider, JwtUtil,
        PasswordEncoder, Permission, PermissionEntry, PreAuthorize, RbacConfig, RbacManager,
        RoleEnum, RolePermission, Roles, Secured, SecurityContext, SecurityContextGuard,
        SecurityExpression, User, UserDetails, UserRole, UserService,
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
