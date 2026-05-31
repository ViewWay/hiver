//! Fork/Join parallel regions for state machines
//! 状态机的 Fork/Join 并行区域

use crate::error::{StateMachineError, StateMachineResult};
use std::collections::HashMap;

/// A parallel region within a state machine
/// 状态机中的并行区域
#[derive(Clone, Debug)]
pub struct Region<S> {
    /// Region identifier
    /// 区域标识符
    pub id: String,
    /// States belonging to this region
    /// 属于此区域的状态
    pub states: Vec<S>,
    /// Initial state of this region
    /// 此区域的初始状态
    pub initial_state: S,
    /// Current state of this region
    /// 此区域的当前状态
    pub current_state: S,
    /// Whether this region has completed (reached end state)
    /// 此区域是否已完成（到达终止状态）
    pub completed: bool,
}

impl<S: Clone + PartialEq + Eq> Region<S> {
    /// Create a new region
    /// 创建新区域
    pub fn new(id: impl Into<String>, initial_state: S) -> Self {
        Self {
            id: id.into(),
            states: Vec::new(),
            initial_state: initial_state.clone(),
            current_state: initial_state,
            completed: false,
        }
    }

    /// Add a state to this region
    /// 向此区域添加状态
    pub fn add_state(mut self, state: S) -> Self {
        self.states.push(state);
        self
    }

    /// Set current state
    /// 设置当前状态
    pub fn set_current(&mut self, state: S) {
        self.current_state = state;
    }

    /// Mark as completed
    /// 标记为已完成
    pub fn complete(&mut self) {
        self.completed = true;
    }

    /// Reset to initial state
    /// 重置到初始状态
    pub fn reset(&mut self) {
        self.current_state = self.initial_state.clone();
        self.completed = false;
    }

    /// Check if state belongs to this region
    /// 检查状态是否属于此区域
    pub fn contains_state(&self, state: &S) -> bool {
        self.states.iter().any(|s| s == state)
    }
}

/// Fork/Join parallel region manager
/// Fork/Join 并行区域管理器
pub struct ForkJoinRegion<S> {
    /// Regions keyed by ID
    /// 按 ID 索引的区域
    regions: HashMap<String, Region<S>>,
    /// Order of region IDs for deterministic iteration
    /// 区域 ID 的顺序，用于确定性迭代
    region_order: Vec<String>,
    /// End states that mark region completion
    /// 标记区域完成的终止状态
    end_states: Vec<S>,
}

impl<S: Clone + PartialEq + Eq + std::fmt::Debug> ForkJoinRegion<S> {
    /// Create a new fork/join region manager
    /// 创建新 Fork/Join 区域管理器
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
            region_order: Vec::new(),
            end_states: Vec::new(),
        }
    }

    /// Add a region
    /// 添加区域
    pub fn add_region(&mut self, region: Region<S>) {
        let id = region.id.clone();
        self.region_order.push(id.clone());
        self.regions.insert(id, region);
    }

    /// Set end states that trigger join
    /// 设置触发 join 的终止状态
    pub fn set_end_states(&mut self, states: Vec<S>) {
        self.end_states = states;
    }

    /// Fork: activate all regions
    /// Fork: 激活所有区域
    pub fn fork(&mut self) -> StateMachineResult<()> {
        if self.regions.is_empty() {
            return Err(StateMachineError::InvalidConfiguration(
                "No regions defined for fork".to_string(),
            ));
        }
        for region in self.regions.values_mut() {
            region.reset();
        }
        Ok(())
    }

    /// Join: check if all regions have completed
    /// Join: 检查是否所有区域都已完成
    pub fn is_joined(&self) -> bool {
        self.regions.values().all(|r| r.completed)
    }

    /// Update a region's current state
    /// 更新区域的当前状态
    pub fn update_region(&mut self, region_id: &str, new_state: S) -> StateMachineResult<()> {
        let region = self
            .regions
            .get_mut(region_id)
            .ok_or_else(|| StateMachineError::StateNotFound(format!("Region '{}' not found", region_id)))?;

        if !region.contains_state(&new_state) && new_state != region.initial_state {
            return Err(StateMachineError::InvalidConfiguration(format!(
                "State {:?} does not belong to region '{}'",
                new_state, region_id
            )));
        }

        region.set_current(new_state);

        // Check if this state is an end state
        if self.end_states.iter().any(|es| es == &region.current_state) {
            region.complete();
        }

        Ok(())
    }

    /// Get a region by ID
    /// 通过 ID 获取区域
    pub fn region(&self, id: &str) -> Option<&Region<S>> {
        self.regions.get(id)
    }

    /// Get all region IDs in order
    /// 按顺序获取所有区域 ID
    pub fn region_ids(&self) -> &[String] {
        &self.region_order
    }

    /// Reset all regions
    /// 重置所有区域
    pub fn reset(&mut self) {
        for region in self.regions.values_mut() {
            region.reset();
        }
    }
}

impl<S> Default for ForkJoinRegion<S>
where
    S: Clone + PartialEq + Eq + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestState {
        Forked,
        Region1A,
        Region1B,
        Region2A,
        Region2B,
        Joined,
    }

    impl State for TestState {}

    #[test]
    fn test_region_creation() {
        let region = Region::new("r1", TestState::Region1A)
            .add_state(TestState::Region1A)
            .add_state(TestState::Region1B);
        assert_eq!(region.id, "r1");
        assert_eq!(region.states.len(), 2);
        assert!(!region.completed);
    }

    #[test]
    fn test_fork_join_basic() {
        let mut fj = ForkJoinRegion::new();

        fj.add_region(
            Region::new("r1", TestState::Region1A)
                .add_state(TestState::Region1A)
                .add_state(TestState::Region1B),
        );
        fj.add_region(
            Region::new("r2", TestState::Region2A)
                .add_state(TestState::Region2A)
                .add_state(TestState::Region2B),
        );
        fj.set_end_states(vec![TestState::Region1B, TestState::Region2B]);

        fj.fork().unwrap();
        assert!(!fj.is_joined());

        fj.update_region("r1", TestState::Region1B).unwrap();
        assert!(!fj.is_joined());

        fj.update_region("r2", TestState::Region2B).unwrap();
        assert!(fj.is_joined());
    }

    #[test]
    fn test_fork_reset() {
        let mut fj = ForkJoinRegion::new();
        fj.add_region(Region::new("r1", TestState::Region1A));

        fj.fork().unwrap();
        fj.reset();
        assert!(!fj.is_joined());
    }

    #[test]
    fn test_unknown_region() {
        let fj: ForkJoinRegion<TestState> = ForkJoinRegion::new();
        assert!(fj.region("missing").is_none());
    }

    #[test]
    fn test_fork_no_regions() {
        let mut fj: ForkJoinRegion<TestState> = ForkJoinRegion::new();
        assert!(fj.fork().is_err());
    }
}
