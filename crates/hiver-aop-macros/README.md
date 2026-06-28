# hiver-aop-macros

[![Crates.io](https://img.shields.io/crates/v/hiver-aop-macros)](https://crates.io/hiver-aop-macros)
[![Documentation](https://docs.rs/hiver-aop-macros/badge.svg)](https://docs.rs/hiver-aop-macros)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Procedural macros for `hiver-aop` — Spring AOP style aspects for the Hiver Framework.
>
> `hiver-aop` 的过程宏 — Hiver 框架的 Spring AOP 风格切面支持。

---

## Overview / 概述

This is the **proc-macro** half of [`hiver-aop`](https://crates.io/hiver-aop). You
normally do **not** depend on it directly — depend on `hiver-aop`, which
re-exports everything.

这是 [`hiver-aop`](https://crates.io/hiver-aop) 的**过程宏**部分。
通常**不需要**直接依赖本 crate — 请依赖 `hiver-aop`，它会重新导出全部内容。

It provides these attribute macros / 它提供以下属性宏:

| Lowercase / 小写 | Uppercase alias / 大写别名 | Purpose / 用途 |
|------------------|----------------------------|----------------|
| `aspect`         | `Aspect`                   | Mark a struct as an aspect / 标记切面 |
| `before`         | `Before`                   | Before advice / 前置通知 |
| `after`          | `After`                    | After advice / 后置通知 |
| `around`         | `Around`                   | Around advice / 环绕通知 |
| `after_returning`| `AfterReturning`           | After-returning advice / 返回后通知 |
| `after_throwing` | `AfterThrowing`            | After-throwing advice / 异常后通知 |
| `pointcut`       | `Pointcut`                 | Reusable pointcut definition / 可重用切点 |

## How it works / 工作原理

Each advice/pointcut macro emits the annotated function unchanged plus a
uniquely-named companion `const` that records the pointcut expression and advice
kind. A companion `const` (not a nested `impl`) is used so the macros are valid
both at module scope **and** inside an `impl` block.

每个通知/切点宏会原样输出被注解的函数，并附加一个唯一命名的伴生 `const`，
记录切点表达式与通知类型。使用伴生 `const`（而非嵌套 `impl`）是为了让宏在
模块作用域**以及** `impl` 块内部都合法。

## Usage / 用法

```toml
[dependencies]
hiver-aop = "0.1"
```

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

See the [`hiver-aop` documentation](https://docs.rs/hiver-aop) for the full API,
runtime types, and pointcut semantics.

## License / 许可证

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option. / 由您选择。
