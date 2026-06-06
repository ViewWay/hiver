//! State machine visualization — generates Mermaid/PlantUML diagrams
//! 状态机可视化 — 生成 Mermaid/PlantUML 图表

use crate::{
    config::{StateMachineConfig, TransitionConfig},
    error::StateMachineResult,
};

/// Diagram output format
/// 图表输出格式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagramFormat
{
    /// Mermaid syntax
    /// Mermaid 语法
    Mermaid,
    /// PlantUML syntax
    /// PlantUML 语法
    PlantUml,
}

/// State machine diagram generator
/// 状态机图表生成器
pub struct StateMachineVisualizer<'a>
{
    config: &'a StateMachineConfig,
    format: DiagramFormat,
}

impl<'a> StateMachineVisualizer<'a>
{
    /// Create a new visualizer
    /// 创建新可视化器
    pub fn new(config: &'a StateMachineConfig) -> Self
    {
        Self {
            config,
            format: DiagramFormat::Mermaid,
        }
    }

    /// Set output format
    /// 设置输出格式
    pub fn format(mut self, format: DiagramFormat) -> Self
    {
        self.format = format;
        self
    }

    /// Generate diagram string
    /// 生成图表字符串
    pub fn render(&self) -> StateMachineResult<String>
    {
        match self.format
        {
            DiagramFormat::Mermaid => self.render_mermaid(),
            DiagramFormat::PlantUml => self.render_plantuml(),
        }
    }

    fn render_mermaid(&self) -> StateMachineResult<String>
    {
        let mut lines = Vec::new();
        lines.push("stateDiagram-v2".to_string());
        lines.push(format!("    [*] --> {}", self.config.initial_state));

        for state in &self.config.states
        {
            if state.final_state
            {
                lines.push(format!("    {} --> [*]", state.id));
            }
        }

        for transition in &self.config.transitions
        {
            lines.push(self.format_mermaid_transition(transition));
        }

        Ok(lines.join("\n"))
    }

    fn format_mermaid_transition(&self, t: &TransitionConfig) -> String
    {
        let label = match &t.event
        {
            Some(event) => format!(" : {}", event),
            None => String::new(),
        };
        format!("    {} --> {}{}", t.source, t.target, label)
    }

    fn render_plantuml(&self) -> StateMachineResult<String>
    {
        let mut lines = Vec::new();
        lines.push("@startuml".to_string());
        lines.push("[*] --> ".to_string() + &self.config.initial_state);

        for state in &self.config.states
        {
            if state.final_state
            {
                lines.push(format!("{} --> [*]", state.id));
            }
        }

        for transition in &self.config.transitions
        {
            let label = match &transition.event
            {
                Some(event) => format!(" : {}", event),
                None => String::new(),
            };
            lines.push(format!("{} --> {}{}", transition.source, transition.target, label));
        }

        lines.push("@enduml".to_string());
        Ok(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::config::{StateConfig, TransitionConfig};

    #[test]
    fn test_mermaid_output()
    {
        let config = StateMachineConfig::new("door", "Locked")
            .add_state(StateConfig::new("Locked"))
            .add_state(StateConfig::new("Unlocked"))
            .add_transition(TransitionConfig::new("Locked", "Unlocked").event("coin"))
            .add_transition(TransitionConfig::new("Unlocked", "Locked").event("push"));

        let viz = StateMachineVisualizer::new(&config);
        let output = viz.render().unwrap();

        assert!(output.contains("stateDiagram-v2"));
        assert!(output.contains("[*] --> Locked"));
        assert!(output.contains("Locked --> Unlocked : coin"));
        assert!(output.contains("Unlocked --> Locked : push"));
    }

    #[test]
    fn test_plantuml_output()
    {
        let config = StateMachineConfig::new("door", "Locked")
            .add_state(StateConfig::new("Locked"))
            .add_state(StateConfig::new("Unlocked").final_state(true))
            .add_transition(TransitionConfig::new("Locked", "Unlocked").event("coin"));

        let viz = StateMachineVisualizer::new(&config).format(DiagramFormat::PlantUml);
        let output = viz.render().unwrap();

        assert!(output.contains("@startuml"));
        assert!(output.contains("@enduml"));
        assert!(output.contains("Locked --> Unlocked : coin"));
    }

    #[test]
    fn test_final_state_mermaid()
    {
        let config = StateMachineConfig::new("test", "A")
            .add_state(StateConfig::new("A"))
            .add_state(StateConfig::new("B").final_state(true))
            .add_transition(TransitionConfig::new("A", "B"));

        let viz = StateMachineVisualizer::new(&config);
        let output = viz.render().unwrap();

        assert!(output.contains("B --> [*]"));
    }
}
