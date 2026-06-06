//! Prompt template system with variable substitution.
//! 带有变量替换的提示模板系统。
//!
//! Provides `PromptTemplate` for building reusable prompts with `{{variable}}`
//! placeholders, plus convenience types for system and user prompts.
//!
//! 提供 `PromptTemplate` 用于构建带有 `{{variable}}` 占位符的可重用提示，
//! 以及系统和用户提示的便捷类型。

use std::collections::HashMap;

use crate::chat_model::ChatMessage;

/// A template for building prompts with variable substitution.
/// 用于通过变量替换构建提示的模板。
///
/// Uses `{{variable_name}}` syntax for placeholders that are replaced
/// with values from a variables map during rendering.
///
/// 使用 `{{variable_name}}` 语法作为占位符，在渲染时用变量映射中的值替换。
///
/// # Example / 示例
///
/// ```
/// use std::collections::HashMap;
///
/// use hiver_ai::prompt::PromptTemplate;
///
/// let template = PromptTemplate::new("Translate the following {{language}} text: {{text}}");
///
/// let mut vars = HashMap::new();
/// vars.insert("language".to_string(), "French".to_string());
/// vars.insert("text".to_string(), "Hello, world!".to_string());
///
/// let rendered = template.render(&vars);
/// assert_eq!(rendered, "Translate the following French text: Hello, world!");
/// ```
#[derive(Debug, Clone)]
pub struct PromptTemplate
{
    /// The template string with `{{variable}}` placeholders.
    /// 带有 `{{variable}}` 占位符的模板字符串。
    template: String,
    /// Default variables to use during rendering.
    /// 渲染期间使用的默认变量。
    variables: HashMap<String, String>,
}

impl PromptTemplate
{
    /// Creates a new prompt template with the given template string.
    /// 使用给定的模板字符串创建新的提示模板。
    ///
    /// Variables are specified using `{{variable_name}}` syntax.
    /// 变量使用 `{{variable_name}}` 语法指定。
    #[must_use]
    pub fn new(template: impl Into<String>) -> Self
    {
        Self {
            template: template.into(),
            variables: HashMap::new(),
        }
    }

    /// Adds a default variable to the template.
    /// 向模板添加默认变量。
    #[must_use]
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Adds multiple default variables to the template.
    /// 向模板添加多个默认变量。
    #[must_use]
    pub fn vars(mut self, variables: HashMap<String, String>) -> Self
    {
        self.variables.extend(variables);
        self
    }

    /// Returns the raw template string.
    /// 返回原始模板字符串。
    #[must_use]
    pub fn template(&self) -> &str
    {
        &self.template
    }

    /// Returns a reference to the default variables.
    /// 返回默认变量的引用。
    #[must_use]
    pub fn variables(&self) -> &HashMap<String, String>
    {
        &self.variables
    }

    /// Renders the template by replacing `{{variable}}` placeholders with values.
    /// 通过替换 `{{variable}}` 占位符为值来渲染模板。
    ///
    /// Values from the `overrides` parameter take precedence over
    /// default variables set on the template.
    ///
    /// `overrides` 参数中的值优先于模板上设置的默认变量。
    pub fn render(&self, overrides: &HashMap<String, String>) -> String
    {
        let mut merged = self.variables.clone();
        merged.extend(overrides.clone());
        Self::substitute(&self.template, &merged)
    }

    /// Renders the template using only default variables.
    /// 仅使用默认变量渲染模板。
    pub fn render_default(&self) -> String
    {
        Self::substitute(&self.template, &self.variables)
    }

    /// Extracts variable names from the template string.
    /// 从模板字符串中提取变量名。
    #[must_use]
    pub fn extract_variables(&self) -> Vec<String>
    {
        Self::find_variables(&self.template)
    }

    /// Substitutes `{{key}}` placeholders in the template with values from the map.
    /// 将模板中的 `{{key}}` 占位符替换为映射中的值。
    ///
    /// Unknown variables are left as-is in the output.
    /// 未知变量在输出中保持原样。
    fn substitute(template: &str, variables: &HashMap<String, String>) -> String
    {
        let mut result = template.to_string();
        for (key, value) in variables
        {
            let placeholder = format!("{{{{{key}}}}}");
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Finds all `{{variable}}` names in a template string.
    /// 查找模板字符串中所有 `{{variable}}` 名称。
    fn find_variables(template: &str) -> Vec<String>
    {
        let mut variables = Vec::new();
        let mut search_from = 0;

        while let Some(start) = template[search_from..].find("{{")
        {
            let abs_start = search_from + start;
            if let Some(end) = template[abs_start..].find("}}")
            {
                let var_name = template[abs_start + 2..abs_start + end].trim().to_string();
                if !var_name.is_empty() && !variables.contains(&var_name)
                {
                    variables.push(var_name);
                }
                search_from = abs_start + end + 2;
            }
            else
            {
                break;
            }
        }

        variables
    }
}

impl std::fmt::Display for PromptTemplate
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.template)
    }
}

/// A system prompt template for AI instructions.
/// 用于 AI 指令的系统提示模板。
///
/// Provides convenience methods for creating system-level prompts
/// that guide the AI's behavior.
///
/// 提供用于创建指导 AI 行为的系统级提示的便捷方法。
#[derive(Debug, Clone)]
pub struct SystemPrompt
{
    /// The inner prompt template.
    /// 内部提示模板。
    template: PromptTemplate,
}

impl SystemPrompt
{
    /// Creates a new system prompt with the given content.
    /// 使用给定内容创建新的系统提示。
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self
    {
        Self {
            template: PromptTemplate::new(content),
        }
    }

    /// Creates a system prompt from a template with default variables.
    /// 从带有默认变量的模板创建系统提示。
    #[must_use]
    pub fn from_template(template: PromptTemplate) -> Self
    {
        Self { template }
    }

    /// Renders the system prompt with the given variables.
    /// 使用给定变量渲染系统提示。
    pub fn render(&self, variables: &HashMap<String, String>) -> String
    {
        self.template.render(variables)
    }

    /// Renders the system prompt with default variables only.
    /// 仅使用默认变量渲染系统提示。
    pub fn render_default(&self) -> String
    {
        self.template.render_default()
    }

    /// Converts this system prompt into a `ChatMessage`.
    /// 将此系统提示转换为 `ChatMessage`。
    #[must_use]
    pub fn to_message(&self) -> ChatMessage
    {
        ChatMessage::system(self.render_default())
    }

    /// Converts this system prompt into a `ChatMessage` with variable overrides.
    /// 使用变量覆盖将此系统提示转换为 `ChatMessage`。
    #[must_use]
    pub fn to_message_with(&self, variables: &HashMap<String, String>) -> ChatMessage
    {
        ChatMessage::system(self.render(variables))
    }
}

/// A user prompt template for user input.
/// 用于用户输入的用户提示模板。
#[derive(Debug, Clone)]
pub struct UserPrompt
{
    /// The inner prompt template.
    /// 内部提示模板。
    template: PromptTemplate,
}

impl UserPrompt
{
    /// Creates a new user prompt with the given content.
    /// 使用给定内容创建新的用户提示。
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self
    {
        Self {
            template: PromptTemplate::new(content),
        }
    }

    /// Creates a user prompt from a template with default variables.
    /// 从带有默认变量的模板创建用户提示。
    #[must_use]
    pub fn from_template(template: PromptTemplate) -> Self
    {
        Self { template }
    }

    /// Renders the user prompt with the given variables.
    /// 使用给定变量渲染用户提示。
    pub fn render(&self, variables: &HashMap<String, String>) -> String
    {
        self.template.render(variables)
    }

    /// Renders the user prompt with default variables only.
    /// 仅使用默认变量渲染用户提示。
    pub fn render_default(&self) -> String
    {
        self.template.render_default()
    }

    /// Converts this user prompt into a `ChatMessage`.
    /// 将此用户提示转换为 `ChatMessage`。
    #[must_use]
    pub fn to_message(&self) -> ChatMessage
    {
        ChatMessage::user(self.render_default())
    }

    /// Converts this user prompt into a `ChatMessage` with variable overrides.
    /// 使用变量覆盖将此用户提示转换为 `ChatMessage`。
    #[must_use]
    pub fn to_message_with(&self, variables: &HashMap<String, String>) -> ChatMessage
    {
        ChatMessage::user(self.render(variables))
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_basic_template_render()
    {
        let template = PromptTemplate::new("Hello, {{name}}!");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let result = template.render(&vars);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_multiple_variables()
    {
        let template = PromptTemplate::new("{{greeting}}, {{name}}! Today is {{day}}.");
        let mut vars = HashMap::new();
        vars.insert("greeting".to_string(), "Hi".to_string());
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("day".to_string(), "Monday".to_string());

        let result = template.render(&vars);
        assert_eq!(result, "Hi, Alice! Today is Monday.");
    }

    #[test]
    fn test_default_variables()
    {
        let template =
            PromptTemplate::new("Hello, {{name}}! You are a {{role}}.").var("role", "assistant");

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Bob".to_string());

        let result = template.render(&vars);
        assert_eq!(result, "Hello, Bob! You are a assistant.");
    }

    #[test]
    fn test_override_defaults()
    {
        let template = PromptTemplate::new("Model: {{model}}, Temp: {{temp}}")
            .var("model", "gpt-4")
            .var("temp", "0.7");

        let mut vars = HashMap::new();
        vars.insert("model".to_string(), "gpt-3.5".to_string());

        let result = template.render(&vars);
        assert_eq!(result, "Model: gpt-3.5, Temp: 0.7");
    }

    #[test]
    fn test_render_default()
    {
        let template = PromptTemplate::new("Hello, {{name}}!").var("name", "default");
        assert_eq!(template.render_default(), "Hello, default!");
    }

    #[test]
    fn test_unknown_variable_left_as_is()
    {
        let template = PromptTemplate::new("Hello, {{name}}! {{unknown}}");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let result = template.render(&vars);
        assert_eq!(result, "Hello, World! {{unknown}}");
    }

    #[test]
    fn test_no_variables()
    {
        let template = PromptTemplate::new("Hello, world!");
        let vars = HashMap::new();
        assert_eq!(template.render(&vars), "Hello, world!");
    }

    #[test]
    fn test_extract_variables()
    {
        let template = PromptTemplate::new("{{greeting}}, {{name}}! {{name}} again.");
        let vars = template.extract_variables();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"greeting".to_string()));
        assert!(vars.contains(&"name".to_string()));
    }

    #[test]
    fn test_extract_variables_empty()
    {
        let template = PromptTemplate::new("No variables here!");
        assert!(template.extract_variables().is_empty());
    }

    #[test]
    fn test_whitespace_in_variables()
    {
        let template = PromptTemplate::new("{{ name }} and {{  other  }}");
        let vars = template.extract_variables();
        assert_eq!(vars[0], "name");
        assert_eq!(vars[1], "other");
    }

    #[test]
    fn test_system_prompt()
    {
        let sys = SystemPrompt::new("You are a {{role}}.");
        let mut vars = HashMap::new();
        vars.insert("role".to_string(), "translator".to_string());

        let msg = sys.to_message_with(&vars);
        assert_eq!(msg.role, crate::chat_model::Role::System);
        assert_eq!(msg.content, "You are a translator.");
    }

    #[test]
    fn test_user_prompt()
    {
        let user = UserPrompt::new("Translate: {{text}}");
        let mut vars = HashMap::new();
        vars.insert("text".to_string(), "Hello".to_string());

        let msg = user.to_message_with(&vars);
        assert_eq!(msg.role, crate::chat_model::Role::User);
        assert_eq!(msg.content, "Translate: Hello");
    }

    #[test]
    fn test_template_display()
    {
        let template = PromptTemplate::new("Hello, {{name}}!");
        assert_eq!(format!("{template}"), "Hello, {{name}}!");
    }

    #[test]
    fn test_repeated_variable()
    {
        let template = PromptTemplate::new("{{x}} and {{x}}");
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), "value".to_string());
        assert_eq!(template.render(&vars), "value and value");
    }

    #[test]
    fn test_vars_builder_method()
    {
        let mut vars = HashMap::new();
        vars.insert("a".to_string(), "1".to_string());
        vars.insert("b".to_string(), "2".to_string());

        let template = PromptTemplate::new("{{a}}-{{b}}").vars(vars);
        assert_eq!(template.render_default(), "1-2");
    }
}
