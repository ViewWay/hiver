# hiver-validation-annotations

**Validation annotations for Hiver Framework**

**Hiver框架的验证注解**

## Overview / 概述

`hiver-validation-annotations` provides validation annotation definitions used by the `hiver-validation` crate for declarative constraint validation.

`hiver-validation-annotations` 提供验证注解定义，供 `hiver-validation` crate 使用，用于声明式约束验证。

## Features / 功能

- **@NotNull** - Non-null validation
- **@Size** - String/collection size constraints
- **@Email** - Email format validation
- **@Pattern** - Regex pattern matching
- **@Min/@Max** - Numeric range validation
- **@Past/@Future** - Date/time validation

- **@NotNull** - 非空验证
- **@Size** - 字符串/集合大小约束
- **@Email** - 邮箱格式验证
- **@Pattern** - 正则匹配
- **@Min/@Max** - 数值范围验证
- **@Past/@Future** - 日期时间验证

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Jakarta Bean Validation, Hibernate Validator | `hiver-validation-annotations` |

## Installation / 安装

```toml
[dependencies]
hiver-validation-annotations = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_validation_annotations::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Jakarta Bean Validation, Hibernate Validator

**Spring 等价物**: Jakarta Bean Validation, Hibernate Validator
