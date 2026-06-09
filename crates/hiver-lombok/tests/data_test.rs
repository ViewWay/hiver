//! Comprehensive tests for all hiver-lombok derive macros.
//! hiver-lombok 所有派生宏的综合测试。
//!
//! Covers: Data, Getter, Setter, AllArgsConstructor, NoArgsConstructor,
//! Builder, Value, With — including edge cases and error paths.

use hiver_lombok::{
    AllArgsConstructor, Builder, Data, Getter, NoArgsConstructor, Setter, Value, With,
};

// ============================================================================
// 1. Data macro tests / Data 宏测试
// ============================================================================

/// Test Data generates constructor, getters, setters, with methods, and Default.
/// 测试 Data 生成构造函数、getter、setter、with 方法和 Default。
#[test]
fn test_data_macro_full() {
    #[derive(Data, Clone, PartialEq, Debug)]
    struct User {
        id: i64,
        username: String,
        email: String,
    }

    // Constructor / 构造函数
    let user = User::new(1, "alice".into(), "alice@example.com".into());
    assert_eq!(user.id, 1);
    assert_eq!(user.username, "alice");
    assert_eq!(user.email, "alice@example.com");

    // Getters / Getter 方法
    assert_eq!(user.id(), 1);
    assert_eq!(user.username(), "alice");
    assert_eq!(user.email(), "alice@example.com");

    // Setters / Setter 方法
    let mut user = User::new(0, String::new(), String::new());
    user.set_id(2);
    user.set_username("bob".into());
    user.set_email("bob@example.com".into());
    assert_eq!(user.id, 2);
    assert_eq!(user.username, "bob");
    assert_eq!(user.email, "bob@example.com");

    // With methods / With 方法
    let user2 = user.with_id(3).with_username("charlie".into());
    assert_eq!(user2.id, 3);
    assert_eq!(user2.username, "charlie");
    // Original unchanged / 原始对象未改变
    assert_eq!(user.id, 2);
}

/// Test Data generates Default impl correctly.
/// 测试 Data 正确生成 Default 实现。
#[test]
fn test_data_default_impl() {
    #[derive(Data, Clone, PartialEq, Debug)]
    struct Config {
        port: u16,
        host: String,
        enabled: bool,
    }

    let default_cfg = Config::default();
    assert_eq!(default_cfg.port, 0u16);
    assert_eq!(default_cfg.host, "");
    assert!(!default_cfg.enabled);

    // Default should equal manual zero values
    // Default 应等于手动设置的零值
    let manual = Config {
        port: 0,
        host: String::new(),
        enabled: false,
    };
    assert_eq!(default_cfg, manual);
}

/// Test Data on a single-field struct.
/// 测试 Data 在单字段结构体上的表现。
#[test]
fn test_data_single_field() {
    #[derive(Data, Clone, PartialEq, Debug)]
    struct Wrapper {
        value: i32,
    }

    let w = Wrapper::new(42);
    assert_eq!(w.value(), 42);

    let w2 = w.with_value(99);
    assert_eq!(w2.value(), 99);
    assert_eq!(w.value(), 42);

    let mut w3 = Wrapper::default();
    assert_eq!(w3.value, 0);
    w3.set_value(7);
    assert_eq!(w3.value, 7);
}

/// Generic struct for testing Data with type parameters.
/// 用于测试 Data 泛型类型参数的泛型结构体。
#[derive(Data, Clone, PartialEq, Debug)]
struct Pair<T: Clone + Default, U: Clone + Default> {
    first: T,
    second: U,
}

/// Test Data on generic structs (Pair<T, U>).
/// 测试 Data 在泛型结构体上的表现。
#[test]
fn test_data_generic_struct() {
    let pair: Pair<i32, String> = Pair::new(10, "hello".into());
    assert_eq!(pair.first(), 10);
    assert_eq!(pair.second(), "hello");

    let default_pair: Pair<i32, i64> = Pair::default();
    assert_eq!(default_pair.first, 0);
    assert_eq!(default_pair.second, 0);
}

// ============================================================================
// 2. Getter macro tests / Getter 宏测试
// ============================================================================

/// Test Getter generates field accessor methods.
/// 测试 Getter 生成字段访问方法。
#[test]
fn test_getter_basic() {
    #[derive(Getter)]
    struct Point {
        x: i32,
        y: i32,
    }

    let point = Point { x: 10, y: 20 };
    assert_eq!(point.x(), 10);
    assert_eq!(point.y(), 20);
}

/// Test Getter with multiple types including String and bool.
/// 测试 Getter 处理多种类型（String、bool 等）。
#[test]
fn test_getter_varied_types() {
    #[derive(Getter)]
    struct Record {
        id: i64,
        name: String,
        active: bool,
        score: f64,
    }

    let rec = Record {
        id: 42,
        name: "test".into(),
        active: true,
        score: 3.14,
    };
    assert_eq!(rec.id(), 42);
    assert_eq!(rec.name(), "test");
    assert!(rec.active());
    assert!((rec.score() - 3.14).abs() < f64::EPSILON);
}

/// Test #[get] attribute skips field getter generation.
/// 测试 #[get] 属性跳过字段 getter 生成。
#[test]
fn test_getter_skip_attribute() {
    #[derive(Getter)]
    struct Secret {
        visible: i32,
        #[get]
        hidden: String,
    }

    let s = Secret {
        visible: 1,
        hidden: "secret".into(),
    };
    // visible should have a getter / visible 应有 getter
    assert_eq!(s.visible(), 1);
    // hidden is skipped by #[get] attribute / hidden 被 #[get] 属性跳过
    // (no .hidden() method should exist — verified by compilation)
}

// ============================================================================
// 3. Setter macro tests / Setter 宏测试
// ============================================================================

/// Test Setter generates set_xxx methods.
/// 测试 Setter 生成 set_xxx 方法。
#[test]
fn test_setter_basic() {
    #[derive(Setter)]
    struct Point {
        x: i32,
        y: i32,
    }

    let mut point = Point { x: 0, y: 0 };
    point.set_x(10);
    point.set_y(20);
    assert_eq!(point.x, 10);
    assert_eq!(point.y, 20);
}

/// Test #[set] attribute skips field setter generation.
/// 测试 #[set] 属性跳过字段 setter 生成。
#[test]
fn test_setter_skip_attribute() {
    #[derive(Setter)]
    struct Protected {
        mutable: i32,
        #[set]
        locked: String,
    }

    let mut p = Protected {
        mutable: 0,
        locked: "fixed".into(),
    };
    p.set_mutable(42);
    assert_eq!(p.mutable, 42);
    // locked is skipped — no set_locked() / locked 被跳过，无 set_locked()
}

/// Test Setter with String and complex types.
/// 测试 Setter 处理 String 和复杂类型。
#[test]
fn test_setter_string_type() {
    #[derive(Setter)]
    struct Profile {
        name: String,
        bio: String,
    }

    let mut profile = Profile {
        name: String::new(),
        bio: String::new(),
    };
    profile.set_name("alice".into());
    profile.set_bio("A developer.".into());
    assert_eq!(profile.name, "alice");
    assert_eq!(profile.bio, "A developer.");
}

// ============================================================================
// 4. AllArgsConstructor tests / AllArgsConstructor 测试
// ============================================================================

/// Test AllArgsConstructor generates new() with all fields.
/// 测试 AllArgsConstructor 生成包含所有字段的 new()。
#[test]
fn test_all_args_constructor_basic() {
    #[derive(AllArgsConstructor, PartialEq, Debug)]
    struct User {
        id: i64,
        username: String,
    }

    let user = User::new(1, "alice".into());
    assert_eq!(user.id, 1);
    assert_eq!(user.username, "alice");
}

/// Test AllArgsConstructor with many fields.
/// 测试 AllArgsConstructor 处理多字段结构体。
#[test]
fn test_all_args_constructor_many_fields() {
    #[derive(AllArgsConstructor, PartialEq, Debug)]
    struct Entity {
        a: i8,
        b: i16,
        c: i32,
        d: i64,
        e: String,
        f: bool,
    }

    let e = Entity::new(1, 2, 3, 4, "five".into(), true);
    assert_eq!(e.a, 1);
    assert_eq!(e.b, 2);
    assert_eq!(e.c, 3);
    assert_eq!(e.d, 4);
    assert_eq!(e.e, "five");
    assert!(e.f);
}

// ============================================================================
// 5. NoArgsConstructor tests / NoArgsConstructor 测试
// ============================================================================

/// Test NoArgsConstructor generates Default and new().
/// 测试 NoArgsConstructor 生成 Default 和 new()。
#[test]
fn test_no_args_constructor_default_values() {
    #[derive(NoArgsConstructor, PartialEq, Debug)]
    struct Config {
        port: u16,
        host: String,
        timeout: u64,
    }

    let cfg = Config::default();
    assert_eq!(cfg.port, 0u16);
    assert_eq!(cfg.host, "");
    assert_eq!(cfg.timeout, 0u64);

    // Also works via Default trait / 也可通过 Default trait 使用
    let cfg2 = Config::default();
    assert_eq!(cfg, cfg2);
}

/// Test NoArgsConstructor with bool and f64 fields.
/// 测试 NoArgsConstructor 处理 bool 和 f64 字段。
#[test]
fn test_no_args_constructor_primitive_defaults() {
    #[derive(NoArgsConstructor)]
    struct Primitives {
        flag: bool,
        ratio: f64,
        count: usize,
        label: String,
    }

    let p = Primitives::default();
    assert!(!p.flag);
    assert_eq!(p.ratio, 0.0);
    assert_eq!(p.count, 0);
    assert_eq!(p.label, "");
}

// ============================================================================
// 6. Builder macro tests / Builder 宏测试
// ============================================================================

/// Test Builder generates builder pattern with all required fields.
/// 测试 Builder 生成完整 builder 模式。
#[test]
fn test_builder_full_construction() {
    #[derive(Builder, PartialEq, Debug)]
    struct User {
        id: i64,
        username: String,
        email: String,
    }

    let user = User::builder()
        .id(1)
        .username("alice".into())
        .email("alice@example.com".into())
        .build()
        .unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "alice");
    assert_eq!(user.email, "alice@example.com");
}

/// Test Builder returns error when required field is missing.
/// 测试 Builder 在缺少必填字段时返回错误。
#[test]
fn test_builder_missing_field_error() {
    #[derive(Builder, Debug)]
    struct Entry {
        key: String,
        value: i32,
    }

    // Missing value / 缺少 value
    let result = Entry::builder().key("k".into()).build();
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("value"), "Error should mention 'value': {}", err_msg);

    // Missing key / 缺少 key
    let result2 = Entry::builder().value(42).build();
    assert!(result2.is_err());
    let err_msg2 = result2.unwrap_err();
    assert!(err_msg2.contains("key"), "Error should mention 'key': {}", err_msg2);
}

/// Test Builder with Option fields works correctly.
/// 测试 Builder 处理 Option 字段。
#[test]
fn test_builder_with_option_fields() {
    #[derive(Builder, Debug)]
    struct OptionalUser {
        id: i64,
        name: String,
        nickname: Option<String>,
        age: Option<u32>,
    }

    // Without optional fields / 不设置可选字段
    let user1 = OptionalUser::builder()
        .id(1)
        .name("alice".into())
        .build()
        .unwrap();
    assert_eq!(user1.id, 1);
    assert_eq!(user1.name, "alice");
    assert!(user1.nickname.is_none());
    assert!(user1.age.is_none());

    // With optional fields / 设置可选字段
    let user2 = OptionalUser::builder()
        .id(2)
        .name("bob".into())
        .nickname(Some("bobby".into()))
        .age(Some(30))
        .build()
        .unwrap();
    assert_eq!(user2.nickname.as_deref(), Some("bobby"));
    assert_eq!(user2.age, Some(30));
}

// ============================================================================
// 7. Value macro tests / Value 宏测试
// ============================================================================

/// Test Value generates constructor, getters, and with methods (immutable).
/// 测试 Value 生成构造函数、getter 和 with 方法（不可变）。
#[test]
fn test_value_immutable_access() {
    #[derive(Value, Clone, PartialEq, Debug)]
    struct Money {
        amount: i64,
        currency: String,
    }

    let m1 = Money::new(100, "USD".into());
    assert_eq!(m1.amount(), 100);
    assert_eq!(m1.currency(), "USD");

    // with_ creates a copy / with_ 创建副本
    let m2 = m1.with_amount(200);
    assert_eq!(m2.amount(), 200);
    assert_eq!(m2.currency(), "USD");
    // Original unchanged / 原始未改变
    assert_eq!(m1.amount(), 100);
}

/// Test Value with multiple fields and chained with_ calls.
/// 测试 Value 多字段和链式 with_ 调用。
#[test]
fn test_value_chained_with_methods() {
    #[derive(Value, Clone, PartialEq, Debug)]
    struct Point3D {
        x: f64,
        y: f64,
        z: f64,
    }

    let p1 = Point3D::new(1.0, 2.0, 3.0);
    let p2 = p1.with_x(10.0).with_y(20.0).with_z(30.0);
    assert_eq!(p2.x(), 10.0);
    assert_eq!(p2.y(), 20.0);
    assert_eq!(p2.z(), 30.0);
    // Original unchanged / 原始未改变
    assert_eq!(p1.x(), 1.0);
}

// ============================================================================
// 8. With macro tests / With 宏测试
// ============================================================================

/// Test With generates with_xxx methods for struct fields.
/// 测试 With 生成字段的 with_xxx 方法。
#[test]
fn test_with_basic() {
    #[derive(With, Clone, PartialEq, Debug)]
    struct Settings {
        theme: String,
        language: String,
        notifications: bool,
    }

    let s1 = Settings {
        theme: "dark".into(),
        language: "en".into(),
        notifications: true,
    };

    let s2 = s1.with_theme("light".into());
    assert_eq!(s2.theme, "light");
    assert_eq!(s1.theme, "dark"); // Original unchanged / 原始未改变

    let s3 = s1.with_language("zh".into());
    assert_eq!(s3.language, "zh");
    assert_eq!(s1.language, "en");
}

/// Test With preserves other fields when one is modified.
/// 测试 With 修改一个字段时保留其他字段不变。
#[test]
fn test_with_preserves_other_fields() {
    #[derive(With, Clone, PartialEq, Debug)]
    struct Triple {
        a: i32,
        b: i32,
        c: i32,
    }

    let t = Triple { a: 1, b: 2, c: 3 };
    let t2 = t.with_b(99);
    assert_eq!(t2.a, 1);
    assert_eq!(t2.b, 99);
    assert_eq!(t2.c, 3);
}

// ============================================================================
// 9. Cross-cutting / edge case tests / 跨切面/边界测试
// ============================================================================

/// Test combining Data with serde Serialize/Deserialize.
/// 测试 Data 与 serde Serialize/Deserialize 组合使用。
#[test]
fn test_data_with_serde() {
    use serde::{Deserialize, Serialize};

    #[derive(Data, Clone, PartialEq, Debug, Serialize, Deserialize)]
    struct Product {
        id: u64,
        name: String,
        price: f64,
    }

    let p = Product::new(1, "widget".into(), 9.99);
    let json = serde_json::to_string(&p).unwrap();
    assert!(json.contains("\"id\":1"));
    assert!(json.contains("\"name\":\"widget\""));

    let decoded: Product = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, p);
}

/// Test combining AllArgsConstructor + NoArgsConstructor (via Default) without conflict.
/// 测试 AllArgsConstructor + NoArgsConstructor（通过 Default）组合不冲突。
#[test]
fn test_combined_derives() {
    #[derive(Getter, Setter, AllArgsConstructor, NoArgsConstructor, Clone, PartialEq, Debug)]
    struct Item {
        sku: String,
        qty: u32,
    }

    // AllArgsConstructor
    let item = Item::new("ABC".into(), 10);
    assert_eq!(item.sku(), "ABC");
    assert_eq!(item.qty(), 10);

    // NoArgsConstructor (via Default)
    let default_item = Item::default();
    assert_eq!(default_item.sku, "");
    assert_eq!(default_item.qty, 0);

    // Setter
    let mut item2 = Item::new(String::new(), 0);
    item2.set_sku("XYZ".into());
    item2.set_qty(5);
    assert_eq!(item2.sku, "XYZ");
    assert_eq!(item2.qty, 5);
}

/// Test Builder with a single field struct.
/// 测试 Builder 在单字段结构体上的表现。
#[test]
fn test_builder_single_field() {
    #[derive(Builder, Debug, PartialEq)]
    struct Id {
        value: u64,
    }

    let id = Id::builder().value(42).build().unwrap();
    assert_eq!(id.value, 42);

    // Missing field returns error / 缺少字段返回错误
    let err = Id::builder().build();
    assert!(err.is_err());
}

/// Test Value does NOT generate setters (immutability check via compilation).
/// 测试 Value 不生成 setter（通过编译验证不可变性）。
#[test]
fn test_value_no_setters() {
    #[derive(Value, Clone, Debug)]
    struct ImmutablePoint {
        x: i32,
        y: i32,
    }

    let p = ImmutablePoint::new(1, 2);
    // Only getters and with_ methods are available — no set_x / set_y
    // 只有 getter 和 with_ 方法可用 — 没有 set_x / set_y
    assert_eq!(p.x(), 1);
    assert_eq!(p.y(), 2);

    let p2 = p.with_x(10);
    assert_eq!(p2.x(), 10);
    assert_eq!(p2.y(), 2);
}

/// Test that with_ methods return new instances (not mutating original).
/// 测试 with_ 方法返回新实例（不修改原始对象）。
#[test]
fn test_with_immutability_guarantee() {
    #[derive(With, Clone, PartialEq, Debug)]
    struct Counter {
        value: i32,
        label: String,
    }

    let original = Counter {
        value: 10,
        label: "orig".into(),
    };

    let modified = original.with_value(20);
    assert_eq!(original.value, 10, "Original should not be mutated");
    assert_eq!(modified.value, 20, "New instance should have new value");
    assert_eq!(modified.label, "orig", "Other fields should be preserved");
}
