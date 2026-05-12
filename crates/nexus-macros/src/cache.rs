use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn cacheable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let cache_name = if attr.is_empty() {
        quote! { "default" }
    } else {
        let cache_name = parse_macro_input!(attr as syn::LitStr);
        quote! { #cache_name }
    };

    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_name_inner = quote::format_ident!("{}_inner", fn_name);

    let expanded = quote! {
        fn #fn_name_inner() {
        }

        #[allow(dead_code)]
        const #fn_name: &str = #fn_name_str;
        const CACHE_NAME: &str = #cache_name;

        #input
    };

    TokenStream::from(expanded)
}

pub fn cache_evict(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

pub fn cache_put(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

pub fn cache_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn caching(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
