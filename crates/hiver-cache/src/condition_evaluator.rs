//! Cache Condition Expression Evaluator
//! 缓存条件表达式求值器
//!
//! Evaluates SpEL-like expressions for cache annotations
//! 评估缓存注解的类似 `SpEL` 的表达式

#![allow(clippy::indexing_slicing)]

use std::collections::HashMap;

use serde_json::Value as JsonValue;

/// Evaluate cache condition expression
/// 评估缓存条件表达式
///
/// # Supported Expressions / 支持的表达式
///
/// - `#param` - Parameter reference 参数引用
/// - `#param == value` - Equality check 相等性检查
/// - `#param != value` - Inequality check 不等性检查
/// - `#param > value`, `#param >= value` - Comparison 比较
/// - `#param < value`, `#param <= value` - Comparison 比较
/// - `#param.isEmpty()` - Check if string/collection is empty 检查字符串/集合是否为空
/// - `#param.length() > 0` - Check length 检查长度
/// - `#result` - Method result (for unless expressions) 方法结果（用于 unless 表达式）
/// - `expr1 and expr2` - Logical AND 逻辑与
/// - `expr1 or expr2` - Logical OR 逻辑或
/// - `!expr` - Logical NOT 逻辑非
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_cache::evaluate_cache_condition;
///
/// let mut args = HashMap::new();
/// args.insert("id".to_string(), JsonValue::Number(123.into()));
///
/// // Cache only if ID > 100
/// let should_cache = evaluate_cache_condition("#id > 100", &args, None);
/// assert!(should_cache);
///
/// // Don't cache if result is null
/// let should_not_cache = evaluate_cache_condition("#result == null", &args, Some(&JsonValue::Null));
/// assert!(should_not_cache);
/// ```
#[allow(clippy::implicit_hasher)]
pub fn evaluate_cache_condition(
    expression: &str,
    args: &HashMap<String, JsonValue>,
    result: Option<&JsonValue>,
) -> bool
{
    let expr = expression.trim();

    // Handle parenthesized expressions — strip outer parens if balanced
    // 处理括号表达式 — 如果外层括号匹配则去掉
    if expr.starts_with('(') && expr.ends_with(')')
    {
        let mut depth = 0;
        let mut balanced = true;
        for (i, c) in expr.char_indices()
        {
            match c
            {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ =>
                {},
            }
            if depth == 0 && i < expr.len() - 1
            {
                balanced = false;
                break;
            }
        }
        if balanced && depth == 0
        {
            return evaluate_cache_condition(&expr[1..expr.len() - 1], args, result);
        }
    }

    // Handle logical NOT
    if let Some(inner) = expr.strip_prefix('!')
    {
        let inner_expr = inner.trim();
        return !evaluate_cache_condition(inner_expr, args, result);
    }

    // Handle method calls like length() — must be checked BEFORE == operator
    // 处理像 length() 这样的方法调用 — 必须在 == 运算符之前检查
    if expr.contains(".length()")
    {
        #[allow(clippy::expect_used)]
        let param_part = expr.split(".length()").next().unwrap_or("");
        let method_end = param_part.len() + ".length()".len();
        let rest = &expr[method_end..];

        let value = get_value(param_part.trim(), args, result);
        let length = get_length(&value);

        if let Some(stripped) = rest.strip_prefix(" >")
        {
            let threshold = stripped.trim().parse::<i64>().unwrap_or(0);
            return (length as i64) > threshold;
        }
        else if let Some(stripped) = rest.strip_prefix(" <")
        {
            let threshold = stripped.trim().parse::<i64>().unwrap_or(0);
            return (length as i64) < threshold;
        }
        else if let Some(stripped) = rest.strip_prefix(" >=")
        {
            let threshold = stripped.trim().parse::<i64>().unwrap_or(0);
            return (length as i64) >= threshold;
        }
        else if let Some(stripped) = rest.strip_prefix(" <=")
        {
            let threshold = stripped.trim().parse::<i64>().unwrap_or(0);
            return (length as i64) <= threshold;
        }
        else if let Some(stripped) = rest.strip_prefix(" ==")
        {
            let threshold = stripped.trim().parse::<i64>().unwrap_or(0);
            return length == (threshold as usize);
        }
    }

    // Handle OR expressions (left to right)
    if let Some(pos) = find_operator(expr, " or ")
    {
        let left = &expr[..pos];
        let right = &expr[pos + 4..];
        return evaluate_cache_condition(left, args, result)
            || evaluate_cache_condition(right, args, result);
    }

    // Handle AND expressions (left to right)
    if let Some(pos) = find_operator(expr, " and ")
    {
        let left = &expr[..pos];
        let right = &expr[pos + 5..];
        return evaluate_cache_condition(left, args, result)
            && evaluate_cache_condition(right, args, result);
    }

    // Handle equality checks
    if expr.contains("==")
    {
        let parts: Vec<&str> = expr.splitn(2, "==").collect();
        if parts.len() == 2
        {
            let left = evaluate_to_value(parts[0].trim(), args, result);
            let right = evaluate_to_value(parts[1].trim(), args, result);
            return left == right;
        }
    }

    // Handle inequality checks
    if expr.contains("!=")
    {
        let parts: Vec<&str> = expr.splitn(2, "!=").collect();
        if parts.len() == 2
        {
            let left = evaluate_to_value(parts[0].trim(), args, result);
            let right = evaluate_to_value(parts[1].trim(), args, result);
            return left != right;
        }
    }

    // Handle greater than
    if expr.contains('>')
    {
        let parts: Vec<&str> = expr.split('>').collect();
        if parts.len() == 2
            && let (Some(left_num), Some(right_num)) = (
                extract_number(parts[0].trim(), args, result),
                extract_number(parts[1].trim(), args, result),
            )
        {
            return left_num > right_num;
        }
    }

    // Handle less than
    if expr.contains('<')
    {
        let parts: Vec<&str> = expr.split('<').collect();
        if parts.len() == 2
            && let (Some(left_num), Some(right_num)) = (
                extract_number(parts[0].trim(), args, result),
                extract_number(parts[1].trim(), args, result),
            )
        {
            return left_num < right_num;
        }
    }

    // Handle greater than or equal
    if expr.contains(">=")
    {
        let parts: Vec<&str> = expr.splitn(2, ">=").collect();
        if parts.len() == 2
            && let (Some(left_num), Some(right_num)) = (
                extract_number(parts[0].trim(), args, result),
                extract_number(parts[1].trim(), args, result),
            )
        {
            return left_num >= right_num;
        }
    }

    // Handle less than or equal
    if expr.contains("<=")
    {
        let parts: Vec<&str> = expr.splitn(2, "<=").collect();
        if parts.len() == 2
            && let (Some(left_num), Some(right_num)) = (
                extract_number(parts[0].trim(), args, result),
                extract_number(parts[1].trim(), args, result),
            )
        {
            return left_num <= right_num;
        }
    }

    // Handle method calls like isEmpty()
    // 处理像 isEmpty() 这样的方法调用
    if expr.ends_with("isEmpty()")
    {
        // Split on '.' to get the base parameter name
        // 使用 '.' 分割来获取基础参数名
        let param_part = expr.split('.').next().unwrap_or(expr);
        let value = get_value(param_part.trim(), args, result);
        return is_empty_value(&value);
    }

    // Handle simple parameter reference - check if it's truthy
    let value = get_value(expr, args, result);
    is_truthy(&value)
}

/// Find operator position (respecting parentheses)
/// 查找运算符位置（考虑括号）
fn find_operator(expr: &str, op: &str) -> Option<usize>
{
    let mut depth = 0;
    let chars = expr.char_indices();

    for (i, c) in chars
    {
        match c
        {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth == 0 && expr[i..].starts_with(op) => return Some(i),
            _ =>
            {},
        }
    }

    None
}

/// Get value from expression
/// 从表达式获取值
fn get_value(expr: &str, args: &HashMap<String, JsonValue>, result: Option<&JsonValue>)
-> JsonValue
{
    let expr = expr.trim();

    // Handle #result
    if expr == "#result"
    {
        return result.cloned().unwrap_or(JsonValue::Null);
    }

    // Handle parameter reference like #param
    if let Some(param_name) = expr.strip_prefix('#')
        && let Some(value) = args.get(param_name)
    {
        return value.clone();
    }

    // Handle string literals (both single and double quotes)
    // 处理字符串字面量（单引号和双引号）
    if (expr.starts_with('"') && expr.ends_with('"'))
        || (expr.starts_with('\'') && expr.ends_with('\''))
    {
        return JsonValue::String(expr[1..expr.len() - 1].to_string());
    }

    // Handle number literals
    if let Ok(num) = expr.parse::<i64>()
    {
        return JsonValue::Number(num.into());
    }

    if let Ok(num) = expr.parse::<f64>()
        && let Some(n) = serde_json::Number::from_f64(num)
    {
        return JsonValue::Number(n);
    }

    // Handle boolean literals
    if expr == "true"
    {
        return JsonValue::Bool(true);
    }

    if expr == "false"
    {
        return JsonValue::Bool(false);
    }

    if expr == "null"
    {
        return JsonValue::Null;
    }

    JsonValue::Null
}

/// Evaluate expression to comparable value
/// 将表达式求值为可比较的值
fn evaluate_to_value(
    expr: &str,
    args: &HashMap<String, JsonValue>,
    result: Option<&JsonValue>,
) -> String
{
    let value = get_value(expr, args, result);

    match value
    {
        JsonValue::String(s) => s,
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Null => "null".to_string(),
        JsonValue::Array(arr) => format!("[...; {} items]", arr.len()),
        JsonValue::Object(obj) => format!("{{...; {} keys}}", obj.len()),
    }
}

/// Extract numeric value from expression
/// 从表达式提取数值
fn extract_number(
    expr: &str,
    args: &HashMap<String, JsonValue>,
    result: Option<&JsonValue>,
) -> Option<f64>
{
    let value = get_value(expr, args, result);

    match value
    {
        JsonValue::Number(n) => n.as_f64(),
        JsonValue::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

/// Check if value is truthy
/// 检查值是否为真
fn is_truthy(value: &JsonValue) -> bool
{
    match value
    {
        JsonValue::Bool(b) => *b,
        JsonValue::Number(n) => n.as_f64().is_some_and(|n| n != 0.0),
        JsonValue::String(s) => !s.is_empty(),
        JsonValue::Array(arr) => !arr.is_empty(),
        JsonValue::Object(obj) => !obj.is_empty(),
        JsonValue::Null => false,
    }
}

/// Check if value is empty
/// 检查值是否为空
fn is_empty_value(value: &JsonValue) -> bool
{
    match value
    {
        JsonValue::String(s) => s.is_empty(),
        JsonValue::Array(arr) => arr.is_empty(),
        JsonValue::Object(obj) => obj.is_empty(),
        JsonValue::Null => true,
        _ => false,
    }
}

/// Get length of value
/// 获取值的长度
fn get_length(value: &JsonValue) -> usize
{
    match value
    {
        JsonValue::String(s) => s.len(),
        JsonValue::Array(arr) => arr.len(),
        JsonValue::Object(obj) => obj.len(),
        _ => 0,
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
mod tests
{
    use super::*;

    #[test]
    fn test_simple_equality()
    {
        let mut args = HashMap::new();
        args.insert("id".to_string(), JsonValue::Number(123.into()));

        assert!(evaluate_cache_condition("#id == 123", &args, None));
        assert!(!evaluate_cache_condition("#id == 456", &args, None));
    }

    #[test]
    fn test_simple_inequality()
    {
        let mut args = HashMap::new();
        args.insert("id".to_string(), JsonValue::Number(123.into()));

        assert!(evaluate_cache_condition("#id != 456", &args, None));
        assert!(!evaluate_cache_condition("#id != 123", &args, None));
    }

    #[test]
    fn test_greater_than()
    {
        let mut args = HashMap::new();
        args.insert("age".to_string(), JsonValue::Number(25.into()));

        assert!(evaluate_cache_condition("#age > 18", &args, None));
        assert!(!evaluate_cache_condition("#age > 30", &args, None));
    }

    #[test]
    fn test_less_than()
    {
        let mut args = HashMap::new();
        args.insert("age".to_string(), JsonValue::Number(25.into()));

        assert!(evaluate_cache_condition("#age < 30", &args, None));
        assert!(!evaluate_cache_condition("#age < 20", &args, None));
    }

    #[test]
    fn test_and_expressions()
    {
        let mut args = HashMap::new();
        args.insert("age".to_string(), JsonValue::Number(25.into()));
        args.insert("active".to_string(), JsonValue::Bool(true));

        assert!(evaluate_cache_condition("#age > 18 and #active", &args, None));
        assert!(!evaluate_cache_condition("#age > 30 and #active", &args, None));
        assert!(!evaluate_cache_condition("#age > 18 and !#active", &args, None));
    }

    #[test]
    fn test_or_expressions()
    {
        let mut args = HashMap::new();
        args.insert("role".to_string(), JsonValue::String("ADMIN".to_string()));
        args.insert("admin".to_string(), JsonValue::Bool(true)); // Fixed: should be true for correct test logic

        assert!(evaluate_cache_condition("#role == 'ADMIN' or #admin", &args, None));
        assert!(evaluate_cache_condition("#role == 'USER' or #admin", &args, None));
        assert!(!evaluate_cache_condition("#role == 'USER' or !#admin", &args, None));
    }

    #[test]
    fn test_not_expressions()
    {
        let mut args = HashMap::new();
        args.insert("active".to_string(), JsonValue::Bool(false));

        assert!(evaluate_cache_condition("!#active", &args, None));
        assert!(!evaluate_cache_condition("#active", &args, None));
    }

    #[test]
    fn test_is_empty()
    {
        let mut args = HashMap::new();
        args.insert("name".to_string(), JsonValue::String(String::new()));
        args.insert("email".to_string(), JsonValue::String("test@example.com".to_string()));

        assert!(evaluate_cache_condition("#name.isEmpty()", &args, None));
        assert!(!evaluate_cache_condition("#email.isEmpty()", &args, None));
    }

    #[test]
    fn test_length_check()
    {
        let mut args = HashMap::new();
        args.insert("username".to_string(), JsonValue::String("alice".to_string()));
        args.insert("name".to_string(), JsonValue::String("Bob".to_string()));

        assert!(evaluate_cache_condition("#username.length() > 3", &args, None));
        assert!(!evaluate_cache_condition("#name.length() > 3", &args, None));
    }

    #[test]
    fn test_result_check()
    {
        let args = HashMap::new();

        // Don't cache if result is null
        assert!(evaluate_cache_condition("#result == null", &args, Some(&JsonValue::Null)));
        assert!(!evaluate_cache_condition(
            "#result == null",
            &args,
            Some(&JsonValue::String("test".to_string()))
        ));

        // Don't cache if result is empty
        assert!(evaluate_cache_condition(
            "#result.isEmpty()",
            &args,
            Some(&JsonValue::String(String::new()))
        ));
        assert!(!evaluate_cache_condition(
            "#result.isEmpty()",
            &args,
            Some(&JsonValue::String("test".to_string()))
        ));
    }

    #[test]
    fn test_complex_expressions()
    {
        let mut args = HashMap::new();
        args.insert("age".to_string(), JsonValue::Number(25.into()));
        args.insert("active".to_string(), JsonValue::Bool(true));
        args.insert("role".to_string(), JsonValue::String("USER".to_string()));

        // Cache only active users over 18
        assert!(evaluate_cache_condition("#age > 18 and #active", &args, None));

        // Cache only admins or active users
        assert!(evaluate_cache_condition("#role == 'ADMIN' or #active", &args, None));

        // Don't cache inactive users
        assert!(!evaluate_cache_condition("#age > 18 and !#active", &args, None));
    }

    #[test]
    fn test_unless_condition()
    {
        let mut args = HashMap::new();
        args.insert("id".to_string(), JsonValue::Number(1.into()));

        // Unless: don't cache if result is null
        let result = JsonValue::Null;
        assert!(evaluate_cache_condition("#result == null", &args, Some(&result)));

        // Unless: don't cache if result is empty
        let result = JsonValue::String(String::new());
        assert!(evaluate_cache_condition("#result.isEmpty()", &args, Some(&result)));

        // Unless: don't cache if id < 10
        assert!(evaluate_cache_condition("#id < 10", &args, None));
        assert!(!evaluate_cache_condition("#id > 10", &args, None));
    }
}
