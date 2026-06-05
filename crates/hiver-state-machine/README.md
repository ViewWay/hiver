# hiver-state-machine

State Machine Framework for Hiver / Hiver 状态机框架

## 功能特性

- **State 和 Event traits**: 定义自定义状态和事件
- **Transition 转换系统**: 支持源状态、目标状态、触发事件
- **Guard 守卫条件**: 在转换前评估布尔条件
- **Action 动作**: 转换时执行的回调函数
- **StateMachine**: 核心状态机实现
- **流式构建器 API**: 方便的状态机构建
- **配置序列化**: 支持 JSON 格式的配置导入/导出
- **扩展状态**: StateData 用于存储额外的键值数据

## Spring State Machine 对应

| Hiver | Spring State Machine |
|-------|---------------------|
| `State` trait | `State` interface |
| `Event` trait | `Event` interface |
| `Transition` | `Transition` |
| `Guard` | `Guard` |
| `Action` | `Action` |
| `StateMachineBuilder` | `StateMachineBuilder` |
| `StateMachineConfig` | `StateMachineFactory` |

## 快速开始

```rust
use hiver_state_machine::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum DoorState {
    Locked,
    Unlocked,
}

impl State for DoorState {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DoorEvent {
    Coin,
    Push,
}

impl Event for DoorEvent {}

// 使用构建器创建状态机
let mut machine = StateMachineBuilder::new()
    .initial_state(DoorState::Locked)
    .transition()
        .source(DoorState::Locked)
        .target(DoorState::Unlocked)
        .event(DoorEvent::Coin)
        .guard(|ctx| {
            // 守卫条件：检查是否允许转换
            Ok(true)
        })
        .action(|ctx| {
            // 转换时执行的动作
            println!("Unlocked!");
            Ok(())
        })
        .and()
    .transition()
        .source(DoorState::Unlocked)
        .target(DoorState::Locked)
        .event(DoorEvent::Push)
        .and()
    .build()
    .unwrap();

// 触发事件
machine.fire(DoorEvent::Coin)?;
assert_eq!(machine.state(), &DoorState::Unlocked);

machine.fire(DoorEvent::Push)?;
assert_eq!(machine.state(), &DoorState::Locked);
```

## 核心组件

### State 状态特征

```rust
pub trait State: Any + Debug + Send + Sync {
    fn id(&self) -> String;
    fn on_entry(&self) -> StateMachineResult<()>;
    fn on_exit(&self) -> StateMachineResult<()>;
}
```

### Event 事件特征

```rust
pub trait Event: Any + Debug + Send + Sync + PartialEq {
    fn id(&self) -> String;
}
```

### StateMachine 状态机

```rust
let mut machine = StateMachine::new(MyState::Initial);

// 添加转换
machine.add_transition(
    Transition::builder()
        .source(MyState::Initial)
        .target(MyState::Final)
        .event(MyEvent::Complete)
        .build()?
);

// 触发事件
machine.fire(MyEvent::Complete)?;

// 检查是否可以触发
if machine.can_fire(&MyEvent::Complete) {
    // ...
}

// 重置到初始状态
machine.reset();
```

### StateContext 状态上下文

```rust
// 守卫中使用
.guard(|ctx: &StateContext<MyState, MyEvent>| {
    // 访问源状态、事件、目标状态
    let source = ctx.source();
    let event = ctx.event();
    let target = ctx.target();
    Ok(true)
})
```

## 配置序列化

```rust
// 从 JSON 加载配置
let config_json = r#"{
    "name": "door-machine",
    "initial_state": "Locked",
    "states": [
        {"id": "Locked", "final_state": false},
        {"id": "Unlocked", "final_state": false}
    ],
    "transitions": [
        {
            "source": "Locked",
            "target": "Unlocked",
            "event": "Coin"
        }
    ]
}"#;

let config: StateMachineConfig = serde_json::from_str(config_json)?;
config.validate()?;
```

## 测试

```bash
cargo test -p hiver-state-machine
```

测试覆盖: 24 个测试全部通过

## 依赖

- `hiver-core`: 核心类型
- `serde`: 序列化支持
- `thiserror`: 错误处理
- `async-trait`: 异步特征


## Installation / 安装

```toml
[dependencies]
hiver-state-machine = "0.1"
```

## 许可证

MIT OR Apache-2.0
