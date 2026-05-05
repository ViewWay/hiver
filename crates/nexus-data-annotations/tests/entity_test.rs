//! Tests for Nexus Data Annotations
//! Nexus Data 注解测试

use nexus_data_annotations::{Entity, Table};

// ========================================================================
// Entity Annotation Tests / @Entity 注解测试
// ========================================================================

#[test]
fn test_entity_macro() {
    // This test verifies that the #[Entity] macro compiles correctly
    // and generates a table_name() method returning the struct name.
    // 此测试验证 #[Entity] 宏能正确编译并生成 table_name() 方法

    #[Entity]
    struct TestUser {
        id: i64,
        username: String,
    }

    // Verify table_name method exists and returns struct name
    // 验证 table_name 方法存在并返回结构体名
    assert_eq!(TestUser::table_name(), "TestUser");
}

#[test]
fn test_table_macro_with_custom_name() {
    // This test verifies that the #[Table] macro compiles correctly
    // and generates a table_name() method with the custom name.
    // 此测试验证 #[Table] 宏能正确编译并生成带自定义名称的 table_name() 方法

    #[Table(name = "custom_users")]
    struct TestUser {
        id: i64,
        username: String,
    }

    // Verify custom table name is used
    // 验证使用了自定义表名
    assert_eq!(TestUser::table_name(), "custom_users");
}

#[test]
fn test_table_default_name() {
    // Test default table name (lowercase struct name)
    // 测试默认表名（小写的结构体名）

    #[Table]
    struct TestUser {
        id: i64,
        username: String,
    }

    // Should use lowercase struct name
    // 应该使用小写的结构体名
    assert_eq!(TestUser::table_name(), "testuser");
}

// ========================================================================
// Column/Id/GeneratedValue Annotation Tests / 列/ID注解测试
// ========================================================================
// Note: #[Column], #[Id], #[GeneratedValue] are field-level proc macro attributes
// that currently pass through without generating code. They cannot be tested in
// unit tests within the proc-macro crate's test harness because proc-macro
// attributes on fields inside functions conflict with the test function's scope.
// Their functionality should be verified via trybuild compile tests instead.
//
// 注意：#[Column], #[Id], #[GeneratedValue] 是字段级过程宏属性，
// 当前只是传递而不生成代码。它们不能在 proc-macro crate 的测试中测试，
// 因为函数内的字段上的过程宏属性与测试函数作用域冲突。
// 应该通过 trybuild 编译测试来验证其功能。

#[test]
fn test_entity_with_multiple_fields() {
    // Test that Entity macro works with multiple fields
    // 测试 Entity 宏能处理多个字段

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

    assert_eq!(TestUser::table_name(), "TestUser");
}

#[test]
fn test_table_with_multiple_fields() {
    // Test that Table macro works with multiple fields and custom name
    // 测试 Table 宏能处理多个字段和自定义名称

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
