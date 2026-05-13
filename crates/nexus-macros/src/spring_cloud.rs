use proc_macro::TokenStream;

pub fn query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn native_query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn read_only(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn modifying(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn jdbc_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn r2dbc_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn mongo_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn redis_hash(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn elasticsearch_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn configuration_properties(attr: TokenStream, item: TokenStream) -> TokenStream {
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    use syn::{Fields, ItemStruct, parse_macro_input};

    let prefix = if attr.is_empty() {
        String::new()
    } else {
        attr.to_string()
            .replace("prefix", "")
            .replace('=', "")
            .trim_matches('"')
            .trim()
            .to_string()
    };

    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let field_bindings = if let Fields::Named(fields) = &input.fields {
        fields.named.iter().map(|f| {
            let fname = f.ident.as_ref().unwrap();
            let key_snake = fname.to_string();
            let key_kebab = key_snake.replace('_', "-");
            let full_key = if prefix.is_empty() {
                key_kebab.clone()
            } else {
                format!("{prefix}.{key_kebab}")
            };
            quote! {
                #fname: loader.get(#full_key)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_default()
            }
        }).collect::<Vec<_>>()
    } else {
        vec![]
    };

    let expanded = quote! {
        #input

        impl #name {
            /// Load from `ApplicationContext` configuration loader.
            /// 从 `ApplicationContext` 配置加载器加载。
            pub fn from_context(ctx: &::nexus_starter::core::ApplicationContext) -> Self {
                let loader = ctx.config_loader();
                Self {
                    #(#field_bindings),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn enable_configuration_properties(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn configuration_properties_scan(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn ignore_unknown_properties(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn default_value(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn nested_configuration_property(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn endpoint_actuator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn read_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn write_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn delete_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ─────────────────────────────────────────────────────────────────────────────
// Feign declarative HTTP client
// ─────────────────────────────────────────────────────────────────────────────

pub fn feign_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    use proc_macro2::TokenStream as TokenStream2;
    use quote::{format_ident, quote};
    use syn::{FnArg, ItemTrait, LitStr, Pat, TraitItem, parse_macro_input};

    let trait_def = parse_macro_input!(item as ItemTrait);
    let trait_name = &trait_def.ident;
    let vis = &trait_def.vis;
    let client_name = format_ident!("{}Client", trait_name);

    let _default_base_url: String = {
        let s = attr.to_string();
        feign_extract_url(&s)
    };

    let mut method_impls = Vec::<TokenStream2>::new();

    for trait_item in &trait_def.items {
        let TraitItem::Fn(method) = trait_item else { continue };
        let method_name = &method.sig.ident;

        let mut http_meth_tok: Option<TokenStream2> = None;
        let mut path_tpl = String::from("/");
        let mut has_body_attr = false;

        for a in &method.attrs {
            let nm = a.path().get_ident().map(|i| i.to_string());
            match nm.as_deref() {
                Some(n @ ("feign_get" | "feign_post" | "feign_put" | "feign_delete" | "feign_patch")) => {
                    if let Ok(lit) = a.parse_args::<LitStr>() {
                        path_tpl = lit.value();
                    }
                    http_meth_tok = Some(match n {
                        "feign_get"    => quote! { ::reqwest::Method::GET },
                        "feign_post"   => quote! { ::reqwest::Method::POST },
                        "feign_put"    => quote! { ::reqwest::Method::PUT },
                        "feign_delete" => quote! { ::reqwest::Method::DELETE },
                        _              => quote! { ::reqwest::Method::PATCH },
                    });
                }
                Some("feign_body") => { has_body_attr = true; }
                _ => {}
            }
        }

        let Some(http_meth) = http_meth_tok else { continue };

        let mut path_params: Vec<syn::Ident> = Vec::new();
        let mut body_param:  Option<syn::Ident> = None;
        let mut query_params: Vec<(String, syn::Ident)> = Vec::new();

        for arg in &method.sig.inputs {
            let FnArg::Typed(pt) = arg else { continue };
            let Pat::Ident(pi) = pt.pat.as_ref() else { continue };
            let ident = pi.ident.clone();

            let is_body = has_body_attr
                || pt.attrs.iter().any(|a| a.path().get_ident().map(|i| i == "feign_body").unwrap_or(false));
            let q = pt.attrs.iter().find_map(|a| {
                if a.path().get_ident().map(|i| i == "feign_query").unwrap_or(false) {
                    a.parse_args::<LitStr>().ok().map(|l| l.value())
                } else {
                    None
                }
            });

            if is_body {
                body_param = Some(ident);
            } else if let Some(qn) = q {
                query_params.push((qn, ident));
            } else {
                path_params.push(ident);
            }
        }

        let (url_fmt, url_args) = feign_build_url_fmt(&path_tpl, &path_params);

        let query_code = if query_params.is_empty() {
            quote! {}
        } else {
            let ks: Vec<_> = query_params.iter().map(|(k,_)| k.clone()).collect();
            let vs: Vec<_> = query_params.iter().map(|(_,v)| v.clone()).collect();
            quote! {
                let url = {
                    let mut qp: Vec<(&str, String)> = Vec::new();
                    #( qp.push((#ks, format!("{}", #vs))); )*
                    let qs = qp.iter().map(|(k,v)| format!("{}={}",k,v)).collect::<Vec<_>>().join("&");
                    if qs.is_empty() { url } else { format!("{}?{}", url, qs) }
                };
            }
        };

        let ret = &method.sig.output;

        let body = if let Some(bp) = &body_param {
            quote! {
                let url = format!(concat!("{}", #url_fmt), self.base_url #(, #url_args)*);
                #query_code
                ::nexus_cloud::feign::execute_request_with_body(&self.client, #http_meth, &url, &#bp, vec![]).await
            }
        } else {
            quote! {
                let url = format!(concat!("{}", #url_fmt), self.base_url #(, #url_args)*);
                #query_code
                ::nexus_cloud::feign::execute_request(&self.client, #http_meth, &url, vec![]).await
            }
        };

        let clean_inputs: Vec<TokenStream2> = method.sig.inputs.iter().map(|arg| match arg {
            FnArg::Receiver(r) => quote! { #r },
            FnArg::Typed(pt) => {
                let clean_attrs: Vec<_> = pt.attrs.iter()
                    .filter(|a| !a.path().get_ident().map(|i| i.to_string().starts_with("feign_")).unwrap_or(false))
                    .collect();
                let p = &pt.pat; let t = &pt.ty;
                quote! { #(#clean_attrs)* #p: #t }
            }
        }).collect();

        method_impls.push(quote! {
            async fn #method_name(#(#clean_inputs),*) #ret { #body }
        });
    }

    let clean_trait = feign_clean_trait(&trait_def);

    quote! {
        #clean_trait

        #vis struct #client_name {
            /// Base URL.
            pub base_url: String,
            client: ::reqwest::Client,
        }

        impl #client_name {
            /// Create a new client pointing at `base_url`.
            pub fn new(base_url: impl Into<String>) -> Self {
                Self { base_url: base_url.into(), client: ::reqwest::Client::new() }
            }

            /// Create from [`::nexus_cloud::feign::FeignClientConfig`].
            pub fn from_config(cfg: ::nexus_cloud::feign::FeignClientConfig) -> Result<Self, ::reqwest::Error> {
                Ok(Self { base_url: cfg.base_url.clone(), client: cfg.build_client()? })
            }
        }

        #[::async_trait::async_trait]
        impl #trait_name for #client_name {
            #(#method_impls)*
        }
    }.into()
}

fn feign_extract_url(attr_str: &str) -> String {
    let key = "url =";
    let idx = attr_str.find(key).unwrap_or(0);
    let rest = &attr_str[idx + key.len()..];
    let t = rest.trim().trim_start_matches('"');
    let end = t.find('"').unwrap_or(0);
    t[..end].to_string()
}

fn feign_build_url_fmt(
    path_tpl: &str,
    params: &[syn::Ident],
) -> (String, Vec<proc_macro2::TokenStream>) {
    use quote::quote;
    let mut fmt = path_tpl.to_string();
    let mut args = Vec::new();
    for id in params {
        let placeholder = format!("{{{}}}", id);
        if fmt.contains(&placeholder) {
            fmt = fmt.replace(&placeholder, "{}");
            args.push(quote! { #id });
        }
    }
    (fmt, args)
}

fn feign_clean_trait(trait_def: &syn::ItemTrait) -> proc_macro2::TokenStream {
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    use syn::{FnArg, TraitItem};

    let feign_attrs = ["feign_get","feign_post","feign_put","feign_delete","feign_patch",
                       "feign_body","feign_query","feign_path","feign_header"];
    let is_feign = |a: &syn::Attribute| a.path().get_ident()
        .map(|i| feign_attrs.contains(&i.to_string().as_str()))
        .unwrap_or(false);

    let vis = &trait_def.vis;
    let ident = &trait_def.ident;
    let generics = &trait_def.generics;
    let supertraits = &trait_def.supertraits;

    let items: Vec<TokenStream2> = trait_def.items.iter().map(|item| {
        if let TraitItem::Fn(m) = item {
            let clean_attrs: Vec<_> = m.attrs.iter().filter(|a| !is_feign(a)).collect();
            let asyncness = &m.sig.asyncness;
            let fn_tok = &m.sig.fn_token;
            let fn_id = &m.sig.ident;
            let fn_gen = &m.sig.generics;
            let ret = &m.sig.output;
            let default = &m.default;
            let semi = m.semi_token;
            let inputs: Vec<TokenStream2> = m.sig.inputs.iter().map(|arg| match arg {
                FnArg::Receiver(r) => quote! { #r },
                FnArg::Typed(pt) => {
                    let ca: Vec<_> = pt.attrs.iter().filter(|a| !is_feign(a)).collect();
                    let p = &pt.pat; let t = &pt.ty;
                    quote! { #(#ca)* #p: #t }
                }
            }).collect();
            quote! { #(#clean_attrs)* #asyncness #fn_tok #fn_id #fn_gen (#(#inputs),*) #ret #default #semi }
        } else {
            quote! { #item }
        }
    }).collect();

    let st = if supertraits.is_empty() { quote!{} } else { quote!{ : #supertraits } };
    quote! {
        #[::async_trait::async_trait]
        #vis trait #ident #generics #st { #(#items)* }
    }
}

pub fn feign_get(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
pub fn feign_post(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
pub fn feign_put(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
pub fn feign_delete(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
pub fn feign_path(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
pub fn feign_query(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
pub fn feign_header(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
pub fn feign_body(_attr: TokenStream, item: TokenStream) -> TokenStream { item }

pub fn circuit_breaker_name(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_timeout(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_retry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_configuration(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_decoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_encoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_logger(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_error_decoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_options(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn query_map_encoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn circuit_breaker_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn time_limiter_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn bulkhead_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn retry_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn fallback(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn circuit_breaker(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn bulkhead(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn time_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn retry_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn request_rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn origin_rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn user_rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn throttling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn gateway_filter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn gateway_predicate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn gateway_route(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn gateway_configuration(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
