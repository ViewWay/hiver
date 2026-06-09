//! State machine configuration
//! 状态机配置

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{StateMachineError, StateMachineResult};

/// Configuration for a state machine
/// 状态机配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateMachineConfig {
    /// Name of the state machine
    /// 状态机名称
    pub name: String,

    /// Initial state ID
    /// 初始状态 ID
    pub initial_state: String,

    /// States in this machine
    /// 状态机中的状态
    #[serde(default)]
    pub states: Vec<StateConfig>,

    /// Transitions between states
    /// 状态之间的转换
    #[serde(default)]
    pub transitions: Vec<TransitionConfig>,
}

impl StateMachineConfig {
    /// Create a new state machine configuration
    /// 创建新状态机配置
    pub fn new(name: impl Into<String>, initial_state: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            initial_state: initial_state.into(),
            states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    /// Add a state
    /// 添加状态
    pub fn add_state(mut self, state: StateConfig) -> Self {
        self.states.push(state);
        self
    }

    /// Add a transition
    /// 添加转换
    pub fn add_transition(mut self, transition: TransitionConfig) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Validate the configuration
    /// 验证配置
    pub fn validate(&self) -> StateMachineResult<()> {
        // Check initial state exists
        // 检查初始状态是否存在
        if !self.states.iter().any(|s| s.id == self.initial_state) {
            return Err(StateMachineError::InvalidConfiguration(format!(
                "Initial state '{}' not found in states",
                self.initial_state
            )));
        }

        // Validate all transitions reference valid states
        // 验证所有转换引用的状态都有效
        for transition in &self.transitions {
            if !self.states.iter().any(|s| s.id == transition.source) {
                return Err(StateMachineError::InvalidConfiguration(format!(
                    "Transition source state '{}' not found",
                    transition.source
                )));
            }
            if !self.states.iter().any(|s| s.id == transition.target) {
                return Err(StateMachineError::InvalidConfiguration(format!(
                    "Transition target state '{}' not found",
                    transition.target
                )));
            }
        }

        Ok(())
    }

    /// Convert to JSON
    /// 转换为 JSON
    pub fn to_json(&self) -> StateMachineResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            StateMachineError::InvalidConfiguration(format!("JSON serialization failed: {}", e))
        })
    }

    /// Parse from JSON
    /// 从 JSON 解析
    pub fn from_json(json: &str) -> StateMachineResult<Self> {
        serde_json::from_str(json).map_err(|e| {
            StateMachineError::InvalidConfiguration(format!("JSON deserialization failed: {}", e))
        })
    }
}

/// Configuration for a single state
/// 单个状态的配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateConfig {
    /// State ID
    /// 状态 ID
    pub id: String,

    /// Whether this is a final state
    /// 是否为终止状态
    #[serde(default)]
    pub final_state: bool,

    /// State entry actions
    /// 状态进入动作
    #[serde(default)]
    pub entry_actions: Vec<String>,

    /// State exit actions
    /// 状态退出动作
    #[serde(default)]
    pub exit_actions: Vec<String>,

    /// Extended state data
    /// 扩展状态数据
    #[serde(default)]
    pub extended_state: HashMap<String, StateDataValueConfig>,
}

impl StateConfig {
    /// Create a new state configuration
    /// 创建新状态配置
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            final_state: false,
            entry_actions: Vec::new(),
            exit_actions: Vec::new(),
            extended_state: HashMap::new(),
        }
    }

    /// Mark as final state
    /// 标记为终止状态
    pub fn final_state(mut self, final_state: bool) -> Self {
        self.final_state = final_state;
        self
    }

    /// Add an entry action
    /// 添加进入动作
    pub fn entry_action(mut self, action: impl Into<String>) -> Self {
        self.entry_actions.push(action.into());
        self
    }

    /// Add an exit action
    /// 添加退出动作
    pub fn exit_action(mut self, action: impl Into<String>) -> Self {
        self.exit_actions.push(action.into());
        self
    }

    /// Add extended state data
    /// 添加扩展状态数据
    pub fn extended_data(mut self, key: impl Into<String>, value: StateDataValueConfig) -> Self {
        self.extended_state.insert(key.into(), value);
        self
    }
}

/// Configuration for state data values
/// 状态数据值配置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StateDataValueConfig {
    /// String value
    /// 字符串值
    #[serde(rename = "string")]
    String { value: String },

    /// Integer value
    /// 整数值
    #[serde(rename = "integer")]
    Integer { value: i64 },

    /// Float value
    /// 浮点数值
    #[serde(rename = "float")]
    Float { value: f64 },

    /// Boolean value
    /// 布尔值
    #[serde(rename = "boolean")]
    Boolean { value: bool },
}

impl StateDataValueConfig {
    /// Create a string value
    /// 创建字符串值
    pub fn string(value: impl Into<String>) -> Self {
        StateDataValueConfig::String {
            value: value.into(),
        }
    }

    /// Create an integer value
    /// 创建整数值
    pub fn integer(value: i64) -> Self {
        StateDataValueConfig::Integer { value }
    }

    /// Create a float value
    /// 创建浮点数值
    pub fn float(value: f64) -> Self {
        StateDataValueConfig::Float { value }
    }

    /// Create a boolean value
    /// 创建布尔值
    pub fn boolean(value: bool) -> Self {
        StateDataValueConfig::Boolean { value }
    }
}

/// Configuration for a single transition
/// 单个转换的配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransitionConfig {
    /// Source state ID
    /// 源状态 ID
    pub source: String,

    /// Target state ID
    /// 目标状态 ID
    pub target: String,

    /// Event that triggers the transition
    /// 触发转换的事件
    pub event: Option<String>,

    /// Guard condition expression
    /// 守卫条件表达式
    #[serde(default)]
    pub guard: Option<String>,

    /// Actions to execute
    /// 执行的动作
    #[serde(default)]
    pub actions: Vec<String>,
}

impl TransitionConfig {
    /// Create a new transition configuration
    /// 创建新转换配置
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            event: None,
            guard: None,
            actions: Vec::new(),
        }
    }

    /// Set the event
    /// 设置事件
    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Set a guard
    /// 设置守卫
    pub fn guard(mut self, guard: impl Into<String>) -> Self {
        self.guard = Some(guard.into());
        self
    }

    /// Add an action
    /// 添加动作
    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.actions.push(action.into());
        self
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_config() {
        let config = StateMachineConfig::new("test-machine", "A")
            .add_state(StateConfig::new("A"))
            .add_state(StateConfig::new("B").final_state(true))
            .add_transition(TransitionConfig::new("A", "B").event("go"));

        assert_eq!(config.name, "test-machine");
        assert_eq!(config.initial_state, "A");
        assert_eq!(config.states.len(), 2);
        assert_eq!(config.transitions.len(), 1);
    }

    #[test]
    fn test_config_validation() {
        let config = StateMachineConfig::new("test-machine", "A")
            .add_state(StateConfig::new("A"))
            .add_state(StateConfig::new("B"))
            .add_transition(TransitionConfig::new("A", "B"));

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_missing_initial_state() {
        let config = StateMachineConfig::new("test-machine", "C")
            .add_state(StateConfig::new("A"))
            .add_state(StateConfig::new("B"));

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_to_json() {
        let config = StateMachineConfig::new("test", "A").add_state(StateConfig::new("A"));

        let json = config.to_json();
        assert!(json.is_ok());
    }

    #[test]
    fn test_config_from_json() {
        let json = r#"{
            "name": "test",
            "initial_state": "A",
            "states": [{"id": "A"}],
            "transitions": []
        }"#;

        let config = StateMachineConfig::from_json(json);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().name, "test");
    }
}
