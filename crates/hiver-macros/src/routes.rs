use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{ItemFn, parse_macro_input};

fn parse_route_path(attr: &TokenStream) -> syn::Result<String>
{
    let attr_str = attr.to_string();

    let path = if attr_str.contains('"')
    {
        let start = attr_str.find('"').unwrap_or(0) + 1;
        let end = attr_str.rfind('"').unwrap_or(attr_str.len());
        attr_str[start..end].to_string()
    }
    else
    {
        attr_str.trim().to_string()
    };

    Ok(path)
}

macro_rules! impl_route_macro {
    ($name:ident, $method:ident, $http_method:expr) => {
        pub fn $name(attr: TokenStream, item: TokenStream) -> TokenStream
        {
            let input = parse_macro_input!(item as ItemFn);
            let func_name = &input.sig.ident;

            let path = match parse_route_path(&attr)
            {
                Ok(p) => p,
                Err(e) => return TokenStream::from(e.to_compile_error()),
            };

            let register_fn = quote::format_ident!("__hiver_route_register_{}", func_name);

            let expanded = quote! {
                #input

                #[allow(non_snake_case)]
                fn #register_fn(router: hiver_router::Router) -> hiver_router::Router {
                    router.route(#path, hiver_http::Method::$method, #func_name)
                }

                #[automatically_derived]
                impl #func_name {
                    /// Register this route onto a Router.
                    /// 将此路由注册到 Router 上。
                    pub fn register_route(router: hiver_router::Router) -> hiver_router::Router {
                        #register_fn(router)
                    }
                }

                ::inventory::submit! {
                    ::hiver_starter::web::route_registry::RouteDescriptor {
                        method: ::hiver_starter::web::route_registry::HttpMethod::$http_method,
                        path: #path,
                        register: #register_fn,
                    }
                }
            };

            TokenStream::from(expanded)
        }
    };
}

impl_route_macro!(get, GET, Get);
impl_route_macro!(post, POST, Post);
impl_route_macro!(put, PUT, Put);
impl_route_macro!(delete, DELETE, Delete);
impl_route_macro!(patch, PATCH, Patch);
impl_route_macro!(head, HEAD, Head);
impl_route_macro!(options, OPTIONS, Options);
impl_route_macro!(trace, TRACE, Trace);

pub fn request_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let attr_str = attr.to_string();
    let path = if attr_str.contains("path")
    {
        if let Some(start) = attr_str.find("path = \"")
        {
            let start = start + 8;
            if let Some(end) = attr_str[start..].find('"')
            {
                attr_str[start..start + end].to_string()
            }
            else
            {
                "/".to_string()
            }
        }
        else
        {
            "/".to_string()
        }
    }
    else
    {
        "/".to_string()
    };

    let method = if attr_str.contains("method")
    {
        if attr_str.contains("GET")
        {
            "GET"
        }
        else if attr_str.contains("POST")
        {
            "POST"
        }
        else if attr_str.contains("PUT")
        {
            "PUT"
        }
        else if attr_str.contains("DELETE")
        {
            "DELETE"
        }
        else if attr_str.contains("PATCH")
        {
            "PATCH"
        }
        else
        {
            "GET"
        }
    }
    else
    {
        "GET"
    };

    let method_ident = Ident::new(method, Span::call_site());

    let expanded = quote! {
        #input

        #[automatically_derived]
        impl #func_name {
            pub fn register_route(router: hiver_router::Router) -> hiver_router::Router {
                router.route(#path, hiver_http::Method::#method_ident, #func_name)
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn cross_origin(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}

pub fn trace_method(attr: TokenStream, item: TokenStream) -> TokenStream
{
    request_mapping(attr, item)
}

pub fn patch_route(attr: TokenStream, item: TokenStream) -> TokenStream
{
    request_mapping(attr, item)
}

pub fn rest_controller(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    super::spring_stereotype::controller(_attr, item)
}

pub fn controller_view(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    super::spring_stereotype::controller(_attr, item)
}

pub fn get_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    get(attr, item)
}

pub fn post_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    post(attr, item)
}

pub fn put_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    put(attr, item)
}

pub fn delete_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    delete(attr, item)
}

pub fn patch_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    patch(attr, item)
}
