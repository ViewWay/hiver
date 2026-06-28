//! Regression: advice macros on free functions (module scope).
//! 回归：自由函数（模块作用域）上的通知宏。
//!
//! Before the fix this failed with
//! `error[E0573]: expected type, found function <name>` because the macro
//! generated `impl #func_name { ... }` where `#func_name` is a function.
//! 修复前此用例失败，报
//! `error[E0573]: expected type, found function <name>`，
//! 因为宏生成了 `impl #func_name { ... }`，而 `#func_name` 是函数而非类型。

use hiver_aop::{after, after_returning, after_throwing, around, before, pointcut, JoinPoint};

#[before("execution(* *..*.*(..))")]
fn free_before(join_point: &JoinPoint)
{
    let _ = join_point.method_name();
}

#[after("execution(* *..*.*(..))")]
fn free_after(join_point: &JoinPoint)
{
    let _ = join_point.method_name();
}

#[around("execution(* *..*.*(..))")]
fn free_around(join_point: &mut hiver_aop::ProceedingJoinPoint)
{
    join_point.proceed();
}

#[after_returning("execution(* *..*.*(..))")]
fn free_after_returning(join_point: &JoinPoint)
{
    let _ = join_point.method_name();
}

#[after_throwing("execution(* *..*.*(..))")]
fn free_after_throwing(join_point: &JoinPoint)
{
    let _ = join_point.method_name();
}

#[pointcut("execution(* *..*.*(..))")]
fn free_pointcut()
{}

// The companion consts exist at module scope and are reachable.
// 伴生 const 存在于模块作用域，且可访问。
const _: (&str, &str) = _HIVER_BEFORE_FREE_BEFORE_META;
const _: (&str, &str) = _HIVER_AFTER_FREE_AFTER_META;
const _: (&str, &str) = _HIVER_AROUND_FREE_AROUND_META;
const _: (&str, &str) = _HIVER_AFTER_RETURNING_FREE_AFTER_RETURNING_META;
const _: (&str, &str) = _HIVER_AFTER_THROWING_FREE_AFTER_THROWING_META;
const _: &str = _HIVER_POINTCUT_FREE_POINTCUT_EXPR;

fn main()
{
    let jp = JoinPoint::new(
        std::sync::Arc::new("target") as std::sync::Arc<dyn std::any::Any + Send + Sync>,
        "do_work".to_string(),
        vec![],
        "do_work()".to_string(),
        "Worker".to_string(),
    );
    free_before(&jp);
    free_after(&jp);
}
