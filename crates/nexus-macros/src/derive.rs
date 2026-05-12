//! Derive macros module
//! 派生宏模块
//!
//! # Overview / 概述
//!
//! This module provides derive macros for common traits.
//! 本模块提供常见trait的派生宏。

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

/// Derive macro for FromRequest trait
/// FromRequest trait的派生宏
///
/// Automatically implements FromRequest for structs with named fields.
/// Each field will be extracted from the request using its own FromRequest implementation.
///
/// 为具有命名字段的结构体自动实现FromRequest。
/// 每个字段将使用其自己的FromRequest实现从请求中提取。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::FromRequest;
/// use nexus_http::{Request, FromRequest as HttpFromRequest};
///
/// #[derive(FromRequest)]
/// struct UserQuery {
///     name: String,
///     age: u32,
/// }
/// ```
pub fn from_request(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "FromRequest can only be derived for structs with named fields",
            )
            .to_compile_error()
            .into()
        }
    };

    let field_names: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let expanded = quote! {
        #[automatically_derived]
        impl ::nexus_http::FromRequest for #struct_name {
            async fn from_request(req: &Request) -> ::nexus_http::Result<Self> {
                Ok(#struct_name {
                    #(
                        #field_names: ::nexus_http::FromRequest::from_request(req).await?,
                    )*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for IntoResponse trait
/// IntoResponse trait的派生宏
///
/// Automatically implements IntoResponse for structs by serializing to JSON.
/// The struct must implement Serialize.
///
/// 为结构体自动实现IntoResponse，通过序列化为JSON。
/// 结构体必须实现Serialize。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::IntoResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize, IntoResponse)]
/// struct User {
///     id: u32,
///     name: String,
/// }
/// ```
pub fn into_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let expanded = quote! {
        #[automatically_derived]
        impl ::nexus_http::IntoResponse for #struct_name {
            fn into_response(self) -> ::nexus_http::Response {
                // Try to serialize as JSON
                // 尝试序列化为JSON
                match serde_json::to_vec(&self) {
                    Ok(json) => ::nexus_http::Response::builder()
                        .status(::nexus_http::StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(json)
                        .unwrap(),
                    Err(_) => ::nexus_http::Response::builder()
                        .status(::nexus_http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body(::nexus_http::Body::from("Failed to serialize response"))
                        .unwrap(),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
