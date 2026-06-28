//! Documents a stable compile error: importing a non-existent macro.
//! 记录一个稳定的编译错误：导入不存在的宏。
//!
//! `hiver-aop` exports both lowercase names (`before`, `after`, ...) and
//! uppercase aliases (`Before`, `After`, ...). It does NOT export a
//! `NonExistentAdvice` macro, so this program is expected to fail with an
//! unresolved-import error.
//! `hiver-aop` 同时导出小写名（`before`、`after`、...）和大写别名（`Before`、`After`、...）。
//! 它不导出 `NonExistentAdvice` 宏，因此该程序应因未解析的导入而编译失败。

use hiver_aop::NonExistentAdvice;

#[NonExistentAdvice("execution(* *..*.*(..))")]
fn broken()
{}

fn main() {}
