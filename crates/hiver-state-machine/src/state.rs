//! State traits and context
//! 状态特征和上下文

use std::{any::Any, fmt::Debug};

use crate::{Event, error::StateMachineResult};

/// State trait for state machine states
/// 状态机状态的特征
pub trait State: Any + Debug + Send + Sync
{
    /// Get state ID
    /// 获取状态 ID
    fn id(&self) -> String
    {
        format!("{:?}", self)
    }

    /// Called when entering this state
    /// 进入此状态时调用
    fn on_entry(&self) -> StateMachineResult<()>
    {
        Ok(())
    }

    /// Called when exiting this state
    /// 退出此状态时调用
    fn on_exit(&self) -> StateMachineResult<()>
    {
        Ok(())
    }
}

/// State context for guards and actions
/// 守卫和动作的状态上下文
pub struct StateContext<'a, S, E>
where
    S: State,
    E: Event,
{
    /// Source state
    /// 源状态
    source: &'a S,

    /// Event that triggered the transition
    /// 触发转换的事件
    event: &'a E,

    /// Target state (if transition is in progress)
    /// 目标状态（如果转换正在进行中）
    target: Option<&'a S>,
}

impl<'a, S, E> StateContext<'a, S, E>
where
    S: State,
    E: Event,
{
    /// Create a new state context
    /// 创建新状态上下文
    pub fn new(source: &'a S, event: &'a E, target: Option<&'a S>) -> Self
    {
        Self {
            source,
            event,
            target,
        }
    }

    /// Get source state
    /// 获取源状态
    pub fn source(&self) -> &S
    {
        self.source
    }

    /// Get event
    /// 获取事件
    pub fn event(&self) -> &E
    {
        self.event
    }

    /// Get target state
    /// 获取目标状态
    pub fn target(&self) -> Option<&S>
    {
        self.target
    }

    /// Check if transition is in progress
    /// 检查转换是否正在进行中
    pub fn is_transitioning(&self) -> bool
    {
        self.target.is_some()
    }
}

/// Extended state data for storing additional information
/// 扩展状态数据用于存储额外信息
#[derive(Clone, Debug)]
pub struct StateData
{
    /// Internal data storage
    /// 内部数据存储
    data: std::collections::HashMap<String, StateDataValue>,
}

impl StateData
{
    /// Create new state data
    /// 创建新状态数据
    pub fn new() -> Self
    {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    /// Insert a value
    /// 插入值
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<StateDataValue>)
    {
        self.data.insert(key.into(), value.into());
    }

    /// Get a value
    /// 获取值
    pub fn get(&self, key: &str) -> Option<&StateDataValue>
    {
        self.data.get(key)
    }

    /// Remove a value
    /// 移除值
    pub fn remove(&mut self, key: &str) -> Option<StateDataValue>
    {
        self.data.remove(key)
    }

    /// Check if key exists
    /// 检查键是否存在
    pub fn contains_key(&self, key: &str) -> bool
    {
        self.data.contains_key(key)
    }

    /// Get all keys
    /// 获取所有键
    pub fn keys(&self) -> impl Iterator<Item = &String>
    {
        self.data.keys()
    }

    /// Clear all data
    /// 清除所有数据
    pub fn clear(&mut self)
    {
        self.data.clear();
    }

    /// Get data length
    /// 获取数据长度
    pub fn len(&self) -> usize
    {
        self.data.len()
    }

    /// Check if empty
    /// 检查是否为空
    pub fn is_empty(&self) -> bool
    {
        self.data.is_empty()
    }
}

impl Default for StateData
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// State data value types
/// 状态数据值类型
#[derive(Clone, Debug)]
pub enum StateDataValue
{
    /// String value
    /// 字符串值
    String(String),

    /// Integer value
    /// 整数值
    Integer(i64),

    /// Float value
    /// 浮点数值
    Float(f64),

    /// Boolean value
    /// 布尔值
    Boolean(bool),

    /// Bytes value
    /// 字节值
    Bytes(Vec<u8>),
}

impl StateDataValue
{
    /// Get as string
    /// 获取字符串
    pub fn as_str(&self) -> Option<&str>
    {
        match self
        {
            StateDataValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as integer
    /// 获取整数
    pub fn as_integer(&self) -> Option<i64>
    {
        match self
        {
            StateDataValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as float
    /// 获取浮点数
    pub fn as_float(&self) -> Option<f64>
    {
        match self
        {
            StateDataValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Get as boolean
    /// 获取布尔值
    pub fn as_bool(&self) -> Option<bool>
    {
        match self
        {
            StateDataValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as bytes
    /// 获取字节
    pub fn as_bytes(&self) -> Option<&[u8]>
    {
        match self
        {
            StateDataValue::Bytes(b) => Some(b),
            _ => None,
        }
    }
}

impl From<String> for StateDataValue
{
    fn from(s: String) -> Self
    {
        StateDataValue::String(s)
    }
}

impl From<&str> for StateDataValue
{
    fn from(s: &str) -> Self
    {
        StateDataValue::String(s.to_string())
    }
}

impl From<i64> for StateDataValue
{
    fn from(i: i64) -> Self
    {
        StateDataValue::Integer(i)
    }
}

impl From<f64> for StateDataValue
{
    fn from(f: f64) -> Self
    {
        StateDataValue::Float(f)
    }
}

impl From<bool> for StateDataValue
{
    fn from(b: bool) -> Self
    {
        StateDataValue::Boolean(b)
    }
}

impl From<Vec<u8>> for StateDataValue
{
    fn from(v: Vec<u8>) -> Self
    {
        StateDataValue::Bytes(v)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestState;

    impl State for TestState {}

    #[test]
    fn test_state_id()
    {
        let state = TestState;
        assert!(!state.id().is_empty());
    }

    #[test]
    fn test_state_data()
    {
        let mut data = StateData::new();
        data.insert("key1", "value1");
        data.insert("key2", 42i64);
        data.insert("key3", true);

        assert_eq!(data.get("key1").and_then(|v| v.as_str()), Some("value1"));
        assert_eq!(data.get("key2").and_then(|v| v.as_integer()), Some(42));
        assert_eq!(data.get("key3").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(data.len(), 3);
        assert!(!data.is_empty());
    }

    #[test]
    fn test_state_data_clear()
    {
        let mut data = StateData::new();
        data.insert("key1", "value1");
        assert_eq!(data.len(), 1);

        data.clear();
        assert!(data.is_empty());
    }
}
