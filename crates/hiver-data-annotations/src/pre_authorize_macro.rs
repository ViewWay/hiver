//! @PreAuthorize 宏实现
//! @PreAuthorize Macro Implementation
//!
//! 提供方法级权限检查注解
//! Provides method-level permission checking annotation

use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// @PreAuthorize 属性
/// @PreAuthorize Attributes
#[derive(Debug, FromMeta)]
#[darling(attributes(authorization))]
pub struct PreAuthorizeAttrs {
    /// 权限表达式 / Permission expression
    pub expression: String,
}

/// @PreAuthorize 注解
/// @PreAuthorize Annotation
///
/// 在方法执行前检查权限
/// Check permissions before method execution
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::PreAuthorize;
///
/// impl UserService {
///     #[PreAuthorize("has_role('ADMIN')")]
///     async fn delete_user(&self, id: i64) -> Result<(), Error> {
///         // 只有 ADMIN 角色才能执行 / Only ADMIN role can execute
///         self.repository.delete(id).await
///     }
///
///     #[PreAuthorize("has_role('ADMIN') or #id == auth.user_id()")]
///     async fn update_profile(&self, auth: &AuthContext, id: i64, data: UpdateData) -> Result<(), Error> {
///         // 管理员或本人可以修改 / Admin or owner can modify
///         self.repository.update(id, data).await
///     }
/// }
/// ```
#[cfg(feature = "security")]
pub fn pre_authorize_macro_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let attr_str = attr.to_string();
    let expression = attr_str
        .trim_matches('"')
        .to_string();

    let visibility = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let attrs = &input.attrs;

    let expanded = quote! {
        #(#attrs)*
        #visibility #sig {
            let __hiver_sec_ctx = ::hiver_security::context::get_security_context()
                .unwrap_or_else(|| std::sync::Arc::new(::hiver_security::SecurityContext::new()));
            let __hiver_sec_opts = ::hiver_security::pre_authorize::PreAuthorizeOptions::new()
                .add_expression_string(#expression);
            if !__hiver_sec_opts.evaluate(__hiver_sec_ctx.as_ref()).await {
                panic!("Access denied by @PreAuthorize: {}", #expression);
            }
            #block
        }
    };

    TokenStream::from(expanded)
}

// Fallback implementation when security feature is not enabled
// 当未启用 security feature 时的后备实现
#[cfg(not(feature = "security"))]
pub fn pre_authorize_macro_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // When security feature is disabled, just return the item unchanged
    // 当未启用 security feature 时，直接返回原项
    item
}
