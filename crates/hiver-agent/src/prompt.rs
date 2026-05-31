//! Prompt templates for common agent patterns.
//! 常见代理模式的提示模板。
//!
//! Provides pre-built prompt templates for:
//! - ReAct agent loop
//! - Summarization tasks
//! - Classification tasks
//! - Custom templates with `{variable}` interpolation
//!
//! 提供预构建的提示模板，用于：
//! - ReAct 代理循环
//! - 摘要任务
//! - 分类任务
//! - 带有 `{variable}` 插值的自定义模板

use std::collections::HashMap;

/// A prompt template with `{variable}` interpolation.
/// 带有 `{variable}` 插值的提示模板。
///
/// Unlike `hiver_ai::PromptTemplate` which uses `{{variable}}` syntax,
/// this template uses single-brace `{variable}` syntax for easier typing
/// in agent contexts.
///
/// 与使用 `{{variable}}` 语法的 `hiver_ai::PromptTemplate` 不同，
/// 此模板使用单大括号 `{variable}` 语法，便于在代理上下文中输入。
///
/// # Example / 示例
///
/// ```
/// use hiver_agent::prompt::AgentPromptTemplate;
/// use std::collections::HashMap;
///
/// let template = AgentPromptTemplate::new("Hello, {name}! Your role is {role}.");
///
/// let mut vars = HashMap::new();
/// vars.insert("name", "Alice");
/// vars.insert("role", "assistant");
///
/// let rendered = template.render(&vars);
/// assert_eq!(rendered, "Hello, Alice! Your role is assistant.");
/// ```
#[derive(Debug, Clone)]
pub struct AgentPromptTemplate {
    /// The template string with `{variable}` placeholders.
    /// 带有 `{variable}` 占位符的模板字符串。
    template: String,
}

impl AgentPromptTemplate {
    /// Creates a new prompt template.
    /// 创建新的提示模板。
    #[must_use]
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    /// Renders the template by replacing `{variable}` placeholders.
    /// 通过替换 `{variable}` 占位符渲染模板。
    pub fn render(&self, variables: &HashMap<&str, &str>) -> String {
        let mut result = self.template.clone();
        for (key, value) in variables {
            let placeholder = format!("{{{key}}}");
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Returns the raw template string.
    /// 返回原始模板字符串。
    #[must_use]
    pub fn template(&self) -> &str {
        &self.template
    }

    /// Extracts variable names from the template.
    /// 从模板中提取变量名。
    #[must_use]
    pub fn extract_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        let chars = self.template.chars().peekable();
        let mut in_brace = false;
        let mut current_var = String::new();

        for ch in chars {
            if ch == '{' {
                in_brace = true;
                current_var.clear();
            } else if ch == '}' && in_brace {
                in_brace = false;
                let trimmed = current_var.trim().to_string();
                if !trimmed.is_empty() && !vars.contains(&trimmed) {
                    vars.push(trimmed);
                }
            } else if in_brace {
                current_var.push(ch);
            }
        }

        vars
    }
}

impl std::fmt::Display for AgentPromptTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.template)
    }
}

/// Pre-built prompt templates for common agent patterns.
/// 常见代理模式的预构建提示模板。
pub struct AgentTemplates;

impl AgentTemplates {
    /// Returns the default ReAct system prompt template.
    /// 返回默认的 ReAct 系统提示模板。
    #[must_use]
    pub fn react_system() -> AgentPromptTemplate {
        AgentPromptTemplate::new(
            "You are a helpful AI assistant that can reason and use tools.\n\
             \n\
             Follow this format:\n\
             Thought: [your reasoning]\n\
             Action: [tool_name]\n\
             Action Input: [JSON arguments]\n\
             \n\
             When you have the final answer:\n\
             Thought: [final reasoning]\n\
             Final Answer: [your answer]\n\
             \n\
             Available tools:\n\
             {tools}",
        )
    }

    /// Returns a summarization prompt template.
    /// 返回摘要提示模板。
    #[must_use]
    pub fn summarization() -> AgentPromptTemplate {
        AgentPromptTemplate::new(
            "Please provide a concise summary of the following text. \
             Focus on the key points and main ideas.\n\
             \n\
             Text to summarize:\n\
             {text}",
        )
    }

    /// Returns a classification prompt template.
    /// 返回分类提示模板。
    #[must_use]
    pub fn classification() -> AgentPromptTemplate {
        AgentPromptTemplate::new(
            "Classify the following input into exactly one of these categories.\n\
             Respond with ONLY the category name.\n\
             \n\
             Categories:\n\
             {categories}\n\
             \n\
             Input: {input}\n\
             \n\
             Category:",
        )
    }

    /// Returns a question-answering prompt template with context.
    /// 返回带上下文的问答提示模板。
    #[must_use]
    pub fn qa_with_context() -> AgentPromptTemplate {
        AgentPromptTemplate::new(
            "Answer the following question based on the provided context. \
             If the context does not contain enough information, say so.\n\
             \n\
             Context:\n\
             {context}\n\
             \n\
             Question: {question}\n\
             \n\
             Answer:",
        )
    }

    /// Returns a code generation prompt template.
    /// 返回代码生成提示模板。
    #[must_use]
    pub fn code_generation() -> AgentPromptTemplate {
        AgentPromptTemplate::new(
            "Generate {language} code for the following task:\n\
             \n\
             Task: {task}\n\
             \n\
             Requirements:\n\
             {requirements}\n\
             \n\
             Code:",
        )
    }

    /// Returns a translation prompt template.
    /// 返回翻译提示模板。
    #[must_use]
    pub fn translation() -> AgentPromptTemplate {
        AgentPromptTemplate::new(
            "Translate the following text from {source_language} to {target_language}.\n\
             Preserve the tone and style of the original text.\n\
             \n\
             Text:\n\
             {text}",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_render() {
        let template = AgentPromptTemplate::new("Hello, {name}!");
        let mut vars = HashMap::new();
        vars.insert("name", "World");
        assert_eq!(template.render(&vars), "Hello, World!");
    }

    #[test]
    fn test_multiple_variables() {
        let template = AgentPromptTemplate::new("{greeting}, {name}! Today is {day}.");
        let mut vars = HashMap::new();
        vars.insert("greeting", "Hi");
        vars.insert("name", "Alice");
        vars.insert("day", "Monday");
        assert_eq!(template.render(&vars), "Hi, Alice! Today is Monday.");
    }

    #[test]
    fn test_missing_variable_left_as_is() {
        let template = AgentPromptTemplate::new("Hello, {name}! {unknown}");
        let mut vars = HashMap::new();
        vars.insert("name", "World");
        assert_eq!(template.render(&vars), "Hello, World! {unknown}");
    }

    #[test]
    fn test_no_variables() {
        let template = AgentPromptTemplate::new("No variables here!");
        let vars = HashMap::new();
        assert_eq!(template.render(&vars), "No variables here!");
    }

    #[test]
    fn test_extract_variables() {
        let template = AgentPromptTemplate::new("{greeting}, {name}! {name} again.");
        let vars = template.extract_variables();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"greeting".to_string()));
        assert!(vars.contains(&"name".to_string()));
    }

    #[test]
    fn test_extract_variables_empty() {
        let template = AgentPromptTemplate::new("No variables!");
        assert!(template.extract_variables().is_empty());
    }

    #[test]
    fn test_template_display() {
        let template = AgentPromptTemplate::new("Hello, {name}!");
        assert_eq!(format!("{template}"), "Hello, {name}!");
    }

    #[test]
    fn test_template_returns_str() {
        let template = AgentPromptTemplate::new("test {var}");
        assert_eq!(template.template(), "test {var}");
    }

    #[test]
    fn test_react_system_template() {
        let template = AgentTemplates::react_system();
        let vars = template.extract_variables();
        assert!(vars.contains(&"tools".to_string()));
    }

    #[test]
    fn test_summarization_template() {
        let template = AgentTemplates::summarization();
        let mut vars = HashMap::new();
        vars.insert("text", "Some long text here");
        let rendered = template.render(&vars);
        assert!(rendered.contains("Some long text here"));
    }

    #[test]
    fn test_classification_template() {
        let template = AgentTemplates::classification();
        let vars = template.extract_variables();
        assert!(vars.contains(&"categories".to_string()));
        assert!(vars.contains(&"input".to_string()));
    }

    #[test]
    fn test_qa_template() {
        let template = AgentTemplates::qa_with_context();
        let mut vars = HashMap::new();
        vars.insert("context", "Paris is the capital of France.");
        vars.insert("question", "What is the capital of France?");
        let rendered = template.render(&vars);
        assert!(rendered.contains("Paris is the capital"));
    }

    #[test]
    fn test_code_generation_template() {
        let template = AgentTemplates::code_generation();
        let vars = template.extract_variables();
        assert!(vars.contains(&"language".to_string()));
        assert!(vars.contains(&"task".to_string()));
        assert!(vars.contains(&"requirements".to_string()));
    }

    #[test]
    fn test_translation_template() {
        let template = AgentTemplates::translation();
        let vars = template.extract_variables();
        assert!(vars.contains(&"source_language".to_string()));
        assert!(vars.contains(&"target_language".to_string()));
        assert!(vars.contains(&"text".to_string()));
    }
}
