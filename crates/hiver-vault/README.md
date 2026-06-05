# hiver-vault

**HashiCorp Vault integration for Hiver Framework**

**Hiver框架的 HashiCorp Vault 集成**

## Overview / 概述

`hiver-vault` provides HashiCorp Vault integration for secrets management, dynamic credentials, and encryption as a service.

`hiver-vault` 提供 HashiCorp Vault 集成，用于密钥管理、动态凭证和加密即服务。

## Features / 功能

- **Secret Engine** - Read/write secrets
- **Dynamic Credentials** - Database credential leasing
- **Transit Engine** - Encryption as a service
- **Token Management** - Token renewal and auth
- KV v1/v2 Support

- **Secret 引擎** - 读写密钥
- **动态凭证** - 数据库凭证租用
- **Transit 引擎** - 加密即服务
- **Token 管理** - Token 续期和认证
- KV v1/v2 支持

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring Vault | `hiver-vault` |

## Installation / 安装

```toml
[dependencies]
hiver-vault = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_vault::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring Vault

**Spring 等价物**: Spring Vault
