# hiver-shell-macros

**Procedural macros for hiver-shell**

**hiver-shell 的过程宏**

## Overview / 概述

`hiver-shell-macros` provides derive macros and attribute macros for building interactive shell commands in Hiver.

`hiver-shell-macros` 提供用于构建 Hiver 交互式 Shell 命令的 derive 宏和属性宏。

## Features / 功能

- **#[shell_command]** - Declare shell commands
- **Argument Parsing** - Automatic argument derivation
- **Help Generation** - Auto-generated help text
- **Completion Support** - Tab completion helpers

- **#[shell_command]** - 声明 Shell 命令
- **参数解析** - 自动参数推导
- **帮助生成** - 自动生成帮助文本
- **补全支持** - Tab 补全辅助

## Equivalent to Spring / 等价于 Spring

| Spring | Hiver |
|--------|-------|
| Spring Shell | `hiver-shell-macros` |

## Installation / 安装

```toml
[dependencies]
hiver-shell-macros = "0.1"
```

## Quick Start / 快速开始

```rust
use hiver_shell_macros::prelude::*;

// See examples directory for detailed usage
```

## License / 许可证

MIT OR Apache-2.0

---

Part of the [Hiver](https://github.com/ViewWay/hiver) framework.

**Spring Equivalence**: Spring Shell

**Spring 等价物**: Spring Shell
