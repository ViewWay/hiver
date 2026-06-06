//! Tests for @PreAuthorize annotation and security expression evaluation
//! @PreAuthorize 注解和安全表达式评估的测试

use std::collections::HashMap;

/// Mock AuthContext for testing
/// 用于测试的模拟 AuthContext
#[derive(Debug, Clone)]
struct TestAuthContext
{
    pub user_id: i64,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

impl TestAuthContext
{
    fn new(user_id: i64, username: &str, roles: Vec<&str>, permissions: Vec<&str>) -> Self
    {
        Self {
            user_id,
            username: username.to_string(),
            roles: roles.into_iter().map(String::from).collect(),
            permissions: permissions.into_iter().map(String::from).collect(),
        }
    }

    fn has_role(&self, role: &str) -> bool
    {
        self.roles.contains(&role.to_string())
    }

    fn has_permission(&self, permission: &str) -> bool
    {
        self.permissions.contains(&permission.to_string())
    }

    fn is_admin(&self) -> bool
    {
        self.has_role("ADMIN")
    }

    fn user_id(&self) -> i64
    {
        self.user_id
    }
}

/// Simple expression evaluator for testing
/// 用于测试的简单表达式求值器
fn evaluate_test_expression(
    expression: &str,
    auth: &TestAuthContext,
    args: &HashMap<String, String>,
) -> bool
{
    // Handle OR expressions (check first to avoid partial matches by individual patterns)
    if expression.contains(" or ")
    {
        let parts: Vec<&str> = expression.split(" or ").collect();
        return parts
            .iter()
            .any(|part| evaluate_test_expression(part.trim(), auth, args));
    }

    // Handle AND expressions (check before individual patterns)
    if expression.contains(" and ")
    {
        let parts: Vec<&str> = expression.split(" and ").collect();
        return parts
            .iter()
            .all(|part| evaluate_test_expression(part.trim(), auth, args));
    }

    // Handle has_role('ROLE_NAME')
    if let Some(rest) = expression.strip_prefix("has_role('")
    {
        if let Some(role) = rest.strip_suffix("')")
        {
            return auth.has_role(role);
        }
    }

    // Handle has_permission('PERMISSION_NAME')
    if let Some(rest) = expression.strip_prefix("has_permission('")
    {
        if let Some(perm) = rest.strip_suffix("')")
        {
            return auth.has_permission(perm);
        }
    }

    // Handle is_admin()
    if expression == "is_admin()"
    {
        return auth.is_admin();
    }

    // Handle parameter checks like #id == auth.user_id()
    if expression.contains("== auth.user_id()")
    {
        if let Some(param_part) = expression.strip_prefix("#")
        {
            if let Some(param_name) = param_part.split(" == ").next()
            {
                if let Some(param_value) = args.get(param_name)
                {
                    if let Ok(value) = param_value.parse::<i64>()
                    {
                        return value == auth.user_id();
                    }
                }
            }
        }
    }

    false
}

// ========================================================================
// Tests / 测试
// ========================================================================

#[test]
fn test_has_role_admin()
{
    let admin = TestAuthContext::new(1, "admin", vec!["ADMIN"], vec![]);
    let args = HashMap::new();

    assert!(
        evaluate_test_expression("has_role('ADMIN')", &admin, &args),
        "Admin should have ADMIN role"
    );
}

#[test]
fn test_has_role_user_not_admin()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec![]);
    let args = HashMap::new();

    assert!(
        !evaluate_test_expression("has_role('ADMIN')", &user, &args),
        "Regular user should not have ADMIN role"
    );
}

#[test]
fn test_has_role_user()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec![]);
    let args = HashMap::new();

    assert!(
        evaluate_test_expression("has_role('USER')", &user, &args),
        "User should have USER role"
    );
}

#[test]
fn test_has_permission()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec!["user:read", "user:write"]);
    let args = HashMap::new();

    assert!(
        evaluate_test_expression("has_permission('user:read')", &user, &args),
        "User should have user:read permission"
    );

    assert!(
        evaluate_test_expression("has_permission('user:write')", &user, &args),
        "User should have user:write permission"
    );

    assert!(
        !evaluate_test_expression("has_permission('admin:delete')", &user, &args),
        "User should not have admin:delete permission"
    );
}

#[test]
fn test_is_admin()
{
    let admin = TestAuthContext::new(1, "admin", vec!["ADMIN"], vec![]);
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec![]);
    let args = HashMap::new();

    assert!(
        evaluate_test_expression("is_admin()", &admin, &args),
        "Admin should pass is_admin() check"
    );

    assert!(
        !evaluate_test_expression("is_admin()", &user, &args),
        "Regular user should not pass is_admin() check"
    );
}

#[test]
fn test_parameter_match()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec![]);
    let mut args = HashMap::new();

    // User accessing their own resource
    args.insert("id".to_string(), "2".to_string());
    assert!(
        evaluate_test_expression("#id == auth.user_id()", &user, &args),
        "User should access their own resource"
    );

    // User accessing another user's resource
    args.insert("id".to_string(), "3".to_string());
    assert!(
        !evaluate_test_expression("#id == auth.user_id()", &user, &args),
        "User should not access another user's resource"
    );
}

#[test]
fn test_or_expression()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec!["user:read"]);
    let args = HashMap::new();

    // Should pass - has permission
    assert!(
        evaluate_test_expression("has_role('ADMIN') or has_permission('user:read')", &user, &args),
        "Should pass: user has user:read permission"
    );

    // Should fail - neither condition met
    assert!(
        !evaluate_test_expression(
            "has_role('ADMIN') or has_permission('admin:delete')",
            &user,
            &args
        ),
        "Should fail: user is neither admin nor has admin:delete permission"
    );
}

#[test]
fn test_and_expression()
{
    let admin = TestAuthContext::new(1, "admin", vec!["ADMIN"], vec!["user:write"]);
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec!["user:read"]);
    let args = HashMap::new();

    // Should pass - admin has both role and permission
    assert!(
        evaluate_test_expression(
            "has_role('ADMIN') and has_permission('user:write')",
            &admin,
            &args
        ),
        "Should pass: admin has both ADMIN role and user:write permission"
    );

    // Should fail - user has neither
    assert!(
        !evaluate_test_expression(
            "has_role('ADMIN') and has_permission('user:write')",
            &user,
            &args
        ),
        "Should fail: user lacks either ADMIN role or user:write permission"
    );
}

#[test]
fn test_complex_expression()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec!["user:read"]);
    let mut args = HashMap::new();

    // User accessing their own resource - should pass
    args.insert("user_id".to_string(), "2".to_string());
    assert!(
        evaluate_test_expression("has_role('ADMIN') or #user_id == auth.user_id()", &user, &args),
        "Should pass: user accessing own resource"
    );

    // User accessing another user's resource - should fail
    args.insert("user_id".to_string(), "3".to_string());
    assert!(
        !evaluate_test_expression("has_role('ADMIN') or #user_id == auth.user_id()", &user, &args),
        "Should fail: user accessing other user's resource"
    );

    // Admin accessing any resource - should pass
    let admin = TestAuthContext::new(1, "admin", vec!["ADMIN"], vec![]);
    args.insert("user_id".to_string(), "3".to_string());
    assert!(
        evaluate_test_expression("has_role('ADMIN') or #user_id == auth.user_id()", &admin, &args),
        "Should pass: admin can access any resource"
    );
}

// ========================================================================
// Integration-style tests / 集成风格测试
// ========================================================================

#[test]
fn test_admin_can_delete_user()
{
    let admin = TestAuthContext::new(1, "admin", vec!["ADMIN"], vec![]);
    let args = HashMap::new();

    let expression = "has_role('ADMIN')";
    let result = evaluate_test_expression(expression, &admin, &args);

    assert!(result, "Admin should be able to delete users");
}

#[test]
fn test_user_cannot_delete_user()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec![]);
    let args = HashMap::new();

    let expression = "has_role('ADMIN')";
    let result = evaluate_test_expression(expression, &user, &args);

    assert!(!result, "Regular user should not be able to delete users");
}

#[test]
fn test_user_can_update_own_profile()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec![]);
    let mut args = HashMap::new();
    args.insert("id".to_string(), "2".to_string());

    let expression = "has_role('ADMIN') or #id == auth.user_id()";
    let result = evaluate_test_expression(expression, &user, &args);

    assert!(result, "User should be able to update their own profile");
}

#[test]
fn test_user_cannot_update_other_profile()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec![]);
    let mut args = HashMap::new();
    args.insert("id".to_string(), "3".to_string());

    let expression = "has_role('ADMIN') or #id == auth.user_id()";
    let result = evaluate_test_expression(expression, &user, &args);

    assert!(!result, "User should not be able to update other user's profile");
}

#[test]
fn test_admin_can_update_any_profile()
{
    let admin = TestAuthContext::new(1, "admin", vec!["ADMIN"], vec![]);
    let mut args = HashMap::new();
    args.insert("id".to_string(), "3".to_string());

    let expression = "has_role('ADMIN') or #id == auth.user_id()";
    let result = evaluate_test_expression(expression, &admin, &args);

    assert!(result, "Admin should be able to update any profile");
}

#[test]
fn test_user_with_read_permission_can_view()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec!["user:read"]);
    let args = HashMap::new();

    let expression = "has_role('ADMIN') or has_permission('user:read')";
    let result = evaluate_test_expression(expression, &user, &args);

    assert!(result, "User with read permission should be able to view");
}

#[test]
fn test_user_without_permission_cannot_view()
{
    let user = TestAuthContext::new(2, "alice", vec!["USER"], vec![]);
    let args = HashMap::new();

    let expression = "has_role('ADMIN') or has_permission('user:read')";
    let result = evaluate_test_expression(expression, &user, &args);

    assert!(!result, "User without read permission should not be able to view");
}
