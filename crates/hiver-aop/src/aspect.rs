//! @Aspect attribute macro
//! @Aspect 属性宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Implements #[Aspect] attribute macro
/// 实现 #[Aspect] 属性宏
///
/// Marks a struct as an AOP aspect
/// 将结构体标记为 AOP 切面
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::Aspect;
///
/// #[Aspect]
/// struct LoggingAspect;
/// ```
pub(crate) fn impl_aspect(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    // Generate aspect marker trait implementation
    // 生成切面标记 trait 实现
    let expanded = quote! {
        #input

        impl #name {
            /// Returns the aspect name
            /// 返回切面名称
            pub fn aspect_name() -> &'static str {
                stringify!(#name)
            }

            /// Returns the aspect order (for multiple aspects)
            /// 返回切面顺序（用于多个切面）
            pub fn aspect_order() -> i32 {
                0
            }
        }
    };

    TokenStream::from(expanded)
}
