# Hiver AOP

[![Crates.io](https://img.shields.io/crates/v/hiver-aop)](https://crates.io/hiver-aop)
[![Documentation](https://docs.rs/hiver-aop/badge.svg)](https://docs.rs/hiver-aop)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Spring AOP style annotations for the Hiver Framework
>
> Hiver 框架的 Spring AOP 风格注解

---

## 📋 Overview / 概述

`hiver-aop` brings Spring AOP / AspectJ-style aspect-oriented programming to
Rust, built on top of the Hiver Framework. It provides procedural macros to
*annotate* advice, and a small runtime to *weave* that advice around method
calls at run time.

`hiver-aop` 为 Rust 带来 Spring AOP / AspectJ 风格的面向切面编程，构建于 Hiver 框架之上。
它提供过程宏用于*标注*通知，并提供一个小型运行时在方法调用周围*织入*这些通知。

**Key Features / 核心特性**:

- ✅ **`#[aspect]`** (alias `#[Aspect]`) — Marks a struct as an aspect / 标记切面
- ✅ **`#[before]`** / **`#[Before]`** — Before advice / 前置通知
- ✅ **`#[after]`** / **`#[After]`** — After advice / 后置通知
- ✅ **`#[around]`** / **`#[Around]`** — Around advice / 环绕通知
- ✅ **`#[after_returning]`** / **`#[AfterReturning]`** — After-returning advice / 返回后通知
- ✅ **`#[after_throwing]`** / **`#[AfterThrowing]`** — After-throwing advice / 异常后通知
- ✅ **`#[pointcut]`** / **`#[Pointcut]`** — Pointcut definition / 切点定义
- ✅ Runtime types: `JoinPoint`, `ProceedingJoinPoint`, `AspectRegistry`, `InterceptChain`

Both the lowercase Rust names and the Spring-familiar uppercase names are
exported, so `use hiver_aop::{aspect, before}` and
`use hiver_aop::{Aspect, Before}` both work. The examples below use the
lowercase names (idiomatic Rust); substitute the uppercase aliases freely.

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

Add to `Cargo.toml`:

```toml
[dependencies]
hiver-aop = "0.1"
```

### Basic Usage / 基本用法

The macros are **attribute macros placed on functions**. They work both on
free functions and on methods inside an `impl` block:

这些宏是**放置在函数上的属性宏**，既可用于自由函数，也可用于 `impl` 块内的方法：

```rust
use hiver_aop::{aspect, before, after, around, JoinPoint, ProceedingJoinPoint};

#[aspect]
struct LoggingAspect;

impl LoggingAspect {
    #[before("execution(* *..*.*(..))")]
    fn log_before(&self, join_point: &JoinPoint) {
        println!("Entering: {}", join_point.method_name());
    }

    #[after("execution(* *..*.*(..))")]
    fn log_after(&self, join_point: &JoinPoint) {
        println!("Exiting: {}", join_point.method_name());
    }

    #[around("execution(* *..*.*(..))")]
    fn log_around(join_point: &mut ProceedingJoinPoint) {
        println!("Before: {}", join_point.method_name());
        join_point.proceed(); // let the target method run / 放行目标方法
        println!("After:  {}", join_point.method_name());
    }
}

// Free-standing advice is supported too.
// 也支持独立的通知函数。
#[before("execution(* *..*.*(..))")]
fn free_advice(join_point: &JoinPoint) {
    println!("Free advice: {}", join_point.method_name());
}
```

Each advice macro records its pointcut expression next to the function as a
companion constant, e.g. the `log_before` example above produces:

每个通知宏会在函数旁以伴生常量记录其切点表达式，例如上面的 `log_before` 会产生：

```rust,ignore
const _HIVER_BEFORE_LOG_BEFORE_META: (&'static str, &'static str) =
    ("execution(* *..*.*(..))", "before");
```

This is valid both at module scope and inside an `impl` block (a nested `impl`
would not be), so advice can be attached to methods on the aspect struct.

---

## 🔌 Compile-time annotations vs runtime weaving
## 编译期注解 vs 运行时织入

`hiver-aop` is split in two:

1. **`hiver-aop`** — the facade crate you depend on. It re-exports the macros
   and the runtime types.
2. **`hiver-aop-macros`** — the proc-macro crate (not depended on directly).

The macros *annotate* advice and *record* pointcut expressions; they do **not**
automatically rewrite your business methods (Rust has no class-loading weave
step the way the JVM does). The actual application of advice happens at run
time through the runtime API:

`hiver-aop` 分为两部分：

1. **`hiver-aop`** — 你依赖的门面 crate，重新导出宏与运行时类型。
2. **`hiver-aop-macros`** — 过程宏 crate（不直接依赖）。

宏负责*标注*通知并*记录*切点表达式；它们**不会**自动改写你的业务方法
（Rust 没有 JVM 那样的类加载织入步骤）。通知的实际应用通过运行时 API 完成：

```rust
use std::sync::Arc;
use hiver_aop::runtime::{InterceptChain, JoinPoint};

let mut chain = InterceptChain::new();
chain.before(|jp| println!("Before: {}", jp.method_name()));
chain.after( |jp| println!("After:  {}", jp.method_name()));

let jp = JoinPoint::new(
    Arc::new("target") as Arc<dyn std::any::Any + Send + Sync>,
    "save_user".to_string(),
    vec![],
    "save_user()".to_string(),
    "service.UserService".to_string(),
);

let result = chain.invoke(jp, || {
    println!("target running");
    Some(Arc::new(200_i32) as Arc<dyn std::any::Any + Send + Sync>)
});
assert_eq!(*result.value::<i32>().unwrap(), 200);
```

For registry-based matching (register many advice, match by pointcut), see
`AspectRegistry` in the [API docs](https://docs.rs/hiver-aop).

---

## 📖 Available Annotations / 可用注解

### `#[aspect]` — mark an aspect / 标记切面

```rust
use hiver_aop::aspect;

#[aspect]
struct LoggingAspect;
```

### `#[before("pointcut")]` — before advice / 前置通知

```rust
#[before("execution(* *..*.*(..))")]
fn log_before(join_point: &JoinPoint) {
    println!("Entering: {}", join_point.method_name());
}
```

### `#[after("pointcut")]` — after advice / 后置通知

Executes after the join point (normal or exceptional exit), like `finally`.
在连接点之后执行（正常或异常退出），类似 `finally`。

```rust
#[after("execution(* *..*.*(..))")]
fn log_after(join_point: &JoinPoint) {
    println!("Exiting: {}", join_point.method_name());
}
```

### `#[around("pointcut")]` — around advice / 环绕通知

Wraps the join point. Take a `&mut ProceedingJoinPoint` and call `proceed()` to
let the target run.
包装连接点。接收 `&mut ProceedingJoinPoint`，调用 `proceed()` 以放行目标方法。

```rust
#[around("execution(* *..*.*(..))")]
fn log_around(join_point: &mut ProceedingJoinPoint) {
    println!("Before: {}", join_point.method_name());
    join_point.proceed();
    println!("After:  {}", join_point.method_name());
}
```

### `#[pointcut("expression")]` — reusable pointcut / 可重用切点

```rust
#[pointcut("execution(* *.service.*.*(..))")]
fn service_layer() {}
```

---

## 📚 Pointcut Expressions in Rust / Rust 中的切点表达式

> **Important / 重要** — Hiver AOP pointcuts are **string patterns matched at
> run time** against a `JoinPoint`'s `method_name()` and `target_class()`.
> There are **no JVM packages**. Patterns like `com.example..*` work as generic
> dotted-name patterns, but they carry no Java package semantics. The idiomatic
> Rust analog is to match module paths / type names such as
> `crate::service::...`, `*.service.*`, etc.
>
> Hiver AOP 的切点是**运行时匹配 `JoinPoint` 的 `method_name()` 与 `target_class()`
> 的字符串模式**。**没有 JVM 包的概念**。`com.example..*` 之类的模式可作为通用的点号名称
> 模式工作，但并不具备 Java 包语义。Rust 惯用的等价物是匹配模块路径/类型名，
> 例如 `crate::service::...`、`*.service.*` 等。

### Execution pattern / 执行模式

```
execution(return-type-pattern declaring-type-pattern? method-name-pattern(param-pattern))
```

- `*` matches a single name segment / 匹配单个名称段
- `..` matches zero or more segments / 匹配零或多个段

**Examples / 示例**:

```text
"execution(* *..*.*(..))"              // any method / 任意方法
"execution(* *.Service.*(..))"         // methods on types whose name is `<X>.Service`
"execution(* *.UserService.save_user(..))" // a specific method on a specific type
"execution(* get*(..))"                // methods whose name matches `get*`
```

### Other designators / 其他指示符

```text
"within(*)"                            // within any type / 任意类型内
"within(service.UserService)"          // within a specific type / 特定类型内
"@annotation(my::Transactional)"       // (parsed; runtime annotation check is not supported)
```

### Combining pointcuts / 组合切点

```text
"execution(* *..*.*(..)) && within(*.Service)"   // AND — both must match / 都须匹配
"execution(* *.Service.*(..)) || execution(* *.Repository.*(..))"  // OR — either / 任一
```

See `PointcutExpression` in the [API docs](https://docs.rs/hiver-aop) for the
full matching semantics.

---

## 🧪 Testing / 测试

Run the unit and runtime tests:

```bash
cargo test --package hiver-aop
```

Downstream compile regression tests (trybuild) guard the documented API shape
against breaking again:

```bash
cargo test --package hiver-aop --test downstream_compile
```

Run the example:

```bash
cargo run --package hiver-aop --example logging_aspect
```

---

## 📖 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-aop](https://docs.rs/hiver-aop)
- **Spring AOP Reference**: [https://docs.spring.io/spring-framework/reference/core/aop.html](https://docs.spring.io/spring-framework/reference/core/aop.html)

---

## 🔄 Migration from Spring AOP / 从 Spring AOP 迁移

### Java / Spring AOP

```java
@Aspect @Component
public class LoggingAspect {
    @Before("execution(* com.example..*.*(..))")
    public void logBefore(JoinPoint joinPoint) {
        logger.info("Entering: {}", joinPoint.getSignature().toShortString());
    }
}
```

### Rust / Hiver AOP

```rust
use hiver_aop::{aspect, before, JoinPoint};

#[aspect]
struct LoggingAspect;

impl LoggingAspect {
    #[before("execution(* *..*.*(..))")]
    fn log_before(&self, join_point: &JoinPoint) {
        println!("Entering: {}", join_point.method_name());
    }
}
```

---

## 📝 License / 许可证

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option. / 由您选择。
