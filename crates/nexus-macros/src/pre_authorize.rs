use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[allow(dead_code)]
fn wrap_pre_authorize(expr_lit: &str, item: TokenStream) -> TokenStream {
    let expr_lit = syn::LitStr::new(expr_lit, proc_macro2::Span::call_site());

    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_async = &input.sig.asyncness;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            use ::nexus_security::context::get_security_context;
            use ::nexus_security::pre_authorize::SecurityExpression;

            let Some(__ctx) = get_security_context() else {
                ::core::panic!("access denied: no security context (expression: {})", #expr_lit);
            };

            let __exprs = SecurityExpression::parse(#expr_lit);
            let mut __allowed = true;
            for __expr in &__exprs {
                if !__expr.evaluate(__ctx.as_ref()).await {
                    __allowed = false;
                    break;
                }
            }
            if !__allowed {
                ::core::panic!("access denied: {}", #expr_lit);
            }

            async move #fn_block.await
        }
    };

    TokenStream::from(expanded)
}

#[allow(dead_code)]
pub fn pre_authorize(attr: TokenStream, item: TokenStream) -> TokenStream {
    let expr = if attr.is_empty() {
        "isAuthenticated()".to_string()
    } else {
        attr.to_string().trim_matches('"').to_string()
    };
    wrap_pre_authorize(&expr, item)
}

#[allow(dead_code)]
pub fn secured(attr: TokenStream, item: TokenStream) -> TokenStream {
    let role = attr.to_string().trim_matches('"').to_string();
    wrap_pre_authorize(&format!("hasRole('{role}')"), item)
}

#[allow(dead_code)]
pub fn roles_allowed(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_s = attr.to_string();
    let roles: Vec<String> = attr_s
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let expr = roles
        .iter()
        .map(|r| format!("hasRole('{r}')"))
        .collect::<Vec<_>>()
        .join(" or ");
    wrap_pre_authorize(&expr, item)
}
