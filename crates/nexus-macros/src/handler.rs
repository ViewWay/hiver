//! Handler attribute macro implementation
//! Handler属性宏实现
//!
//! # Overview / 概述
//!
//! This module provides the `#[handler]` procedural macro for defining
//! async HTTP handler functions with automatic parameter extraction.
//!
//! 本模块提供 `#[handler]` 过程宏，用于定义带自动参数提取的
//! 异步HTTP处理函数。
//!
//! The macro:
//! - Validates that the function is `async`
//! - Generates parameter extraction via `FromRequest`
//! - Generates a wrapper function that accepts `Request`, extracts params,
//!   calls the original function, and converts the result via `IntoResponse`

use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, parse_macro_input};

/// Implementation of the `#[handler]` attribute macro
/// `#[handler]` 属性宏的实现
///
/// Equivalent to Spring's handler method with automatic parameter resolution.
/// 等价于Spring的handler方法（带自动参数解析）。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::handler;
/// use nexus_http::{Request, FromRequest, IntoResponse};
///
/// #[handler]
/// async fn create_user(body: CreateUserRequest) -> impl IntoResponse {
///     // body is automatically extracted from the request
///     // body 自动从请求中提取
///     nexus_http::Response::builder()
///         .status(201)
///         .body(format!("Created user: {:?}", body))
///         .unwrap()
/// }
/// ```
pub(crate) fn handler_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Validate the function is async
    // 验证函数是异步的
    if input.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &input.sig.ident,
            "#[handler] can only be applied to async functions / #[handler] 只能应用于异步函数",
        )
        .to_compile_error()
        .into();
    }

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;

    // Collect parameter names and types for extraction
    // 收集参数名和类型用于提取
    let param_info: Vec<_> = fn_inputs
        .iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                let pat = &*pat_type.pat;
                let ty = &*pat_type.ty;
                (pat.clone(), ty.clone())
            } else {
                // Skip `self` parameters (shouldn't appear in handlers)
                panic!("#[handler] does not support self parameters / #[handler] 不支持 self 参数");
            }
        })
        .collect();

    // Generate extraction statements for each parameter
    // 为每个参数生成提取语句
    let extract_statements = param_info.iter().map(|(pat, ty)| {
        let ty = ty;
        quote! {
            let #pat: #ty = match <#ty as nexus_http::FromRequest>::from_request(&req).await {
                Ok(val) => val,
                Err(e) => {
                    return nexus_http::IntoResponse::into_response(
                        nexus_http::Response::builder()
                            .status(nexus_http::StatusCode::BAD_REQUEST)
                            .body(format!("Parameter extraction failed: {:?}", e))
                            .unwrap_or_else(|_| nexus_http::Response::new(nexus_http::Body::empty()))
                    );
                }
            };
        }
    });

    // Collect parameter names for the function call
    // 收集参数名用于函数调用
    let param_names: Vec<_> = param_info.iter().map(|(pat, _)| pat).collect();

    // Generate the original function and the wrapper
    // 生成原始函数和包装器
    let expanded = quote! {
        // Original function (kept as-is)
        // 原始函数（保持不变）
        #(#fn_attrs)*
        #fn_vis async fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }

        // Wrapper function that accepts a Request and dispatches to the original
        // 接受 Request 并分发到原始函数的包装器
        ::nexus_http::handler::register_handler(
            stringify!(#fn_name),
            |req: nexus_http::Request| async move {
                // Extract all parameters from the request via FromRequest
                // 通过 FromRequest 从请求中提取所有参数
                #(#extract_statements)*

                // Call the original handler function
                // 调用原始处理函数
                let result = #fn_name(#(#param_names),*).await;

                // Convert the result into a Response via IntoResponse
                // 通过 IntoResponse 将结果转换为 Response
                nexus_http::IntoResponse::into_response(result)
            }
        );
    };

    TokenStream::from(expanded)
}
