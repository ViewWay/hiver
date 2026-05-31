//! # @PreAuthorize Annotation Examples
//! # @PreAuthorize 注解示例
//!
//! This example demonstrates method-level security using the @PreAuthorize annotation
//! 本示例演示使用 @PreAuthorize 注解进行方法级安全控制
//!
//! ## Run Example / 运行示例
//!
//! ```bash
//! cargo run --example pre_authorize_example
//! ```

use hiver_data_annotations::{PreAuthorize, CrudRepository, PagingRepository, Page, PageRequest};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ========================================================================
// Domain Models / 领域模型
// ========================================================================

/// User entity
/// 用户实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

/// Authentication context
/// 认证上下文
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i64,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

impl AuthContext {
    /// Create a new auth context
    /// 创建新的认证上下文
    pub fn new(user_id: i64, username: String, roles: Vec<String>, permissions: Vec<String>) -> Self {
        Self {
            user_id,
            username,
            roles,
            permissions,
        }
    }

    /// Check if user has role
    /// 检查用户是否拥有角色
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Check if user has permission
    /// 检查用户是否拥有权限
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Check if user is admin
    /// 检查用户是否为管理员
    pub fn is_admin(&self) -> bool {
        self.has_role("ADMIN")
    }

    /// Get current user ID
    /// 获取当前用户 ID
    pub fn user_id(&self) -> i64 {
        self.user_id
    }
}

/// Mock repository for demonstration
/// 演示用的模拟 repository
pub struct MockUserRepository {
    users: Vec<User>,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: vec![
                User {
                    id: 1,
                    username: "admin".to_string(),
                    email: "admin@example.com".to_string(),
                    roles: vec!["ADMIN".to_string()],
                },
                User {
                    id: 2,
                    username: "alice".to_string(),
                    email: "alice@example.com".to_string(),
                    roles: vec!["USER".to_string()],
                },
                User {
                    id: 3,
                    username: "bob".to_string(),
                    email: "bob@example.com".to_string(),
                    roles: vec!["USER".to_string()],
                },
            ],
        }
    }

    pub fn find_by_id(&self, id: i64) -> Option<User> {
        self.users.iter().find(|u| u.id == id).cloned()
    }

    pub fn find_all(&self) -> Vec<User> {
        self.users.clone()
    }

    pub fn delete(&mut self, id: i64) -> Result<(), String> {
        if let Some(pos) = self.users.iter().position(|u| u.id == id) {
            self.users.remove(pos);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn update_email(&mut self, id: i64, email: String) -> Result<(), String> {
        if let Some(user) = self.users.iter_mut().find(|u| u.id == id) {
            user.email = email;
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
}

// ========================================================================
// Service Layer with Security Annotations / 带安全注解的服务层
// ========================================================================

/// User service with method-level security
/// 带方法级安全的用户服务
pub struct UserService {
    repository: MockUserRepository,
}

impl UserService {
    pub fn new(repository: MockUserRepository) -> Self {
        Self { repository }
    }

    /// Only admins can delete users
    /// 只有管理员可以删除用户
    ///
    /// # Security / 安全
    ///
    /// Expression: `has_role('ADMIN')`
    /// Checks: Current user must have ADMIN role
    /// 检查：当前用户必须拥有 ADMIN 角色
    #[PreAuthorize("has_role('ADMIN')")]
    pub async fn delete_user(&self, auth: &AuthContext, id: i64) -> Result<String, String> {
        self.repository.delete(id)
            .map(|_| format!("User {} deleted successfully", id))
            .map_err(|e| format!("Delete failed: {}", e))
    }

    /// Admins or the user themselves can update profiles
    /// 管理员或用户本人可以更新资料
    ///
    /// # Security / 安全
    ///
    /// Expression: `has_role('ADMIN') or #id == auth.user_id()`
    /// Checks: Current user is admin or updating their own profile
    /// 检查：当前用户是管理员或更新自己的资料
    #[PreAuthorize("has_role('ADMIN') or #id == auth.user_id()")]
    pub async fn update_profile(
        &self,
        auth: &AuthContext,
        id: i64,
        email: String
    ) -> Result<String, String> {
        self.repository.update_email(id, email)
            .map(|_| format!("User {} profile updated", id))
            .map_err(|e| format!("Update failed: {}", e))
    }

    /// Only users with user:write permission can create users
    /// 只有拥有 user:write 权限的用户可以创建用户
    ///
    /// # Security / 安全
    ///
    /// Expression: `has_permission('user:write')`
    /// Checks: Current user must have user:write permission
    /// 检查：当前用户必须拥有 user:write 权限
    #[PreAuthorize("has_permission('user:write')")]
    pub async fn create_user(
        &self,
        auth: &AuthContext,
        username: String,
        email: String
    ) -> Result<String, String> {
        Ok(format!("User {} created successfully", username))
    }

    /// Only admins can view all users
    /// 只有管理员可以查看所有用户
    ///
    /// # Security / 安全
    ///
    /// Expression: `is_admin()`
    /// Checks: Current user must be admin
    /// 检查：当前用户必须是管理员
    #[PreAuthorize("is_admin()")]
    pub async fn get_all_users(&self, auth: &AuthContext) -> Result<Vec<User>, String> {
        Ok(self.repository.find_all())
    }

    /// Complex expression: Admins or users with read permission
    /// 复杂表达式：管理员或拥有读取权限的用户
    ///
    /// # Security / 安全
    ///
    /// Expression: `has_role('ADMIN') or has_permission('user:read')`
    /// Checks: Current user is admin OR has user:read permission
    /// 检查：当前用户是管理员或拥有 user:read 权限
    #[PreAuthorize("has_role('ADMIN') or has_permission('user:read')")]
    pub async fn get_user(&self, auth: &AuthContext, id: i64) -> Result<User, String> {
        self.repository.find_by_id(id)
            .ok_or_else(|| format!("User {} not found", id))
    }

    /// Admins or the user themselves can view profiles
    /// 管理员或用户本人可以查看资料
    ///
    /// # Security / 安全
    ///
    /// Expression: `has_role('ADMIN') or #user_id == auth.user_id()`
    /// Checks: Current user is admin OR viewing their own profile
    /// 检查：当前用户是管理员或查看自己的资料
    #[PreAuthorize("has_role('ADMIN') or #user_id == auth.user_id()")]
    pub async fn view_profile(
        &self,
        auth: &AuthContext,
        user_id: i64
    ) -> Result<User, String> {
        self.repository.find_by_id(user_id)
            .ok_or_else(|| format!("User {} not found", user_id))
    }
}

// ========================================================================
// Manual Expression Evaluator (for demonstration) / 手动表达式求值器（演示用）
// ========================================================================

/// Evaluate security expression
/// 评估安全表达式
pub fn evaluate_expression(
    expression: &str,
    auth: &AuthContext,
    args: &HashMap<String, String>
) -> bool {
    // Parse and evaluate the expression
    // 解析并评估表达式

    // Handle has_role('ROLE_NAME')
    if let Some(rest) = expression.strip_prefix("has_role('") {
        if let Some(role) = rest.strip_suffix("')") {
            return auth.has_role(role);
        }
    }

    // Handle has_permission('PERMISSION_NAME')
    if let Some(rest) = expression.strip_prefix("has_permission('") {
        if let Some(perm) = rest.strip_suffix("')") {
            return auth.has_permission(perm);
        }
    }

    // Handle is_admin()
    if expression == "is_admin()" {
        return auth.is_admin();
    }

    // Handle parameter checks like #id == auth.user_id()
    if expression.contains("== auth.user_id()") {
        if let Some(param_part) = expression.strip_prefix("#") {
            if let Some(param_name) = param_part.split(" == ").next() {
                if let Some(param_value) = args.get(param_name) {
                    if let Ok(value) = param_value.parse::<i64>() {
                        return value == auth.user_id();
                    }
                }
            }
        }
    }

    // Handle OR expressions
    if expression.contains(" or ") {
        let parts: Vec<&str> = expression.split(" or ").collect();
        return parts.iter().any(|part| evaluate_expression(part, auth, args));
    }

    // Handle AND expressions
    if expression.contains(" and ") {
        let parts: Vec<&str> = expression.split(" and ").collect();
        return parts.iter().all(|part| evaluate_expression(part, auth, args));
    }

    false
}

// ========================================================================
// Examples / 示例
// ========================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     @PreAuthorize Annotation Examples / @PreAuthorize 注解示例      ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let service = UserService::new(MockUserRepository::new());

    // Example 1: Admin deleting a user
    // 示例 1：管理员删除用户
    println!("📋 Example 1: Admin deletes user / 示例 1：管理员删除用户");
    println!("─────────────────────────────────────────────────────────────");

    let admin_auth = AuthContext::new(
        1,
        "admin".to_string(),
        vec!["ADMIN".to_string()],
        vec!["user:write".to_string(), "user:read".to_string()],
    );

    println!("Auth: {:?} (ADMIN)", admin_auth.username);
    println!("Action: Delete user ID 2");
    println!("Expression: has_role('ADMIN')");

    let mut args = HashMap::new();
    args.insert("id".to_string(), "2".to_string());

    let can_execute = evaluate_expression("has_role('ADMIN')", &admin_auth, &args);
    println!("✅ Authorization: {}", if can_execute { "GRANTED" } else { "DENIED" });

    if can_execute {
        match service.delete_user(&admin_auth, 2).await {
            Ok(msg) => println!("✅ Result: {}", msg),
            Err(e) => println!("❌ Error: {}", e),
        }
    }
    println!();

    // Example 2: Regular user tries to delete (should fail)
    // 示例 2：普通用户尝试删除（应该失败）
    println!("📋 Example 2: Regular user attempts delete / 示例 2：普通用户尝试删除");
    println!("─────────────────────────────────────────────────────────────");

    let user_auth = AuthContext::new(
        2,
        "alice".to_string(),
        vec!["USER".to_string()],
        vec!["user:read".to_string()],
    );

    println!("Auth: {:?} (USER)", user_auth.username);
    println!("Action: Delete user ID 3");
    println!("Expression: has_role('ADMIN')");

    let can_execute = evaluate_expression("has_role('ADMIN')", &user_auth, &args);
    println!("❌ Authorization: {}", if can_execute { "GRANTED" } else { "DENIED" });
    println!();

    // Example 3: User updates their own profile
    // 示例 3：用户更新自己的资料
    println!("📋 Example 3: User updates own profile / 示例 3：用户更新自己的资料");
    println!("─────────────────────────────────────────────────────────────");

    println!("Auth: alice (USER, id=2)");
    println!("Action: Update profile for user ID 2");
    println!("Expression: has_role('ADMIN') or #id == auth.user_id()");

    let mut args = HashMap::new();
    args.insert("id".to_string(), "2".to_string());

    let can_execute = evaluate_expression(
        "has_role('ADMIN') or #id == auth.user_id()",
        &user_auth,
        &args
    );
    println!("✅ Authorization: {}", if can_execute { "GRANTED" } else { "DENIED" });

    if can_execute {
        match service.update_profile(&user_auth, 2, "alice.new@example.com".to_string()).await {
            Ok(msg) => println!("✅ Result: {}", msg),
            Err(e) => println!("❌ Error: {}", e),
        }
    }
    println!();

    // Example 4: User tries to update another user's profile (should fail)
    // 示例 4：用户尝试更新其他用户的资料（应该失败）
    println!("📋 Example 4: User attempts to update another user / 示例 4：用户尝试更新其他用户");
    println!("─────────────────────────────────────────────────────────────");

    println!("Auth: alice (USER, id=2)");
    println!("Action: Update profile for user ID 3");
    println!("Expression: has_role('ADMIN') or #id == auth.user_id()");

    let mut args = HashMap::new();
    args.insert("id".to_string(), "3".to_string());

    let can_execute = evaluate_expression(
        "has_role('ADMIN') or #id == auth.user_id()",
        &user_auth,
        &args
    );
    println!("❌ Authorization: {}", if can_execute { "GRANTED" } else { "DENIED" });
    println!();

    // Example 5: Admin can update any user
    // 示例 5：管理员可以更新任何用户
    println!("📋 Example 5: Admin updates any user / 示例 5：管理员更新任何用户");
    println!("─────────────────────────────────────────────────────────────");

    println!("Auth: admin (ADMIN)");
    println!("Action: Update profile for user ID 3");
    println!("Expression: has_role('ADMIN') or #id == auth.user_id()");

    let can_execute = evaluate_expression(
        "has_role('ADMIN') or #id == auth.user_id()",
        &admin_auth,
        &args
    );
    println!("✅ Authorization: {}", if can_execute { "GRANTED" } else { "DENIED" });

    if can_execute {
        match service.update_profile(&admin_auth, 3, "bob.updated@example.com".to_string()).await {
            Ok(msg) => println!("✅ Result: {}", msg),
            Err(e) => println!("❌ Error: {}", e),
        }
    }
    println!();

    // Example 6: Permission-based access
    // 示例 6：基于权限的访问
    println!("📋 Example 6: Permission-based access / 示例 6：基于权限的访问");
    println!("─────────────────────────────────────────────────────────────");

    println!("Auth: alice (USER, user:read permission)");
    println!("Action: View user ID 2");
    println!("Expression: has_role('ADMIN') or has_permission('user:read')");

    let can_execute = evaluate_expression(
        "has_role('ADMIN') or has_permission('user:read')",
        &user_auth,
        &args
    );
    println!("✅ Authorization: {}", if can_execute { "GRANTED" } else { "DENIED" });

    if can_execute {
        match service.get_user(&user_auth, 2).await {
            Ok(user) => println!("✅ Result: Found user {}", user.username),
            Err(e) => println!("❌ Error: {}", e),
        }
    }
    println!();

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              Examples completed! / 示例完成！                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    Ok(())
}
