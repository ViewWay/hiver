//! Tests for nexus-validation-annotations attribute parsing.
//! 验证注解属性解析的测试。
//!
//! These tests verify that derive macros actually use the attribute values
//! (e.g., `#[min(5)]` uses 5, not a hardcoded default).
//! 这些测试验证派生宏确实使用了属性值（例如 `#[min(5)]` 使用 5，而非硬编码默认值）。

use proc_macro2::TokenStream;
use quote::quote;

/// Helper: parse a DeriveInput from a token stream.
/// 辅助函数：从 token 流解析 DeriveInput。
fn parse_input(tokens: TokenStream) -> syn::DeriveInput {
    syn::parse2::<syn::DeriveInput>(tokens).expect("Failed to parse DeriveInput")
}

// ========================================================================
// Min attribute parsing tests / Min 属性解析测试
// ========================================================================

/// Test that `#[min(5)]` extracts value 5, not the hardcoded default 0.
/// 测试 `#[min(5)]` 提取值 5，而非硬编码默认值 0。
#[test]
fn test_min_bare_value_is_parsed() {
    let input = parse_input(quote! {
        struct Order {
            #[min(5)]
            quantity: i32,
        }
    });

    let fields = super::extract_fields_with_min(&input);

    assert_eq!(fields.len(), 1, "Should find exactly one field with #[min]");
    let (ident, value) = &fields[0];
    assert_eq!(ident.to_string(), "quantity");
    assert_eq!(*value, 5, "#[min(5)] should parse value as 5, not 0");
}

/// Test that `#[min(value = 10)]` extracts value 10.
/// 测试 `#[min(value = 10)]` 提取值 10。
#[test]
fn test_min_named_value_is_parsed() {
    let input = parse_input(quote! {
        struct Order {
            #[min(value = 10)]
            quantity: i32,
        }
    });

    let fields = super::extract_fields_with_min(&input);

    assert_eq!(fields.len(), 1);
    let (_, value) = &fields[0];
    assert_eq!(*value, 10, "#[min(value = 10)] should parse value as 10");
}

/// Test that `#[min(0)]` correctly parses zero.
/// 测试 `#[min(0)]` 正确解析零值。
#[test]
fn test_min_zero_value() {
    let input = parse_input(quote! {
        struct Data {
            #[min(0)]
            val: i32,
        }
    });

    let fields = super::extract_fields_with_min(&input);
    assert_eq!(fields[0].1, 0);
}

/// Test that `#[min(999)]` parses large values.
/// 测试 `#[min(999)]` 解析大数值。
#[test]
fn test_min_large_value() {
    let input = parse_input(quote! {
        struct Data {
            #[min(999)]
            val: u32,
        }
    });

    let fields = super::extract_fields_with_min(&input);
    assert_eq!(fields[0].1, 999);
}

// ========================================================================
// Max attribute parsing tests / Max 属性解析测试
// ========================================================================

/// Test that `#[max(100)]` extracts value 100, not u32::MAX.
/// 测试 `#[max(100)]` 提取值 100，而非 u32::MAX。
#[test]
fn test_max_bare_value_is_parsed() {
    let input = parse_input(quote! {
        struct Order {
            #[max(100)]
            quantity: i32,
        }
    });

    let fields = super::extract_fields_with_max(&input);

    assert_eq!(fields.len(), 1);
    let (ident, value) = &fields[0];
    assert_eq!(ident.to_string(), "quantity");
    assert_eq!(*value, 100, "#[max(100)] should parse value as 100, not u32::MAX");
}

/// Test that `#[max(value = 50)]` extracts value 50.
/// 测试 `#[max(value = 50)]` 提取值 50。
#[test]
fn test_max_named_value_is_parsed() {
    let input = parse_input(quote! {
        struct Order {
            #[max(value = 50)]
            quantity: i32,
        }
    });

    let fields = super::extract_fields_with_max(&input);

    assert_eq!(fields.len(), 1);
    let (_, value) = &fields[0];
    assert_eq!(*value, 50, "#[max(value = 50)] should parse value as 50");
}

/// Test multiple fields with different max values.
/// 测试多个字段使用不同的 max 值。
#[test]
fn test_max_multiple_fields() {
    let input = parse_input(quote! {
        struct Data {
            #[max(10)]
            small: u32,
            #[max(1000)]
            large: u32,
        }
    });

    let fields = super::extract_fields_with_max(&input);
    assert_eq!(fields.len(), 2);
    // Fields should be in declaration order / 字段应按声明顺序
    assert_eq!(fields[0].0.to_string(), "small");
    assert_eq!(fields[0].1, 10);
    assert_eq!(fields[1].0.to_string(), "large");
    assert_eq!(fields[1].1, 1000);
}

// ========================================================================
// Pattern attribute parsing tests / Pattern 属性解析测试
// ========================================================================

/// Test that `#[pattern("^[a-z]+$")]` extracts the actual regex, not ".*".
/// 测试 `#[pattern("^[a-z]+$")]` 提取实际正则，而非 ".*"。
#[test]
fn test_pattern_bare_string_is_parsed() {
    let input = parse_input(quote! {
        struct User {
            #[pattern("^[a-z]+$")]
            username: String,
        }
    });

    let fields = super::extract_fields_with_pattern(&input);

    assert_eq!(fields.len(), 1);
    let (ident, pattern) = &fields[0];
    assert_eq!(ident.to_string(), "username");
    assert_eq!(
        pattern.as_str(),
        "^[a-z]+$",
        "#[pattern(\"^[a-z]+$\")] should parse the actual regex, not \".*\""
    );
}

/// Test that `#[pattern(regex = "^[0-9]{3}$")]` extracts the regex via named arg.
/// 测试 `#[pattern(regex = \"^[0-9]{3}$\")] 通过命名参数提取正则。
#[test]
fn test_pattern_named_value_is_parsed() {
    let input = parse_input(quote! {
        struct Data {
            #[pattern(regex = "^[0-9]{3}$")]
            code: String,
        }
    });

    let fields = super::extract_fields_with_pattern(&input);

    assert_eq!(fields.len(), 1);
    let (_, pattern) = &fields[0];
    assert_eq!(pattern.as_str(), "^[0-9]{3}$");
}

/// Test pattern with complex regex containing special characters.
/// 测试包含特殊字符的复杂正则。
#[test]
fn test_pattern_complex_regex() {
    let input = parse_input(quote! {
        struct Data {
            #[pattern("^[A-Z][a-zA-Z0-9._%+-]+@[A-Z0-9.-]+\\.[A-Z]{2,}$")]
            email: String,
        }
    });

    let fields = super::extract_fields_with_pattern(&input);
    assert_eq!(fields[0].1.as_str(), "^[A-Z][a-zA-Z0-9._%+-]+@[A-Z0-9.-]+\\.[A-Z]{2,}$");
}

// ========================================================================
// Length attribute parsing tests / Length 属性解析测试
// ========================================================================

/// Test that `#[length(min = 3, max = 20)]` extracts both values, not defaults.
/// 测试 `#[length(min = 3, max = 20)]` 提取两个值，而非默认值。
#[test]
fn test_length_min_max_parsed() {
    let input = parse_input(quote! {
        struct User {
            #[length(min = 3, max = 20)]
            username: String,
        }
    });

    let fields = super::extract_fields_with_length(&input);

    assert_eq!(fields.len(), 1);
    let (ident, min, max) = &fields[0];
    assert_eq!(ident.to_string(), "username");
    assert_eq!(*min, 3, "#[length(min = 3, ...)] should parse min as 3, not 0");
    assert_eq!(*max, 20, "#[length(..., max = 20)] should parse max as 20, not u32::MAX");
}

/// Test `#[length(min = 5)]` with only min specified.
/// 测试仅指定 min 的 `#[length(min = 5)]`。
#[test]
fn test_length_min_only() {
    let input = parse_input(quote! {
        struct Data {
            #[length(min = 5)]
            text: String,
        }
    });

    let fields = super::extract_fields_with_length(&input);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].1, 5, "min should be 5");
    assert_eq!(fields[0].2, u32::MAX, "max should default to u32::MAX when not specified");
}

/// Test `#[length(max = 100)]` with only max specified.
/// 测试仅指定 max 的 `#[length(max = 100)]`。
#[test]
fn test_length_max_only() {
    let input = parse_input(quote! {
        struct Data {
            #[length(max = 100)]
            text: String,
        }
    });

    let fields = super::extract_fields_with_length(&input);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].1, 0, "min should default to 0 when not specified");
    assert_eq!(fields[0].2, 100, "max should be 100");
}

/// Test `#[length(min = 1, max = 1)]` with equal min and max.
/// 测试 min 和 max 相等的 `#[length(min = 1, max = 1)]`。
#[test]
fn test_length_equal_min_max() {
    let input = parse_input(quote! {
        struct Data {
            #[length(min = 1, max = 1)]
            single_char: String,
        }
    });

    let fields = super::extract_fields_with_length(&input);
    assert_eq!(fields[0].1, 1);
    assert_eq!(fields[0].2, 1);
}

// ========================================================================
// Multiple attributes on same struct / 同一结构体上的多个属性
// ========================================================================

/// Test that fields without the target attribute are skipped.
/// 测试没有目标属性的字段会被跳过。
#[test]
fn test_only_tagged_fields_extracted() {
    let input = parse_input(quote! {
        struct Mixed {
            #[min(5)]
            tagged: i32,
            untagged: i32,
        }
    });

    let min_fields = super::extract_fields_with_min(&input);
    assert_eq!(min_fields.len(), 1, "Only tagged field should be extracted");
    assert_eq!(min_fields[0].0.to_string(), "tagged");
}

/// Test struct with no matching attributes returns empty vec.
/// 测试没有匹配属性的结构体返回空向量。
#[test]
fn test_no_matching_attributes() {
    let input = parse_input(quote! {
        struct Plain {
            field: i32,
        }
    });

    assert!(super::extract_fields_with_min(&input).is_empty());
    assert!(super::extract_fields_with_max(&input).is_empty());
    assert!(super::extract_fields_with_pattern(&input).is_empty());
    assert!(super::extract_fields_with_length(&input).is_empty());
}
