//! Tests for cache condition evaluation
//! 缓存条件评估的测试

use std::collections::HashMap;

use hiver_cache::{CacheEvictOptions, CachePutOptions, CacheableOptions, evaluate_cache_condition};
use serde_json::Value as JsonValue;

// ========================================================================
// Test Cacheable with Conditions / 测试带条件的 Cacheable
// ========================================================================

#[test]
fn test_condition_id_positive()
{
    let mut args = HashMap::new();
    args.insert("id".to_string(), JsonValue::Number(123.into()));

    assert!(evaluate_cache_condition("#id > 0", &args, None));
    assert!(!evaluate_cache_condition("#id < 0", &args, None));
}

#[test]
fn test_condition_age_check()
{
    let mut args = HashMap::new();
    args.insert("age".to_string(), JsonValue::Number(25.into()));

    assert!(evaluate_cache_condition("#age > 18", &args, None));
    assert!(evaluate_cache_condition("#age >= 25", &args, None));
    assert!(!evaluate_cache_condition("#age < 18", &args, None));
}

#[test]
fn test_condition_string_equality()
{
    let mut args = HashMap::new();
    args.insert("role".to_string(), JsonValue::String("ADMIN".to_string()));

    assert!(evaluate_cache_condition("#role == 'ADMIN'", &args, None));
    assert!(!evaluate_cache_condition("#role == 'USER'", &args, None));
    assert!(evaluate_cache_condition("#role != 'USER'", &args, None));
}

#[test]
fn test_condition_and_expressions()
{
    let mut args = HashMap::new();
    args.insert("age".to_string(), JsonValue::Number(25.into()));
    args.insert("active".to_string(), JsonValue::Bool(true));

    assert!(evaluate_cache_condition("#age > 18 and #active", &args, None));
    assert!(!evaluate_cache_condition("#age > 30 and #active", &args, None));
    assert!(!evaluate_cache_condition("#age > 18 and !#active", &args, None));
}

#[test]
fn test_condition_or_expressions()
{
    let mut args = HashMap::new();
    args.insert("role".to_string(), JsonValue::String("USER".to_string()));
    args.insert("admin".to_string(), JsonValue::Bool(false));

    assert!(evaluate_cache_condition("#role == 'ADMIN' or #admin", &args, None) == false);
    assert!(evaluate_cache_condition("#role == 'USER' or #admin", &args, None) == true);
}

#[test]
fn test_condition_not_expressions()
{
    let mut args = HashMap::new();
    args.insert("active".to_string(), JsonValue::Bool(false));
    args.insert("deleted".to_string(), JsonValue::Bool(true));

    assert!(evaluate_cache_condition("!#active", &args, None));
    assert!(!evaluate_cache_condition("!#deleted", &args, None));
}

#[test]
fn test_condition_is_empty()
{
    let mut args = HashMap::new();
    args.insert("name".to_string(), JsonValue::String(String::new()));
    args.insert("email".to_string(), JsonValue::String("test@example.com".to_string()));
    args.insert("list".to_string(), JsonValue::Array(vec![]));
    args.insert("items".to_string(), JsonValue::Array(vec![JsonValue::Number(1.into())]));

    assert!(evaluate_cache_condition("#name.isEmpty()", &args, None));
    assert!(!evaluate_cache_condition("#email.isEmpty()", &args, None));
    assert!(evaluate_cache_condition("#list.isEmpty()", &args, None));
    assert!(!evaluate_cache_condition("#items.isEmpty()", &args, None));
}

#[test]
fn test_condition_length_checks()
{
    let mut args = HashMap::new();
    args.insert("username".to_string(), JsonValue::String("alice".to_string()));
    args.insert("name".to_string(), JsonValue::String("Bo".to_string()));

    assert!(evaluate_cache_condition("#username.length() > 3", &args, None));
    assert!(evaluate_cache_condition("#username.length() >= 5", &args, None));
    assert!(!evaluate_cache_condition("#name.length() > 3", &args, None));
    assert!(evaluate_cache_condition("#name.length() == 2", &args, None));
}

#[test]
fn test_unless_result_null()
{
    let args = HashMap::new();

    // Unless: don't cache if result is null
    assert!(evaluate_cache_condition("#result == null", &args, Some(&JsonValue::Null)));
    assert!(!evaluate_cache_condition(
        "#result == null",
        &args,
        Some(&JsonValue::String("test".to_string()))
    ));
}

#[test]
fn test_unless_result_empty()
{
    let args = HashMap::new();

    // Unless: don't cache if result is empty string
    assert!(evaluate_cache_condition(
        "#result.isEmpty()",
        &args,
        Some(&JsonValue::String(String::new()))
    ));
    assert!(!evaluate_cache_condition(
        "#result.isEmpty()",
        &args,
        Some(&JsonValue::String("test".to_string()))
    ));
}

#[test]
fn test_complex_conditions()
{
    let mut args = HashMap::new();
    args.insert("age".to_string(), JsonValue::Number(25.into()));
    args.insert("active".to_string(), JsonValue::Bool(true));
    args.insert("role".to_string(), JsonValue::String("USER".to_string()));
    args.insert("score".to_string(), JsonValue::Number(85.into()));

    // Cache only active adults
    assert!(evaluate_cache_condition("#age >= 18 and #active", &args, None));

    // Cache admins or high scorers
    assert!(evaluate_cache_condition("#role == 'ADMIN' or #score > 80", &args, None));

    // Don't cache inactive users regardless of score
    assert!(!evaluate_cache_condition("#age > 18 and !#active", &args, None));

    // Cache active users with good scores
    assert!(evaluate_cache_condition("#active and #score > 70", &args, None));
}

#[test]
fn test_condition_with_multiple_parameters()
{
    let mut args = HashMap::new();
    args.insert("min_age".to_string(), JsonValue::Number(18.into()));
    args.insert("max_age".to_string(), JsonValue::Number(65.into()));
    args.insert("age".to_string(), JsonValue::Number(35.into()));

    assert!(evaluate_cache_condition("#age >= #min_age and #age <= #max_age", &args, None));
    assert!(!evaluate_cache_condition("#age < #min_age or #age > #max_age", &args, None));
}

#[test]
fn test_condition_with_string_comparison()
{
    let mut args = HashMap::new();
    args.insert("status".to_string(), JsonValue::String("ACTIVE".to_string()));
    args.insert("role".to_string(), JsonValue::String("USER".to_string()));

    assert!(evaluate_cache_condition("#status == 'ACTIVE'", &args, None));
    assert!(evaluate_cache_condition("#status != 'INACTIVE'", &args, None));
    assert!(!evaluate_cache_condition("#role == 'ADMIN'", &args, None));
}

#[test]
fn test_nested_conditions()
{
    let mut args = HashMap::new();
    args.insert("age".to_string(), JsonValue::Number(25.into()));
    args.insert("active".to_string(), JsonValue::Bool(true));
    args.insert("verified".to_string(), JsonValue::Bool(false));
    args.insert("premium".to_string(), JsonValue::Bool(true));

    // Complex nested condition
    assert!(evaluate_cache_condition(
        "#active and (#premium or #verified) and #age > 18",
        &args,
        None
    ));

    // Another complex condition
    assert!(evaluate_cache_condition(
        "(#age >= 18 and #active) or (#premium and !#verified)",
        &args,
        None
    ));
}

#[test]
fn test_cacheable_options_builder()
{
    let options = CacheableOptions::new()
        .cache_name("users")
        .key("#id")
        .condition("#id > 0")
        .unless("#result == null")
        .cache_null(false);

    assert_eq!(options.cache_names, vec!["users"]);
    assert_eq!(options.key, Some("#id".to_string()));
    assert_eq!(options.condition, Some("#id > 0".to_string()));
    assert_eq!(options.unless, Some("#result == null".to_string()));
    assert!(!options.cache_null);
}

#[test]
fn test_cache_put_options_builder()
{
    let options = CachePutOptions::new()
        .cache_name("users")
        .key("#user.id")
        .condition("#user.active");

    assert_eq!(options.cache_names, vec!["users"]);
    assert_eq!(options.key, Some("#user.id".to_string()));
    assert_eq!(options.condition, Some("#user.active".to_string()));
}

#[test]
fn test_cache_evict_options_builder()
{
    let options = CacheEvictOptions::new()
        .cache_name("users")
        .key("#id")
        .all_entries(false)
        .before_invocation(false)
        .condition("#id > 0");

    assert_eq!(options.cache_names, vec!["users"]);
    assert_eq!(options.key, Some("#id".to_string()));
    assert!(!options.all_entries);
    assert!(!options.before_invocation);
    assert_eq!(options.condition, Some("#id > 0".to_string()));
}

#[test]
fn test_condition_with_zero()
{
    let mut args = HashMap::new();
    args.insert("count".to_string(), JsonValue::Number(0.into()));

    assert!(evaluate_cache_condition("#count == 0", &args, None));
    assert!(!evaluate_cache_condition("#count > 0", &args, None));
    assert!(evaluate_cache_condition("#count >= 0", &args, None));
}

#[test]
fn test_condition_with_negative_numbers()
{
    let mut args = HashMap::new();
    args.insert("temperature".to_string(), JsonValue::Number((-10).into()));

    assert!(evaluate_cache_condition("#temperature < 0", &args, None));
    assert!(evaluate_cache_condition("#temperature <= -10", &args, None));
    assert!(!evaluate_cache_condition("#temperature > 0", &args, None));
}

#[test]
fn test_condition_with_floating_point()
{
    let mut args = HashMap::new();
    args.insert(
        "price".to_string(),
        JsonValue::Number(serde_json::Number::from_f64(99.99).unwrap()),
    );

    assert!(evaluate_cache_condition("#price > 50", &args, None));
    assert!(evaluate_cache_condition("#price < 100", &args, None));
    assert!(!evaluate_cache_condition("#price > 100", &args, None));
}

#[test]
fn test_condition_equality_with_different_types()
{
    let mut args = HashMap::new();
    args.insert("id".to_string(), JsonValue::Number(123.into()));
    args.insert("name".to_string(), JsonValue::String("123".to_string()));

    // Number comparison
    assert!(evaluate_cache_condition("#id == 123", &args, None));

    // String comparison
    assert!(evaluate_cache_condition("#name == '123'", &args, None));
}

#[test]
fn test_multiple_or_conditions()
{
    let mut args = HashMap::new();
    args.insert("role".to_string(), JsonValue::String("USER".to_string()));
    args.insert("admin".to_string(), JsonValue::Bool(false));
    args.insert("moderator".to_string(), JsonValue::Bool(false));

    assert!(
        evaluate_cache_condition("#role == 'ADMIN' or #admin or #moderator", &args, None) == false
    );
}

#[test]
fn test_multiple_and_conditions()
{
    let mut args = HashMap::new();
    args.insert("age".to_string(), JsonValue::Number(25.into()));
    args.insert("active".to_string(), JsonValue::Bool(true));
    args.insert("verified".to_string(), JsonValue::Bool(true));
    args.insert("premium".to_string(), JsonValue::Bool(true));

    // All conditions must be true
    assert!(evaluate_cache_condition(
        "#age > 18 and #active and #verified and #premium",
        &args,
        None
    ));

    // One false makes the whole expression false
    args.insert("premium".to_string(), JsonValue::Bool(false));
    assert!(!evaluate_cache_condition(
        "#age > 18 and #active and #verified and #premium",
        &args,
        None
    ));
}

#[test]
fn test_condition_precedence()
{
    let mut args = HashMap::new();
    args.insert("a".to_string(), JsonValue::Bool(true));
    args.insert("b".to_string(), JsonValue::Bool(false));
    args.insert("c".to_string(), JsonValue::Bool(true));

    // AND has higher precedence than OR
    // This should be: a or (b and c) = true or (false and true) = true
    assert!(evaluate_cache_condition("#a or #b and #c", &args, None));

    // With explicit parentheses
    assert!(evaluate_cache_condition("#a or (#b and #c)", &args, None));
    assert!(evaluate_cache_condition("(#a or #b) and #c", &args, None));
}

#[test]
fn test_real_world_cache_conditions()
{
    // Test 1: Only cache active users
    let mut args = HashMap::new();
    args.insert("active".to_string(), JsonValue::Bool(true));
    args.insert("deleted".to_string(), JsonValue::Bool(false));
    assert!(evaluate_cache_condition("#active and !#deleted", &args, None));

    // Test 2: Only cache products in stock
    let mut args2 = HashMap::new();
    args2.insert("stock".to_string(), JsonValue::Number(10.into()));
    args2.insert("available".to_string(), JsonValue::Bool(true));
    assert!(evaluate_cache_condition("#stock > 0 and #available", &args2, None));

    // Test 3: Don't cache sensitive data
    let mut args3 = HashMap::new();
    args3.insert("sensitive".to_string(), JsonValue::Bool(true));
    assert!(!evaluate_cache_condition("!#sensitive", &args3, None));

    // Test 4: Cache only valid sessions
    let mut args4 = HashMap::new();
    args4.insert("expired".to_string(), JsonValue::Bool(false));
    args4.insert("revoked".to_string(), JsonValue::Bool(false));
    assert!(evaluate_cache_condition("!#expired and !#revoked", &args4, None));
}
