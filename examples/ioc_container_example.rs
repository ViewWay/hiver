//! IoC Container Example / IoC容器示例
//!
//! This example demonstrates the Inversion of Control (IoC) container
//! and dependency injection features equivalent to Spring Boot's:
//!
//! 此示例演示了控制反转(IoC)容器和依赖注入功能，等价于Spring Boot的：
//!
//! - `@Component` → Component scanning
//! - `@Service` → Service layer beans
//! - `@Repository` → Data access layer beans
//! - `@Autowired` → Constructor injection
//! - `ApplicationContext` → Bean lookup and lifecycle management

use hiver_core::container::{
    ApplicationContext, BeanRegistration, Container, PostConstruct, PreDestroy,
};
use hiver_core::error::{Error, Result};
use std::sync::Arc;

// ============================================================================
// Domain Models / 领域模型
// ============================================================================

/// User entity
/// 用户实体
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: u64, name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            email: email.into(),
        }
    }
}

// ============================================================================
// Repository Layer (@Repository equivalent)
// ============================================================================

/// User repository
/// 用户仓库
///
/// Equivalent to Spring's `@Repository`.
/// 等价于Spring的`@Repository`。
#[derive(Debug)]
pub struct UserRepository {
    users: Arc<std::sync::Mutex<Vec<User>>>,
}

impl UserRepository {
    /// Create a new repository
    /// 创建新仓库
    pub fn new() -> Self {
        Self {
            users: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Find user by ID
    /// 按ID查找用户
    pub fn find_by_id(&self, id: u64) -> Option<User> {
        let users = self.users.lock().unwrap();
        users.iter().find(|u| u.id == id).cloned()
    }

    /// Save user
    /// 保存用户
    pub fn save(&self, user: &User) {
        let mut users = self.users.lock().unwrap();
        if let Some(existing) = users.iter_mut().find(|u| u.id == user.id) {
            *existing = user.clone();
        } else {
            users.push(user.clone());
        }
    }

    /// Get all users
    /// 获取所有用户
    pub fn find_all(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        users.clone()
    }
}

impl Default for UserRepository {
    fn default() -> Self {
        Self::new()
    }
}

// Bean is automatically implemented via blanket impl
// Bean通过blanket impl自动实现

// ============================================================================
// Service Layer (@Service equivalent)
// ============================================================================

/// User service
/// 用户服务
///
/// Equivalent to Spring's `@Service` with constructor injection.
/// 等价于Spring的`@Service`配合构造函数注入。
#[derive(Debug)]
pub struct UserService {
    /// Injected dependency (@Autowired)
    /// 注入的依赖
    repository: Arc<UserRepository>,
}

impl UserService {
    /// Constructor with dependency injection
    /// 带依赖注入的构造函数
    ///
    /// Equivalent to Spring's:
    /// 等价于Spring的：
    /// ```java
    /// @Autowired
    /// public UserService(UserRepository repository) {
    ///     this.repository = repository;
    /// }
    /// ```
    pub fn new(repository: Arc<UserRepository>) -> Self {
        Self { repository }
    }

    /// Get user by ID
    /// 按ID获取用户
    pub fn get_user(&self, id: u64) -> Option<User> {
        self.repository.find_by_id(id)
    }

    /// Create or update user
    /// 创建或更新用户
    pub fn save_user(&self, user: User) -> Result<User> {
        self.repository.save(&user);
        Ok(user)
    }

    /// Get all users
    /// 获取所有用户
    pub fn list_users(&self) -> Result<Vec<User>> {
        Ok(self.repository.find_all())
    }
}

// ============================================================================
// Component with Lifecycle Callbacks
// 带生命周期回调的组件
// ============================================================================

/// Email service component with lifecycle callbacks
/// 带生命周期回调的邮件服务组件
///
/// Demonstrates `@PostConstruct` and `@PreDestroy` equivalent functionality.
/// 演示`@PostConstruct`和`@PreDestroy`等价功能。
#[derive(Debug)]
pub struct EmailService {
    pub smtp_server: String,
}

impl EmailService {
    pub fn new(smtp_server: impl Into<String>) -> Self {
        Self {
            smtp_server: smtp_server.into(),
        }
    }

    pub fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        println!("Sending email to {} via {}: {}", to, self.smtp_server, subject);
        println!("Body: {}", body);
        Ok(())
    }
}

// ============================================================================
// PostConstruct and PreDestroy Implementation
// PostConstruct和PreDestroy实现
// ============================================================================

/// User service with lifecycle callbacks
/// 带生命周期回调的用户服务
pub struct UserServiceWithLifecycle {
    repository: Arc<UserRepository>,
    pub post_construct_called: bool,
}

impl UserServiceWithLifecycle {
    pub fn new(repository: Arc<UserRepository>) -> Self {
        Self {
            repository,
            post_construct_called: false,
        }
    }

    pub fn get_user(&self, id: u64) -> Option<User> {
        self.repository.find_by_id(id)
    }
}

impl PostConstruct for UserServiceWithLifecycle {
    fn post_construct(&self) -> Result<()> {
        println!("[UserService] @PostConstruct called - initializing service");
        // Simulate initialization logic
        // 模拟初始化逻辑
        Ok(())
    }
}

impl PreDestroy for UserServiceWithLifecycle {
    fn pre_destroy(&self) -> Result<()> {
        println!("[UserService] @PreDestroy called - cleaning up resources");
        // Simulate cleanup logic
        // 模拟清理逻辑
        Ok(())
    }
}

// ============================================================================
// Main Application (Demonstrates IoC Container Usage)
// 主应用程序（演示IoC容器用法）
// ============================================================================

fn main() -> Result<()> {
    println!("=== IoC Container Example / IoC容器示例 ===\n");

    // ========================================================================
    // Example 1: Basic Container Usage
    // 示例1：基本容器用法
    // ========================================================================

    println!("--- Example 1: Basic Container Usage ---");
    println!("--- 示例1：基本容器用法 ---\n");

    let mut container = Container::new();

    // Register beans with factory functions (constructor injection)
    // 使用工厂函数注册bean（构造函数注入）
    //
    // Equivalent to Spring's @Bean method in @Configuration class:
    // 等价于Spring中@Configuration类里的@Bean方法：
    // ```java
    // @Bean
    // public UserRepository userRepository() {
    //     return new UserRepository();
    // }
    // ```
    container.register::<UserRepository, _>(|_c| Ok(UserRepository::new()))?;

    // Register UserService with dependency injection
    // 注册UserService并注入依赖
    //
    // Equivalent to:
    // 等价于：
    // ```java
    // @Bean
    // public UserService userService(UserRepository repository) {
    //     return new UserService(repository);
    // }
    // ```
    container.register::<UserService, _>(|c| {
        let repo = c.get_bean::<UserRepository>()?;
        Ok(UserService::new(repo))
    })?;

    // Get bean from container (singleton scope by default)
    // 从容器获取bean（默认为单例作用域）
    let user_service: Arc<UserService> = container.get_bean()?;

    // Use the service
    // 使用服务
    let user = User::new(1, "Alice", "alice@example.com");
    user_service.save_user(user)?;
    user_service.save_user(User::new(2, "Bob", "bob@example.com"))?;

    println!("Users saved successfully!");

    if let Some(found) = user_service.get_user(1) {
        println!("Found user: {} ({})", found.name, found.email);
    }

    println!();

    // ========================================================================
    // Example 2: ApplicationContext (Spring Boot equivalent)
    // 示例2：ApplicationContext（Spring Boot等价物）
    // ========================================================================

    println!("--- Example 2: ApplicationContext ---");
    println!("--- 示例2：ApplicationContext ---\n");

    let mut context = ApplicationContext::new();

    // Set active profile (equivalent to --spring.profiles.active)
    // 设置活动配置文件（等价于 --spring.profiles.active）
    context.set_profile("dev");
    println!("Active profile: {}", context.profile());

    // Register beans directly
    // 直接注册bean
    context.register(UserRepository::new())?;
    context.register_with::<UserService, _>(|c| {
        let repo = c.get_bean::<UserRepository>()?;
        Ok(UserService::new(repo))
    })?;

    // Start the context (initializes eager singletons)
    // 启动上下文（初始化急切单例）
    context.start()?;
    println!("Context started, active: {}", context.is_active());

    println!();

    // ========================================================================
    // Example 3: Bean Registration with Configuration
    // 示例3：带配置的Bean注册
    // ========================================================================

    println!("--- Example 3: Advanced Bean Registration ---");
    println!("--- 示例3：高级Bean注册 ---\n");

    let mut container = Container::new();

    // Register with full configuration options
    // 使用完整配置选项注册
    let email_registration = BeanRegistration::new("emailService")
        .factory(Arc::new(|_: &Container| Ok(EmailService::new("smtp.example.com"))))
        .post_construct(|email: &EmailService| {
            println!("[EmailService] @PostConstruct - connecting to {}", email.smtp_server);
            Ok(())
        })
        .pre_destroy(|email: &EmailService| {
            println!("[EmailService] @PreDestroy - disconnecting from {}", email.smtp_server);
            Ok(())
        })
        .scope(hiver_core::bean::Scope::Singleton)
        .lazy(false);

    container.register_with(email_registration)?;

    // Get and use the bean
    // 获取并使用bean
    let email_service: Arc<EmailService> = container.get_bean()?;
    email_service.send_email("user@example.com", "Welcome", "Hello!")?;

    println!();

    // ========================================================================
    // Example 4: Bean Lookup by Name
    // 示例4：按名称查找Bean
    // ========================================================================

    println!("--- Example 4: Bean Lookup by Name ---");
    println!("--- 示例4：按名称查找Bean ---\n");

    let mut container = Container::new();

    container.register::<UserRepository, _>(|_| Ok(UserRepository::new()))?;

    // Get bean by name using the type name
    // 使用类型名称按名称获取bean
    // Note: Bean names use the full type path from std::any::type_name
    // 注意：Bean名称使用来自std::any::type_name的完整类型路径
    let repo: Arc<UserRepository> =
        container.get_bean_by_name("ioc_container_example::UserRepository")?;

    println!("Retrieved repository by name: {:?}", repo);

    // Check if bean exists
    // 检查bean是否存在
    println!("Has UserRepository: {}", container.has_bean::<UserRepository>());

    println!();

    // ========================================================================
    // Example 5: Shutdown and Cleanup
    // 示例5：关闭和清理
    // ========================================================================

    println!("--- Example 5: Container Lifecycle ---");
    println!("--- 示例5：容器生命周期 ---\n");

    let mut context = ApplicationContext::new();
    context.register(UserRepository::new())?;
    context.start()?;

    println!("Context is active: {}", context.is_active());

    // Close the context (calls @PreDestroy callbacks)
    // 关闭上下文（调用@PreDestroy回调）
    context.close()?;
    println!("Context closed successfully");

    println!();
    println!("=== Example Complete / 示例完成 ===");

    Ok(())
}
