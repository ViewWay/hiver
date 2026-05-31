use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ItemFn, ItemStatic, ItemStruct, ItemTrait, parse_macro_input};

pub fn hiver_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn run() -> anyhow::Result<()> {
                use hiver_starter::core::{
                    ApplicationContext, AutoConfigurationLoader,
                    AutoConfigurationRegistry, CoreAutoConfiguration,
                    AutoConfiguration, logging,
                };
                use hiver_starter::web::{
                    WebServerAutoConfiguration, RouterAutoConfiguration,
                    MiddlewareAutoConfiguration,
                };
                use std::time::Instant;

                fn format_timestamp() -> String {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let duration = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default();
                    let secs = duration.as_secs();
                    let millis = duration.subsec_millis();

                    let days_since_epoch = secs / 86400;
                    let year = 1970 + (days_since_epoch / 365);
                    let day_of_year = (days_since_epoch % 365) as u32;
                    let month = (day_of_year / 30) + 1;
                    let day = (day_of_year % 30) + 1;

                    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {:03}",
                        year, month, day,
                        (secs % 86400 / 3600) as u32,
                        (secs % 3600 / 60) as u32,
                        (secs % 60) as u32,
                        millis)
                }

                let start_time = Instant::now();

                let mut ctx = ApplicationContext::new();

                let mut registry = AutoConfigurationRegistry::new();
                let _ = registry.load_from_defaults();

                let mut configs: Vec<Box<dyn AutoConfiguration>> = vec![
                    Box::new(CoreAutoConfiguration::new()),
                    Box::new(WebServerAutoConfiguration::from_config(&ctx)),
                    Box::new(RouterAutoConfiguration::new()),
                    Box::new(MiddlewareAutoConfiguration::from_config(&ctx)),
                ];

                configs.sort_by_key(|c| c.order());

                for config in &configs {
                    if config.condition() {
                        config.configure(&mut ctx)?;
                    }
                }

                let rt = ::tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    ctx.start().await
                })?;

                let elapsed = start_time.elapsed().as_millis();
                let class_name = "hiver.Application";
                let timestamp = format_timestamp();
                println!();
                println!(
                    "{} {} {} --- [           main] {} : Tomcat started on port(s): {} (http)",
                    timestamp,
                    "\x1b[32mINFO\x1b[0m",
                    std::process::id(),
                    "\x1b[90mo.s.b.w.e.tomcat.TomcatWebServer\x1b[0m",
                    "\x1b[36m8080\x1b[0m"
                );
                println!(
                    "{} {} {} --- [           main] {} : Started Application in {} seconds (JVM running for {})",
                    timestamp,
                    "\x1b[32mINFO\x1b[0m",
                    std::process::id(),
                    class_name,
                    format!("\x1b[36m{}.{:03}\x1b[0m", elapsed / 1000, elapsed % 1000),
                    format!("\x1b[36m{}.{:03}\x1b[0m", elapsed / 1000, elapsed % 1000)
                );
                println!();

                Ok(())
            }

            pub fn context() -> anyhow::Result<ApplicationContext> {
                Ok(ApplicationContext::new())
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn run() -> std::io::Result<()> {
                use hiver_runtime::Runtime;

                let runtime = Runtime::new()?;
                runtime.block_on(async {
                    hiver_http::Server::new().run().await
                })
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let expanded = quote! {
        #input

        impl #input {
            pub fn register() -> hiver_router::Router {
                hiver_router::Router::new()
                    .prefix(concat!("/", stringify!(#input)))
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    let fields = &input.fields;

    let field_names: Vec<_> = fields.iter().filter_map(|f| f.ident.as_ref()).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let bean_reg = crate::bean_register::generate_bean_registration(
        &input,
        quote! { ::hiver_starter::core::registry::BeanScope::Singleton },
    );

    let expanded = quote! {
        #input

        impl #name {
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }
        }

        impl ::std::convert::Into<::std::sync::Arc<Self>> for #name {
            fn into(self) -> ::std::sync::Arc<Self> {
                ::std::sync::Arc::new(self)
            }
        }

        #bean_reg
    };

    TokenStream::from(expanded)
}

pub fn repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(input) = syn::parse::<ItemStruct>(item.clone()) {
        let _name = &input.ident;
        let bean_reg = crate::bean_register::generate_bean_registration(
            &input,
            quote! { ::hiver_starter::core::registry::BeanScope::Singleton },
        );
        let expanded = quote! {
            #input
            #bean_reg
        };
        return TokenStream::from(expanded);
    }

    let input = parse_macro_input!(item as ItemTrait);
    let expanded = quote! { #input };
    TokenStream::from(expanded)
}

pub fn config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let prefix = if attr.is_empty() {
        Ident::new("config", Span::call_site())
    } else {
        parse_macro_input!(attr as ConfigArgs).prefix
    };

    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn load() -> Result<Self, config::ConfigError> {
                let mut cfg = config::Config::builder();

                cfg = cfg.add_source(config::File::with_name("application"));

                cfg = cfg.add_source(
                    config::Environment::with_prefix(stringify!(#prefix))
                        .separator("__")
                        .try_parsing(true)
                );

                let config = cfg.build()?;
                config.try_deserialize()
            }
        }
    };

    TokenStream::from(expanded)
}

struct ConfigArgs {
    prefix: Ident,
}

impl Parse for ConfigArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _eq_token = syn::token::Eq::parse(input)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitStr) {
            let lit_str: syn::LitStr = input.parse()?;
            Ok(Self {
                prefix: Ident::new(&lit_str.value(), lit_str.span()),
            })
        } else {
            let ident: Ident = input.parse()?;
            Ok(Self { prefix: ident })
        }
    }
}

pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let bean_reg = crate::bean_register::generate_bean_registration(
        &input,
        quote! { ::hiver_starter::core::registry::BeanScope::Singleton },
    );

    let expanded = quote! {
        #input

        impl #name {
            pub fn init() -> Self {
                ::std::default::Default::default()
            }
        }

        #bean_reg
    };

    TokenStream::from(expanded)
}

pub fn autowired(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn configuration(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn register_beans() {
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn bean(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let expanded = quote! {
        #input

        #[automatically_derived]
        impl #func_name {
            pub fn bean_name() -> &'static str {
                stringify!(#func_name)
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let profile = parse_macro_input!(attr as syn::LitStr);
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    let expanded = quote! {
        #input

        impl #struct_name {
            fn is_active_profile() -> bool {
                const REQUIRED_PROFILE: &str = #profile;

                std::env::var("SPRING_PROFILES_ACTIVE")
                    .map(|active_profiles| {
                        for active in active_profiles.split(',') {
                            let active = active.trim();
                            if active == REQUIRED_PROFILE || active == "default" {
                                return true;
                            }
                        }
                        false
                    })
                    .unwrap_or_else(|_| {
                        REQUIRED_PROFILE == "default"
                    })
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn value(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStatic);

    let attr_str = attr.to_string();

    let property_name = if attr_str.contains("${") && attr_str.contains('}') {
        let start = attr_str.find("${").map_or(0, |i| i + 2);
        let end = attr_str.find('}').unwrap_or(attr_str.len());

        let prop = &attr_str[start..end];
        if let Some(colon_pos) = prop.find(':') {
            prop[..colon_pos].to_string()
        } else {
            prop.to_string()
        }
    } else {
        attr_str.trim().to_string()
    };

    let default_value = if let Expr::Lit(expr_lit) = &*input.expr {
        Some(expr_lit.clone())
    } else {
        None
    };

    let name = &input.ident;
    let ty = &input.ty;

    let expanded = if let Some(default) = default_value {
        quote! {
            #input

            impl #name {
                fn load_value() -> #ty {
                    std::env::var(#property_name)
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_else(|| #default)
                }
            }
        }
    } else {
        quote! {
            #input
        }
    };

    TokenStream::from(expanded)
}
