//! @Transactional Annotation Examples / @Transactional 注解示例
//!
//! This example demonstrates the use of the @Transactional annotation
//! for automatic transaction management in the Hiver framework.
//!
//! 此示例演示了在 Hiver 框架中使用 @Transactional 注解进行自动事务管理。

use hiver_data_annotations::Transactional;
use hiver_data_annotations::transactional::{
    IsolationLevel, Propagation, TransactionalConfig,
};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Example 1: Basic Transactional Method / 基础事务方法
// ============================================================================

struct UserService {
    // In a real application, this would be a database connection pool
    // 在实际应用中，这应该是一个数据库连接池
    users: Arc<RwLock<Vec<User>>>,
}

#[derive(Debug, Clone)]
struct User {
    id: i64,
    username: String,
    email: String,
    balance: i64,
}

impl UserService {
    fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Basic transactional method - uses default configuration
    /// 基础事务方法 - 使用默认配置
    ///
    /// Equivalent to Spring's:
    /// 等价于 Spring 的：
    /// ```java
    /// @Transactional
    /// public void createUser(User user) { ... }
    /// ```
    #[Transactional]
    async fn create_user(&self, user: User) -> Result<(), String> {
        // Simulate database operations
        // 模拟数据库操作
        let mut users = self.users.write().await;
        users.push(user);
        Ok(())
    }

    /// Transactional method with custom isolation level
    /// 带有自定义隔离级别的事务方法
    ///
    /// Equivalent to Spring's:
    /// 等价于 Spring 的：
    /// ```java
    /// @Transactional(isolation = Isolation.READ_COMMITTED)
    /// public void transferFunds(Long from, Long to, Long amount) { ... }
    /// ```
    #[Transactional(isolation = ReadCommitted)]
    async fn transfer_funds(&self, from_id: i64, to_id: i64, amount: i64) -> Result<(), String> {
        let mut users = self.users.write().await;

        // Find users
        // 查找用户
        let from_user = users
            .iter_mut()
            .find(|u| u.id == from_id)
            .ok_or("From user not found")?;
        let to_user = users
            .iter_mut()
            .find(|u| u.id == to_id)
            .ok_or("To user not found")?;

        // Check balance
        // 检查余额
        if from_user.balance < amount {
            return Err("Insufficient funds".to_string());
        }

        // Transfer funds
        // 转账
        from_user.balance -= amount;
        to_user.balance += amount;

        println!(
            "✅ Transferred {} from {} to {}",
            amount, from_user.username, to_user.username
        );

        Ok(())
    }
}

// ============================================================================
// Example 2: Advanced Transactional Configuration / 高级事务配置
// ============================================================================

struct BankingService {
    users: Arc<RwLock<Vec<User>>>,
}

impl BankingService {
    fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Transactional method with full configuration
    /// 完整配置的事务方法
    ///
    /// Equivalent to Spring's:
    /// 等价于 Spring 的：
    /// ```java
    /// @Transactional(
    ///     isolation = Isolation.SERIALIZABLE,
    ///     propagation = Propagation.REQUIRES_NEW,
    ///     timeout = 60,
    ///     readOnly = false
    /// )
    /// public void criticalOperation() { ... }
    /// ```
    #[Transactional(
        isolation = Serializable,
        propagation = RequiresNew,
        timeout = 60,
        read_only = false,
        max_retries = 5
    )]
    async fn critical_operation(&self, user_id: i64, amount: i64) -> Result<(), String> {
        let mut users = self.users.write().await;

        let user = users
            .iter_mut()
            .find(|u| u.id == user_id)
            .ok_or("User not found")?;

        user.balance += amount;

        println!("✅ Critical operation completed for user {}", user.username);
        Ok(())
    }

    /// Read-only transactional method
    /// 只读事务方法
    ///
    /// Equivalent to Spring's:
    /// 等价于 Spring 的：
    /// ```java
    /// @Transactional(readOnly = true)
    /// public User getUserById(Long id) { ... }
    /// ```
    #[Transactional(read_only = true)]
    async fn get_user_by_id(&self, id: i64) -> Result<Option<User>, String> {
        let users = self.users.read().await;
        Ok(users.iter().find(|u| u.id == id).cloned())
    }

    /// Nested transactional method
    /// 嵌套事务方法
    ///
    /// Equivalent to Spring's:
    /// 等价于 Spring 的：
    /// ```java
    /// @Transactional(propagation = Propagation.NESTED)
    /// public void nestedOperation() { ... }
    /// ```
    #[Transactional(propagation = Nested)]
    async fn nested_operation(&self, user_id: i64) -> Result<(), String> {
        println!("🔄 Executing nested transaction for user {}", user_id);
        Ok(())
    }
}

// ============================================================================
// Example 3: Transaction Propagation Scenarios / 事务传播场景
// ============================================================================

struct OrderService {
    users: Arc<RwLock<Vec<User>>>,
    orders: Arc<RwLock<Vec<Order>>>,
}

#[derive(Debug, Clone)]
struct Order {
    id: i64,
    user_id: i64,
    amount: i64,
}

impl OrderService {
    fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(Vec::new())),
            orders: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// REQUIRED propagation (default) - joins existing transaction or creates new one
    /// REQUIRED 传播（默认）- 加入现有事务或创建新事务
    ///
    /// This is the most common propagation type.
    /// 这是最常见的传播类型。
    #[Transactional(propagation = Required)]
    async fn create_order(&self, user_id: i64, amount: i64) -> Result<(), String> {
        let mut orders = self.orders.write().await;
        orders.push(Order {
            id: orders.len() as i64 + 1,
            user_id,
            amount,
        });

        println!("✅ Order created for user {}", user_id);
        Ok(())
    }

    /// REQUIRES_NEW - always creates a new transaction, suspending existing one
    /// REQUIRES_NEW - 总是创建新事务，挂起现有事务
    ///
    /// Useful for logging or auditing that should commit independently.
    /// 适用于应该独立提交的日志或审计。
    #[Transactional(propagation = RequiresNew)]
    async fn log_audit(&self, action: String) -> Result<(), String> {
        println!("📝 AUDIT LOG: {}", action);
        Ok(())
    }

    /// NOT_SUPPORTED - executes non-transactionally, suspending existing transaction
    /// NOT_SUPPORTED - 非事务执行，挂起现有事务
    ///
    /// Useful for operations that should not be part of the transaction.
    /// 适用于不应成为事务一部分的操作。
    #[Transactional(propagation = NotSupported)]
    async fn send_notification(&self, user_id: i64, message: String) -> Result<(), String> {
        println!("📧 Notification sent to user {}: {}", user_id, message);
        Ok(())
    }

    /// NEVER - executes non-transactionally, errors if transaction exists
    /// NEVER - 非事务执行，如果存在事务则报错
    ///
    /// Useful for operations that must never run in a transaction.
    /// 适用于绝不能在事务中运行的操作。
    #[Transactional(propagation = Never)]
    async fn cache_invalidate(&self, user_id: i64) -> Result<(), String> {
        println!("🗑️ Cache invalidated for user {}", user_id);
        Ok(())
    }
}

// ============================================================================
// Example 4: Combining with Other Annotations / 与其他注解结合
// ============================================================================

use hiver_data_annotations::{Entity, Table, Id, Column};
use hiver_http::validation::{Validatable, ValidationHelpers};
use hiver_http::validation::ValidationErrors;

/// User entity with annotations
/// 带注解的用户实体
#[derive(Debug, Clone)]
#[Entity]
#[Table(name = "users")]
struct AnnotatedUser {
    #[Id]
    #[Column(name = "id")]
    id: i64,

    #[Column(name = "username", nullable = false)]
    username: String,

    #[Column(name = "email", nullable = false)]
    email: String,

    #[Column(name = "balance")]
    balance: i64,
}

/// Validation implementation
/// 验证实现
impl Validatable for AnnotatedUser {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if let Some(error) = ValidationHelpers::require_min_length("username", &self.username, 3) {
            errors.add(error);
        }

        if let Some(error) = ValidationHelpers::require_email_format("email", &self.email) {
            errors.add(error);
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

struct ComprehensiveUserService {
    users: Arc<RwLock<Vec<AnnotatedUser>>>,
}

impl ComprehensiveUserService {
    fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Combining validation and transaction management
    /// 结合验证和事务管理
    ///
    /// This method demonstrates how to use @Transactional with validation.
    /// 此方法演示如何将 @Transactional 与验证结合使用。
    #[Transactional(isolation = ReadCommitted)]
    async fn create_validated_user(&self, user: AnnotatedUser) -> Result<(), String> {
        // Validate first
        // 先验证
        user.validate().map_err(|e| format!("Validation failed: {}", e.error_count()))?;

        // Then save in transaction
        // 然后在事务中保存
        let mut users = self.users.write().await;
        users.push(user);

        println!("✅ Validated user created: {}", user.username);
        Ok(())
    }
}

// ============================================================================
// Main / 主函数
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║   @Transactional Annotation Examples                      ║");
    println!("║   @Transactional 注解示例                                  ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Example 1: Basic usage
    // 示例 1：基本用法
    println!("📊 Example 1: Basic Transactional Methods");
    println!("═══════════════════════════════════════════════════════════\n");

    let user_service = UserService::new();

    // Create users
    // 创建用户
    user_service
        .create_user(User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            balance: 1000,
        })
        .await?;

    user_service
        .create_user(User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            balance: 500,
        })
        .await?;

    println!("✅ Created 2 users\n");

    // Transfer funds with ReadCommitted isolation
    // 使用 ReadCommitted 隔离级别转账
    user_service
        .transfer_funds(1, 2, 200)
        .await?;

    println!();

    // Example 2: Advanced configuration
    // 示例 2：高级配置
    println!("📊 Example 2: Advanced Transactional Configuration");
    println!("═══════════════════════════════════════════════════════════\n");

    let banking_service = BankingService::new();

    banking_service
        .create_user(User {
            id: 3,
            username: "charlie".to_string(),
            email: "charlie@example.com".to_string(),
            balance: 0,
        })
        .await?;

    banking_service
        .critical_operation(3, 1000)
        .await?;

    println!();

    // Example 3: Propagation scenarios
    // 示例 3：传播场景
    println!("📊 Example 3: Transaction Propagation");
    println!("═══════════════════════════════════════════════════════════\n");

    let order_service = OrderService::new();

    order_service
        .create_user(User {
            id: 4,
            username: "david".to_string(),
            email: "david@example.com".to_string(),
            balance: 2000,
        })
        .await?;

    order_service.create_order(4, 500).await?;
    order_service.log_audit("Order created".to_string()).await?;
    order_service
        .send_notification(4, "Order confirmed".to_string())
        .await?;
    order_service.cache_invalidate(4).await?;

    println!();

    // Example 4: Combining annotations
    // 示例 4：结合注解
    println!("📊 Example 4: Combining with Validation");
    println!("═══════════════════════════════════════════════════════════\n");

    let comprehensive_service = ComprehensiveUserService::new();

    let valid_user = AnnotatedUser {
        id: 5,
        username: "eve".to_string(),
        email: "eve@example.com".to_string(),
        balance: 1500,
    };

    comprehensive_service
        .create_validated_user(valid_user)
        .await?;

    println!();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║   All @Transactional examples completed successfully!      ║");
    println!("║   所有 @Transactional 示例成功完成！                          ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_transactional() {
        let service = UserService::new();
        let result = service
            .create_user(User {
                id: 1,
                username: "test".to_string(),
                email: "test@example.com".to_string(),
                balance: 100,
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transfer_funds() {
        let service = UserService::new();

        service
            .create_user(User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                balance: 1000,
            })
            .await
            .unwrap();

        service
            .create_user(User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                balance: 500,
            })
            .await
            .unwrap();

        let result = service.transfer_funds(1, 2, 200).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_insufficient_funds() {
        let service = UserService::new();

        service
            .create_user(User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                balance: 100,
            })
            .await
            .unwrap();

        let result = service.transfer_funds(1, 2, 200).await;
        assert!(result.is_err());
    }
}
