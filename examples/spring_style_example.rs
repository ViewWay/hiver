//! Hiver Framework - Spring Boot Style Example
//! Hiver框架 - Spring Boot风格示例
//!
//! This example shows how to use Hiver framework with Spring Boot-like annotations.
//! 此示例展示如何使用Hiver框架和类似Spring Boot的注解。
//!
//! # Equivalent Spring Boot Code / 等价的Spring Boot代码
//!
//! ```java
//! @SpringBootApplication
//! @RestController
//! public class DemoApplication {
//!
//!     private static final Logger log = LoggerFactory.getLogger(DemoApplication.class);
//!
//!     @Value("${app.name:Hiver Application}")
//!     private String appName;
//!
//!     @GetMapping("/helloworld")
//!     public String hello() {
//!         log.info("Handling hello request");
//!         return "Hello World!";
//!     }
//! }
//! ```

use hiver_observability::log::{Logger, LoggerFactory};

// ============================================================================
// @SpringBootApplication equivalent
// @SpringBootApplication 等价物
// ============================================================================

#[hiver_macros::main]
struct Application;

// ============================================================================
// @RestController equivalent
// @RestController 等价物
// ============================================================================

#[hiver_macros::controller]
#[hiver_macros::slf4j] // Equivalent to Lombok @Slf4j
struct DemoController {
    // Logger is automatically added by #[slf4j] macro
    // 日志记录器由 #[slf4j] 宏自动添加
}

impl DemoController {
    // Equivalent to @GetMapping("/helloworld")
    // 等价于 @GetMapping("/helloworld")
    #[hiver_macros::get("/helloworld")]
    fn hello(&self) -> &'static str {
        self.log()
            .info(format_args!("Handling /helloworld request"));
        "Hello World!"
    }

    // Equivalent to @GetMapping("/user/{id}")
    // 等价于 @GetMapping("/user/{id}")
    #[hiver_macros::get("/user/{id}")]
    fn get_user(&self, id: u64) -> String {
        self.log()
            .info(format_args!("Getting user with id: {}", id));
        format!("User: {}", id)
    }

    // Equivalent to @PostMapping("/user")
    // 等价于 @PostMapping("/user")
    #[hiver_macros::post("/user")]
    fn create_user(&self, #[hiver_macros::body] user: String) -> String {
        self.log().info(format_args!("Creating user: {}", user));
        format!("Created: {}", user)
    }
}

// ============================================================================
// @Service equivalent
// @Service 等价物
// ============================================================================

#[hiver_macros::service]
#[hiver_macros::slf4j]
struct UserService {
    repository: std::sync::Arc<UserRepository>,
}

impl UserService {
    fn new(repository: std::sync::Arc<UserRepository>) -> Self {
        Self { repository }
    }

    #[hiver_macros::cacheable("users")]
    async fn find_by_id(&self, id: u64) -> Option<String> {
        self.log().debug(format_args!("Finding user by id: {}", id));
        self.repository.find_by_id(id).await
    }
}

// ============================================================================
// @Repository equivalent
// @Repository 等价物
// ============================================================================

#[hiver_macros::repository]
trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: u64) -> Option<String>;
}

// ============================================================================
// Main function with logger initialization
// 带日志初始化的主函数
// ============================================================================

fn main() -> std::io::Result<()> {
    // Initialize Spring Boot style logging
    // 初始化Spring Boot风格日志
    Logger::init_spring_style().expect("Failed to initialize logger");

    let log = LoggerFactory::get("main");
    log.info(format_args!("Starting Hiver Application..."));

    Application::run()
}

// ============================================================================
// More examples with Spring annotations
// 更多Spring注解示例
// ============================================================================

/*
// Equivalent to @ConfigurationProperties
// 等价于 @ConfigurationProperties
#[hiver_macros::config(prefix = "app")]
struct AppConfig {
    name: String,
    port: u16,
}

// Equivalent to @Profile("dev")
// 等价于 @Profile("dev")
#[hiver_macros::profile("dev")]
#[hiver_macros::service]
struct DevService {
    // Only available in dev profile
    // 仅在dev配置文件中可用
}

// Equivalent to @Transactional
// 等价于 @Transactional
#[hiver_macros::transactional]
async fn transfer_money(from: u64, to: u64, amount: f64) -> Result<(), Error> {
    // Database operations here will be executed in a transaction
    // 这里的数据库操作将在事务中执行
    Ok(())
}

// Equivalent to @Scheduled(cron = "0 * * * * *")
// 等价于 @Scheduled(cron = "0 * * * * *")
#[hiver_macros::scheduled(cron = "0 * * * * *")]
async fn cleanup_task() {
    // Run every hour
    // 每小时运行一次
}

// Equivalent to @ExceptionHandler
// 等价于 @ExceptionHandler
#[hiver_macros::exception_handler]
async fn handle_not_found(e: NotFoundError) -> Response {
    Response::builder()
        .status(404)
        .body(e.to_string())
        .unwrap()
}
*/
