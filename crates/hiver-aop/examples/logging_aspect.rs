//! hiver AOP Macro Example / hiver AOP 宏示例
//!
//! This example demonstrates the AOP (Aspect-Oriented Programming) attribute
//! macros compiling end-to-end — both as free functions and as methods on an
//! aspect struct — and shows how the runtime registry wires advice to a join
//! point.
//!
//! 此示例演示 AOP（面向切面编程）属性宏的端到端编译——既可作为自由函数，
//! 也可作为切面结构体的方法——并展示运行时注册表如何将通知连接到连接点。
//!
//! Run / 运行:
//! ```bash
//! cargo run -p hiver-aop --example logging_aspect
//! ```

// Advice methods in this example are demonstrative (their shape matters, not
// their calls), so silence the expected dead-code warnings.
// 本示例中的通知方法仅作演示（重要的是其形式而非调用），
// 因此屏蔽预期的死代码告警。
#![allow(dead_code)]

use std::sync::Arc;

use hiver_aop::{
    AdviceType, After, Around, Aspect, Before, InterceptChain, JoinPoint, Pointcut,
    PointcutExpression, ProceedingJoinPoint,
};

// ============================================================================
// Compile-time: aspect + pointcut + advice definitions
// 编译期：切面 + 切点 + 通知定义
// ============================================================================

/// Logging aspect — applies to service-layer methods.
/// 日志切面 — 应用于服务层方法。
#[Aspect]
struct LoggingAspect;

impl LoggingAspect
{
    /// Companion metadata recorded by the `before` macro is reachable as an
    /// associated constant.
    /// 由 `before` 宏记录的伴生元数据可作为关联常量被访问。
    const META: (&'static str, &'static str) = Self::_HIVER_BEFORE_LOG_BEFORE_META;

    /// Run before matched methods.
    /// 在匹配的方法之前执行。
    #[Before("execution(* *..*.*(..))")]
    fn log_before(&self, join_point: &JoinPoint)
    {
        println!("[before] entering: {}", join_point.method_name());
    }

    /// Run after matched methods (always, like `finally`).
    /// 在匹配的方法之后执行（始终执行，类似 `finally`）。
    #[After("execution(* *..*.*(..))")]
    fn log_after(&self, join_point: &JoinPoint)
    {
        println!("[after]  exiting: {}", join_point.method_name());
    }

    /// Wrap matched methods; call `proceed()` to let the target run.
    /// 包装匹配的方法；调用 `proceed()` 以放行目标方法。
    #[Around("execution(* *..*.*(..))")]
    fn log_around(join_point: &mut ProceedingJoinPoint)
    {
        println!("[around] before: {}", join_point.method_name());
        join_point.proceed(); // let the target run / 放行目标方法
        println!("[around] after:  {}", join_point.method_name());
    }

    /// A reusable pointcut expression.
    /// 可重用的切点表达式。
    #[Pointcut("execution(* *..*.*(..))")]
    fn service_layer() {}
}

/// A free-standing advice function (module scope) — also supported.
/// 模块作用域的自由通知函数 — 同样受支持。
#[Before("execution(* *..*.*(..))")]
fn free_advice(join_point: &JoinPoint)
{
    println!("[free]   {}", join_point.method_name());
}

// ============================================================================
// Runtime: build a join point and run an InterceptChain around a target
// 运行时：构造连接点，并在目标周围运行拦截链
// ============================================================================

fn main()
{
    println!("=== hiver AOP Macro Demo ===\n");

    // 1. The aspect compiles and is constructible.
    // 切面编译通过且可构造。
    let _ = LoggingAspect;
    println!("LoggingAspect created; recorded metadata = {:?}", LoggingAspect::META);

    // 2. Pointcut matching works against a join point's method/class names.
    // 切点匹配针对连接点的方法名/类名工作。
    let wildcard = PointcutExpression::new("execution(* *..*.*(..))".to_string());
    let jp = JoinPoint::new(
        Arc::new("user_service") as Arc<dyn std::any::Any + Send + Sync>,
        "save_user".to_string(),
        vec![Arc::new(42_i64) as Arc<dyn std::any::Any + Send + Sync>],
        "save_user(i64)".to_string(),
        "service.UserService".to_string(),
    );
    println!(
        "pointcut \"execution(* *..*.*(..))\" matches save_user? {}",
        wildcard.matches(&jp)
    );

    // 3. Build a runtime InterceptChain (the actual weaving mechanism) and invoke a target through
    //    it.
    // 构建运行时拦截链（真正的织入机制）并通过它调用目标。
    let mut chain = InterceptChain::new();
    chain.before(|join_point| println!("[chain before] {}", join_point.method_name()));
    chain.around(|pjp| {
        println!("[chain around->proceed]");
        pjp.proceed();
    });
    chain.after(|join_point| println!("[chain after]  {}", join_point.method_name()));

    println!("\n--- invoking target through InterceptChain ---");
    let result = chain.invoke(jp, || {
        println!("[target] save_user running");
        Some(Arc::new(200_i32) as Arc<dyn std::any::Any + Send + Sync>)
    });
    println!("target returned: {:?}", result.value::<i32>());

    // 4. The registry exposes `AdviceType` and registers callable advice too.
    // 注册表还暴露 `AdviceType` 并可注册可调用通知。
    println!("\nAdviceType::Before == {:?}", AdviceType::Before);
    println!("\n=== Demo Complete ===");

    // Silence unused warnings for the compile-time-only advice items.
    // 抑制仅编译期使用的通知项的未使用告警。
    let _ = free_advice;
    let _ = |_jp: &JoinPoint| {};
    let _ = LoggingAspect::service_layer;
}
