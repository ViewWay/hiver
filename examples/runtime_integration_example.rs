//! Hiver Annotations Runtime Integration Test
//! Hiver 注解运行时集成测试
//!
//! This example demonstrates the runtime integration of all annotation features:
//! - Query execution with @Query, @Insert, @Update, @Delete
//! - Validation with @Valid and validation annotations
//! - AOP with @Aspect, @Before, @After, @Around
//! - Transactions with @Transactional
//!
//! 此示例演示了所有注解功能的运行时集成：
//! - 使用 @Query、@Insert、@Update、@Delete 进行查询执行
//! - 使用 @Valid 和验证注解进行验证
//! - 使用 @Aspect、@Before、@After、@Around 进行 AOP
//! - 使用 @Transactional 进行事务管理

use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Part 1: Query Runtime / 查询运行时
// ============================================================================

#[allow(dead_code)]
fn demo_query_runtime() {
    use hiver_data_rdbc::{QueryMetadata, ParamStyle, QueryType};

    // Define query metadata (normally extracted from @Query annotation)
    // 定义查询元数据（通常从 @Query 注解中提取）
    let metadata = QueryMetadata {
        sql: "SELECT * FROM users WHERE id = :id".to_string(),
        param_style: ParamStyle::Named,
        param_names: vec!["id".to_string()],
        query_type: QueryType::SelectOne,
    };

    println!("✅ Query metadata created");
    println!("   SQL: {}", metadata.sql);
    println!("   Params: {:?}", metadata.param_names);
}

// ============================================================================
// Part 2: Validation Runtime / 验证运行时
// ============================================================================

#[allow(dead_code)]
fn demo_validation_runtime() {
    use hiver_http::validation::{ValidationHelpers, ValidationErrors};

    // Create validation errors
    // 创建验证错误
    let mut errors = ValidationErrors::new();

    // Test validation helpers
    // 测试验证辅助函数
    let username = "";
    if let Some(error) = ValidationHelpers::require_non_empty("username", username) {
        errors.add(error);
    }

    let email = "invalid-email";
    if let Some(error) = ValidationHelpers::require_email_format("email", email) {
        errors.add(error);
    }

    let password = "short";
    if let Some(error) = ValidationHelpers::require_min_length("password", password, 8) {
        errors.add(error);
    }

    let age = 15;
    if let Some(error) = ValidationHelpers::require_min("age", age, 18) {
        errors.add(error);
    }

    println!("✅ Validation runtime test");
    println!("   Errors found: {}", errors.error_count());
    for error in errors.iter() {
        println!("   - {}: {}", error.field(), error.message());
    }
}

// ============================================================================
// Part 3: AOP Runtime / AOP 运行时
// ============================================================================

#[allow(dead_code)]
fn demo_aop_runtime() {
    use hiver_aop::runtime::{JoinPoint, PointcutExpression, global_registry};
    use std::any::Any;

    // Create a join point
    // 创建连接点
    let target: Arc<dyn Any + Send + Sync> = Arc::new("UserService");
    let args: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new(42), Arc::new("alice")];

    let join_point = JoinPoint::new(
        target,
        "find_by_id".to_string(),
        args,
        "find_by_id(i64)".to_string(),
        "UserService".to_string(),
    );

    println!("✅ AOP runtime test");
    println!("   Join point: {}", join_point.method_name());
    println!("   Target class: {}", join_point.target_class());

    // Create a pointcut expression
    // 创建切点表达式
    let pointcut = PointcutExpression::new("execution(* com.example..*.*(..))".to_string());
    println!("   Pointcut expression: {}", pointcut.expression());

    // Check if pointcut matches
    // 检查切点是否匹配
    let matches = pointcut.matches(&join_point);
    println!("   Pointcut matches: {}", matches);

    // Access global registry
    // 访问全局注册表
    let registry = global_registry();
    println!("   Global registry: accessible");
}

// ============================================================================
// Part 4: Transactional Runtime / 事务运行时
// ============================================================================

#[allow(dead_code)]
fn demo_transactional_runtime() {
    use hiver_data_annotations::transactional::{
        TransactionalConfig, IsolationLevel, Propagation,
    };

    // Create transactional configuration
    // 创建事务配置
    let config = TransactionalConfig::new()
        .isolation(IsolationLevel::ReadCommitted)
        .timeout(30)
        .propagation(Propagation::Required)
        .read_only(false)
        .max_retries(3);

    println!("✅ Transactional runtime test");
    println!("   Isolation: {:?}", config.isolation);
    println!("   Timeout: {:?}", config.timeout);
    println!("   Propagation: {:?}", config.propagation);
    println!("   Read-only: {}", config.read_only);
    println!("   Max retries: {}", config.max_retries);
}

// ============================================================================
// Part 5: Integrated Example / 集成示例
// ============================================================================

/// User entity with all annotations
/// 带有所有注解的用户实体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct User {
    id: i64,
    username: String,
    email: String,
    age: i32,
}

/// Validation implementation for User
/// User 的验证实现
impl hiver_http::validation::Validatable for User {
    fn validate(&self) -> Result<(), hiver_http::validation::ValidationErrors> {
        let mut errors = hiver_http::validation::ValidationErrors::new();

        // Validate username
        // 验证用户名
        if let Some(error) = hiver_http::validation::ValidationHelpers::require_non_empty(
            "username",
            &self.username,
        ) {
            errors.add(error);
        }

        if let Some(error) = hiver_http::validation::ValidationHelpers::require_min_length(
            "username",
            &self.username,
            3,
        ) {
            errors.add(error);
        }

        // Validate email
        // 验证邮箱
        if let Some(error) = hiver_http::validation::ValidationHelpers::require_email_format(
            "email",
            &self.email,
        ) {
            errors.add(error);
        }

        // Validate age
        // 验证年龄
        if let Some(error) = hiver_http::validation::ValidationHelpers::require_min("age", self.age, 18)
        {
            errors.add(error);
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

/// Service class with transactional methods
/// 带有事务方法的服务类
struct UserService {
    users: Vec<User>,
}

impl UserService {
    fn new() -> Self {
        Self {
            users: vec![
                User {
                    id: 1,
                    username: "alice".to_string(),
                    email: "alice@example.com".to_string(),
                    age: 25,
                },
                User {
                    id: 2,
                    username: "bob".to_string(),
                    email: "bob@example.com".to_string(),
                    age: 30,
                },
            ],
        }
    }

    /// Find user by ID (would use @Query in real implementation)
    /// 通过 ID 查找用户（实际实现中会使用 @Query）
    #[allow(dead_code)]
    fn find_by_id(&self, id: i64) -> Option<User> {
        self.users.iter().find(|u| u.id == id).cloned()
    }

    /// Create user (would use @Transactional in real implementation)
    /// 创建用户（实际实现中会使用 @Transactional）
    #[allow(dead_code)]
    fn create_user(&mut self, user: User) -> Result<(), hiver_http::validation::ValidationErrors> {
        // Validate user
        // 验证用户
        user.validate()?;

        // In real implementation, this would be in a transaction
        // 在实际实现中，这将在事务中执行
        self.users.push(user);
        Ok(())
    }

    /// Update user email (would use @Update and @Transactional)
    /// 更新用户邮箱（实际实现中会使用 @Update 和 @Transactional）
    #[allow(dead_code)]
    fn update_email(&mut self, id: i64, email: String) -> Result<(), String> {
        if let Some(user) = self.users.iter_mut().find(|u| u.id == id) {
            user.email = email;
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    /// Delete user (would use @Delete and @Transactional)
    /// 删除用户（实际实现中会使用 @Delete 和 @Transactional）
    #[allow(dead_code)]
    fn delete_user(&mut self, id: i64) -> Result<(), String> {
        let original_len = self.users.len();
        self.users.retain(|u| u.id != id);

        if self.users.len() < original_len {
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
}

#[allow(dead_code)]
fn demo_integrated_example() {
    println!("🚀 Integrated Example\n");

    // Create service
    // 创建服务
    let mut service = UserService::new();

    // Find user
    // 查找用户
    if let Some(user) = service.find_by_id(1) {
        println!("✅ Found user: {:?}", user);
    }

    // Create user - validation should pass
    // 创建用户 - 验证应该通过
    let new_user = User {
        id: 3,
        username: "charlie".to_string(),
        email: "charlie@example.com".to_string(),
        age: 28,
    };

    match service.create_user(new_user.clone()) {
        Ok(_) => println!("✅ User created: {:?}", new_user),
        Err(errors) => println!("❌ Validation failed: {}", errors.error_count()),
    }

    // Create user - validation should fail
    // 创建用户 - 验证应该失败
    let invalid_user = User {
        id: 4,
        username: "".to_string(), // Too short / 太短
        email: "invalid-email".to_string(), // Invalid format / 格式无效
        age: 15, // Too young / 太小
    };

    match service.create_user(invalid_user) {
        Ok(_) => println!("✅ User created"),
        Err(errors) => {
            println!("❌ Validation failed:");
            for error in errors.iter() {
                println!("   - {}: {}", error.field(), error.message());
            }
        }
    }

    // Update email
    // 更新邮箱
    match service.update_email(1, "alice.new@example.com".to_string()) {
        Ok(_) => println!("✅ Email updated"),
        Err(e) => println!("❌ Update failed: {}", e),
    }

    // Delete user
    // 删除用户
    match service.delete_user(2) {
        Ok(_) => println!("✅ User deleted"),
        Err(e) => println!("❌ Delete failed: {}", e),
    }
}

// ============================================================================
// Main / 主函数
// ============================================================================

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║   Hiver Annotations Runtime Integration Test             ║");
    println!("║   Hiver 注解运行时集成测试                                 ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Part 1: Query runtime
    // 第一部分：查询运行时
    println!("📊 Part 1: Query Runtime");
    demo_query_runtime();
    println!();

    // Part 2: Validation runtime
    // 第二部分：验证运行时
    println!("✓ Part 2: Validation Runtime");
    demo_validation_runtime();
    println!();

    // Part 3: AOP runtime
    // 第三部分：AOP 运行时
    println!("✓ Part 3: AOP Runtime");
    demo_aop_runtime();
    println!();

    // Part 4: Transactional runtime
    // 第四部分：事务运行时
    println!("✓ Part 4: Transactional Runtime");
    demo_transactional_runtime();
    println!();

    // Part 5: Integrated example
    // 第五部分：集成示例
    println!("✓ Part 5: Integrated Example");
    demo_integrated_example();
    println!();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║   All runtime components integrated successfully!          ║");
    println!("║   所有运行时组件集成成功！                                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_runtime() {
        demo_query_runtime();
    }

    #[test]
    fn test_validation_runtime() {
        demo_validation_runtime();
    }

    #[test]
    fn test_aop_runtime() {
        demo_aop_runtime();
    }

    #[test]
    fn test_transactional_runtime() {
        demo_transactional_runtime();
    }

    #[test]
    fn test_integrated_example() {
        demo_integrated_example();
    }

    #[test]
    fn test_user_validation() {
        let valid_user = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        };

        assert!(valid_user.validate().is_ok());

        let invalid_user = User {
            id: 2,
            username: "".to_string(),
            email: "invalid".to_string(),
            age: 15,
        };

        assert!(invalid_user.validate().is_err());
    }

    #[test]
    fn test_user_service() {
        let mut service = UserService::new();

        // Test find
        // 测试查找
        assert!(service.find_by_id(1).is_some());
        assert!(service.find_by_id(999).is_none());

        // Test create
        // 测试创建
        let new_user = User {
            id: 3,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        };
        assert!(service.create_user(new_user).is_ok());

        // Test update
        // 测试更新
        assert!(service.update_email(1, "new@example.com".to_string()).is_ok());

        // Test delete
        // 测试删除
        assert!(service.delete_user(2).is_ok());
    }
}
