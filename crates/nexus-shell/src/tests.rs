//! Comprehensive tests for nexus-shell
//! nexus-shell的综合测试
//!
//! Tests cover: command registration, execution, result formatting,
//! validation, completion, prompt, and built-in commands.
//!
//! 测试覆盖：命令注册、执行、结果格式化、验证、补全、提示符和内置命令。

use std::sync::{Arc, Mutex};

use crate::builtin::{BuiltinState, ClearCommand, EchoCommand, ExitCommand, HistoryCommand};
use crate::command::{Command, CommandBox, CommandMeta, CommandRegistry, ParameterMeta};
use crate::completion::CompletionProvider;
use crate::prompt::{Banner, PromptColor, PromptStyle};
use crate::result::{
    JsonResult, OutputFormat, ResultHandler, ShellError, ShellOutput, TableResult, TextResult,
};
use crate::validation::{InputValidator, ValidatedInput};
use crate::{Shell, ShellBuilder, ShellConfig};

// ============================================================================
// Helper: Simple test command / 辅助：简单测试命令
// ============================================================================

struct TestCommand {
    name: String,
    desc: String,
    handler: fn(&[&str]) -> Result<String, ShellError>,
}

impl TestCommand {
    fn new(name: &str, desc: &str) -> Self {
        Self {
            name: name.to_string(),
            desc: desc.to_string(),
            handler: |_args| Ok("test output".to_string()),
        }
    }

    fn with_handler(
        name: &str,
        desc: &str,
        handler: fn(&[&str]) -> Result<String, ShellError>,
    ) -> Self {
        Self {
            name: name.to_string(),
            desc: desc.to_string(),
            handler,
        }
    }
}

impl Command for TestCommand {
    fn meta(&self) -> CommandMeta {
        CommandMeta::new(&self.name).description(&self.desc)
    }

    fn execute(&self, args: &[&str]) -> Result<String, ShellError> {
        (self.handler)(args)
    }
}

// ============================================================================
// 1. Command Registry Tests / 命令注册表测试
// ============================================================================

#[test]
fn test_command_registry_new() {
    let registry = CommandRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_command_registry_register_and_lookup() {
    let mut registry = CommandRegistry::new();
    registry.register(TestCommand::new("greet", "Greet someone"));

    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 1);
    assert!(registry.contains("greet"));
    assert!(!registry.contains("unknown"));

    let cmd = registry.get("greet").expect("should find greet");
    assert_eq!(cmd.name(), "greet");
    assert_eq!(cmd.description(), "Greet someone");
}

#[test]
fn test_command_registry_aliases() {
    let mut registry = CommandRegistry::new();

    struct AliasCmd;
    impl Command for AliasCmd {
        fn meta(&self) -> CommandMeta {
            CommandMeta::new("hello")
                .description("Say hello")
                .aliases(&["hi", "hey"])
        }
        fn execute(&self, args: &[&str]) -> Result<String, ShellError> {
            Ok(format!("Hello, {}!", args.first().unwrap_or(&"World")))
        }
    }

    registry.register(AliasCmd);

    assert!(registry.contains("hello"));
    assert!(registry.contains("hi"));
    assert!(registry.contains("hey"));
    assert_eq!(registry.len(), 1);

    // Execute via alias / 通过别名执行
    let result = registry.execute_line("hi Alice");
    assert_eq!(result.unwrap(), "Hello, Alice!");
}

#[test]
fn test_command_registry_execute_line() {
    let mut registry = CommandRegistry::new();
    registry.register(TestCommand::with_handler(
        "add",
        "Add numbers",
        |args| {
            let sum: i32 = args.iter().filter_map(|s| s.parse::<i32>().ok()).sum();
            Ok(sum.to_string())
        },
    ));

    let result = registry.execute_line("add 1 2 3").unwrap();
    assert_eq!(result, "6");
}

#[test]
fn test_command_registry_unknown_command() {
    let registry = CommandRegistry::new();
    let result = registry.execute_line("unknown_cmd");
    assert!(result.is_err());
    match result.unwrap_err() {
        ShellError::CommandNotFound(name) => assert_eq!(name, "unknown_cmd"),
        _ => panic!("Expected CommandNotFound error"),
    }
}

#[test]
fn test_command_registry_all_commands() {
    let mut registry = CommandRegistry::new();
    registry.register(TestCommand::new("alpha", "First"));
    registry.register(TestCommand::new("beta", "Second"));
    registry.register(TestCommand::new("gamma", "Third"));

    let names = registry.command_names();
    assert_eq!(names, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn test_command_registry_hidden_commands() {
    let mut registry = CommandRegistry::new();

    struct HiddenCmd;
    impl Command for HiddenCmd {
        fn meta(&self) -> CommandMeta {
            CommandMeta::new("secret").description("Hidden cmd").hidden(true)
        }
        fn execute(&self, _args: &[&str]) -> Result<String, ShellError> {
            Ok("secret".to_string())
        }
    }

    registry.register(HiddenCmd);
    registry.register(TestCommand::new("visible", "Visible cmd"));

    let visible = registry.all_commands();
    assert_eq!(visible.len(), 1); // secret is hidden
    assert_eq!(visible[0].name, "visible");

    let all = registry.all_commands_including_hidden();
    assert_eq!(all.len(), 2);
}

// ============================================================================
// 2. Result Handler Tests / 结果处理器测试
// ============================================================================

#[test]
fn test_result_handler_plain() {
    let handler = ResultHandler::with_format(OutputFormat::Plain);
    let text = TextResult::new("hello world");
    assert_eq!(handler.handle(&text), "hello world");
}

#[test]
fn test_result_handler_json() {
    let handler = ResultHandler::with_format(OutputFormat::Json);
    let text = TextResult::new("hello");
    let output = handler.handle(&text);
    assert!(output.contains("\"result\""));
    assert!(output.contains("hello"));
}

#[test]
fn test_result_handler_error() {
    let handler = ResultHandler::new();
    let error = ShellError::CommandNotFound("test".to_string());
    let output = handler.handle_error(&error);
    assert!(output.contains("Command not found"));
}

#[test]
fn test_table_result_render() {
    let table = TableResult::new(vec!["Name", "Age", "City"])
        .row(vec!["Alice", "30", "NYC"])
        .row(vec!["Bob", "25", "LA"]);

    let rendered = table.render_table();
    assert!(rendered.contains("Name"));
    assert!(rendered.contains("Alice"));
    assert!(rendered.contains("Bob"));
    assert!(rendered.contains("NYC"));
}

#[test]
fn test_table_result_json() {
    let table = TableResult::new(vec!["Name", "Age"])
        .row(vec!["Alice", "30"]);

    let json = table.render_json();
    assert!(json.contains("\"Name\""));
    assert!(json.contains("\"Alice\""));
}

#[test]
fn test_json_result() {
    let json_result = JsonResult::from_str(r#"{"key": "value"}"#).unwrap();
    let rendered = json_result.render_json();
    assert!(rendered.contains("key"));
    assert!(rendered.contains("value"));
}

#[test]
fn test_text_result_json() {
    let text = TextResult::new("hello world");
    let json = text.render_json();
    assert!(json.contains("hello world"));
}

// ============================================================================
// 3. Validation Tests / 验证测试
// ============================================================================

#[test]
fn test_validation_valid_input() {
    let validator = InputValidator::new();
    let result = validator.validate("greet Alice").unwrap();
    assert_eq!(result.command, "greet");
    assert_eq!(result.args, vec!["Alice"]);
}

#[test]
fn test_validation_empty_input() {
    let validator = InputValidator::new().allow_empty(true);
    let result = validator.validate("  ").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_validation_empty_not_allowed() {
    let validator = InputValidator::new().allow_empty(false);
    let result = validator.validate("");
    assert!(result.is_err());
}

#[test]
fn test_validation_too_long() {
    let validator = InputValidator::new().max_length(10);
    let result = validator.validate("this is a very long input string");
    assert!(result.is_err());
}

#[test]
fn test_validation_null_bytes() {
    let validator = InputValidator::new();
    let result = validator.validate("cmd\0arg");
    assert!(result.is_err());
}

#[test]
fn test_validation_invalid_command_name() {
    use crate::validation::validate_command_name;
    assert!(validate_command_name("123invalid").is_err());
    assert!(validate_command_name("valid-cmd").is_ok());
    assert!(validate_command_name("valid_cmd").is_ok());
    assert!(validate_command_name("valid:cmd").is_ok());
    assert!(validate_command_name("inv@lid").is_err());
}

// ============================================================================
// 4. Completion Tests / 补全测试
// ============================================================================

#[test]
fn test_completion_basic() {
    let mut registry = CommandRegistry::new();
    registry.register(TestCommand::new("greet", "Greet"));
    registry.register(TestCommand::new("goodbye", "Goodbye"));
    registry.register(TestCommand::new("help", "Help"));

    let provider = CompletionProvider::new(&registry);

    let completions = provider.complete("gr");
    assert_eq!(completions, vec!["greet"]);

    let completions = provider.complete("go");
    assert_eq!(completions, vec!["goodbye"]);

    let completions = provider.complete("g");
    assert!(completions.contains(&"greet".to_string()));
    assert!(completions.contains(&"goodbye".to_string()));

    let completions = provider.complete("");
    assert_eq!(completions.len(), 3);
}

#[test]
fn test_completion_no_match() {
    let mut registry = CommandRegistry::new();
    registry.register(TestCommand::new("hello", "Hello"));

    let provider = CompletionProvider::new(&registry);
    let completions = provider.complete("xyz");
    assert!(completions.is_empty());
}

// ============================================================================
// 5. Prompt Tests / 提示符测试
// ============================================================================

#[test]
fn test_prompt_style_default() {
    let style = PromptStyle::new();
    let rendered = style.render();
    assert!(rendered.contains("nexus"));
    assert!(rendered.contains(">"));
}

#[test]
fn test_prompt_style_custom() {
    let style = PromptStyle::new()
        .app_name("myapp")
        .color(PromptColor::Green)
        .suffix("$ ");

    let rendered = style.render();
    assert!(rendered.contains("myapp"));
    assert!(rendered.contains("$"));
}

#[test]
fn test_banner_render() {
    let banner = Banner::new();
    let rendered = banner.render();
    assert!(!rendered.is_empty());
    assert!(rendered.contains("Nexus Shell"));
}

#[test]
fn test_banner_disabled() {
    let banner = Banner::disabled();
    let rendered = banner.render();
    assert!(rendered.is_empty());
}

// ============================================================================
// 6. Built-in Command Tests / 内置命令测试
// ============================================================================

#[test]
fn test_builtin_echo_command() {
    let cmd = EchoCommand;
    assert_eq!(cmd.name(), "echo");

    let result = cmd.execute(&["hello", "world"]).unwrap();
    assert_eq!(result, "hello world");

    let result = cmd.execute(&[]).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_builtin_clear_command() {
    let cmd = ClearCommand;
    assert_eq!(cmd.name(), "clear");

    let result = cmd.execute(&[]).unwrap();
    // Should contain ANSI escape sequences
    assert!(result.contains('\x1b'));
}

#[test]
fn test_builtin_exit_command() {
    let cmd = ExitCommand;
    assert_eq!(cmd.name(), "exit");

    let result = cmd.execute(&[]);
    assert!(result.is_err());
    match result.unwrap_err() {
        ShellError::ExitRequested => {}
        _ => panic!("Expected ExitRequested"),
    }
}

#[test]
fn test_builtin_history_command() {
    let state = Arc::new(Mutex::new(BuiltinState::new()));
    {
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        s.add_history("echo hello");
        s.add_history("greet world");
    }

    let cmd = HistoryCommand::new(state);
    let result = cmd.execute(&[]).unwrap();
    assert!(result.contains("echo hello"));
    assert!(result.contains("greet world"));
}

// ============================================================================
// 7. CommandMeta Tests / 命令元数据测试
// ============================================================================

#[test]
fn test_command_meta_builder() {
    let meta = CommandMeta::new("test")
        .description("A test command")
        .group("Testing")
        .aliases(&["t", "tst"])
        .hidden(false)
        .parameter(ParameterMeta::required("name", "The name"))
        .parameter(ParameterMeta::optional("verbose", "Verbose output"));

    assert_eq!(meta.name, "test");
    assert_eq!(meta.description, "A test command");
    assert_eq!(meta.group, "Testing");
    assert_eq!(meta.aliases, vec!["t", "tst"]);
    assert!(!meta.hidden);
    assert_eq!(meta.parameters.len(), 2);
    assert!(meta.parameters[0].required);
    assert!(!meta.parameters[1].required);
    assert!(meta.matches("test"));
    assert!(meta.matches("t"));
    assert!(meta.matches("tst"));
    assert!(!meta.matches("other"));
}

#[test]
fn test_command_meta_display() {
    let meta = CommandMeta::new("hello")
        .description("Say hello")
        .aliases(&["hi"]);

    let display = format!("{meta}");
    assert!(display.contains("hello"));
    assert!(display.contains("hi"));
    assert!(display.contains("Say hello"));
}

// ============================================================================
// 8. ShellBuilder Tests / ShellBuilder测试
// ============================================================================

#[test]
fn test_shell_builder_default() {
    let shell = ShellBuilder::new().build();
    // Should have built-in commands registered
    // 应该已注册内置命令
    assert!(shell.registry().contains("help"));
    assert!(shell.registry().contains("exit"));
    assert!(shell.registry().contains("clear"));
    assert!(shell.registry().contains("echo"));
    assert!(shell.registry().contains("history"));
    assert!(shell.registry().contains("stacktrace"));
    assert!(shell.registry().contains("script"));
}

#[test]
fn test_shell_builder_no_builtins() {
    let shell = ShellBuilder::new()
        .register_builtins(false)
        .build();
    assert!(shell.registry().is_empty());
}

#[test]
fn test_shell_builder_custom_commands() {
    let shell = ShellBuilder::new()
        .register(TestCommand::new("custom1", "Custom 1"))
        .register(TestCommand::new("custom2", "Custom 2"))
        .build();

    assert!(shell.registry().contains("custom1"));
    assert!(shell.registry().contains("custom2"));
    // Built-in commands too / 还有内置命令
    assert!(shell.registry().contains("help"));
}

#[test]
fn test_shell_execute_line() {
    let shell = ShellBuilder::new()
        .register(TestCommand::with_handler(
            "double",
            "Double a number",
            |args| {
                let n: i32 = args.first().unwrap_or(&"0").parse().unwrap_or(0);
                Ok((n * 2).to_string())
            },
        ))
        .build();

    let result = shell.execute("double 21").unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_shell_execute_script() {
    let shell = ShellBuilder::new()
        .register(TestCommand::with_handler(
            "echo",
            "Echo",
            |args| Ok(args.join(" ")),
        ))
        .build();

    let script = "echo hello\n# comment\necho world\n";
    let results = shell.execute_script(script);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].as_ref().unwrap(), "hello");
    assert_eq!(results[1].as_ref().unwrap(), "world");
}

// ============================================================================
// 9. ShellConfig Tests / ShellConfig测试
// ============================================================================

#[test]
fn test_shell_config_builder() {
    let config = ShellConfig::new()
        .app_name("test-app")
        .show_banner(false)
        .register_builtins(false);

    assert_eq!(config.app_name, "test-app");
    assert!(!config.show_banner);
    assert!(!config.register_builtins);
}

// ============================================================================
// 10. BuiltinState Tests / 内置状态测试
// ============================================================================

#[test]
fn test_builtin_state_history() {
    let mut state = BuiltinState::new();
    state.add_history("cmd1");
    state.add_history("cmd2");
    state.add_history("cmd3");

    assert_eq!(state.history.len(), 3);
    assert_eq!(state.history[0], "cmd1");
    assert_eq!(state.history[2], "cmd3");
}

#[test]
fn test_builtin_state_history_max() {
    let mut state = BuiltinState::new();
    for i in 0..150 {
        state.add_history(&format!("cmd{i}"));
    }
    // Should be capped at MAX_HISTORY (100)
    assert_eq!(state.history.len(), 100);
}

#[test]
fn test_builtin_state_history_skip_empty() {
    let mut state = BuiltinState::new();
    state.add_history("cmd1");
    state.add_history("   ");
    state.add_history("");
    state.add_history("cmd2");

    assert_eq!(state.history.len(), 2);
}

#[test]
fn test_builtin_state_record_error() {
    let mut state = BuiltinState::new();
    let error = ShellError::CommandNotFound("test".to_string());
    state.record_error(&error);

    assert!(state.last_error.is_some());
    let trace = state.last_error.unwrap();
    assert!(trace.contains("Command not found"));
}

// ============================================================================
// 11. Shell Macro test (via shell_command! macro)
// ============================================================================

#[test]
fn test_shell_command_macro() {
    let cmd: CommandBox = crate::shell_command!(
        "test-macro",
        "A macro-defined command",
        |args: &[&str]| {
            Ok(format!("macro output: {}", args.join(",")))
        }
    );

    assert_eq!(cmd.name(), "test-macro");
    assert_eq!(cmd.description(), "A macro-defined command");

    let result = cmd.execute(&["a", "b"]).unwrap();
    assert_eq!(result, "macro output: a,b");
}

// ============================================================================
// 12. ParameterMeta Tests / 参数元数据测试
// ============================================================================

#[test]
fn test_parameter_meta() {
    let required = ParameterMeta::required("name", "Your name");
    assert!(required.required);
    assert_eq!(required.name, "name");
    assert!(required.default_value.is_none());

    let optional = ParameterMeta::optional("verbose", "Verbose");
    assert!(!optional.required);

    let with_default = ParameterMeta::with_default("count", "Count", "10");
    assert!(!with_default.required);
    assert_eq!(with_default.default_value, Some("10".to_string()));
}
