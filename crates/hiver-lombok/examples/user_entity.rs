//! Nexus Lombok Examples / Nexus Lombok 示例
//!
//! This example demonstrates all Lombok-style annotations available in Nexus.
//! 此示例演示了 Nexus 中所有可用的 Lombok 风格注解。

use hiver_lombok::{
    AllArgsConstructor, Builder, Data, Getter, NoArgsConstructor, Setter, Value, With,
};

// ============================================================================
// Example 1: #[Data] - Most Common / 最常用
// ============================================================================
///
/// User entity with automatic getters, setters, constructor, and with methods
/// 自动生成 getters, setters, constructor 和 with methods 的用户实体
#[derive(Data, Clone, PartialEq, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
}

#[allow(dead_code)]
fn example_data() {
    // Constructor / 构造函数
    let user = User::new(1, "alice".into(), "alice@example.com".into(), 25);

    // Getters / Getters
    println!("Username: {}", user.username());
    println!("Email: {}", user.email());

    // Setters / Setters
    let mut user = User::default();
    user.set_id(2);
    user.set_username("bob".into());

    // With methods (chaining) / With 方法（链式调用）
    let user = User::default()
        .with_id(3)
        .with_username("charlie".into())
        .with_email("charlie@example.com".into())
        .with_age(30);

    println!("User: {:?}", user);
}

// ============================================================================
// Example 2: Individual Derives / 单独派生
// ============================================================================

/// Point struct with explicit getter/setter
/// 显式 getter/setter 的点结构体
#[derive(Getter, Setter, PartialEq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[allow(dead_code)]
fn example_individual() {
    let mut point = Point { x: 0, y: 0 };

    // Getter
    println!("X: {}", point.x());

    // Setter
    point.set_x(10);
    point.set_y(20);
}

// ============================================================================
// Example 3: Constructors / 构造函数
// ============================================================================

/// Config with all-args constructor
/// 全参数构造函数的 Config
#[derive(AllArgsConstructor, PartialEq, Debug)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub timeout: u64,
}

/// Config with no-args constructor (uses Default)
/// 无参数构造函数的 Config（使用 Default）
#[derive(NoArgsConstructor, PartialEq, Debug)]
pub struct DefaultConfig {
    pub port: u16,
    pub host: String,
    pub timeout: u64,
}

#[allow(dead_code)]
fn example_constructors() {
    // All args / 所有参数
    let config = Config::new(8080, "localhost".into(), 30000);
    println!("Config: {:?}", config);

    // No args (uses Default) / 无参数（使用 Default）
    let config = DefaultConfig::default();
    println!("Default config: {:?}", config);
}

// ============================================================================
// Example 4: @Value - Immutable / 不可变
// ============================================================================

/// Immutable value object / 不可变值对象
#[derive(Value, Clone, PartialEq, Debug)]
pub struct Money {
    pub amount: i64,
    pub currency: String,
}

#[allow(dead_code)]
fn example_value() {
    let money1 = Money::new(100, "USD".into());

    // Read-only access / 只读访问
    println!("Amount: {}", money1.amount());

    // Create modified copy / 创建修改后的副本
    let money2 = money1.with_amount(200);

    // Original unchanged / 原始保持不变
    assert_eq!(money1.amount(), 100);
    assert_eq!(money2.amount(), 200);
}

// ============================================================================
// Example 5: @Builder - Builder Pattern / Builder 模式
// ============================================================================

/// Complex user with many optional fields
/// 有许多可选字段的复杂用户
#[derive(Builder, PartialEq, Debug)]
pub struct ComplexUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: Option<i32>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[allow(dead_code)]
fn example_builder() {
    let user = ComplexUser::builder()
        .id(1)
        .username("alice".into())
        .email("alice@example.com".into())
        .age(Some(25))
        .build()
        .unwrap();

    println!("User: {:?}", user);
}

// ============================================================================
// Example 6: @With - With Methods / With 方法
// ============================================================================

/// Struct with with methods
/// 带有 with 方法的结构体
#[derive(With, Clone, PartialEq, Debug)]
pub struct Settings {
    pub theme: String,
    pub language: String,
    pub notifications: bool,
}

#[allow(dead_code)]
fn example_with() {
    let settings = Settings {
        theme: "dark".into(),
        language: "en".into(),
        notifications: true,
    };

    // Create modified copies / 创建修改后的副本
    let light_mode = settings.with_theme("light".into());
    let chinese = settings.with_language("zh".into());

    assert_eq!(settings.theme, "dark");
    assert_eq!(light_mode.theme, "light");
}

// ============================================================================
// Example 7: Combined Macros / 组合宏
// ============================================================================

/// User combining multiple macros
/// 组合多个宏的用户
#[derive(
    Data,      // Getters, setters, constructor, with methods
    Clone,     // Required for with methods
    PartialEq, // For comparisons
    Debug,     // For debugging
)]
pub struct CompleteUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
}

#[allow(dead_code)]
fn example_complete() {
    // Constructor / 构造函数
    let mut user = CompleteUser::new(1, "alice".into(), "alice@example.com".into(), 25);

    // All Data methods available / 所有 Data 方法可用
    println!("ID: {}", user.id());
    user.set_username("bob".into());
    let user2 = user.with_age(30);
    let _ = user2;
}

fn main() {
    println!("=== Example 1: #[Data] ===");
    example_data();

    println!("\n=== Example 4: @Value ===");
    example_value();

    println!("\n=== Example 5: @Builder ===");
    example_builder();

    println!("\n=== Example 7: Combined ===");
    example_complete();
}
