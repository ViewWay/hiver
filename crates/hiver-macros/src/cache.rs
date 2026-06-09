use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Pat, ReturnType, parse_macro_input};

struct CacheAttr
{
    cache_name: String,
    key_template: Option<String>,
}

fn parse_cache_attr(attr: TokenStream) -> CacheAttr
{
    let s = attr.to_string();
    let mut cache_name = "default".to_string();
    let mut key_template = None;
    for part in s.split(',')
    {
        let part = part.trim();
        if part.starts_with("key")
        {
            if let Some((_, v)) = part.split_once('=')
            {
                key_template = Some(v.trim().trim_matches('"').to_string());
            }
        }
        else if !part.is_empty() && !part.contains('=')
        {
            cache_name = part.trim_matches('"').to_string();
        }
    }
    CacheAttr {
        cache_name,
        key_template,
    }
}

fn param_idents(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Vec<syn::Ident>
{
    inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg
                && let Pat::Ident(pat_ident) = &*pat_type.pat
            {
                return Some(pat_ident.ident.clone());
            }
            None
        })
        .collect()
}

fn build_key_expr(
    fn_name: &syn::Ident,
    params: &[syn::Ident],
    template: Option<&str>,
) -> proc_macro2::TokenStream
{
    if let Some(tpl) = template
    {
        let mut fmt = tpl.to_string();
        for p in params
        {
            fmt = fmt.replace(&format!("#{p}"), &format!("{{{p}}}"));
        }
        quote! { format!(#fmt, #(#params = #params),*) }
    }
    else if params.is_empty()
    {
        quote! { stringify!(#fn_name).to_string() }
    }
    else
    {
        quote! { format!(concat!(stringify!(#fn_name), ":", "{:?}"), (#(#params),*)) }
    }
}

fn return_type_tokens(output: &ReturnType) -> proc_macro2::TokenStream
{
    match output
    {
        ReturnType::Type(_, ty) => quote! { #ty },
        ReturnType::Default => quote! { () },
    }
}

pub fn cacheable(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let attr = parse_cache_attr(attr);
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_async = &input.sig.asyncness;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;
    let params = param_idents(fn_inputs);
    let key_expr = build_key_expr(fn_name, &params, attr.key_template.as_deref());
    let ret_ty = return_type_tokens(fn_output);
    let cache_name = attr.cache_name;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            let __cache_key: String = #key_expr;
            if let Some(__raw) = ::hiver_cache::cache_get(#cache_name, &__cache_key).await {
                if let Ok(__hit) = ::serde_json::from_str::<#ret_ty>(&__raw) {
                    return __hit;
                }
            }

            let __result: #ret_ty = async move #fn_block.await;

            if let Ok(__json) = ::serde_json::to_string(&__result) {
                ::hiver_cache::cache_put(#cache_name, __cache_key, __json).await;
            }

            __result
        }
    };

    TokenStream::from(expanded)
}

pub fn cache_evict(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let attr = parse_cache_attr(attr);
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_async = &input.sig.asyncness;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;
    let params = param_idents(fn_inputs);
    let key_expr = build_key_expr(fn_name, &params, attr.key_template.as_deref());
    let cache_name = attr.cache_name;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            let __result = async move #fn_block.await;
            let __cache_key: String = #key_expr;
            ::hiver_cache::cache_evict_key(#cache_name, &__cache_key).await;
            __result
        }
    };

    TokenStream::from(expanded)
}

pub fn cache_put(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let attr = parse_cache_attr(attr);
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_async = &input.sig.asyncness;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;
    let params = param_idents(fn_inputs);
    let key_expr = build_key_expr(fn_name, &params, attr.key_template.as_deref());
    let ret_ty = return_type_tokens(fn_output);
    let cache_name = attr.cache_name;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            let __result: #ret_ty = async move #fn_block.await;
            let __cache_key: String = #key_expr;
            if let Ok(__json) = ::serde_json::to_string(&__result) {
                ::hiver_cache::cache_put(#cache_name, __cache_key, __json).await;
            }
            __result
        }
    };

    TokenStream::from(expanded)
}

pub fn cache_config(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}

pub fn caching(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}
