use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemImpl, parse_macro_input};

pub fn controller_advice(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let impl_item = &input;
    let type_name = &impl_item.self_ty;

    let expanded = quote! {
        #impl_item

        #[automatically_derived]
        impl #type_name {
            pub fn get_exception_handlers() -> ::nexus_http::exception::ExceptionHandlerRegistry {
                ::nexus_http::exception::ExceptionHandlerRegistry::new()
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn rest_controller_advice(_attr: TokenStream, item: TokenStream) -> TokenStream {
    controller_advice(_attr, item)
}

pub fn exception_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let expanded = quote! {
        #input

        #[automatically_derived]
        impl #func_name {
            pub fn exception_types() -> Vec<std::any::TypeId> {
                vec![]
            }
        }
    };

    TokenStream::from(expanded)
}
