# Annotations Reference / 注解参考

> **Status**: ✅ Available
> **状态**: 可用

---

## Overview / 概述

Hiver provides 40+ Spring-like annotations via procedural macros for declarative programming.

Hiver 通过过程宏提供 40+ 类 Spring 注解，支持声明式编程。

---

## Core Annotations / 核心注解

| Annotation | Description | Spring Equivalent |
|------------|-------------|-------------------|
| `#[handler]` | HTTP handler method | `@RequestMapping` |
| `#[get("/path")]` | GET endpoint | `@GetMapping` |
| `#[post("/path")]` | POST endpoint | `@PostMapping` |
| `#[middleware]` | Middleware component | `@Component` + Filter |
| `#[inject]` | Dependency injection | `@Autowired` |

## Data Annotations / 数据注解

| Annotation | Description | Spring Equivalent |
|------------|-------------|-------------------|
| `#[derive(Model)]` | ORM model derive | `@Entity` |
| `#[derive(Repository)]` | Repository derive | `@Repository` |
| `#[derive(PropertiesConfig)]` | Config binding | `@ConfigurationProperties` |

---

> 📝 Full annotation reference is being compiled.

---

*← [Previous](../ai/ollama.md) | [Next](./configuration.md) →*
