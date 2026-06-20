//! # Annotated Application Example (full `#[hiver_main]` path)
//! # 注解驱动应用示例(完整 `#[hiver_main]` 路径)
//!
//! Demonstrates the Spring-Boot-style fully-annotated entry point, where the
//! framework auto-wires everything:
//! 1. `#[hiver_main]` boots Hiver's own runtime, loads config, runs
//!    auto-configurations, collects routes via `inventory`, and binds + serves.
//! 2. `#[controller]` registers the controller as a singleton bean.
//! 3. `#[get]` contributes routes discovered at startup via `inventory`.
//!
//! 演示 Spring Boot 风格的全注解入口,框架自动接通一切:
//! 1. `#[hiver_main]` 启动 Hiver 自有 runtime、加载配置、运行自动配置、经
//!    `inventory` 收集路由,并绑定 + 提供服务。
//! 2. `#[controller]` 将控制器注册为单例 bean。
//! 3. `#[get]` 经 `inventory` 贡献启动时被发现的路由。
//!
//! ## Run / 运行
//!
//! ```bash
//! cargo run -p hiver-examples --bin annotated_app_example
//! curl http://127.0.0.1:8080/hello
//! curl http://127.0.0.1:8080/greet/world
//! ```
//!
//! Unlike `http_server_example` (which wires routes by hand via `.get()`),
//! this one uses *only* annotations — no manual router construction.
//! 与 `http_server_example`(手动经 `.get()` 接线路由)不同,本示例*仅*使用
//! 注解 —— 无手动 router 构造。

use hiver_macros::{controller, get, hiver_main};
use hiver_http::Request;

/// The application entry point. `#[hiver_main]` generates `MyApp::run()` which
/// boots the runtime, runs auto-configurations, collects all `#[get]` routes,
/// and serves on `server.port` (default 8080).
/// 应用入口。`#[hiver_main]` 生成 `MyApp::run()`,启动 runtime、运行自动配置、
/// 收集所有 `#[get]` 路由,并在 `server.port`(默认 8080)上服务。
#[hiver_main]
struct MyApp;

/// A REST controller, registered as a singleton bean. It exists to show the
/// `#[controller]` stereotype working end-to-end; the routes below are free
/// functions discovered via `inventory`.
/// REST 控制器,注册为单例 bean。其存在是为展示 `#[controller]` 构造型端到端
/// 生效;下面的路由是经 `inventory` 发现的自由函数。
#[derive(Default)]
#[controller]
struct HelloController;

/// `GET /hello` — no-arg handler returning a static string. Registered via the
/// `#[get]` macro + `inventory`; `#[hiver_main]`'s `RouterAutoConfiguration`
/// collects it automatically.
/// `GET /hello` —— 无参处理程序,返回静态字符串。经 `#[get]` 宏 + `inventory`
/// 注册;`#[hiver_main]` 的 `RouterAutoConfiguration` 自动收集它。
#[get("/hello")]
async fn hello() -> &'static str
{
    "Hello from annotated Hiver!"
}

/// `GET /greet/{name}` — demonstrates a path-parameter extractor via the raw
/// `Request` (identity extractor). Returns a static string to keep the handler
/// signature aligned with the macro's `IntoResponse` path.
/// `GET /greet/{name}` —— 经原始 `Request`(身份提取器)演示路径参数提取。
/// 返回静态字符串,使处理程序签名与宏的 `IntoResponse` 路径一致。
#[get("/greet/{name}")]
async fn greet(_req: Request) -> &'static str
{
    "Hello, friend!"
}

/// Standard binary entry point — just delegate to the macro-generated runner.
/// 标准二进制入口 —— 仅委托给宏生成的运行器。
fn main() -> anyhow::Result<()>
{
    MyApp::run()
}
