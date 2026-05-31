//! Spring Data + MyBatis-Plus Style Example / Spring Data + MyBatis-Plus 风格示例
//!
//! This example demonstrates combining Spring Data JPA annotations with MyBatis-Plus patterns.
//! 此示例演示了如何将 Spring Data JPA 注解与 MyBatis-Plus 模式结合使用。

use hiver_data_annotations::{Entity, Table};

// ============================================================================
// Example 1: Basic Entity with @Table Annotation / 带有 @Table 注解的基本实体
// ============================================================================

/// User entity mapped to "users" table
/// 映射到 "users" 表的用户实体
#[Table(name = "users")]
#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
}

// ============================================================================
// Example 2: Entity with default table name / 带有默认表名的实体
// ============================================================================

/// Product entity — default table name is "Product" (struct name)
/// 产品实体 — 默认表名为 "Product"（结构体名）
#[Entity]
#[derive(Debug, Clone)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: f64,
}

// ============================================================================
// Example 3: Demonstration / 演示
// ============================================================================

fn main() {
    println!("=== Nexus Data Annotations Demo ===\n");

    // Table annotation generates table_name()
    // Table 注解生成 table_name() 方法
    println!("User table: {}", User::table_name());
    assert_eq!(User::table_name(), "users");

    // Entity annotation generates table_name() (defaults to struct name)
    // Entity 注解生成 table_name()（默认为结构体名）
    println!("Product table: {}", Product::table_name());
    assert_eq!(Product::table_name(), "Product");

    // Create and use entities
    // 创建并使用实体
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };
    println!("\nUser: {:?}", user);

    let product = Product {
        id: 101,
        name: "Rust Book".to_string(),
        price: 49.99,
    };
    println!("Product: {:?}", product);

    println!("\n=== Demo Complete ===");
}
