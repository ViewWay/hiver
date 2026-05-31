# hiver-tx

[![Crates.io](https://img.shields.io/crates/v/hiver-tx)](https://crates.io/crates/hiver-tx)
[![Documentation](https://docs.rs/hiver-tx/badge.svg)](https://docs.rs/hiver-tx)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Transaction management for Hiver Framework
> 
> Hiver框架的事务管理

---

## 📋 Overview / 概述

`hiver-tx` provides declarative transaction management for Hiver applications, similar to Spring's `@Transactional` annotation.

`hiver-tx` 为Hiver应用程序提供声明式事务管理，类似于Spring的`@Transactional`注解。

**Key Features** / **核心特性**:
- ✅ **@Transactional** - Declarative transactions
- ✅ **Transaction Manager** - Transaction lifecycle
- ✅ **Isolation Levels** - Transaction isolation
- ✅ **Propagation** - Transaction propagation
- ✅ **Rollback Rules** - Custom rollback behavior

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

```toml
[dependencies]
hiver-tx = "0.1.0-alpha"
hiver-macros = "0.1.0-alpha"
```

### Basic Usage / 基本用法

```rust
use hiver_tx::Transactional;
use hiver_macros::transactional;

struct UserService;

impl UserService {
    // Declarative transaction / 声明式事务
    #[transactional]
    async fn create_user(&self, user: User) -> Result<User, Error> {
        // All operations in transaction / 所有操作在事务中
        save_user(user.clone()).await?;
        create_profile(user.id).await?;
        Ok(user)
    }
    
    // With isolation level / 带隔离级别
    #[transactional(isolation = "SERIALIZABLE")]
    async fn transfer_money(&self, from: u64, to: u64, amount: f64) -> Result<(), Error> {
        debit_account(from, amount).await?;
        credit_account(to, amount).await?;
        Ok(())
    }
}
```

---

## 📖 Transaction Features / 事务功能

### Transaction Propagation / 事务传播

```rust
use hiver_tx::Propagation;

// REQUIRED (default) - Join existing or create new
#[transactional(propagation = "REQUIRED")]

// REQUIRES_NEW - Always create new transaction
#[transactional(propagation = "REQUIRES_NEW")]

// NESTED - Nested transaction
#[transactional(propagation = "NESTED")]

// SUPPORTS - Join if exists, otherwise no transaction
#[transactional(propagation = "SUPPORTS")]

// NOT_SUPPORTED - Suspend current transaction
#[transactional(propagation = "NOT_SUPPORTED")]

// NEVER - Fail if transaction exists
#[transactional(propagation = "NEVER")]

// MANDATORY - Fail if no transaction
#[transactional(propagation = "MANDATORY")]
```

### Isolation Levels / 隔离级别

```rust
use hiver_tx::IsolationLevel;

// READ_UNCOMMITTED - Lowest isolation
#[transactional(isolation = "READ_UNCOMMITTED")]

// READ_COMMITTED - Default for most databases
#[transactional(isolation = "READ_COMMITTED")]

// REPEATABLE_READ - Prevent non-repeatable reads
#[transactional(isolation = "REPEATABLE_READ")]

// SERIALIZABLE - Highest isolation
#[transactional(isolation = "SERIALIZABLE")]
```

### Rollback Rules / 回滚规则

```rust
// Rollback on specific exceptions / 特定异常时回滚
#[transactional(rollback_for = "ValidationError")]

// Don't rollback on specific exceptions / 特定异常时不回滚
#[transactional(no_rollback_for = "BusinessException")]

// Rollback on all exceptions (default) / 所有异常时回滚（默认）
#[transactional]
```

---

## 🚦 Roadmap / 路线图

### Phase 3: Core Transactions ✅ (Completed / 已完成)
- [x] @Transactional annotation
- [x] Transaction manager
- [x] Isolation levels
- [x] Propagation

### Phase 4: Advanced Features 🔄 (In Progress / 进行中)
- [ ] Distributed transactions
- [ ] Transaction synchronization
- [ ] Savepoints

---

## 📚 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-tx](https://docs.rs/hiver-tx)

---

**Built with ❤️ for transaction management**

**为事务管理构建 ❤️**
