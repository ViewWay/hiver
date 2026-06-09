use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn scheduled(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let schedule_expr = parse_schedule_args(&attr);

    let expanded = quote! {
        #input

        #[automatically_derived]
        impl #func_name {
            pub fn schedule(self) {
                use hiver_runtime::time::Duration;
                use hiver_runtime::spawn;

                spawn(async move {
                    #schedule_expr
                });
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_schedule_args(attr: &TokenStream) -> proc_macro2::TokenStream
{
    let attr_str = attr.to_string();

    if attr_str.contains("cron")
    {
        let interval_ms: u64 = 60_000;
        quote! {
            loop {
                self().await;
                hiver_runtime::sleep(hiver_runtime::Duration::from_millis(#interval_ms)).await;
            }
        }
    }
    else if attr_str.contains("fixed_rate")
    {
        quote! {
            loop {
                self().await;
                tokio::time::sleep(Duration::from_millis(5000)).await;
            }
        }
    }
    else if attr_str.contains("fixed_delay")
    {
        quote! {
            loop {
                self().await;
                tokio::time::sleep(Duration::from_millis(5000)).await;
            }
        }
    }
    else
    {
        quote! {
            self().await;
        }
    }
}

pub fn async_fn(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as ItemFn);

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

pub fn slf4j(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as syn::ItemStruct);
    let name = &input.ident;

    let has_log_field = input
        .fields
        .iter()
        .any(|f| f.ident.as_ref().is_some_and(|i| *i == "log"));

    if has_log_field
    {
        return TokenStream::from(quote! { #input });
    }

    let expanded = quote! {
        #input

        impl #name {
            fn log(&self) -> hiver_observability::log::LoggerHandle {
                hiver_observability::log::LoggerFactory::get_for::<#name>()
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn logger(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let expanded = quote! {
        #input

        let log = hiver_observability::log::LoggerFactory::get(stringify!(#func_name));
    };

    TokenStream::from(expanded)
}
