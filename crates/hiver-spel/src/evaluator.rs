//! SpEL expression evaluator.
//! SpEL 表达式求值器。

use serde_json::Value;

use crate::{
    context::SpelContext,
    parser::{self, CmpOp, SpelError, SpelExpr},
};

/// Compiles and evaluates a SpEL expression against a context.
/// 编译并针对上下文求值 SpEL 表达式。
pub struct SpelEvaluator
{
    expr: Result<SpelExpr, SpelError>,
}

impl SpelEvaluator
{
    /// Creates a new evaluator from an expression string.
    /// 从表达式字符串创建新的求值器。
    pub fn new(expression: &str) -> Self
    {
        Self {
            expr: parser::parse(expression),
        }
    }

    /// Evaluates the expression against the given context.
    /// 针对给定上下文求值表达式。
    pub fn evaluate(&self, ctx: &SpelContext) -> Result<bool, SpelError>
    {
        let expr = self
            .expr
            .as_ref()
            .map_err(|e| SpelError::Parse(e.to_string()))?;
        eval_bool(expr, ctx)
    }
}

fn eval_bool(expr: &SpelExpr, ctx: &SpelContext) -> Result<bool, SpelError>
{
    match expr
    {
        SpelExpr::HasRole(role) => Ok(ctx.has_role(role)),
        SpelExpr::HasAuthority(auth) => Ok(ctx.has_authority(auth)),
        SpelExpr::HasAnyRole(roles) => Ok(ctx.has_any_role(roles)),
        SpelExpr::PermitAll => Ok(true),
        SpelExpr::DenyAll => Ok(false),
        SpelExpr::IsAuthenticated => Ok(ctx.is_authenticated()),
        SpelExpr::IsAnonymous => Ok(ctx.is_anonymous()),
        SpelExpr::And(a, b) => Ok(eval_bool(a, ctx)? && eval_bool(b, ctx)?),
        SpelExpr::Or(a, b) => Ok(eval_bool(a, ctx)? || eval_bool(b, ctx)?),
        SpelExpr::Not(e) => Ok(!eval_bool(e, ctx)?),
        SpelExpr::Compare(left, op, right) =>
        {
            let lv = eval_value(left, ctx)?;
            let rv = eval_value(right, ctx)?;
            compare(&lv, &rv, *op)
        },
        SpelExpr::LiteralBool(b) => Ok(*b),
        _ => Err(SpelError::Evaluation("expression does not evaluate to bool".into())),
    }
}

fn eval_value(expr: &SpelExpr, ctx: &SpelContext) -> Result<Value, SpelError>
{
    match expr
    {
        SpelExpr::Variable(name) => ctx
            .get_variable(name)
            .cloned()
            .ok_or_else(|| SpelError::Evaluation(format!("undefined variable: '{name}'"))),
        SpelExpr::PropertyAccess(obj, prop) =>
        {
            let val = eval_value(obj, ctx)?;
            match &val
            {
                Value::Object(map) => map
                    .get(prop)
                    .cloned()
                    .ok_or_else(|| SpelError::Evaluation(format!("property '{prop}' not found"))),
                Value::String(s) => eval_string_property(s, prop),
                Value::Array(arr) => eval_array_property(arr, prop),
                Value::Number(n) => eval_number_property(n, prop),
                _ => Err(SpelError::Evaluation(format!(
                    "cannot access property '{prop}' on {:?}",
                    val
                ))),
            }
        },
        SpelExpr::LiteralNumber(n) => Ok(Value::from(*n)),
        SpelExpr::LiteralString(s) => Ok(Value::from(s.as_str())),
        SpelExpr::LiteralBool(b) => Ok(Value::from(*b)),
        _ => Err(SpelError::Evaluation("expected a value expression".into())),
    }
}

/// Evaluate property access on string values.
/// 对字符串值进行属性访问求值。
fn eval_string_property(s: &str, prop: &str) -> Result<Value, SpelError>
{
    match prop
    {
        "length" | "len" => Ok(Value::from(s.len() as f64)),
        "isEmpty" | "is_empty" => Ok(Value::from(s.is_empty())),
        "trim" => Ok(Value::from(s.trim())),
        "toUpper" | "toUpperCase" | "to_uppercase" => Ok(Value::from(s.to_uppercase())),
        "toLower" | "toLowerCase" | "to_lowercase" => Ok(Value::from(s.to_lowercase())),
        _ => Err(SpelError::Evaluation(format!(
            "unknown string property: '{prop}'"
        ))),
    }
}

/// Evaluate property access on array values.
/// 对数组值进行属性访问求值。
fn eval_array_property(arr: &[Value], prop: &str) -> Result<Value, SpelError>
{
    match prop
    {
        "length" | "len" | "size" => Ok(Value::from(arr.len() as f64)),
        "isEmpty" | "is_empty" => Ok(Value::from(arr.is_empty())),
        _ => Err(SpelError::Evaluation(format!(
            "unknown array property: '{prop}'"
        ))),
    }
}

/// Evaluate property access on number values.
/// 对数值进行属性访问求值。
fn eval_number_property(n: &serde_json::Number, prop: &str) -> Result<Value, SpelError>
{
    match prop
    {
        "intValue" | "as_i64" if n.is_i64() => Ok(Value::from(n.as_i64().unwrap() as f64)),
        "doubleValue" | "as_f64" if n.is_f64() => Ok(Value::from(n.as_f64().unwrap())),
        _ => Err(SpelError::Evaluation(format!(
            "unknown number property: '{prop}'"
        ))),
    }
}

#[allow(clippy::float_cmp)]
fn compare(l: &Value, r: &Value, op: CmpOp) -> Result<bool, SpelError>
{
    let ln = l.as_f64();
    let rn = r.as_f64();
    if let (Some(lv), Some(rv)) = (ln, rn)
    {
        return Ok(match op
        {
            CmpOp::Eq => lv == rv,
            CmpOp::NotEq => lv != rv,
            CmpOp::Gt => lv > rv,
            CmpOp::Lt => lv < rv,
            CmpOp::GtEq => lv >= rv,
            CmpOp::LtEq => lv <= rv,
        });
    }
    let ls = l.as_str();
    let rs = r.as_str();
    if let (Some(lv), Some(rv)) = (ls, rs)
    {
        return Ok(match op
        {
            CmpOp::Eq => lv == rv,
            CmpOp::NotEq => lv != rv,
            CmpOp::Gt => lv > rv,
            CmpOp::Lt => lv < rv,
            CmpOp::GtEq => lv >= rv,
            CmpOp::LtEq => lv <= rv,
        });
    }
    let lb = l.as_bool();
    let rb = r.as_bool();
    if let (Some(lv), Some(rv)) = (lb, rb)
    {
        return Ok(match op
        {
            CmpOp::Eq => lv == rv,
            CmpOp::NotEq => lv != rv,
            _ =>
            {
                return Err(SpelError::Evaluation(
                    "ordered comparison not supported for booleans".into(),
                ));
            },
        });
    }
    Ok(l == r)
}
