//! Derive macros module
//! 派生宏模块
//!
//! # Overview / 概述
//!
//! This module provides derive macros for common traits.
//! 本模块提供常见trait的派生宏。

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Fields, Ident, Token, parse::ParseStream, parse_macro_input,
    punctuated::Punctuated,
};

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
/// use hiver_macros::FromRequest;
/// use hiver_http::{Request, FromRequest as HttpFromRequest};
///
/// #[derive(FromRequest)]
/// struct UserQuery {
///     name: String,
///     age: u32,
/// }
/// ```
pub fn from_request(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = match &input.data
    {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ =>
        {
            return syn::Error::new_spanned(
                struct_name,
                "FromRequest can only be derived for structs with named fields",
            )
            .to_compile_error()
            .into();
        },
    };

    let field_names: Vec<_> = fields.iter().filter_map(|f| f.ident.as_ref()).collect();

    let expanded = quote! {
        #[automatically_derived]
        impl ::hiver_http::FromRequest for #struct_name {
            async fn from_request(req: &Request) -> ::hiver_http::Result<Self> {
                Ok(#struct_name {
                    #(
                        #field_names: ::hiver_http::FromRequest::from_request(req).await?,
                    )*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Bean trait with optional dependency declaration.
/// Bean trait 的派生宏，支持可选的依赖声明。
///
/// # Attributes / 属性
///
/// - `#[bean(depends(Type1, Type2, ...))]` — declare bean dependencies
///
/// # Examples / 示例
///
/// ```rust,ignore
/// // Simple bean without dependencies / 无依赖的简单 bean
/// #[derive(Bean)]
/// struct SimpleService;
///
/// // Bean with declared dependencies / 带声明的依赖
/// #[derive(Bean)]
/// #[bean(depends(UserRepository, EmailService))]
/// struct UserService { /* ... */ }
/// ```
pub fn bean_derive(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Parse #[bean(depends(Type1, Type2, ...))] helper attribute
    let deps = parse_bean_depends(&input.attrs);

    let dep_impl = if deps.is_empty()
    {
        // No dependencies declared — skip BeanDependencies impl
        quote! {}
    }
    else
    {
        let type_infos: Vec<_> = deps
            .iter()
            .map(|dep| {
                quote! {
                    ::hiver_core::DependencyInfo {
                        type_id: std::any::TypeId::of::<#dep>(),
                        type_name: std::any::type_name::<#dep>(),
                    }
                }
            })
            .collect();

        quote! {
            #[automatically_derived]
            impl ::hiver_core::BeanDependencies for #name
            {
                fn dependencies() -> Vec<::hiver_core::DependencyInfo>
                {
                    vec![#(#type_infos),*]
                }
            }
        }
    };

    let expanded = quote! {
        #[automatically_derived]
        impl ::hiver_core::Bean for #name {}

        #dep_impl
    };

    TokenStream::from(expanded)
}

/// Parse `#[bean(depends(Type1, Type2))]` from derive helper attributes.
fn parse_bean_depends(attrs: &[syn::Attribute]) -> Vec<Ident>
{
    for attr in attrs
    {
        if !attr.path().is_ident("bean")
        {
            continue;
        }

        let result: Result<Vec<Ident>, syn::Error> = attr.parse_args_with(|input: ParseStream| {
            // Parse: depends(Type1, Type2, ...)
            let keyword: Ident = input.parse()?;
            if keyword != "depends"
            {
                return Ok(Vec::new());
            }

            let content;
            syn::parenthesized!(content in input);
            let types = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
            Ok(types.into_iter().collect())
        });

        if let Ok(deps) = result
        {
            return deps;
        }
    }
    Vec::new()
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
/// use hiver_macros::IntoResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize, IntoResponse)]
/// struct User {
///     id: u32,
///     name: String,
/// }
/// ```
pub fn into_response(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let expanded = quote! {
        #[automatically_derived]
        impl ::hiver_http::IntoResponse for #struct_name {
            fn into_response(self) -> ::hiver_http::Response {
                // Try to serialize as JSON
                // 尝试序列化为JSON
                match serde_json::to_vec(&self) {
                    Ok(json) => ::hiver_http::Response::builder()
                        .status(::hiver_http::StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(json)
                        .unwrap(),
                    Err(_) => ::hiver_http::Response::builder()
                        .status(::hiver_http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body(::hiver_http::Body::from("Failed to serialize response"))
                        .unwrap(),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
