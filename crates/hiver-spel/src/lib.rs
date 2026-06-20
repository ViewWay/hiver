//! # Hiver SpEL — Simplified Spring Expression Language Engine
//!
//! A lightweight expression parser and evaluator for security annotations
//! in the Hiver framework, inspired by Spring's SpEL.
//!
//! # Supported Expressions / 支持的表达式
//!
//! - `hasRole('ADMIN')` — check role / 检查角色
//! - `hasAuthority('WRITE')` — check authority / 检查权限
//! - `hasAnyRole('ADMIN', 'SUPERUSER')` — any of roles / 任一角色
//! - `permitAll` / `denyAll` — always true/false
//! - `#var == value` — variable comparison / 变量比较
//! - `expr1 and expr2` / `expr1 or expr2` — logical ops / 逻辑运算
//! - `not expr` / `!expr` — negation / 取反

#![warn(missing_docs)]
#![warn(unreachable_pub)]

mod context;
mod evaluator;
mod parser;

pub use context::SpelContext;
pub use evaluator::SpelEvaluator;
pub use parser::{SpelError, SpelExpr};

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_has_role()
    {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        assert!(
            SpelEvaluator::new("hasRole('ADMIN')")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            !SpelEvaluator::new("hasRole('USER')")
                .evaluate(&ctx)
                .unwrap()
        );
    }

    #[test]
    fn test_has_authority()
    {
        let mut ctx = SpelContext::new();
        ctx.add_authority("WRITE");
        assert!(
            SpelEvaluator::new("hasAuthority('WRITE')")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            !SpelEvaluator::new("hasAuthority('READ')")
                .evaluate(&ctx)
                .unwrap()
        );
    }

    #[test]
    fn test_has_any_role()
    {
        let mut ctx = SpelContext::new();
        ctx.add_role("EDITOR");
        assert!(
            SpelEvaluator::new("hasAnyRole('ADMIN', 'EDITOR')")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            !SpelEvaluator::new("hasAnyRole('ADMIN', 'SUPERUSER')")
                .evaluate(&ctx)
                .unwrap()
        );
    }

    #[test]
    fn test_permit_all_deny_all()
    {
        let ctx = SpelContext::new();
        assert!(SpelEvaluator::new("permitAll").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("denyAll").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_logical_and_or()
    {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        ctx.add_authority("WRITE");
        assert!(
            SpelEvaluator::new("hasRole('ADMIN') and hasAuthority('WRITE')")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            SpelEvaluator::new("hasRole('USER') or hasRole('ADMIN')")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            !SpelEvaluator::new("hasRole('ADMIN') and hasAuthority('READ')")
                .evaluate(&ctx)
                .unwrap()
        );
    }

    #[test]
    fn test_logical_not()
    {
        let ctx = SpelContext::new();
        assert!(SpelEvaluator::new("not denyAll").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("!denyAll").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_number_comparison()
    {
        let mut ctx = SpelContext::new();
        ctx.set_variable("age", serde_json::json!(42));
        assert!(SpelEvaluator::new("#age == 42").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#age > 40").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#age < 50").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_variable_string_eq()
    {
        let mut ctx = SpelContext::new();
        ctx.set_variable("name", serde_json::json!("alice"));
        assert!(
            SpelEvaluator::new("#name == 'alice'")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(!SpelEvaluator::new("#name == 'bob'").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_complex_expression()
    {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        ctx.set_variable("userId", serde_json::json!(1));
        assert!(
            SpelEvaluator::new("hasRole('ADMIN') or #userId == 1")
                .evaluate(&ctx)
                .unwrap()
        );
    }

    #[test]
    fn test_parenthesized()
    {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        assert!(
            SpelEvaluator::new("(hasRole('ADMIN') or hasRole('USER')) and true")
                .evaluate(&ctx)
                .unwrap()
        );
    }

    #[test]
    fn test_parse_error()
    {
        let ctx = SpelContext::new();
        assert!(SpelEvaluator::new("hasRole(").evaluate(&ctx).is_err());
    }

    // ========================================================================
    // Additional coverage: auth checks, property access, all comparison
    // operators, lexer/parser errors, and context mutators.
    // 额外覆盖：认证检查、属性访问、全部比较算子、词法/解析错误与上下文 mutator。
    // ========================================================================

    #[test]
    fn test_is_authenticated_and_is_anonymous()
    {
        // A fresh context is anonymous (not authenticated) by default.
        // 新上下文默认为匿名（未认证）。
        let ctx = SpelContext::new();
        assert!(SpelEvaluator::new("isAnonymous()").evaluate(&ctx).unwrap());
        assert!(
            !SpelEvaluator::new("isAuthenticated()")
                .evaluate(&ctx)
                .unwrap()
        );

        // After set_authenticated(true), the flags flip.
        // set_authenticated(true) 后，标志翻转。
        let mut ctx = SpelContext::new();
        ctx.set_authenticated(true);
        assert!(
            SpelEvaluator::new("isAuthenticated()")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(!SpelEvaluator::new("isAnonymous()").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_comparison_operators_neq_gteq_lteq()
    {
        // Only ==, >, < are covered above; exercise !=, >=, <=.
        // 上面只覆盖了 ==, >, <；此处覆盖 !=, >=, <=。
        let mut ctx = SpelContext::new();
        ctx.set_variable("n", serde_json::json!(5));
        assert!(SpelEvaluator::new("#n != 6").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("#n != 5").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#n >= 5").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#n >= 4").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#n <= 5").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#n <= 6").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("#n >= 6").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("#n <= 4").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_string_comparison_ordering()
    {
        // String ordering uses lexicographic comparison via as_str().
        // 字符串比较通过 as_str() 进行字典序比较。
        let mut ctx = SpelContext::new();
        ctx.set_variable("s", serde_json::json!("banana"));
        assert!(SpelEvaluator::new("#s > 'apple'").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#s < 'cherry'").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#s != 'apple'").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_property_access_on_object()
    {
        // #user.name reads the "name" field of an object variable.
        // #user.name 读取对象变量的 "name" 字段。
        let mut ctx = SpelContext::new();
        ctx.set_variable("user", serde_json::json!({"name": "alice", "age": 30}));
        assert!(
            SpelEvaluator::new("#user.name == 'alice'")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            SpelEvaluator::new("#user.age == 30")
                .evaluate(&ctx)
                .unwrap()
        );
    }

    #[test]
    fn test_string_properties_length_empty_case()
    {
        // String property access: length, isEmpty, toUpper, toLower, trim.
        // 字符串属性访问：length、isEmpty、toUpper、toLower、trim。
        let mut ctx = SpelContext::new();
        ctx.set_variable("s", serde_json::json!("Hi"));
        assert!(SpelEvaluator::new("#s.length == 2").evaluate(&ctx).unwrap());
        assert!(
            SpelEvaluator::new("#s.isEmpty == false")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            SpelEvaluator::new("#s.toUpper == 'HI'")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            SpelEvaluator::new("#s.toLower == 'hi'")
                .evaluate(&ctx)
                .unwrap()
        );

        // trim: a padded string trims to the inner token.
        // trim：带空白的字符串修剪为内部 token。
        let mut ctx2 = SpelContext::new();
        ctx2.set_variable("padded", serde_json::json!("  x  "));
        assert!(
            SpelEvaluator::new("#padded.trim == 'x'")
                .evaluate(&ctx2)
                .unwrap()
        );
    }

    #[test]
    fn test_array_properties_length_empty()
    {
        // Array property access: length/size, isEmpty/is_empty.
        // 数组属性访问：length/size、isEmpty/is_empty。
        let mut ctx = SpelContext::new();
        ctx.set_variable("arr", serde_json::json!([1, 2, 3]));
        assert!(
            SpelEvaluator::new("#arr.length == 3")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(SpelEvaluator::new("#arr.size == 3").evaluate(&ctx).unwrap());
        assert!(
            SpelEvaluator::new("#arr.isEmpty == false")
                .evaluate(&ctx)
                .unwrap()
        );

        let mut ctx2 = SpelContext::new();
        ctx2.set_variable("empty", serde_json::json!([]));
        assert!(
            SpelEvaluator::new("#empty.is_empty == true")
                .evaluate(&ctx2)
                .unwrap()
        );
    }

    #[test]
    fn test_number_properties_intvalue_doublevalue()
    {
        // intValue/as_i64 work for integer JSON; doubleValue/as_f64 for floats.
        // intValue/as_i64 对整数 JSON 有效；doubleValue/as_f64 对浮点有效。
        let mut ctx = SpelContext::new();
        ctx.set_variable("i", serde_json::json!(7));
        assert!(
            SpelEvaluator::new("#i.intValue == 7")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(SpelEvaluator::new("#i.as_i64 == 7").evaluate(&ctx).unwrap());

        let mut ctx2 = SpelContext::new();
        ctx2.set_variable("d", serde_json::json!(2.5));
        assert!(
            SpelEvaluator::new("#d.doubleValue == 2.5")
                .evaluate(&ctx2)
                .unwrap()
        );
        assert!(
            SpelEvaluator::new("#d.as_f64 == 2.5")
                .evaluate(&ctx2)
                .unwrap()
        );
    }

    #[test]
    fn test_lexer_errors()
    {
        let ctx = SpelContext::new();
        // Unterminated string literal.
        // 未闭合的字符串字面量。
        assert!(SpelEvaluator::new("'unclosed").evaluate(&ctx).is_err());
        // '#' with no variable name following.
        // '#' 后无变量名。
        assert!(SpelEvaluator::new("#").evaluate(&ctx).is_err());
        // Unexpected character (e.g. '@').
        // 非法字符（如 '@'）。
        assert!(SpelEvaluator::new("@oops").evaluate(&ctx).is_err());
        // Trailing tokens after a complete expression.
        // 完整表达式后的多余 token。
        assert!(
            SpelEvaluator::new("permitAll permitAll")
                .evaluate(&ctx)
                .is_err()
        );
        // Unknown identifier.
        // 未知标识符。
        assert!(SpelEvaluator::new("frobnicate").evaluate(&ctx).is_err());
    }

    #[test]
    fn test_max_depth_guard()
    {
        // Nest deeper than MAX_DEPTH (64) via nested parentheses; must error.
        // 通过嵌套括号超过 MAX_DEPTH（64）；必须报错。
        let ctx = SpelContext::new();
        // Build 70 nested parens around "permitAll": (((... permitAll ...)))
        // 用 70 层嵌套括号包裹 "permitAll"：(((... permitAll ...)))
        let expr = "(".repeat(70) + "permitAll" + &")".repeat(70);
        assert!(SpelEvaluator::new(&expr).evaluate(&ctx).is_err());
    }

    #[test]
    fn test_context_bulk_mutators_and_default()
    {
        // add_roles / add_authorities bulk mutators.
        // add_roles / add_authorities 批量 mutator。
        let mut ctx = SpelContext::new();
        ctx.add_roles(["ADMIN", "SUPERUSER"]);
        ctx.add_authorities(["READ", "WRITE"]);
        assert!(
            SpelEvaluator::new("hasRole('ADMIN')")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            SpelEvaluator::new("hasRole('SUPERUSER')")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            SpelEvaluator::new("hasAuthority('READ')")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            SpelEvaluator::new("hasAnyRole('ADMIN', 'GUEST')")
                .evaluate(&ctx)
                .unwrap()
        );

        // set_principal auto-marks authenticated and binds the principal variable.
        // set_principal 自动标记已认证并绑定 principal 变量。
        ctx.set_principal("alice");
        assert!(
            SpelEvaluator::new("isAuthenticated()")
                .evaluate(&ctx)
                .unwrap()
        );
        assert!(
            SpelEvaluator::new("#principal == 'alice'")
                .evaluate(&ctx)
                .unwrap()
        );

        // Default::default() == new(): anonymous and empty.
        // Default::default() == new()：匿名且为空。
        let dctx = SpelContext::default();
        assert!(SpelEvaluator::new("isAnonymous()").evaluate(&dctx).unwrap());
    }
}
