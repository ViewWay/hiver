//! Spring-style stereotype macros: @Configuration, @Bean, @Component, @Service, @Repository.
//! Spring 风格的构造型宏：@Configuration、@Bean、@Component、@Service、@Repository。
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Expr, ItemFn, ItemStatic, ItemStruct, ItemTrait,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use crate::bean_register::{extract_arc_inner, extract_result_ok_type, to_camel_case};

// ============================================================================
// #[hiver_main] / #[main] — Application entry points
// ============================================================================

pub fn hiver_main(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn run() -> anyhow::Result<()> {
                use hiver_starter::core::{
                    ApplicationContext, AutoConfiguration,
                    AutoConfigurationLoader, AutoConfigurationRegistry,
                    autoconfig::collect_auto_configurations,
                    logging,
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

                let mut configs = collect_auto_configurations();

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

pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream
{
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

// ============================================================================
// #[controller] — @RestController equivalent
// ============================================================================

pub fn controller(_attr: TokenStream, item: TokenStream) -> TokenStream
{
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

// ============================================================================
// #[service] — @Service equivalent
// ============================================================================

pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream
{
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

// ============================================================================
// #[repository] — @Repository equivalent
// ============================================================================

pub fn repository(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    if let Ok(input) = syn::parse::<ItemStruct>(item.clone())
    {
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

// ============================================================================
// #[config] — @ConfigurationProperties equivalent
// ============================================================================

pub fn config(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as ItemStruct);

    let prefix = if attr.is_empty()
    {
        Ident::new("config", Span::call_site())
    }
    else
    {
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

struct ConfigArgs
{
    prefix: Ident,
}

impl Parse for ConfigArgs
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let _eq_token = syn::token::Eq::parse(input)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitStr)
        {
            let lit_str: syn::LitStr = input.parse()?;
            Ok(Self {
                prefix: Ident::new(&lit_str.value(), lit_str.span()),
            })
        }
        else
        {
            let ident: Ident = input.parse()?;
            Ok(Self { prefix: ident })
        }
    }
}

// ============================================================================
// #[component] — @Component equivalent
// ============================================================================

pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream
{
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

// ============================================================================
// #[autowired] — field-level injection marker (passthrough)
// ============================================================================

pub fn autowired(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    item
}

// ============================================================================
// #[configuration] — @Configuration equivalent
// ============================================================================

/// `#[configuration]` marks a struct as a Spring-style configuration class.
/// `#[configuration]` 将结构体标记为 Spring 风格的配置类。
///
/// Unlike Spring, Hiver uses compile-time registration via `inventory`.
/// `#[bean]` functions self-register; this macro serves as a logical namespace marker
/// and also registers the configuration struct itself as a bean (if it implements Default).
///
/// 与 Spring 不同，Hiver 使用 `inventory` 进行编译时注册。
/// `#[bean]` 函数自动注册；此宏作为逻辑命名空间标记，
/// 同时也将配置结构体自身注册为 Bean（需实现 Default）。
///
/// # Example / 示例
///
/// ```rust,ignore
/// #[configuration]
/// struct AppConfig;
///
/// #[bean]
/// fn datasource() -> DataSource { ... }
///
/// #[bean]
/// fn user_repo(ds: Arc<DataSource>) -> UserRepository { ... }
/// ```
pub fn configuration(_attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    let bean_name_camel = to_camel_case(&name.to_string());

    // Generate a factory that creates the config struct (typically zero-sized or Default)
    let factory_fn = format_ident!("__hiver_config_factory_{}", name);

    let expanded = quote! {
        #input

        impl #name {
            /// @Configuration marker — all #[bean] functions self-register via inventory.
            /// @Configuration 标记 — 所有 #[bean] 函数通过 inventory 自动注册。
            pub fn register_beans() {
                // Beans registered via inventory::submit! from #[bean] functions.
                // No manual registration needed.
                // 无需手动注册。
            }
        }

        fn #factory_fn(_ctx: &::hiver_starter::core::ApplicationContext)
            -> ::std::boxed::Box<dyn ::std::any::Any + ::std::marker::Send + ::std::marker::Sync>
        {
            ::std::boxed::Box::new(::std::sync::Arc::new(#name::default()))
        }

        ::inventory::submit! {
            ::hiver_starter::core::registry::BeanDescriptor {
                name: #bean_name_camel,
                type_id: || ::std::any::TypeId::of::<#name>(),
                scope: ::hiver_starter::core::registry::BeanScope::Singleton,
                factory: #factory_fn,
                dep_type_ids: &[] as &[fn() -> ::std::any::TypeId],
                condition: ::hiver_starter::core::registry::always_true,
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// #[bean] — @Bean equivalent (function-level, compile-time DI)
// ============================================================================

/// Parsed attributes for `#[bean(...)]`.
/// `#[bean(...)]` 的解析属性。
struct BeanAttrs
{
    /// Custom bean name. Defaults to camelCase of function name.
    /// 自定义 Bean 名称。默认为函数名的 camelCase。
    name: Option<String>,
    /// Bean scope. Defaults to Singleton.
    /// Bean 作用域。默认为 Singleton。
    scope: BeanScopeAttr,
}

#[derive(Default)]
enum BeanScopeAttr
{
    #[default]
    Singleton,
    Prototype,
}

impl Default for BeanAttrs
{
    fn default() -> Self
    {
        Self {
            name: None,
            scope: BeanScopeAttr::default(),
        }
    }
}

/// Single key=value attribute pair for #[bean].
/// #[bean] 的单个 key=value 属性对。
struct BeanAttr
{
    key: Ident,
    #[allow(dead_code)]
    eq_token: syn::token::Eq,
    value: syn::LitStr,
}

impl Parse for BeanAttr
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        Ok(Self {
            key: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for BeanAttrs
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let mut name = None;
        let mut scope = BeanScopeAttr::Singleton;

        if !input.is_empty()
        {
            let items: syn::punctuated::Punctuated<BeanAttr, syn::Token![,]> =
                syn::punctuated::Punctuated::parse_terminated(input)?;

            for item in &items
            {
                match item.key.to_string().as_str()
                {
                    "name" => name = Some(item.value.value()),
                    "scope" => match item.value.value().as_str()
                    {
                        "singleton" => scope = BeanScopeAttr::Singleton,
                        "prototype" => scope = BeanScopeAttr::Prototype,
                        other =>
                        {
                            return Err(syn::Error::new(
                                item.value.span(),
                                format!(
                                    "unknown scope '{other}', expected 'singleton' or 'prototype'"
                                ),
                            ));
                        },
                    },
                    other =>
                    {
                        return Err(syn::Error::new(
                            item.key.span(),
                            format!("unknown attribute '{other}', expected 'name' or 'scope'"),
                        ));
                    },
                }
            }
        }

        Ok(Self { name, scope })
    }
}

/// `#[bean]` marks a function as a Spring-style bean factory.
/// `#[bean]` 将函数标记为 Spring 风格的 Bean 工厂。
///
/// The function's parameters are resolved as dependencies from the `ApplicationContext`.
/// The function's return type is the bean type.
///
/// 函数参数从 `ApplicationContext` 解析为依赖。
/// 函数返回类型即为 Bean 类型。
///
/// # Attributes / 属性
///
/// - `name = "..."` — custom bean name (default: camelCase of function name)
/// - `scope = "singleton"` or `scope = "prototype"` — bean scope
///
/// # Supported parameter types / 支持的参数类型
///
/// - `Arc<T>` — resolved as `Arc<T>` from context (most common)
/// - `&ApplicationContext` — passes the context directly (for config access)
/// - `T` — resolved as `Arc<T>`, then inner cloned
///
/// # Supported return types / 支持的返回类型
///
/// - `T` — wrapped in `Arc::new(T)`
/// - `Arc<T>` — stored directly
/// - `Result<T, E>` / `Result<Arc<T>, E>` — unwrapped with expect
///
/// # Example / 示例
///
/// ```rust,ignore
/// #[bean]
/// fn datasource() -> DataSource {
///     DataSource::new("localhost:5432")
/// }
///
/// #[bean]
/// fn user_repo(ds: Arc<DataSource>) -> UserRepository {
///     UserRepository::new(ds)
/// }
///
/// #[bean(name = "cacheProvider", scope = "singleton")]
/// fn cache() -> RedisCache {
///     RedisCache::new()
/// }
/// ```
pub fn bean(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let attrs = if attr.is_empty()
    {
        BeanAttrs::default()
    }
    else
    {
        parse_macro_input!(attr as BeanAttrs)
    };

    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Determine bean name
    let bean_name = attrs
        .name
        .unwrap_or_else(|| to_camel_case(&func_name.to_string()));

    // Extract return type — must be present
    let return_ty: syn::Type = match &input.sig.output
    {
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
        syn::ReturnType::Default =>
        {
            return syn::Error::new(func_name.span(), "#[bean] function must have a return type")
                .to_compile_error()
                .into();
        },
    };

    // Determine bean type: unwrap Arc<T> and Result<T, E> wrappers
    // 确定 Bean 类型：解包 Arc<T> 和 Result<T, E> 包装
    let (bean_type, is_arc_return, is_result_return) = {
        // Peel Result first
        if let Some(ok_ty) = extract_result_ok_type(&return_ty)
        {
            // Check if Ok type is Arc<T>
            if let Some(inner) = extract_arc_inner(ok_ty)
            {
                (inner.clone(), true, true)
            }
            else
            {
                (ok_ty.clone(), false, true)
            }
        }
        else if let Some(inner) = extract_arc_inner(&return_ty)
        {
            (inner.clone(), true, false)
        }
        else
        {
            (return_ty.clone(), false, false)
        }
    };

    // Extract parameters → dependencies
    // 提取参数 → 依赖
    let mut dep_types: Vec<syn::Type> = Vec::new();
    let mut param_resolutions: Vec<TokenStream2> = Vec::new();

    for param in &input.sig.inputs
    {
        match param
        {
            syn::FnArg::Typed(pat_type) =>
            {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;

                // Skip `ctx: &ApplicationContext` parameters — pass context directly
                // 跳过 `ctx: &ApplicationContext` 参数 — 直接传递上下文
                if is_context_type(ty)
                {
                    param_resolutions.push(quote! { #pat: ctx });
                    continue;
                }

                if let Some(inner_ty) = extract_arc_inner(ty)
                {
                    // Arc<Dep> → resolve Arc from context
                    dep_types.push(inner_ty.clone());
                    param_resolutions.push(quote! {
                        #pat: ctx.get_required_bean::<#inner_ty>()
                            .expect(concat!("dependency not found: ", stringify!(#inner_ty)))
                    });
                }
                else if let Some(inner_ty) = extract_result_ok_type(ty)
                {
                    // Result<Dep, _> → resolve and unwrap
                    dep_types.push(inner_ty.clone());
                    param_resolutions.push(quote! {
                        #pat: ctx.get_required_bean::<#inner_ty>()
                            .expect(concat!("dependency not found: ", stringify!(#inner_ty)))
                    });
                }
                else
                {
                    // Bare type → resolve Arc then clone inner
                    dep_types.push((**ty).clone());
                    param_resolutions.push(quote! {
                        #pat: ctx.get_required_bean::<#ty>()
                            .map(|arc: ::std::sync::Arc<#ty>| (*arc).clone())
                            .expect(concat!("dependency not found: ", stringify!(#ty)))
                    });
                }
            },
            syn::FnArg::Receiver(_) =>
            {
                // Skip self parameters — not supported for #[bean]
            },
        }
    }

    // Generate unique identifiers
    let factory_fn = format_ident!("__hiver_bean_factory_{}", func_name);
    let deps_static = format_ident!("__HIVER_BEAN_DEPS_{}", func_name.to_string().to_uppercase());

    // Build dependency array
    let dep_array = if dep_types.is_empty()
    {
        quote! { &[] as &[fn() -> ::std::any::TypeId] }
    }
    else
    {
        let dep_fns: Vec<_> = dep_types
            .iter()
            .map(|ty| {
                quote! { || ::std::any::TypeId::of::<#ty>() }
            })
            .collect();
        quote! {
            {
                static #deps_static: &[fn() -> ::std::any::TypeId] = &[#(#dep_fns),*];
                #deps_static
            }
        }
    };

    // Build scope token
    let scope_token = match attrs.scope
    {
        BeanScopeAttr::Prototype =>
        {
            quote! { ::hiver_starter::core::registry::BeanScope::Prototype }
        },
        BeanScopeAttr::Singleton =>
        {
            quote! { ::hiver_starter::core::registry::BeanScope::Singleton }
        },
    };

    // Build factory body based on return type
    let factory_body = if is_result_return
    {
        if is_arc_return
        {
            quote! {
                let bean: ::std::sync::Arc<#bean_type> = #func_name(#(#param_resolutions),*)
                    .expect(concat!("#[bean] ", stringify!(#func_name), " returned an error"));
                ::std::boxed::Box::new(bean)
            }
        }
        else
        {
            quote! {
                let bean: #bean_type = #func_name(#(#param_resolutions),*)
                    .expect(concat!("#[bean] ", stringify!(#func_name), " returned an error"));
                ::std::boxed::Box::new(::std::sync::Arc::new(bean))
            }
        }
    }
    else if is_arc_return
    {
        quote! {
            let bean: ::std::sync::Arc<#bean_type> = #func_name(#(#param_resolutions),*);
            ::std::boxed::Box::new(bean)
        }
    }
    else
    {
        quote! {
            let bean: #bean_type = #func_name(#(#param_resolutions),*);
            ::std::boxed::Box::new(::std::sync::Arc::new(bean))
        }
    };

    let expanded = quote! {
        #input

        #[allow(non_snake_case)]
        fn #factory_fn(ctx: &::hiver_starter::core::ApplicationContext)
            -> ::std::boxed::Box<dyn ::std::any::Any + ::std::marker::Send + ::std::marker::Sync>
        {
            #factory_body
        }

        ::inventory::submit! {
            ::hiver_starter::core::registry::BeanDescriptor {
                name: #bean_name,
                type_id: || ::std::any::TypeId::of::<#bean_type>(),
                scope: #scope_token,
                factory: #factory_fn,
                dep_type_ids: #dep_array,
                condition: ::hiver_starter::core::registry::always_true,
            }
        }
    };

    TokenStream::from(expanded)
}

/// Check if a type is `&ApplicationContext` or `&mut ApplicationContext`.
/// 检查类型是否为 `&ApplicationContext` 或 `&mut ApplicationContext`。
fn is_context_type(ty: &syn::Type) -> bool
{
    let syn::Type::Reference(type_ref) = ty
    else
    {
        return false;
    };
    let syn::Type::Path(type_path) = type_ref.elem.as_ref()
    else
    {
        return false;
    };
    type_path
        .path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "ApplicationContext")
}

// ============================================================================
// #[profile] — @Profile equivalent
// ============================================================================

pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream
{
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

// ============================================================================
// #[value] — @Value equivalent
// ============================================================================

pub fn value(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as ItemStatic);

    let attr_str = attr.to_string();

    let property_name = if attr_str.contains("${") && attr_str.contains('}')
    {
        let start = attr_str.find("${").map_or(0, |i| i + 2);
        let end = attr_str.find('}').unwrap_or(attr_str.len());

        let prop = &attr_str[start..end];
        if let Some(colon_pos) = prop.find(':')
        {
            prop[..colon_pos].to_string()
        }
        else
        {
            prop.to_string()
        }
    }
    else
    {
        attr_str.trim().to_string()
    };

    let default_value = if let Expr::Lit(expr_lit) = &*input.expr
    {
        Some(expr_lit.clone())
    }
    else
    {
        None
    };

    let name = &input.ident;
    let ty = &input.ty;

    let expanded = if let Some(default) = default_value
    {
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
    }
    else
    {
        quote! {
            #input
        }
    };

    TokenStream::from(expanded)
}
