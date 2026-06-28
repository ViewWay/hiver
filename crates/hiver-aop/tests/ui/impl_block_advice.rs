//! Regression: advice macros on methods inside an `impl` block.
//! 回归：`impl` 块内方法上的通知宏。
//!
//! This is the primary failing case from issue #60. Before the fix it failed
//! with `error: implementation is not supported in traits or impls` because the
//! macro generated a nested `impl` while already inside one.
//! 这是 issue #60 的主要失败用例。修复前报
//! `error: implementation is not supported in traits or impls`，
//! 因为宏在已位于 `impl` 内时又生成了嵌套 `impl`。

use hiver_aop::{after, around, aspect, before, pointcut, JoinPoint};

#[aspect]
struct LoggingAspect;

impl LoggingAspect
{
    #[before("execution(* *..*.*(..))")]
    fn log_before(&self, join_point: &JoinPoint)
    {
        let _ = join_point.method_name();
    }

    #[after("execution(* *..*.*(..))")]
    fn log_after(&self, join_point: &JoinPoint)
    {
        let _ = join_point.method_name();
    }

    #[around("execution(* *..*.*(..))")]
    fn log_around(join_point: &mut hiver_aop::ProceedingJoinPoint)
    {
        join_point.proceed();
    }

    // A reusable pointcut defined on the aspect.
    // 在切面上定义可重用切点。
    #[pointcut("execution(* *..*.*(..))")]
    fn service_layer()
    {}

    // The companion consts are reachable as associated consts of the aspect.
    // 伴生 const 可作为切面的关联常量被访问。
    const _PROBE: (&str, &str) = Self::_HIVER_BEFORE_LOG_BEFORE_META;
}

fn main()
{
    let aspect = LoggingAspect;
    let jp = JoinPoint::new(
        std::sync::Arc::new("target") as std::sync::Arc<dyn std::any::Any + Send + Sync>,
        "do_work".to_string(),
        vec![],
        "do_work()".to_string(),
        "Worker".to_string(),
    );
    aspect.log_before(&jp);
}
