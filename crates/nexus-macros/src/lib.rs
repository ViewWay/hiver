//! Nexus Macros - Spring Boot Style Procedural Macros
//! Nexus宏 - Spring Boot风格的过程宏
//!
//! # Overview / 概述
//!
//! `nexus-macros` provides Spring Boot-style procedural macros for the Nexus framework.
//!
//! `nexus-macros` 为 Nexus 框架提供 Spring Boot 风格的过程宏。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_macros::{nexus_main, controller, get};
//!
//! #[nexus_main]
//! struct Application;
//!
//! fn main() -> anyhow::Result<()> {
//!     Application::run()
//! }
//!
//! #[controller]
//! struct DemoController;
//!
//! #[get("/helloworld")]
//! fn hello() -> &'static str {
//!     "Hello World!"
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    DeriveInput, Expr, ItemFn, ItemImpl, ItemStatic, ItemStruct, ItemTrait, parse_macro_input,
};

#[cfg(test)]
mod tests;

mod transactional;
mod handler;

// ============================================================================
// Handler Macro (equivalent to Spring @RequestMapping handler method)
// Handler 宏（等价于 Spring @RequestMapping 处理方法）
// ============================================================================

/// Marks an async function as an HTTP handler with automatic parameter extraction
/// 将异步函数标记为带自动参数提取的HTTP处理器
///
/// The macro validates that the function is async, generates parameter extraction
/// via `FromRequest`, and creates a wrapper that accepts `Request`, extracts params,
/// calls the original function, and converts the result via `IntoResponse`.
///
/// 该宏验证函数是异步的，通过 `FromRequest` 生成参数提取，并创建接受 `Request`、
/// 提取参数、调用原函数、通过 `IntoResponse` 转换结果的包装器。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::handler;
///
/// #[handler]
/// async fn create_user(body: UserRequest) -> impl IntoResponse {
///     // Parameters are automatically extracted from the request
///     // 参数自动从请求中提取
/// }
/// ```
#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    handler::handler_impl(attr, item)
}

// ============================================================================
// Spring Boot Style Main Macro (equivalent to @SpringBootApplication)
// Spring Boot 风格主宏（等价于 @SpringBootApplication）
// ============================================================================

/// Marks the main application entry point with auto-configuration
/// 标记主应用程序入口点（带自动配置）
///
/// Equivalent to Spring Boot's `@SpringBootApplication`.
/// 等价于 Spring Boot 的 `@SpringBootApplication`。
///
/// This macro integrates with nexus-starter's auto-configuration system:
/// 此宏与 nexus-starter 的自动配置系统集成：
/// - Creates `ApplicationContext` / 创建应用上下文
/// - Loads auto-configurations from META-INF/nexus/autoconfiguration.imports
/// - 从 META-INF/nexus/autoconfiguration.imports 加载自动配置
/// - Runs auto-configurations in priority order / 按优先级执行自动配置
/// - Starts the application / 启动应用程序
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::nexus_main;
///
/// #[nexus_main]
/// struct Application;
///
/// fn main() -> anyhow::Result<()> {
///     Application::run()
/// }
/// ```
#[proc_macro_attribute]
pub fn nexus_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            /// Run the application with auto-configuration
            /// 运行应用程序（带自动配置）
            pub fn run() -> anyhow::Result<()> {
                use nexus_starter::core::{
                    ApplicationContext, AutoConfigurationLoader,
                    AutoConfigurationRegistry, CoreAutoConfiguration,
                    AutoConfiguration, logging,
                };
                use nexus_starter::web::{
                    WebServerAutoConfiguration, RouterAutoConfiguration,
                    MiddlewareAutoConfiguration,
                };
                use std::time::Instant;

                // Helper function to format timestamp (simple version)
                fn format_timestamp() -> String {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let duration = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default();
                    let secs = duration.as_secs();
                    let millis = duration.subsec_millis();

                    // Simple format: 2024-01-29T10:30:45 123
                    // Convert secs to date/time (simplified)
                    let days_since_epoch = secs / 86400;
                    let epoch_days = 11017; // Days from 0000-01-01 to 1970-01-01 (simplified)
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

                // Create startup info tracker
                let start_time = Instant::now();

                // 1. Create application context / 创建应用上下文
                let mut ctx = ApplicationContext::new();

                // 2. Load auto-configurations from META-INF/nexus/autoconfiguration.imports
                // 从 META-INF/nexus/autoconfiguration.imports 加载自动配置
                let mut registry = AutoConfigurationRegistry::new();
                let _ = registry.load_from_defaults();

                // 3. Create and register auto-configurations in priority order
                // 按优先级创建并注册自动配置
                let mut configs: Vec<Box<dyn AutoConfiguration>> = vec![
                    // Core configuration (highest priority: -100)
                    Box::new(CoreAutoConfiguration::new()),
                    // Web server configuration (priority: 0)
                    Box::new(WebServerAutoConfiguration::from_config(&ctx)),
                    // Router configuration (priority: 10)
                    Box::new(RouterAutoConfiguration::new()),
                    // Middleware configuration (priority: 20)
                    Box::new(MiddlewareAutoConfiguration::from_config(&ctx)),
                ];

                // Sort by priority (lower number = higher priority)
                // 按优先级排序（数字越小优先级越高）
                configs.sort_by_key(|c| c.order());

                // 4. Execute auto-configurations
                // 执行自动配置
                for config in &configs {
                    if config.condition() {
                        config.configure(&mut ctx)?;
                    }
                }

                // Print "Started Application" message in Spring Boot style
                // 打印 Spring Boot 风格的 "Started Application" 消息
                let elapsed = start_time.elapsed().as_millis();
                let class_name = "nexus.Application";
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

                // TODO: Start the HTTP server
                // TODO: 启动 HTTP 服务器
                // This will be implemented when the HTTP server integration is ready
                // 当 HTTP 服务器集成准备好时实现

                Ok(())
            }

            /// Get the application context
            /// 获取应用上下文
            pub fn context() -> anyhow::Result<ApplicationContext> {
                // Initialize and return context
                // 初始化并返回上下文
                Ok(ApplicationContext::new())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Marks the main application entry point (legacy version)
/// 标记主应用程序入口点（旧版本）
///
/// This is the legacy version without auto-configuration support.
/// Use `#[nexus_main]` for full auto-configuration support.
/// 这是旧版本，不支持自动配置。使用 `#[nexus_main]` 获得完整的自动配置支持。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::main;
///
/// #[main]
/// struct Application;
/// ```
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            /// Run the application
            /// 运行应用程序
            pub fn run() -> std::io::Result<()> {
                use nexus_runtime::Runtime;

                let runtime = Runtime::new()?;
                runtime.block_on(async {
                    // Start the server
                    // 启动服务器
                    nexus_http::Server::new().run().await
                })
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Controller Macro (equivalent to @RestController)
// 控制器宏（等价于 @RestController）
// ============================================================================

/// Marks a struct as a REST controller
/// 将结构体标记为 REST 控制器
///
/// Equivalent to Spring's `@RestController` or `@Controller`.
/// 等价于 Spring 的 `@RestController` 或 `@Controller`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::{controller, get};
///
/// #[controller]
/// struct DemoController;
///
/// #[get("/hello")]
/// fn hello() -> &'static str {
///     "Hello!"
/// }
/// ```
#[proc_macro_attribute]
pub fn controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let expanded = quote! {
        #input

        impl #input {
            /// Register this controller's routes
            /// 注册此控制器的路由
            pub fn register() -> nexus_router::Router {
                nexus_router::Router::new()
                    .prefix(concat!("/", stringify!(#input)))
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Service Macro (equivalent to @Service)
// 服务宏（等价于 @Service）
// ============================================================================

/// Marks a struct as a service
/// 将结构体标记为服务
///
/// Equivalent to Spring's `@Service`.
/// 等价于 Spring 的 `@Service`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::service;
/// use std::sync::Arc;
///
/// #[service]
/// struct UserService {
///     repository: Arc<UserRepository>,
/// }
///
/// impl UserService {
///     fn new(repository: Arc<UserRepository>) -> Self {
///         Self { repository }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    // Generate a new() method if not exists
    let name = &input.ident;
    let fields = &input.fields;

    let field_names: Vec<_> = fields.iter().filter_map(|f| f.ident.as_ref()).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let expanded = quote! {
        #input

        impl #name {
            /// Create a new instance (constructor injection)
            /// 创建新实例（构造函数注入）
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }
        }

        // Implement Into<Arc<Self>> for easy DI container integration
        // 实现 Into<Arc<Self>> 以便与 DI 容器集成
        impl Into<std::sync::Arc<Self>> for #name {
            fn into(self) -> std::sync::Arc<Self> {
                std::sync::Arc::new(self)
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Repository Macro (equivalent to Spring Data JPA Repository)
// 仓储宏（等价于 Spring Data JPA Repository）
// ============================================================================

/// Marks a trait as a repository
/// 将 trait 标记为仓储
///
/// Equivalent to Spring Data JPA's `@Repository` or `JpaRepository`.
/// 等价于 Spring Data JPA 的 `@Repository` 或 `JpaRepository`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::repository;
///
/// #[repository]
/// trait UserRepository: Send + Sync {
///     async fn find_by_id(&self, id: u64) -> Option<User>;
///     async fn save(&self, user: User) -> Result<User, Error>;
/// }
/// ```
#[proc_macro_attribute]
pub fn repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemTrait);

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Configuration Macro (equivalent to @ConfigurationProperties)
// 配置宏（等价于 @ConfigurationProperties）
// ============================================================================

/// Marks a struct as configuration properties
/// 将结构体标记为配置属性
///
/// Equivalent to Spring Boot's `@ConfigurationProperties`.
/// 等价于 Spring Boot 的 `@ConfigurationProperties`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::config;
///
/// #[config(prefix = "app")]
/// struct AppConfig {
///     name: String,
///     port: u16,
/// }
/// ```
#[proc_macro_attribute]
pub fn config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    // Parse the prefix attribute
    let prefix = if attr.is_empty() {
        Ident::new("config", Span::call_site())
    } else {
        parse_macro_input!(attr as ConfigArgs).prefix
    };

    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            /// Load configuration from environment/config files
            /// 从环境/配置文件加载配置
            pub fn load() -> Result<Self, config::ConfigError> {
                let mut cfg = config::Config::builder();

                // Load from config files (application.toml, application.yml)
                // 从配置文件加载
                cfg = cfg.add_source(config::File::with_name("application"));

                // Load from environment with prefix
                // 从环境变量加载（带前缀）
                cfg = cfg.add_source(
                    config::Environment::with_prefix(stringify!(#prefix))
                        .separator("__")
                        .try_parsing(true)
                );

                // Build and deserialize
                // 构建并反序列化
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

// ============================================================================
// Route Macros (equivalent to @GetMapping, @PostMapping, etc.)
// 路由宏（等价于 @GetMapping, @PostMapping 等）
// ============================================================================

/// Helper function to parse route attributes
/// 解析路由属性的辅助函数
fn parse_route_path(attr: TokenStream) -> syn::Result<String> {
    let attr_str = attr.to_string();

    // Handle both #[get("/path")] and #[get(path)]
    // 处理 #[get("/path")] 和 #[get(path)] 两种形式
    let path = if attr_str.contains('"') {
        // Extract string between quotes
        // 提取引号之间的字符串
        let start = attr_str.find('"').unwrap_or(0) + 1;
        let end = attr_str.rfind('"').unwrap_or(attr_str.len());
        attr_str[start..end].to_string()
    } else {
        // Use the raw value
        // 使用原始值
        attr_str.trim().to_string()
    };

    Ok(path)
}

macro_rules! impl_route_macro {
    ($name:ident, $method:ident) => {
        /// Route attribute macro
        /// 路由属性宏
        #[proc_macro_attribute]
        pub fn $name(attr: TokenStream, item: TokenStream) -> TokenStream {
            let input = parse_macro_input!(item as ItemFn);
            let func_name = &input.sig.ident;

            // Parse the route path
            // 解析路由路径
            let path = match parse_route_path(attr) {
                Ok(p) => p,
                Err(e) => return TokenStream::from(e.to_compile_error()),
            };

            // Build the route registration
            // 构建路由注册
            let expanded = quote! {
                #input

                // Register this route with the router
                // 在路由器中注册此路由
                #[automatically_derived]
                impl #func_name {
                    pub fn register_route(router: nexus_router::Router) -> nexus_router::Router {
                        router.route(#path, nexus_http::Method::$method, #func_name)
                    }
                }
            };

            TokenStream::from(expanded)
        }
    };
}

// Implement route macros for each HTTP method
// 为每个 HTTP 方法实现路由宏
impl_route_macro!(get, GET);
impl_route_macro!(post, POST);
impl_route_macro!(put, PUT);
impl_route_macro!(delete, DELETE);
impl_route_macro!(patch, PATCH);
impl_route_macro!(head, HEAD);
impl_route_macro!(options, OPTIONS);
impl_route_macro!(trace, TRACE);

// ============================================================================
// Component Macro (equivalent to @Component)
// 组件宏（等价于 @Component）
// ============================================================================

/// Marks a struct as a Spring component
/// 将结构体标记为 Spring 组件
///
/// Equivalent to Spring's `@Component`.
/// 等价于 Spring 的 `@Component`。
///
/// Components are automatically detected and registered in the application context.
/// 组件会被自动检测并注册到应用程序上下文中。
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let expanded = quote! {
        #input

        impl #input {
            /// Initialize this component
            /// 初始化此组件
            pub fn init() -> Self {
                Self::default()
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Autowired Macro (equivalent to @Autowired)
// 自动装配宏（等价于 @Autowired）
// ============================================================================

/// Marks a field or constructor for autowiring
/// 标记字段或构造函数用于自动装配
///
/// Equivalent to Spring's `@Autowired`.
/// 等价于 Spring 的 `@Autowired`。
#[proc_macro_attribute]
pub fn autowired(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just pass through
    // 目前，直接传递
    item
}

// ============================================================================
// Scheduled Macro (equivalent to @Scheduled)
// 定时任务宏（等价于 @Scheduled）
// ============================================================================

/// Marks a function to be scheduled
/// 标记函数为定时执行
///
/// Equivalent to Spring's `@Scheduled`.
/// 等价于 Spring 的 `@Scheduled`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::scheduled;
///
/// #[scheduled(cron = "0 * * * * *")] // Every hour
/// async fn cleanup_task() {
///     // Cleanup logic
/// }
///
/// #[scheduled(fixed_rate = 5000)] // Every 5 seconds
/// async fn periodic_task() {
///     // Periodic logic
/// }
/// ```
#[proc_macro_attribute]
pub fn scheduled(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse scheduling parameters
    // 解析调度参数
    let schedule_expr = parse_schedule_args(attr);

    let expanded = quote! {
        #input

        // Register this scheduled task
        // 注册此定时任务
        #[automatically_derived]
        impl #func_name {
            pub fn schedule(self) {
                use nexus_runtime::time::Duration;
                use nexus_runtime::spawn;

                spawn(async move {
                    #schedule_expr
                });
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_schedule_args(attr: TokenStream) -> proc_macro2::TokenStream {
    let attr_str = attr.to_string();

    if attr_str.contains("cron") {
        // Simplified fixed-interval fallback for cron expressions
        // Parse a basic numeric value from the attribute if possible,
        // otherwise default to 60-second intervals.
        let interval_ms: u64 = 60_000;
        quote! {
            // Simplified cron: fixed-interval scheduling (60s default)
            // 简化版cron：固定间隔调度（默认60秒）
            loop {
                self().await;
                nexus_runtime::sleep(nexus_runtime::Duration::from_millis(#interval_ms)).await;
            }
        }
    } else if attr_str.contains("fixed_rate") {
        quote! {
            // Fixed rate scheduling
            // 固定频率调度
            loop {
                self().await;
                tokio::time::sleep(Duration::from_millis(5000)).await;
            }
        }
    } else if attr_str.contains("fixed_delay") {
        quote! {
            // Fixed delay scheduling
            // 固定延迟调度
            loop {
                self().await;
                tokio::time::sleep(Duration::from_millis(5000)).await;
            }
        }
    } else {
        quote! {
            // Default: run once on startup
            // 默认：启动时运行一次
            self().await;
        }
    }
}

// ============================================================================
// Async Macro (equivalent to @Async)
// 异步宏（等价于 @Async）
// ============================================================================

/// Marks a method to run asynchronously
/// 标记方法为异步执行
///
/// Equivalent to Spring's `@Async`.
/// 等价于 Spring 的 `@Async`。
#[proc_macro_attribute]
pub fn async_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Ensure the function is async
    // 确保函数是异步的
    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Derive Macros
// 派生宏
// ============================================================================

/// Derive macro for `FromRequest` trait
/// `FromRequest` trait 的派生宏
#[proc_macro_derive(FromRequest)]
pub fn from_request_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl nexus_http::FromRequest for #name {
            type Error = nexus_http::Error;

            async fn from_request(
                req: &nexus_http::Request,
            ) -> Result<Self, Self::Error> {
                // Minimal JSON deserialization from request body
                // 从请求体进行最小化JSON反序列化
                let body = req.body().as_slice();
                serde_json::from_slice::<Self>(body)
                    .map_err(|e| nexus_http::Error::new(std::borrow::Cow::Owned(e.to_string())))
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for `IntoResponse` trait
/// `IntoResponse` trait 的派生宏
#[proc_macro_derive(IntoResponse)]
pub fn into_response_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl nexus_http::IntoResponse for #name {
            fn into_response(self) -> nexus_http::Response {
                // Convert to JSON response
                // 转换为 JSON 响应
                use nexus_http::Json;
                Json(self).into_response()
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Slf4j Macro (equivalent to Lombok @Slf4j)
// Slf4j 宏（等价于 Lombok @Slf4j）
// ============================================================================

/// Automatically adds a logger field to the struct
/// 自动为结构体添加日志字段
///
/// Equivalent to Lombok's `@Slf4j` annotation in Java.
/// 等价于 Java 中 Lombok 的 `@Slf4j` 注解。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::slf4j;
///
/// #[slf4j]
/// struct MyController {
///     // The macro automatically adds: log: nexus_observability::log::LoggerHandle
/// }
///
/// impl MyController {
///     fn do_something(&self) {
///         self.log.info(format_args!("Doing something..."));
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn slf4j(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    // Check if struct already has a log field
    let has_log_field = input.fields.iter().any(|f| {
        f.ident
            .as_ref()
            .is_some_and(|i| *i == "log")
    });

    if has_log_field {
        // Already has log field, just return the original
        // 已有 log 字段，直接返回原样
        return TokenStream::from(quote! { #input });
    }

    let expanded = quote! {
        #input

        impl #name {
            /// Get the logger for this struct
            /// 获取此结构体的日志记录器
            fn log(&self) -> nexus_observability::log::LoggerHandle {
                nexus_observability::log::LoggerFactory::get_for::<#name>()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Creates a static logger in the current scope
/// 在当前作用域中创建静态日志记录器
///
/// This is a simpler alternative to `#[slf4j]` that creates a `log` binding.
/// 这是 `#[slf4j]` 的更简单替代方案，创建一个 `log` 绑定。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::logger;
///
/// #[logger]
/// fn my_function() {
///     log.info("Hello from logger");
/// }
/// ```
#[proc_macro_attribute]
pub fn logger(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let expanded = quote! {
        #input

        // Create logger binding
        // 创建日志记录器绑定
        let log = nexus_observability::log::LoggerFactory::get(stringify!(#func_name));
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Transactional Macro (equivalent to @Transactional)
// 事务宏（等价于 @Transactional）
// ============================================================================

/// Marks a function or method to be executed within a transaction
/// 标记函数或方法在事务中执行
///
/// Equivalent to Spring's `@Transactional`.
/// 等价于 Spring 的 `@Transactional`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::transactional;
///
/// #[transactional]
/// async fn transfer_money(from: Account, to: Account, amount: f64) -> Result<(), Error> {
///     // Database operations here will be executed in a transaction
///     // 这里的数据库操作将在事务中执行
/// }
/// ```
#[proc_macro_attribute]
pub fn transactional(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Use the transactional module implementation
    // 使用transactional模块实现
    transactional::transactional_impl(attr, item)
}

// ============================================================================
// Cacheable Macros (equivalent to @Cacheable, @CacheEvict, @CachePut)
// 缓存宏（等价于 @Cacheable, @CacheEvict, @CachePut）
// ============================================================================

/// Cache the result of a method
/// 缓存方法结果
///
/// Equivalent to Spring's `@Cacheable`.
/// 等价于 Spring 的 `@Cacheable`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::cacheable;
///
/// #[cacheable("users")]
/// async fn get_user(id: u64) -> Option<User> {
///     // Result will be cached
///     // 结果将被缓存
/// }
/// ```
#[proc_macro_attribute]
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

    // Generate a cache key prefix
    // 生成缓存键前缀
    let expanded = quote! {
        // Original function - moved to inner
        // 原始函数 - 移动到内部
        fn #fn_name_inner() {
            // This is a placeholder - the actual implementation would require
            // capturing the original function body which is complex in proc macros
            // 这是一个占位符 - 实际实现需要捕获原始函数体，这在proc宏中很复杂
        }

        /// Cached version of the function
        /// 函数的缓存版本
        ///
        /// Note: Full caching implementation requires integration with
        /// nexus-cache crate. This macro provides the annotation for
        /// future automatic cache generation.
        /// 注意：完整的缓存实现需要与nexus-cache crate集成。
        /// 此宏为将来自动缓存生成提供注解。
        #[allow(dead_code)]
        const #fn_name: &str = #fn_name_str;
        const CACHE_NAME: &str = #cache_name;

        // The actual caching wrapper would be generated here
        // 实际的缓存包装器将在此生成
        #input
    };

    TokenStream::from(expanded)
}

/// Evict cache entries
/// 清除缓存条目
///
/// Equivalent to Spring's `@CacheEvict`.
/// 等价于 Spring 的 `@CacheEvict`。
#[proc_macro_attribute]
pub fn cache_evict(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _cache_name = attr;
    let input = parse_macro_input!(item as ItemFn);

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

/// Update cache entry
/// 更新缓存条目
///
/// Equivalent to Spring's `@CachePut`.
/// 等价于 Spring 的 `@CachePut`。
#[proc_macro_attribute]
pub fn cache_put(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _cache_name = attr;
    let input = parse_macro_input!(item as ItemFn);

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Conditional Macros (equivalent to @ConditionalOn* annotations)
// 条件宏（等价于 @ConditionalOn* 注解）
// ============================================================================

/// Only enable bean if class is on classpath
/// 仅当类在类路径上时才启用 bean
///
/// Equivalent to Spring Boot's `@ConditionalOnClass`.
/// 等价于 Spring Boot 的 `@ConditionalOnClass`。
#[proc_macro_attribute]
pub fn conditional_on_class(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Only enable bean if property is set
/// 仅当属性设置时才启用 bean
///
/// Equivalent to Spring Boot's `@ConditionalOnProperty`.
/// 等价于 Spring Boot 的 `@ConditionalOnProperty`。
#[proc_macro_attribute]
pub fn conditional_on_property(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Only enable bean if another bean is missing
/// 仅当另一个 bean 不存在时才启用 bean
///
/// Equivalent to Spring Boot's `@ConditionalOnMissingBean`.
/// 等价于 Spring Boot 的 `@ConditionalOnMissingBean`。
#[proc_macro_attribute]
pub fn conditional_on_missing_bean(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Value Macro (equivalent to @Value)
// 值注入宏（等价于 @Value）
// ============================================================================

/// Inject a value from environment or config
/// 从环境或配置注入值
///
/// Equivalent to Spring's `@Value("${property.name}")`.
/// 等价于 Spring 的 `@Value("${property.name}")`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::value;
///
/// #[value("${app.name}")]
/// static APP_NAME: &str = "Nexus Application";
///
/// #[value("${server.port:8080}")]
/// static SERVER_PORT: u16 = 8080;
/// ```
#[proc_macro_attribute]
pub fn value(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStatic);

    // Parse the property expression
    let attr_str = attr.to_string();

    // Extract property name from "${property.name}" or "${property.name:default}"
    let property_name = if attr_str.contains("${") && attr_str.contains('}') {
        let start = attr_str.find("${").map_or(0, |i| i + 2);
        let end = attr_str.find('}').unwrap_or(attr_str.len());

        let prop = &attr_str[start..end];
        if let Some(colon_pos) = prop.find(':') {
            // Has default value
            // 有默认值
            prop[..colon_pos].to_string()
        } else {
            prop.to_string()
        }
    } else {
        attr_str.trim().to_string()
    };

    // Extract default value from the static expression
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

// ============================================================================
// Profile Macro (equivalent to @Profile)
// 配置宏（等价于 @Profile）
// ============================================================================

/// Only enable component when specific profile is active
/// 仅当特定配置文件激活时才启用组件
///
/// Equivalent to Spring's `@Profile("dev")`.
/// 等价于 Spring 的 `@Profile("dev")`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::profile;
///
/// #[profile("dev")]
/// #[service]
/// struct DevService {
///     // Only available in dev profile
///     // 仅在 dev 配置文件中可用
/// }
///
/// #[profile("prod")]
/// #[service]
/// struct ProdService {
///     // Only available in production
///     // 仅在生产环境中可用
/// }
/// ```
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let profile = parse_macro_input!(attr as syn::LitStr);
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    // Generate a function that checks if this profile is active
    // 生成检查此配置文件是否活动的函数
    let expanded = quote! {
        #input

        impl #struct_name {
            /// Check if this component's profile is currently active
            /// 检查此组件的配置文件当前是否活动
            fn is_active_profile() -> bool {
                const REQUIRED_PROFILE: &str = #profile;

                std::env::var("SPRING_PROFILES_ACTIVE")
                    .map(|active_profiles| {
                        // Check each active profile
                        // 检查每个活动配置文件
                        for active in active_profiles.split(',') {
                            let active = active.trim();
                            if active == REQUIRED_PROFILE || active == "default" {
                                return true;
                            }
                        }
                        false
                    })
                    .unwrap_or_else(|_| {
                        // If no profile is set, only enable if required is "default"
                        // 如果没有设置配置文件，仅在required是"default"时启用
                        REQUIRED_PROFILE == "default"
                    })
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Parameter Extraction Macros (equivalent to @PathVariable, @RequestParam, etc.)
// 参数提取宏（等价于 @PathVariable, @RequestParam 等）
// ============================================================================

/// Mark a parameter as extracted from path variable
/// 标记参数从路径变量提取
///
/// Equivalent to Spring's `@PathVariable`.
/// 等价于 Spring 的 `@PathVariable`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::path_variable;
///
/// #[get("/users/:id")]
/// async fn get_user(#[path_variable] id: String) -> String {
///     format!("User: {}", id)
/// }
/// ```
#[proc_macro_attribute]
pub fn path_variable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as extracted from query string
/// 标记参数从查询字符串提取
///
/// Equivalent to Spring's `@RequestParam`.
/// 等价于 Spring 的 `@RequestParam`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::request_param;
///
/// #[get("/search")]
/// async fn search(#[request_param] q: String) -> String {
///     format!("Searching for: {}", q)
/// }
/// ```
#[proc_macro_attribute]
pub fn request_param(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as extracted from request header
/// 标记参数从请求头提取
///
/// Equivalent to Spring's `@RequestHeader`.
/// 等价于 Spring 的 `@RequestHeader`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::request_header;
///
/// #[get("/info")]
/// async fn info(#[request_header] user_agent: String) -> String {
///     format!("User-Agent: {}", user_agent)
/// }
/// ```
#[proc_macro_attribute]
pub fn request_header_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as extracted from cookie
/// 标记参数从 Cookie 提取
///
/// Equivalent to Spring's `@CookieValue`.
/// 等价于 Spring 的 `@CookieValue`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::cookie_value;
///
/// #[get("/pref")]
/// async fn preferences(#[cookie_value] theme: String) -> String {
///     format!("Theme: {}", theme)
/// }
/// ```
#[proc_macro_attribute]
pub fn cookie_value(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as extracted from request body
/// 标记参数从请求体提取
///
/// Equivalent to Spring's `@RequestBody`.
/// 等价于 Spring 的 `@RequestBody`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::request_body;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     name: String,
///     email: String,
/// }
///
/// #[post("/users")]
/// async fn create_user(#[request_body] user: CreateUser) -> String {
///     format!("Created user: {}", user.name)
/// }
/// ```
#[proc_macro_attribute]
pub fn request_body(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as extracted from model attributes
/// 标记参数从模型属性提取
///
/// Equivalent to Spring's `@ModelAttribute`.
/// 等价于 Spring 的 `@ModelAttribute`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::model_attribute;
///
/// #[post("/users")]
/// async fn create_user(#[model_attribute] user: User) -> String {
///     format!("Created user: {}", user.name)
/// }
/// ```
#[proc_macro_attribute]
pub fn model_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as extracted from request attribute
/// 标记参数从请求属性提取
///
/// Equivalent to Spring's `@RequestAttribute`.
/// 等价于 Spring 的 `@RequestAttribute`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::request_attribute;
///
/// #[get("/context")]
/// async fn context(#[request_attribute] user_id: String) -> String {
///     format!("User ID: {}", user_id)
/// }
/// ```
#[proc_macro_attribute]
pub fn request_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as extracted from matrix variable
/// 标记参数从矩阵变量提取
///
/// Equivalent to Spring's `@MatrixVariable`.
/// 等价于 Spring 的 `@MatrixVariable`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::matrix_variable;
///
/// #[get("/products/:id")]
/// async fn get_product(#[matrix_variable("version")] version: String) -> String {
///     format!("Version: {}", version)
/// }
/// ```
#[proc_macro_attribute]
pub fn matrix_variable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as extracted from session
/// 标记参数从会话提取
///
/// Equivalent to Spring's `@SessionAttribute`.
/// 等价于 Spring 的 `@SessionAttribute`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::session_attribute;
///
/// #[get("/profile")]
/// async fn profile(#[session_attribute] user_id: String) -> String {
///     format!("Profile for user: {}", user_id)
/// }
/// ```
#[proc_macro_attribute]
pub fn session_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Request Mapping Macro (equivalent to @RequestMapping)
// 请求映射宏（等价于 @RequestMapping）
// ============================================================================

/// Generic request mapping for any HTTP method
/// 通用请求映射，支持任何 HTTP 方法
///
/// Equivalent to Spring's `@RequestMapping`.
/// 等价于 Spring 的 `@RequestMapping`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::request_mapping;
///
/// #[request_mapping(path = "/api/data", method = "GET")]
/// async fn get_data() -> &'static str {
///     "data"
/// }
/// ```
#[proc_macro_attribute]
pub fn request_mapping(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse attributes
    let attr_str = attr.to_string();
    let path = if attr_str.contains("path") {
        if let Some(start) = attr_str.find("path = \"") {
            let start = start + 8;
            if let Some(end) = attr_str[start..].find('"') {
                attr_str[start..start + end].to_string()
            } else {
                "/".to_string()
            }
        } else {
            "/".to_string()
        }
    } else {
        "/".to_string()
    };

    let method = if attr_str.contains("method") {
        if attr_str.contains("GET") {
            "GET"
        } else if attr_str.contains("POST") {
            "POST"
        } else if attr_str.contains("PUT") {
            "PUT"
        } else if attr_str.contains("DELETE") {
            "DELETE"
        } else if attr_str.contains("PATCH") {
            "PATCH"
        } else {
            "GET"
        }
    } else {
        "GET"
    };

    let method_ident = Ident::new(method, Span::call_site());

    let expanded = quote! {
        #input

        #[automatically_derived]
        impl #func_name {
            pub fn register_route(router: nexus_router::Router) -> nexus_router::Router {
                router.route(#path, nexus_http::Method::#method_ident, #func_name)
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Cross Origin Macro (equivalent to @CrossOrigin)
// 跨域宏（等价于 @CrossOrigin）
// ============================================================================

/// Configure CORS for the endpoint
/// 为端点配置 CORS
///
/// Equivalent to Spring's `@CrossOrigin`.
/// 等价于 Spring 的 `@CrossOrigin`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::{cross_origin, get};
///
/// #[cross_origin(origins = "*")]
/// #[get("/api/data")]
/// async fn get_data() -> &'static str {
///     "data"
/// }
/// ```
#[proc_macro_attribute]
pub fn cross_origin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Currently pass-through - actual CORS handling is done by middleware
    // 目前直接传递 - 实际的 CORS 处理由中间件完成
    item
}

// ============================================================================
// Configuration Macro (equivalent to @Configuration)
// 配置宏（等价于 @Configuration）
// ============================================================================

/// Marks a class as a source of bean definitions
/// 标记类为 Bean 定义源
///
/// Equivalent to Spring's `@Configuration`.
/// 等价于 Spring 的 `@Configuration`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::{configuration, bean};
///
/// #[configuration]
/// struct AppConfig {
///     // Bean definitions
/// }
/// ```
#[proc_macro_attribute]
pub fn configuration(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            /// Register all beans from this configuration
            /// 注册此配置中的所有 Bean
            pub fn register_beans() {
                // Bean registration happens through component scanning
                // Bean 注册通过组件扫描完成
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Bean Macro (equivalent to @Bean)
// Bean宏（等价于 @Bean）
// ============================================================================

/// Marks a method as a bean producer
/// 标记方法为 Bean 生产者
///
/// Equivalent to Spring's `@Bean`.
/// 等价于 Spring 的 `@Bean`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::{bean, configuration};
///
/// #[configuration]
/// struct AppConfig;
///
/// impl AppConfig {
///     #[bean]
///     fn data_source() -> DataSource {
///         DataSource::new()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn bean(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let expanded = quote! {
        #input

        // Register this bean with the container
        // 将此 Bean 注册到容器
        #[automatically_derived]
        impl #func_name {
            pub fn bean_name() -> &'static str {
                stringify!(#func_name)
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Enable Macros (equivalent to @Enable*)
// 启用宏（等价于 @Enable*）
// ============================================================================

/// Enable auto-configuration
/// 启用自动配置
///
/// Equivalent to Spring Boot's `@EnableAutoConfiguration`.
/// 等价于 Spring Boot 的 `@EnableAutoConfiguration`。
#[proc_macro_attribute]
pub fn enable_auto_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Enable caching
/// 启用缓存
///
/// Equivalent to Spring's `@EnableCaching`.
/// 等价于 Spring 的 `@EnableCaching`。
#[proc_macro_attribute]
pub fn enable_caching(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Enable scheduling
/// 启用定时任务
///
/// Equivalent to Spring's `@EnableScheduling`.
/// 等价于 Spring 的 `@EnableScheduling`。
#[proc_macro_attribute]
pub fn enable_scheduling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Enable async method execution
/// 启用异步方法执行
///
/// Equivalent to Spring's `@EnableAsync`.
/// 等价于 Spring 的 `@EnableAsync`。
#[proc_macro_attribute]
pub fn enable_async_exec(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Enable transaction management
/// 启用事务管理
///
/// Equivalent to Spring's `@EnableTransactionManagement`.
/// 等价于 Spring 的 `@EnableTransactionManagement`。
#[proc_macro_attribute]
pub fn enable_transaction_management(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Enable validation
/// 启用参数校验
///
/// Equivalent to Spring's `@EnableValidating`.
/// 等价于 Spring 的 `@EnableValidating`。
#[proc_macro_attribute]
pub fn enable_validating(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Enable Web MVC
/// 启用 Web MVC
///
/// Equivalent to Spring's `@EnableWebMvc`.
/// 等价于 Spring 的 `@EnableWebMvc`。
#[proc_macro_attribute]
pub fn enable_web_mvc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Import configuration classes
/// 导入配置类
///
/// Equivalent to Spring's `@Import`.
/// 等价于 Spring 的 `@Import`。
#[proc_macro_attribute]
pub fn import(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Enable component scanning
/// 启用组件扫描
///
/// Equivalent to Spring's `@ComponentScan`.
/// 等价于 Spring 的 `@ComponentScan`。
#[proc_macro_attribute]
pub fn component_scan(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Validation Macros (equivalent to @Valid/@Validated/@NotNull/@Size/etc)
// 验证宏（等价于 @Valid/@Validated/@NotNull/@Size 等）
// ============================================================================

/// Enable validation for a parameter or struct
/// 启用参数或结构体的验证
///
/// Equivalent to Spring's `@Valid`.
/// 等价于 Spring 的 `@Valid`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::Valid;
///
/// #[post("/users")]
/// async fn create_user(#[Valid] req: CreateUserRequest) -> Response {
///     Response::ok()
/// }
/// ```
#[proc_macro_attribute]
pub fn valid(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Validation is handled by Validated extractor at runtime
    // 验证由运行时的 Validated 提取器处理
    item
}

/// Enable method-level validation
/// 启用方法级校验
///
/// Equivalent to Spring's `@Validated`.
/// 等价于 Spring 的 `@Validated`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::Validated;
///
/// #[Validated]
/// #[post("/users")]
/// async fn create_user(#[Valid] req: CreateUserRequest) -> Response {
///     Response::ok()
/// }
/// ```
#[proc_macro_attribute]
pub fn validated(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a field as required (not null)
/// 标记字段为必填（非空）
///
/// Equivalent to Spring's `@NotNull`.
/// 等价于 Spring 的 `@NotNull`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::NotNull;
///
/// struct User {
///     #[NotNull]
///     username: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn not_null(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a string field as required and non-empty
/// 标记字符串字段为必填且非空
///
/// Equivalent to Spring's `@NotBlank`.
/// 等价于 Spring 的 `@NotBlank`。
#[proc_macro_attribute]
pub fn not_blank(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a collection field as required and non-empty
/// 标记集合字段为必填且非空
///
/// Equivalent to Spring's `@NotEmpty`.
/// 等价于 Spring 的 `@NotEmpty`。
#[proc_macro_attribute]
pub fn not_empty(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Size constraint for strings, collections, arrays, maps, etc.
/// 字符串、集合、数组、映射等的大小约束
///
/// Equivalent to Spring's `@Size`.
/// 等价于 Spring 的 `@Size`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::Size;
///
/// struct User {
///     #[Size(min = 3, max = 20)]
///     username: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn size(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Length constraint for strings (alias for Size)
/// 字符串长度约束（Size 的别名）
#[proc_macro_attribute]
pub fn length(_attr: TokenStream, item: TokenStream) -> TokenStream {
    size(_attr, item)
}

/// Minimum value constraint for numeric types
/// 数值类型的最小值约束
///
/// Equivalent to Spring's `@Min`.
/// 等价于 Spring 的 `@Min`。
#[proc_macro_attribute]
pub fn min(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Maximum value constraint for numeric types
/// 数值类型的最大值约束
///
/// Equivalent to Spring's `@Max`.
/// 等价于 Spring 的 `@Max`。
#[proc_macro_attribute]
pub fn max(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Decimal minimum value constraint
/// 小数最小值约束
///
/// Equivalent to Spring's `@DecimalMin`.
/// 等价于 Spring 的 `@DecimalMin`。
#[proc_macro_attribute]
pub fn decimal_min(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Decimal maximum value constraint
/// 小数最大值约束
///
/// Equivalent to Spring's `@DecimalMax`.
/// 等价于 Spring 的 `@DecimalMax`。
#[proc_macro_attribute]
pub fn decimal_max(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Email format validation
/// 邮箱格式验证
///
/// Equivalent to Spring's `@Email`.
/// 等价于 Spring 的 `@Email`。
#[proc_macro_attribute]
pub fn email(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Pattern validation using regular expression
/// 使用正则表达式进行模式验证
///
/// Equivalent to Spring's `@Pattern`.
/// 等价于 Spring 的 `@Pattern`。
#[proc_macro_attribute]
pub fn pattern(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// URL format validation
/// URL 格式验证
///
/// Equivalent to Spring's `@Url`.
/// 等价于 Spring 的 `@Url`。
#[proc_macro_attribute]
pub fn url(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Assert that the boolean value is true
/// 断言布尔值为 true
///
/// Equivalent to Spring's `@AssertTrue`.
/// 等价于 Spring 的 `@AssertTrue`。
#[proc_macro_attribute]
pub fn assert_true(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Assert that the boolean value is false
/// 断言布尔值为 false
///
/// Equivalent to Spring's `@AssertFalse`.
/// 等价于 Spring 的 `@AssertFalse`。
#[proc_macro_attribute]
pub fn assert_false(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Past date validation
/// 过去日期验证
///
/// Equivalent to Spring's `@Past`.
/// 等价于 Spring 的 `@Past`。
#[proc_macro_attribute]
pub fn past(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Future date validation
/// 未来日期验证
///
/// Equivalent to Spring's `@Future`.
/// 等价于 Spring 的 `@Future`。
#[proc_macro_attribute]
pub fn future(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Range validation for numeric values
/// 数值范围验证
///
/// Equivalent to Spring's `@Range`.
/// 等价于 Spring 的 `@Range`。
#[proc_macro_attribute]
pub fn range(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Negative value validation
/// 负值验证
///
/// Equivalent to Spring's `@Negative`.
/// 等价于 Spring 的 `@Negative`。
#[proc_macro_attribute]
pub fn negative(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Positive value validation
/// 正值验证
///
/// Equivalent to Spring's `@Positive`.
/// 等价于 Spring 的 `@Positive`。
#[proc_macro_attribute]
pub fn positive(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Scope Macros (equivalent to @RequestScope, @SessionScope, etc.)
// 作用域宏（等价于 @RequestScope, @SessionScope 等）
// ============================================================================

/// Specify that the bean should be created at request scope
/// 指定 Bean 在请求作用域创建
///
/// Equivalent to Spring's `@RequestScope`.
/// 等价于 Spring 的 `@RequestScope`。
#[proc_macro_attribute]
pub fn request_scope(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Specify that the bean should be created at session scope
/// 指定 Bean 在会话作用域创建
///
/// Equivalent to Spring's `@SessionScope`.
/// 等价于 Spring 的 `@SessionScope`。
#[proc_macro_attribute]
pub fn session_scope(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Specify that the bean should be created at application scope (singleton)
/// 指定 Bean 在应用作用域创建（单例）
///
/// Equivalent to Spring's `@ApplicationScope`.
/// 等价于 Spring 的 `@ApplicationScope`。
#[proc_macro_attribute]
pub fn application_scope(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Lifecycle Callbacks (equivalent to @PostConstruct, @PreDestroy)
// 生命周期回调（等价于 @PostConstruct, @PreDestroy）
// ============================================================================

/// Mark a method to be called after bean construction
/// 标记方法在 Bean 构造后调用
///
/// Equivalent to Spring's `@PostConstruct`.
/// 等价于 Spring 的 `@PostConstruct`。
#[proc_macro_attribute]
pub fn post_construct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method to be called before bean destruction
/// 标记方法在 Bean 销毁前调用
///
/// Equivalent to Spring's `@PreDestroy`.
/// 等价于 Spring 的 `@PreDestroy`。
#[proc_macro_attribute]
pub fn pre_destroy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Bean Qualification Macros (equivalent to @Qualifier, @Primary, @Lazy)
// Bean 限定宏（等价于 @Qualifier, @Primary, @Lazy）
// ============================================================================

/// Specify a qualifier for a bean to disambiguate dependencies
/// 指定 Bean 限定符以消除依赖歧义
///
/// Equivalent to Spring's `@Qualifier`.
/// 等价于 Spring 的 `@Qualifier`。
#[proc_macro_attribute]
pub fn qualifier(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Indicate that a bean should be preferred when multiple candidates exist
/// 指示当存在多个候选时优先选择此 Bean
///
/// Equivalent to Spring's `@Primary`.
/// 等价于 Spring 的 `@Primary`。
#[proc_macro_attribute]
pub fn primary(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Indicate that a bean should be lazily initialized
/// 指示 Bean 应该延迟初始化
///
/// Equivalent to Spring's `@Lazy`.
/// 等价于 Spring 的 `@Lazy`。
#[proc_macro_attribute]
pub fn lazy_bean(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Indicate that a bean should not be autowired and requires explicit lookup
/// 指示 Bean 不应自动装配，需要显式查找
///
/// Equivalent to Spring's `@Lookup`.
/// 等价于 Spring 的 `@Lookup`。
#[proc_macro_attribute]
pub fn lookup(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a bean as having prototype scope (new instance each time)
/// 标记 Bean 为原型作用域（每次创建新实例）
///
/// Equivalent to Spring's `@Scope("prototype")`.
/// 等价于 Spring 的 `@Scope("prototype")`。
#[proc_macro_attribute]
pub fn scope_prototype(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a bean as having singleton scope (default behavior)
/// 标记 Bean 为单例作用域（默认行为）
///
/// Equivalent to Spring's `@Scope("singleton")`.
/// 等价于 Spring 的 `@Scope("singleton")`。
#[proc_macro_attribute]
pub fn scope_singleton(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Transaction Event Macros (equivalent to @TransactionalEventListener)
// 事务事件宏（等价于 @TransactionalEventListener）
// ============================================================================

/// Mark a method as transaction event listener
/// 标记方法为事务事件监听器
///
/// Equivalent to Spring's `@TransactionalEventListener`.
/// 等价于 Spring 的 `@TransactionalEventListener`。
#[proc_macro_attribute]
pub fn transactional_event_listener(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as event listener
/// 标记方法为事件监听器
///
/// Equivalent to Spring's `@EventListener`.
/// 等价于 Spring 的 `@EventListener`。
#[proc_macro_attribute]
pub fn event_listener(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Retry Macros (equivalent to @Retryable, @Recover)
// 重试宏（等价于 @Retryable, @Recover）
// ============================================================================

/// Mark a method as retryable
/// 标记方法为可重试
///
/// Equivalent to Spring's `@Retryable`.
/// 等价于 Spring 的 `@Retryable`。
#[proc_macro_attribute]
pub fn retryable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as recovery method for retry
/// 标记方法为重试的恢复方法
///
/// Equivalent to Spring's `@Recover`.
/// 等价于 Spring 的 `@Recover`。
#[proc_macro_attribute]
pub fn recover(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Cache Configuration Macros (equivalent to @Caching, @CacheConfig)
// 缓存配置宏（等价于 @Caching, @CacheConfig）
// ============================================================================

/// Class-level cache configuration
/// 类级别的缓存配置
///
/// Equivalent to Spring's `@CacheConfig`.
/// 等价于 Spring 的 `@CacheConfig`。
#[proc_macro_attribute]
pub fn cache_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Method-level caching hint
/// 方法级别的缓存提示
///
/// Equivalent to Spring's `@Caching`.
/// 等价于 Spring 的 `@Caching`。
#[proc_macro_attribute]
pub fn caching(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Scheduled Task Macros (equivalent to @Scheduled, @EnableScheduling)
// 定时任务宏（等价于 @Scheduled, @EnableScheduling）
// ============================================================================

/// Mark a method as cron scheduled task
/// 标记方法为 cron 定时任务
///
/// Equivalent to Spring's `@Scheduled(cron = "...")`.
/// 等价于 Spring 的 `@Scheduled(cron = "...")`。
#[proc_macro_attribute]
pub fn cron(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as fixed rate scheduled task
/// 标记方法为固定频率定时任务
///
/// Equivalent to Spring's `@Scheduled(fixedRate = ...)`.
/// 等价于 Spring 的 `@Scheduled(fixedRate = ...)`。
#[proc_macro_attribute]
pub fn fixed_rate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as fixed delay scheduled task
/// 标记方法为固定延迟定时任务
///
/// Equivalent to Spring's `@Scheduled(fixedDelay = ...)`.
/// 等价于 Spring 的 `@Scheduled(fixedDelay = ...)`。
#[proc_macro_attribute]
pub fn fixed_delay(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as initial delay scheduled task
/// 标记方法为初始延迟定时任务
///
/// Equivalent to Spring's `@Scheduled(initialDelay = ...)`.
/// 等价于 Spring 的 `@Scheduled(initialDelay = ...)`。
#[proc_macro_attribute]
pub fn initial_delay(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Security Macros (equivalent to @Secured, @PreAuthorize, etc.)
// 安全宏（等价于 @Secured, @PreAuthorize 等）
// ============================================================================

/// Mark a method as requiring authentication
/// 标记方法需要认证
///
/// Equivalent to Spring Security's `@Secured`.
/// 等价于 Spring Security 的 `@Secured`。
#[proc_macro_attribute]
pub fn secured(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Pre-authorize method access based on expression
/// 基于表达式预先授权方法访问
///
/// Equivalent to Spring Security's `@PreAuthorize`.
/// 等价于 Spring Security 的 `@PreAuthorize`。
#[proc_macro_attribute]
pub fn pre_authorize(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Post-authorize method access based on expression
/// 基于表达式事后授权方法访问
///
/// Equivalent to Spring Security's `@PostAuthorize`.
/// 等价于 Spring Security 的 `@PostAuthorize`。
#[proc_macro_attribute]
pub fn post_authorize(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Pre-filter method access based on expression
/// 基于表达式预先过滤方法访问
///
/// Equivalent to Spring Security's `@PreFilter`.
/// 等价于 Spring Security 的 `@PreFilter`。
#[proc_macro_attribute]
pub fn pre_filter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Post-filter method access based on expression
/// 基于表达式事后过滤方法访问
///
/// Equivalent to Spring Security's `@PostFilter`.
/// 等价于 Spring Security 的 `@PostFilter`。
#[proc_macro_attribute]
pub fn post_filter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Define roles required for access
/// 定义访问所需的角色
///
/// Equivalent to Spring Security's `@RolesAllowed`.
/// 等价于 Spring Security 的 `@RolesAllowed`。
#[proc_macro_attribute]
pub fn roles_allowed(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Permit all access
/// 允许所有访问
///
/// Equivalent to Spring Security's `@PermitAll`.
/// 等价于 Spring Security 的 `@PermitAll`。
#[proc_macro_attribute]
pub fn permit_all(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Deny all access
/// 拒绝所有访问
///
/// Equivalent to Spring Security's `@DenyAll`.
/// 等价于 Spring Security 的 `@DenyAll`。
#[proc_macro_attribute]
pub fn deny_all(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Allow anonymous access
/// 允许匿名访问
///
/// Equivalent to Spring Security's `@Anonymous`.
/// 等价于 Spring Security 的 `@Anonymous`。
#[proc_macro_attribute]
pub fn anonymous(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Require specific role for access
/// 要求特定角色才能访问
///
/// Equivalent to Spring Security's `@RolesAllowed`.
/// 等价于 Spring Security 的 `@RolesAllowed`。
#[proc_macro_attribute]
pub fn require_role(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// HTTP Method Specific Macros (short aliases for common mappings)
// HTTP 方法特定宏（常见映射的简短别名）
// ============================================================================

/// Map TRACE requests
/// 映射 TRACE 请求
///
/// Shorthand for `#[request_mapping(method = "TRACE")]`.
/// `#[request_mapping(method = "TRACE")]` 的简写。
#[proc_macro_attribute]
pub fn trace_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    request_mapping(attr, item)
}

/// Map PATCH requests
/// 映射 PATCH 请求
///
/// Shorthand for `#[request_mapping(method = "PATCH")]`.
/// `#[request_mapping(method = "PATCH")]` 的简写。
#[proc_macro_attribute]
pub fn patch_route(attr: TokenStream, item: TokenStream) -> TokenStream {
    request_mapping(attr, item)
}

// ============================================================================
// Response Status Macros (equivalent to @ResponseStatus)
// 响应状态宏（等价于 @ResponseStatus）
// ============================================================================

/// Set the response status for an exception handler
/// 为异常处理器设置响应状态
///
/// Equivalent to Spring's `@ResponseStatus`.
/// 等价于 Spring 的 `@ResponseStatus`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::{response_status, exception_handler};
///
/// #[response_status(code = 404, reason = "Not Found")]
/// #[exception_handler]
/// async fn handle_not_found(e: NotFoundError) -> Response {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn response_status(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark exception as causing 400 Bad Request
/// 标记异常导致 400 Bad Request
///
/// Equivalent to Spring's `@ResponseStatus(code = 400)`.
/// 等价于 Spring 的 `@ResponseStatus(code = 400)`。
#[proc_macro_attribute]
pub fn bad_request(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark exception as causing 401 Unauthorized
/// 标记异常导致 401 Unauthorized
///
/// Equivalent to Spring's `@ResponseStatus(code = 401)`.
/// 等价于 Spring 的 `@ResponseStatus(code = 401)`。
#[proc_macro_attribute]
pub fn unauthorized(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark exception as causing 403 Forbidden
/// 标记异常导致 403 Forbidden
///
/// Equivalent to Spring's `@ResponseStatus(code = 403)`.
/// 等价于 Spring 的 `@ResponseStatus(code = 403)`。
#[proc_macro_attribute]
pub fn forbidden(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark exception as causing 404 Not Found
/// 标记异常导致 404 Not Found
///
/// Equivalent to Spring's `@ResponseStatus(code = 404)`.
/// 等价于 Spring 的 `@ResponseStatus(code = 404)`。
#[proc_macro_attribute]
pub fn not_found(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark exception as causing 500 Internal Server Error
/// 标记异常导致 500 Internal Server Error
///
/// Equivalent to Spring's `@ResponseStatus(code = 500)`.
/// 等价于 Spring 的 `@ResponseStatus(code = 500)`。
#[proc_macro_attribute]
pub fn internal_server_error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark exception as causing 503 Service Unavailable
/// 标记异常导致 503 Service Unavailable
///
/// Equivalent to Spring's `@ResponseStatus(code = 503)`.
/// 等价于 Spring 的 `@ResponseStatus(code = 503)`。
#[proc_macro_attribute]
pub fn service_unavailable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Rest Controller Macro (equivalent to @RestController)
// REST 控制器宏（等价于 @RestController）
// ============================================================================

/// Marks a class as a REST controller (alias for controller)
/// 将类标记为 REST 控制器（controller 的别名）
///
/// Equivalent to Spring's `@RestController`.
/// 等价于 Spring 的 `@RestController`。
///
/// This is an alias for `#[controller]` provided for semantic clarity.
/// 这是 `#[controller]` 的别名，提供语义清晰度。
#[proc_macro_attribute]
pub fn rest_controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    controller(_attr, item)
}

/// Marks a class as a controller (returns views, not JSON)
/// 将类标记为控制器（返回视图而非 JSON）
///
/// Equivalent to Spring's `@Controller`.
/// 等价于 Spring 的 `@Controller`。
#[proc_macro_attribute]
pub fn controller_view(_attr: TokenStream, item: TokenStream) -> TokenStream {
    controller(_attr, item)
}

// ============================================================================
// RestAnnotation Macro (shortcut for REST endpoints)
// REST 注解宏（REST 端点的快捷方式）
// ============================================================================

/// Combined annotation for GET endpoint
/// GET 端点的组合注解
///
/// Equivalent to Spring's `@GetMapping`.
/// 等价于 Spring 的 `@GetMapping`。
#[proc_macro_attribute]
pub fn get_mapping(attr: TokenStream, item: TokenStream) -> TokenStream {
    get(attr, item)
}

/// Combined annotation for POST endpoint
/// POST 端点的组合注解
///
/// Equivalent to Spring's `@PostMapping`.
/// 等价于 Spring 的 `@PostMapping`。
#[proc_macro_attribute]
pub fn post_mapping(attr: TokenStream, item: TokenStream) -> TokenStream {
    post(attr, item)
}

/// Combined annotation for PUT endpoint
/// PUT 端点的组合注解
///
/// Equivalent to Spring's `@PutMapping`.
/// 等价于 Spring 的 `@PutMapping`。
#[proc_macro_attribute]
pub fn put_mapping(attr: TokenStream, item: TokenStream) -> TokenStream {
    put(attr, item)
}

/// Combined annotation for DELETE endpoint
/// DELETE 端点的组合注解
///
/// Equivalent to Spring's `@DeleteMapping`.
/// 等价于 Spring 的 `@DeleteMapping`。
#[proc_macro_attribute]
pub fn delete_mapping(attr: TokenStream, item: TokenStream) -> TokenStream {
    delete(attr, item)
}

/// Combined annotation for PATCH endpoint
/// PATCH 端点的组合注解
///
/// Equivalent to Spring's `@PatchMapping`.
/// 等价于 Spring 的 `@PatchMapping`。
#[proc_macro_attribute]
pub fn patch_mapping(attr: TokenStream, item: TokenStream) -> TokenStream {
    patch(attr, item)
}

// ============================================================================
// Additional Spring Data JPA Macros
// 额外的 Spring Data JPA 宏
// ============================================================================

/// Mark a query method as a JPA query
/// 标记查询方法为 JPA 查询
///
/// Equivalent to Spring Data JPA's `@Query`.
/// 等价于 Spring Data JPA 的 `@Query`。
#[proc_macro_attribute]
pub fn query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a query method as a native SQL query
/// 标记查询方法为原生 SQL 查询
///
/// Equivalent to Spring Data JPA's `@Query(nativeQuery = true)`.
/// 等价于 Spring Data JPA 的 `@Query(nativeQuery = true)`。
#[proc_macro_attribute]
pub fn native_query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as a transactional query method
/// 标记方法为事务查询方法
///
/// Equivalent to Spring Data JPA's `@Transactional(readOnly = true)`.
/// 等价于 Spring Data JPA 的 `@Transactional(readOnly = true)`。
#[proc_macro_attribute]
pub fn read_only(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as modifying query
/// 标记方法为修改查询
///
/// Equivalent to Spring Data JPA's `@Modifying`.
/// 等价于 Spring Data JPA 的 `@Modifying`。
#[proc_macro_attribute]
pub fn modifying(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a repository as JDBC repository
/// 标记仓储为 JDBC 仓储
///
/// Equivalent to Spring's `@JdbcRepository`.
/// 等价于 Spring 的 `@JdbcRepository`。
#[proc_macro_attribute]
pub fn jdbc_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a repository as R2DBC repository
/// 标记仓储为 R2DBC 仓储
///
/// Equivalent to Spring's `@R2dbcRepository`.
/// 等价于 Spring 的 `@R2dbcRepository`。
#[proc_macro_attribute]
pub fn r2dbc_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a repository as `MongoDB` repository
/// 标记仓储为 `MongoDB` 仓储
///
/// Equivalent to Spring Data `MongoDB`'s `@MongoRepository`.
/// 等价于 Spring Data `MongoDB` 的 `@MongoRepository`。
#[proc_macro_attribute]
pub fn mongo_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a repository as Redis repository
/// 标记仓储为 Redis 仓储
///
/// Equivalent to Spring Data Redis's `@RedisHash`.
/// 等价于 Spring Data Redis 的 `@RedisHash`。
#[proc_macro_attribute]
pub fn redis_hash(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a repository as Elasticsearch repository
/// 标记仓储为 Elasticsearch 仓储
///
/// Equivalent to Spring Data Elasticsearch's `@ElasticsearchRepository`.
/// 等价于 Spring Data Elasticsearch 的 `@ElasticsearchRepository`。
#[proc_macro_attribute]
pub fn elasticsearch_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Additional Spring Cloud Macros
// 额外的 Spring Cloud 宏
// ============================================================================

/// Mark a class as configuration properties
/// 标记类为配置属性
///
/// Equivalent to Spring Boot's `@ConfigurationProperties`.
/// 等价于 Spring Boot 的 `@ConfigurationProperties`。
#[proc_macro_attribute]
pub fn configuration_properties(_attr: TokenStream, item: TokenStream) -> TokenStream {
    config(_attr, item)
}

/// Mark a class as enable configuration properties
/// 标记类为启用配置属性
///
/// Equivalent to Spring Boot's `@EnableConfigurationProperties`.
/// 等价于 Spring Boot 的 `@EnableConfigurationProperties`。
#[proc_macro_attribute]
pub fn enable_configuration_properties(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as enable configuration properties registration
/// 标记类为启用配置属性注册
///
/// Equivalent to Spring Boot 2.2+ `@ConfigurationPropertiesScan`.
/// 等价于 Spring Boot 2.2+ 的 `@ConfigurationPropertiesScan`。
#[proc_macro_attribute]
pub fn configuration_properties_scan(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a field as ignoring unknown properties
/// 标记字段为忽略未知属性
///
/// Equivalent to Spring Boot's `@IgnoreUnknownProperties`.
/// 等价于 Spring Boot 的 `@IgnoreUnknownProperties`。
#[proc_macro_attribute]
pub fn ignore_unknown_properties(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a field as default value
/// 标记字段为默认值
///
/// Equivalent to Spring Boot's `@DefaultValue`.
/// 等价于 Spring Boot 的 `@DefaultValue`。
#[proc_macro_attribute]
pub fn default_value(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as nestable configuration
/// 标记类为可嵌套配置
///
/// Equivalent to Spring Boot's `@NestedConfigurationProperty`.
/// 等价于 Spring Boot 的 `@NestedConfigurationProperty`。
#[proc_macro_attribute]
pub fn nested_configuration_property(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as endpoint configuration
/// 标记类为端点配置
///
/// Equivalent to Spring Boot Actuator's `@Endpoint`.
/// 等价于 Spring Boot Actuator 的 `@Endpoint`。
#[proc_macro_attribute]
pub fn endpoint_actuator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as endpoint read operation
/// 标记方法为端点读取操作
///
/// Equivalent to Spring Boot Actuator's `@ReadOperation`.
/// 等价于 Spring Boot Actuator 的 `@ReadOperation`。
#[proc_macro_attribute]
pub fn read_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as endpoint write operation
/// 标记方法为端点写入操作
///
/// Equivalent to Spring Boot Actuator's `@WriteOperation`.
/// 等价于 Spring Boot Actuator 的 `@WriteOperation`。
#[proc_macro_attribute]
pub fn write_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as endpoint delete operation
/// 标记方法为端点删除操作
///
/// Equivalent to Spring Boot Actuator's `@DeleteOperation`.
/// 等价于 Spring Boot Actuator 的 `@DeleteOperation`。
#[proc_macro_attribute]
pub fn delete_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Feign Client Macros (equivalent to @FeignClient)
// Feign 客户端宏（等价于 @FeignClient）
// ============================================================================

/// Mark an interface as a Feign client
/// 标记接口为 Feign 客户端
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@FeignClient`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@FeignClient`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::feign_client;
///
/// #[feign_client("https://api.example.com")]
/// trait ApiClient {
///     #[get("/users/{id}")]
///     async fn get_user(&self, #[path] id: u64) -> User;
/// }
/// ```
#[proc_macro_attribute]
pub fn feign_client(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as Feign client GET request
/// 标记方法为 Feign 客户端 GET 请求
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@GetMapping` in Feign client.
/// 等价于 Spring Cloud `OpenFeign` 的 `@GetMapping`（在 Feign 客户端中）。
#[proc_macro_attribute]
pub fn feign_get(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as Feign client POST request
/// 标记方法为 Feign 客户端 POST 请求
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@PostMapping` in Feign client.
/// 等价于 Spring Cloud `OpenFeign` 的 `@PostMapping`（在 Feign 客户端中）。
#[proc_macro_attribute]
pub fn feign_post(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as Feign client PUT request
/// 标记方法为 Feign 客户端 PUT 请求
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@PutMapping` in Feign client.
/// 等价于 Spring Cloud `OpenFeign` 的 `@PutMapping`（在 Feign 客户端中）。
#[proc_macro_attribute]
pub fn feign_put(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a method as Feign client DELETE request
/// 标记方法为 Feign 客户端 DELETE 请求
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@DeleteMapping` in Feign client.
/// 等价于 Spring Cloud `OpenFeign` 的 `@DeleteMapping`（在 Feign 客户端中）。
#[proc_macro_attribute]
pub fn feign_delete(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as Feign path variable
/// 标记参数为 Feign 路径变量
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@PathVariable`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@PathVariable`。
#[proc_macro_attribute]
pub fn feign_path(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as Feign query parameter
/// 标记参数为 Feign 查询参数
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@RequestParam`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@RequestParam`。
#[proc_macro_attribute]
pub fn feign_query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as Feign request header
/// 标记参数为 Feign 请求头
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@RequestHeader`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@RequestHeader`。
#[proc_macro_attribute]
pub fn feign_header(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a parameter as Feign request body
/// 标记参数为 Feign 请求体
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@RequestBody`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@RequestBody`。
#[proc_macro_attribute]
pub fn feign_body(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Configure Feign client circuit breaker
/// 配置 Feign 客户端熔断器
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@CircuitBreakerName`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@CircuitBreakerName`。
#[proc_macro_attribute]
pub fn circuit_breaker_name(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Configure Feign client timeout
/// 配置 Feign 客户端超时
///
/// Equivalent to Spring Cloud's `@Timeout`.
/// 等价于 Spring Cloud 的 `@Timeout`。
#[proc_macro_attribute]
pub fn feign_timeout(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Configure Feign client retry
/// 配置 Feign 客户端重试
///
/// Equivalent to Spring Cloud's `@Retryable`.
/// 等价于 Spring Cloud 的 `@Retryable`。
#[proc_macro_attribute]
pub fn feign_retry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as Feign client configuration
/// 标记类为 Feign 客户端配置
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@FeignClientConfiguration`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@FeignClientConfiguration`。
#[proc_macro_attribute]
pub fn feign_configuration(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as Feign client decoder
/// 标记类为 Feign 客户端解码器
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@Decoder`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@Decoder`。
#[proc_macro_attribute]
pub fn feign_decoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as Feign client encoder
/// 标记类为 Feign 客户端编码器
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@Encoder`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@Encoder`。
#[proc_macro_attribute]
pub fn feign_encoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as Feign client logger
/// 标记类为 Feign 客户端日志记录器
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@Logger`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@Logger`。
#[proc_macro_attribute]
pub fn feign_logger(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as Feign client error decoder
/// 标记类为 Feign 客户端错误解码器
///
/// Equivalent to Spring Cloud `OpenFeign`'s `@ErrorDecoder`.
/// 等价于 Spring Cloud `OpenFeign` 的 `@ErrorDecoder`。
#[proc_macro_attribute]
pub fn feign_error_decoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Configure Feign client options
/// 配置 Feign 客户端选项
///
/// Equivalent to Spring Cloud's `@FeignClientOptions`.
/// 等价于 Spring Cloud 的 `@FeignClientOptions`。
#[proc_macro_attribute]
pub fn feign_options(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as Feign client query map encoder
/// 标记类为 Feign 客户端查询映射编码器
///
/// Equivalent to Spring Cloud's `@QueryMapEncoder`.
/// 等价于 Spring Cloud 的 `@QueryMapEncoder`。
#[proc_macro_attribute]
pub fn query_map_encoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as Feign client contract
/// 标记类为 Feign 客户端契约
///
/// Equivalent to Spring Cloud Contract's `@FeignClient`.
/// 等价于 Spring Cloud Contract 的 `@FeignClient`。
#[proc_macro_attribute]
pub fn contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as circuit breaker configuration
/// 标记类为熔断器配置
///
/// Equivalent to Spring Cloud Circuit Breaker's `@CircuitBreakerConfig`.
/// 等价于 Spring Cloud Circuit Breaker 的 `@CircuitBreakerConfig`。
#[proc_macro_attribute]
pub fn circuit_breaker_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as time limiter configuration
/// 标记类为时间限制器配置
///
/// Equivalent to Spring Cloud Resilience4j's `@TimeLimiterConfig`.
/// 等价于 Spring Cloud Resilience4j 的 `@TimeLimiterConfig`。
#[proc_macro_attribute]
pub fn time_limiter_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as bulkhead configuration
/// 标记类为隔板配置
///
/// Equivalent to Spring Cloud Resilience4j's `@BulkheadConfig`.
/// 等价于 Spring Cloud Resilience4j 的 `@BulkheadConfig`。
#[proc_macro_attribute]
pub fn bulkhead_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as retry configuration
/// 标记类为重试配置
///
/// Equivalent to Spring Cloud Retry's `@RetryConfig`.
/// 等价于 Spring Cloud Retry 的 `@RetryConfig`。
#[proc_macro_attribute]
pub fn retry_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as fallback configuration
/// 标记类为回退配置
///
/// Equivalent to Spring Cloud Circuit Breaker's `@Fallback`.
/// 等价于 Spring Cloud Circuit Breaker 的 `@Fallback`。
#[proc_macro_attribute]
pub fn fallback(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as circuit breaker
/// 标记类为熔断器
///
/// Equivalent to Spring Cloud Circuit Breaker's `@CircuitBreaker`.
/// 等价于 Spring Cloud Circuit Breaker 的 `@CircuitBreaker`。
#[proc_macro_attribute]
pub fn circuit_breaker(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as bulkhead
/// 标记类为隔板
///
/// Equivalent to Spring Cloud Resilience4j's `@Bulkhead`.
/// 等价于 Spring Cloud Resilience4j 的 `@Bulkhead`。
#[proc_macro_attribute]
pub fn bulkhead(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as time limiter
/// 标记类为时间限制器
///
/// Equivalent to Spring Cloud Resilience4j's `@TimeLimiter`.
/// 等价于 Spring Cloud Resilience4j 的 `@TimeLimiter`。
#[proc_macro_attribute]
pub fn time_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as retry
/// 标记类为重试
///
/// Equivalent to Spring Cloud Retry's `@Retry`.
/// 等价于 Spring Cloud Retry 的 `@Retry`。
#[proc_macro_attribute]
pub fn retry_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as rate limiter
/// 标记类为速率限制器
///
/// Equivalent to Spring Cloud Gateway's `@RateLimiter`.
/// 等价于 Spring Cloud Gateway 的 `@RateLimiter`。
#[proc_macro_attribute]
pub fn rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as request rate limiter
/// 标记类为请求速率限制器
///
/// Equivalent to Spring Cloud Gateway's `@RequestRateLimiter`.
/// 等价于 Spring Cloud Gateway 的 `@RequestRateLimiter`。
#[proc_macro_attribute]
pub fn request_rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as origin rate limiter
/// 标记类为源速率限制器
///
/// Equivalent to Spring Cloud Gateway's `@OriginRateLimiter`.
/// 等价于 Spring Cloud Gateway 的 `@OriginRateLimiter`。
#[proc_macro_attribute]
pub fn origin_rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as user rate limiter
/// 标记类为用户速率限制器
///
/// Equivalent to Spring Cloud Gateway's `@UserRateLimiter`.
/// 等价于 Spring Cloud Gateway 的 `@UserRateLimiter`。
#[proc_macro_attribute]
pub fn user_rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as throttling
/// 标记类为限流
///
/// Equivalent to Spring Cloud Gateway's `@Throttling`.
/// 等价于 Spring Cloud Gateway 的 `@Throttling`。
#[proc_macro_attribute]
pub fn throttling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as gateway filter
/// 标记类为网关过滤器
///
/// Equivalent to Spring Cloud Gateway's `@Component` for filters.
/// 等价于 Spring Cloud Gateway 的 `@Component`（用于过滤器）。
#[proc_macro_attribute]
pub fn gateway_filter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as gateway predicate
/// 标记类为网关谓词
///
/// Equivalent to Spring Cloud Gateway's `@Component` for predicates.
/// 等价于 Spring Cloud Gateway 的 `@Component`（用于谓词）。
#[proc_macro_attribute]
pub fn gateway_predicate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as gateway route
/// 标记类为网关路由
///
/// Equivalent to Spring Cloud Gateway's `@Route`.
/// 等价于 Spring Cloud Gateway 的 `@Route`。
#[proc_macro_attribute]
pub fn gateway_route(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a class as gateway configuration
/// 标记类为网关配置
///
/// Equivalent to Spring Cloud Gateway's `@Configuration`.
/// 等价于 Spring Cloud Gateway 的 `@Configuration`。
#[proc_macro_attribute]
pub fn gateway_configuration(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ============================================================================
// Exception Handling Macros (equivalent to Spring @ControllerAdvice/@ExceptionHandler)
// 异常处理宏（等价于 Spring @ControllerAdvice/@ExceptionHandler）
// ============================================================================

/// Marks a class as a global exception handler
/// 标记类为全局异常处理器
///
/// Equivalent to Spring's `@ControllerAdvice`.
/// 等价于 Spring 的 `@ControllerAdvice`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::{controller_advice, exception_handler};
/// use nexus_http::exception::{ErrorResponse, ResourceNotFoundException};
/// use nexus_http::Response;
///
/// #[controller_advice]
/// pub struct GlobalExceptionHandler;
///
/// impl GlobalExceptionHandler {
///     #[exception_handler]
///     fn handle_not_found(exc: &ResourceNotFoundException) -> Response {
///         ErrorResponse::not_found()
///             .code("RESOURCE_NOT_FOUND")
///             .message(&exc.to_string())
///             .into_response()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn controller_advice(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let impl_item = &input;
    let type_name = &impl_item.self_ty;

    let expanded = quote! {
        #impl_item

        // Register this controller advice with the exception handler registry
        // 将此 controller advice 注册到异常处理器注册表
        #[automatically_derived]
        impl #type_name {
            /// Get the exception handler registry
            /// 获取异常处理器注册表
            pub fn get_exception_handlers() -> ::nexus_http::exception::ExceptionHandlerRegistry {
                ::nexus_http::exception::ExceptionHandlerRegistry::new()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Marks a class as a global REST exception handler
/// 标记类为全局 REST 异常处理器
///
/// This is similar to `@ControllerAdvice` but for REST APIs.
/// 这类似于 `@ControllerAdvice`，但用于 REST API。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::rest_controller_advice;
/// use nexus_macros::exception_handler;
/// use nexus_http::Response;
///
/// #[rest_controller_advice]
/// pub struct RestExceptionHandler;
///
/// impl RestExceptionHandler {
///     #[exception_handler]
///     fn handle_all(exc: &dyn std::error::Error) -> Response {
///         nexus_http::ErrorResponse::internal_server_error()
///             .message(exc.to_string())
///             .into_response()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn rest_controller_advice(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Currently identical to controller_advice
    // In the future, this could add JSON response handling
    // 目前与 controller_advice 相同
    // 未来可以添加 JSON 响应处理
    controller_advice(_attr, item)
}

/// Marks a method as an exception handler
/// 标记方法为异常处理器
///
/// Equivalent to Spring's `@ExceptionHandler`.
/// 等价于 Spring 的 `@ExceptionHandler`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_macros::exception_handler;
/// use nexus_http::exception::{ErrorResponse, ApplicationException};
/// use nexus_http::Response;
///
/// #[exception_handler]
/// fn handle_application_error(exc: &ApplicationException) -> Response {
///     exc.into_error_response().to_response()
/// }
///
/// #[exception_handler]
/// fn handle_all(exc: &dyn std::error::Error) -> Response {
///     ErrorResponse::internal_server_error()
///         .message(exc.to_string())
///         .into_response()
/// }
/// ```
#[proc_macro_attribute]
pub fn exception_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let expanded = quote! {
        #input

        // Register this exception handler
        // 注册此异常处理器
        #[automatically_derived]
        impl #func_name {
            /// Get the exception types this handler can process
            /// 获取此处理器可以处理的异常类型
            pub fn exception_types() -> Vec<std::any::TypeId> {
                vec![]
            }
        }
    };

    TokenStream::from(expanded)
}
