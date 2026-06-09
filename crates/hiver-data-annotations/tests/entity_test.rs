//! Tests for Hiver Data Annotations
//! Hiver Data 注解测试

use hiver_data_annotations::{Entity, Table};

// ========================================================================
// Entity Annotation Tests / @Entity 注解测试
// ========================================================================

#[test]
fn test_entity_macro() {
    #[Entity]
    struct TestUser {
        id: i64,
        username: String,
    }

    // Default table name is snake_case of struct name
    // 默认表名是结构体名的 snake_case 形式
    assert_eq!(TestUser::table_name(), "test_user");
    assert_eq!(TestUser::field_names(), &["id", "username"]);
    assert_eq!(TestUser::field_count(), 2);
}

#[test]
fn test_table_macro_with_custom_name() {
    #[Table(name = "custom_users")]
    struct TestUser {
        id: i64,
        username: String,
    }

    assert_eq!(TestUser::table_name(), "custom_users");
}

#[test]
fn test_table_default_name() {
    #[Table]
    struct TestUser {
        id: i64,
        username: String,
    }

    // Default table name is snake_case
    // 默认表名为 snake_case
    assert_eq!(TestUser::table_name(), "test_user");
}

#[test]
fn test_entity_with_multiple_fields() {
    #[Entity]
    struct TestUser {
        id: i64,
        username: String,
        email: String,
    }

    let _user = TestUser {
        id: 1,
        username: "test".to_string(),
        email: "test@example.com".to_string(),
    };

    assert_eq!(TestUser::table_name(), "test_user");
    assert_eq!(TestUser::field_names(), &["id", "username", "email"]);
    assert_eq!(TestUser::field_count(), 3);
}

#[test]
fn test_table_with_multiple_fields() {
    #[Table(name = "users")]
    struct User {
        id: i64,
        username: String,
        email: String,
    }

    assert_eq!(User::table_name(), "users");

    let _user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
}

#[test]
fn test_entity_no_id_field() {
    // Entity without #[Id] returns None for id_field_name
    // 没有 #[Id] 的实体 id_field_name 返回 None
    #[Entity]
    struct AuditLog {
        action: String,
        timestamp: i64,
    }

    assert_eq!(AuditLog::table_name(), "audit_log");
    assert_eq!(AuditLog::field_names(), &["action", "timestamp"]);
    assert_eq!(AuditLog::id_field_name(), None);
    assert_eq!(AuditLog::id_generation_strategy(), None);
    assert_eq!(AuditLog::relations(), &[]);
}
