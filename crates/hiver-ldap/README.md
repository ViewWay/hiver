# hiver-ldap

**LDAP integration for Hiver Framework**

**Hiver框架的 LDAP 集成**

## Overview / 概述

`hiver-ldap` provides LDAP directory integration with authentication, user management, and query support.

`hiver-ldap` 提供 LDAP 目录集成，包括认证、用户管理和查询支持。

## Features / 功能

- **LDAP Authentication** - Bind authentication
- **LdapTemplate** - High-level LDAP operations
- **User Management** - CRUD operations on directory entries
- **Query Builder** - Type-safe LDAP queries
- Connection Pooling

- **LDAP 认证** - 绑定认证
- **LdapTemplate** - 高级 LDAP 操作
- **用户管理** - 目录条目的 CRUD 操作
- **查询构建器** - 类型安全的 LDAP 查询
- 连接池

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring LDAP | `hiver-ldap` |

## Installation / 安装

```toml
[dependencies]
hiver-ldap = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_ldap::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring LDAP

**Spring 等价物**: Spring LDAP
