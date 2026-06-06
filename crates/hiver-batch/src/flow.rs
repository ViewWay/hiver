//! Conditional job flow and step transitions.
//! 条件作业流程和步骤转换。
//!
//! Equivalent to Spring Batch's `JobFlowBuilder` and `Flow` interfaces.
//! 等价于 Spring Batch 的 JobFlowBuilder 和 Flow 接口。

use crate::{
    error::{BatchError, BatchResult},
    execution::ExitStatus,
};

/// Decision outcome for step transitions.
/// 步骤转换的决策结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowDecision
{
    /// Continue to the named step.
    Next(String),
    /// Transition to a different branch.
    Branch(String),
    /// Stop the job.
    Stop,
    /// End the job successfully.
    End,
    /// Fail the job.
    Fail,
}

/// A condition that determines step transition.
/// 决定步骤转换的条件。
pub trait FlowCondition: Send + Sync
{
    /// Evaluate the condition against the current exit status.
    fn evaluate(&self, exit_status: &ExitStatus) -> FlowDecision;
}

/// A closure-based flow condition.
/// 基于闭包的流程条件。
pub struct FnFlowCondition<F: Fn(&ExitStatus) -> FlowDecision + Send + Sync>(pub F);

impl<F: Fn(&ExitStatus) -> FlowDecision + Send + Sync> FlowCondition for FnFlowCondition<F>
{
    fn evaluate(&self, exit_status: &ExitStatus) -> FlowDecision
    {
        (self.0)(exit_status)
    }
}

/// Transitions to one step on COMPLETED, another on FAILED.
/// COMPLETED 时转到下一步，FAILED 时转到另一步。
pub struct OnCompletedOrFailed
{
    /// Step on success.
    pub on_completed: String,
    /// Step on failure.
    pub on_failed: String,
}

impl OnCompletedOrFailed
{
    /// Create a new conditional transition.
    pub fn new(on_completed: impl Into<String>, on_failed: impl Into<String>) -> Self
    {
        Self {
            on_completed: on_completed.into(),
            on_failed: on_failed.into(),
        }
    }
}

impl FlowCondition for OnCompletedOrFailed
{
    fn evaluate(&self, exit_status: &ExitStatus) -> FlowDecision
    {
        if exit_status.code == "COMPLETED"
        {
            FlowDecision::Next(self.on_completed.clone())
        }
        else
        {
            FlowDecision::Next(self.on_failed.clone())
        }
    }
}

/// A step in a flow graph with associated transition rules.
/// 流程图中的一个步骤及其关联的转换规则。
#[derive(Debug, Clone)]
pub struct FlowStep
{
    /// Step name.
    pub name: String,
    /// Transitions: (exit_code_pattern, target_step_name).
    pub transitions: Vec<(String, String)>,
    /// Default next step if no transition matches.
    pub default_next: Option<String>,
}

impl FlowStep
{
    /// Create a new flow step.
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            transitions: Vec::new(),
            default_next: None,
        }
    }

    /// Add a transition on a specific exit code.
    pub fn on(mut self, exit_code: &str, next_step: impl Into<String>) -> Self
    {
        self.transitions
            .push((exit_code.to_string(), next_step.into()));
        self
    }

    /// Set the default next step.
    pub fn default_next(mut self, step: impl Into<String>) -> Self
    {
        self.default_next = Some(step.into());
        self
    }

    /// Resolve the next step given an exit status.
    pub fn resolve_next(&self, exit_status: &ExitStatus) -> FlowDecision
    {
        for (pattern, target) in &self.transitions
        {
            if pattern == "*" || pattern == &exit_status.code
            {
                return FlowDecision::Next(target.clone());
            }
        }
        if let Some(ref next) = self.default_next
        {
            FlowDecision::Next(next.clone())
        }
        else
        {
            FlowDecision::End
        }
    }
}

/// A complete job flow definition.
/// 完整的作业流程定义。
#[derive(Debug, Clone, Default)]
pub struct JobFlow
{
    /// Ordered flow steps.
    pub steps: Vec<FlowStep>,
    /// Name of the first step.
    pub start: Option<String>,
}

impl JobFlow
{
    /// Create a new empty flow.
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set the starting step.
    pub fn start(mut self, step_name: impl Into<String>) -> Self
    {
        self.start = Some(step_name.into());
        self
    }

    /// Add a flow step.
    pub fn add_step(mut self, step: FlowStep) -> Self
    {
        self.steps.push(step);
        self
    }

    /// Resolve the first step name.
    pub fn first_step(&self) -> BatchResult<&str>
    {
        if let Some(ref name) = self.start
        {
            Ok(name.as_str())
        }
        else if let Some(first) = self.steps.first()
        {
            Ok(first.name.as_str())
        }
        else
        {
            Err(BatchError::Other("Flow has no steps".into()))
        }
    }

    /// Find a flow step by name.
    pub fn find_step(&self, name: &str) -> Option<&FlowStep>
    {
        self.steps.iter().find(|s| s.name == name)
    }

    /// Execute the flow: walk through steps using transition rules.
    pub fn execute(&self, executor: &mut dyn StepExecutor) -> BatchResult<ExitStatus>
    {
        let mut current = self.first_step()?.to_string();

        loop
        {
            let step_result = executor.execute_step(&current)?;

            let flow_step = match self.find_step(&current)
            {
                Some(s) => s,
                None => return Ok(step_result),
            };

            match flow_step.resolve_next(&step_result)
            {
                FlowDecision::Next(next) =>
                {
                    current = next;
                },
                FlowDecision::End => return Ok(ExitStatus::completed()),
                FlowDecision::Stop => return Ok(ExitStatus::stopped()),
                FlowDecision::Fail => return Ok(ExitStatus::failed()),
                FlowDecision::Branch(name) =>
                {
                    current = name;
                },
            }
        }
    }
}

/// Builder for `JobFlow`.
/// JobFlow 的构建器。
#[derive(Default)]
pub struct FlowBuilder
{
    flow: JobFlow,
}

impl FlowBuilder
{
    /// Create a new flow builder.
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set the starting step.
    pub fn start(mut self, step_name: impl Into<String>) -> Self
    {
        self.flow.start = Some(step_name.into());
        self
    }

    /// Add a flow step.
    pub fn add_step(mut self, step: FlowStep) -> Self
    {
        self.flow.steps.push(step);
        self
    }

    /// Build the flow.
    pub fn build(self) -> JobFlow
    {
        self.flow
    }
}

/// Trait for executing steps in a flow.
/// 在流程中执行步骤的 trait。
pub trait StepExecutor
{
    /// Execute a step by name and return its exit status.
    fn execute_step(&mut self, step_name: &str) -> BatchResult<ExitStatus>;
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_flow_step_resolve()
    {
        let step = FlowStep::new("step1")
            .on("COMPLETED", "step2")
            .on("FAILED", "error-handler");

        assert_eq!(step.resolve_next(&ExitStatus::completed()), FlowDecision::Next("step2".into()));
        assert_eq!(
            step.resolve_next(&ExitStatus::failed()),
            FlowDecision::Next("error-handler".into())
        );
    }

    #[test]
    fn test_flow_step_wildcard()
    {
        let step = FlowStep::new("step1").on("*", "step2");
        assert_eq!(
            step.resolve_next(&ExitStatus::new("ANYTHING")),
            FlowDecision::Next("step2".into())
        );
    }

    #[test]
    fn test_flow_step_default_next()
    {
        let step = FlowStep::new("step1").default_next("step2");
        assert_eq!(step.resolve_next(&ExitStatus::completed()), FlowDecision::Next("step2".into()));
    }

    #[test]
    fn test_flow_step_no_match_ends()
    {
        let step = FlowStep::new("step1");
        assert_eq!(step.resolve_next(&ExitStatus::completed()), FlowDecision::End);
    }

    #[test]
    fn test_on_completed_or_failed()
    {
        let cond = OnCompletedOrFailed::new("success-step", "failure-step");
        assert_eq!(
            cond.evaluate(&ExitStatus::completed()),
            FlowDecision::Next("success-step".into())
        );
        assert_eq!(cond.evaluate(&ExitStatus::failed()), FlowDecision::Next("failure-step".into()));
    }

    #[test]
    fn test_flow_execute_simple()
    {
        struct SimpleExecutor;
        impl StepExecutor for SimpleExecutor
        {
            fn execute_step(&mut self, _name: &str) -> BatchResult<ExitStatus>
            {
                Ok(ExitStatus::completed())
            }
        }

        let flow = FlowBuilder::new()
            .start("step1")
            .add_step(FlowStep::new("step1").on("COMPLETED", "step2"))
            .add_step(FlowStep::new("step2"))
            .build();

        let mut executor = SimpleExecutor;
        let result = flow.execute(&mut executor).unwrap();
        assert_eq!(result.code, "COMPLETED");
    }

    #[test]
    fn test_flow_no_steps()
    {
        let flow = JobFlow::new();
        assert!(flow.first_step().is_err());
    }

    #[test]
    fn test_flow_stop()
    {
        struct StopThenCompleteExecutor
        {
            count: usize,
        }
        impl StepExecutor for StopThenCompleteExecutor
        {
            fn execute_step(&mut self, _name: &str) -> BatchResult<ExitStatus>
            {
                self.count += 1;
                if self.count == 1
                {
                    Ok(ExitStatus::stopped())
                }
                else
                {
                    Ok(ExitStatus::completed())
                }
            }
        }

        let flow = FlowBuilder::new()
            .start("step1")
            .add_step(FlowStep::new("step1").on("STOPPED", "step2"))
            .add_step(FlowStep::new("step2"))
            .build();

        let mut executor = StopThenCompleteExecutor { count: 0 };
        let result = flow.execute(&mut executor).unwrap();
        // step1 → STOPPED → step2 → COMPLETED → no transitions → End → COMPLETED
        assert_eq!(result.code, "COMPLETED");
    }
}
