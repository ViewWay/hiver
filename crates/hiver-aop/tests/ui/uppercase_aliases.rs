//! Regression: Spring-familiar UPPERCASE macro aliases resolve.
//! 回归：Spring 风格的大写宏别名可正确解析。
//!
//! The README documents `#[Aspect]`, `#[Before]`, `#[After]`, `#[Around]`, and
//! `#[Pointcut]`. These uppercase aliases must resolve so the documented import
//! shape compiles from downstream crates (issue #60 reproduction 1).
//! README 中使用了 `#[Aspect]`、`#[Before]`、`#[After]`、`#[Around]`、`#[Pointcut]`。
//! 这些大写别名必须可解析，使文档中的导入形式能从下游 crate 编译（issue #60 复现 1）。

use hiver_aop::{After, Around, Aspect, Before, JoinPoint, Pointcut};

#[Aspect]
struct LoggingAspect;

impl LoggingAspect
{
    #[Before("execution(* *..*.*(..))")]
    fn log_before(&self, join_point: &JoinPoint)
    {
        let _ = join_point.method_name();
    }

    #[After("execution(* *..*.*(..))")]
    fn log_after(&self, join_point: &JoinPoint)
    {
        let _ = join_point.method_name();
    }

    #[Around("execution(* *..*.*(..))")]
    fn log_around(&self, _join_point: &JoinPoint)
    {
        // Real around advice takes a &mut ProceedingJoinPoint; this fixture only
        // checks the uppercase alias resolves on a method.
        // 真正的 around 通知接收 &mut ProceedingJoinPoint；此夹具仅验证大写别名在方法上可解析。
    }

    #[Pointcut("execution(* *..*.*(..))")]
    fn service_layer()
    {}
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
