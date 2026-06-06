//! State persistence for state machines
//! 状态机持久化

use std::collections::HashMap;

use crate::{error::StateMachineResult, state::StateData};

/// Snapshot of a state machine's current state
/// 状态机当前状态的快照
#[derive(Clone, Debug)]
pub struct StateMachineSnapshot<S>
{
    /// Current state
    /// 当前状态
    pub current_state: S,
    /// Extended state data
    /// 扩展状态数据
    pub extended_state: StateData,
    /// Machine ID
    /// 机器 ID
    pub machine_id: String,
}

/// Persistence backend for state machine snapshots
/// 状态机快照的持久化后端
pub trait StateMachinePersist<S>: Send + Sync
{
    /// Save a snapshot
    /// 保存快照
    fn save(&self, snapshot: &StateMachineSnapshot<S>) -> StateMachineResult<()>;

    /// Load a snapshot by machine ID
    /// 通过机器 ID 加载快照
    fn load(&self, machine_id: &str) -> StateMachineResult<Option<StateMachineSnapshot<S>>>;

    /// Delete a snapshot
    /// 删除快照
    fn delete(&self, machine_id: &str) -> StateMachineResult<()>;
}

/// In-memory state machine repository
/// 内存中的状态机仓库
pub struct InMemoryStateMachineRepository<S>
{
    snapshots: std::sync::RwLock<HashMap<String, StateMachineSnapshot<S>>>,
}

impl<S> InMemoryStateMachineRepository<S>
{
    /// Create a new in-memory repository
    /// 创建新的内存仓库
    pub fn new() -> Self
    {
        Self {
            snapshots: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl<S> Default for InMemoryStateMachineRepository<S>
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl<S: Clone + Send + Sync> StateMachinePersist<S> for InMemoryStateMachineRepository<S>
{
    fn save(&self, snapshot: &StateMachineSnapshot<S>) -> StateMachineResult<()>
    {
        self.snapshots
            .write()
            .unwrap()
            .insert(snapshot.machine_id.clone(), snapshot.clone());
        Ok(())
    }

    fn load(&self, machine_id: &str) -> StateMachineResult<Option<StateMachineSnapshot<S>>>
    {
        Ok(self.snapshots.read().unwrap().get(machine_id).cloned())
    }

    fn delete(&self, machine_id: &str) -> StateMachineResult<()>
    {
        self.snapshots.write().unwrap().remove(machine_id);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::state::State;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestState
    {
        A,
        B,
    }

    impl State for TestState {}

    #[test]
    fn test_save_and_load()
    {
        let repo = InMemoryStateMachineRepository::new();
        let snapshot = StateMachineSnapshot {
            current_state: TestState::A,
            extended_state: StateData::new(),
            machine_id: "test-machine".to_string(),
        };

        repo.save(&snapshot).unwrap();
        let loaded = repo.load("test-machine").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().current_state, TestState::A);
    }

    #[test]
    fn test_load_missing()
    {
        let repo: InMemoryStateMachineRepository<TestState> = InMemoryStateMachineRepository::new();
        let loaded = repo.load("missing").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_delete()
    {
        let repo = InMemoryStateMachineRepository::new();
        let snapshot = StateMachineSnapshot {
            current_state: TestState::A,
            extended_state: StateData::new(),
            machine_id: "test-machine".to_string(),
        };

        repo.save(&snapshot).unwrap();
        repo.delete("test-machine").unwrap();
        let loaded = repo.load("test-machine").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_overwrite()
    {
        let repo = InMemoryStateMachineRepository::new();
        let snapshot1 = StateMachineSnapshot {
            current_state: TestState::A,
            extended_state: StateData::new(),
            machine_id: "test-machine".to_string(),
        };
        let snapshot2 = StateMachineSnapshot {
            current_state: TestState::B,
            extended_state: StateData::new(),
            machine_id: "test-machine".to_string(),
        };

        repo.save(&snapshot1).unwrap();
        repo.save(&snapshot2).unwrap();
        let loaded = repo.load("test-machine").unwrap().unwrap();
        assert_eq!(loaded.current_state, TestState::B);
    }
}
