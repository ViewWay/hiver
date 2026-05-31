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

mod parser;
mod evaluator;
mod context;

pub use context::SpelContext;
pub use evaluator::SpelEvaluator;
pub use parser::{SpelExpr, SpelError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_role() {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        assert!(SpelEvaluator::new("hasRole('ADMIN')").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("hasRole('USER')").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_has_authority() {
        let mut ctx = SpelContext::new();
        ctx.add_authority("WRITE");
        assert!(SpelEvaluator::new("hasAuthority('WRITE')").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("hasAuthority('READ')").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_has_any_role() {
        let mut ctx = SpelContext::new();
        ctx.add_role("EDITOR");
        assert!(SpelEvaluator::new("hasAnyRole('ADMIN', 'EDITOR')").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("hasAnyRole('ADMIN', 'SUPERUSER')").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_permit_all_deny_all() {
        let ctx = SpelContext::new();
        assert!(SpelEvaluator::new("permitAll").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("denyAll").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_logical_and_or() {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        ctx.add_authority("WRITE");
        assert!(SpelEvaluator::new("hasRole('ADMIN') and hasAuthority('WRITE')")
            .evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("hasRole('USER') or hasRole('ADMIN')")
            .evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("hasRole('ADMIN') and hasAuthority('READ')")
            .evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_logical_not() {
        let ctx = SpelContext::new();
        assert!(SpelEvaluator::new("not denyAll").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("!denyAll").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_number_comparison() {
        let mut ctx = SpelContext::new();
        ctx.set_variable("age", serde_json::json!(42));
        assert!(SpelEvaluator::new("#age == 42").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#age > 40").evaluate(&ctx).unwrap());
        assert!(SpelEvaluator::new("#age < 50").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_variable_string_eq() {
        let mut ctx = SpelContext::new();
        ctx.set_variable("name", serde_json::json!("alice"));
        assert!(SpelEvaluator::new("#name == 'alice'").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("#name == 'bob'").evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_complex_expression() {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        ctx.set_variable("userId", serde_json::json!(1));
        assert!(SpelEvaluator::new("hasRole('ADMIN') or #userId == 1")
            .evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_parenthesized() {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        assert!(SpelEvaluator::new("(hasRole('ADMIN') or hasRole('USER')) and true")
            .evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_parse_error() {
        let ctx = SpelContext::new();
        assert!(SpelEvaluator::new("hasRole(").evaluate(&ctx).is_err());
    }
}
