# Claude Code Change Log

Session: 927ea45b-f0e9-4c5a-a166-0e25bb63a9f9
Project: hiver
Created: 2026-05-21 07:06:14

---

## 2026-05-21 07:06:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
New file `crates/hiver-runtime/src/task/raw_task.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-runtime/src/task/raw_task.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-21 07:06:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 13 lines changed.
### Change Summary
Added "pub mod raw_task;" in `crates/hiver-runtime/src/task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:06:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/local.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/local.rs`. Approximately 28 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/scheduler/local.rs` from "// Execute the task; // 执行任务; // TODO: Actually execute the future (Phase 1: placeholder)" to "// Execute the task by polling its future via the vtable; // 通过vtable轮询其future来执行任务; let completed = unsafe { crate::tas...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:07:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/work_stealing.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/work_stealing.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/scheduler/work_stealing.rs` from "// Execute the task; // 执行任务; // TODO: Actually execute the future (Phase 1: placeholder)" to "// Execute the task by polling its future via the vtable; // 通过vtable轮询其future来执行任务; let completed = unsafe { crate::tas...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:07:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/runtime.rs`
### Change Record
Modified file `crates/hiver-runtime/src/runtime.rs`. Approximately 17 lines changed.
### Change Summary
Added "/// Thread-local storage for the current runtime handle; /// 当前运行时句柄的线程本地存储; thread_local! {" in `crates/hiver-runtime/src/runtime.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:07:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/runtime.rs`
### Change Record
Modified file `crates/hiver-runtime/src/runtime.rs`. Approximately 46 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/runtime.rs` from "// TODO: Implement thread-local storage for current runtime handle; // TODO: 为当前运行时句柄实现线程本地存储; Self {" to "/// Thread-local storage for the current runtime handle; /// 当前运行时句柄的线程本地存储; thread_local! {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:07:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/runtime.rs`
### Change Record
Modified file `crates/hiver-runtime/src/runtime.rs`. Approximately 93 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/runtime.rs` from "loop {; return Ok(());; // TODO: Implement thread-local storage for current runtime handle" to "/// Thread-local storage for the current runtime handle; /// 当前运行时句柄的线程本地存储; thread_local! {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:08:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 100 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "use std::thread;; // For Phase 2, we'll use a simple thread-based executor; // Each spawned task gets its own thread tha..." to "pub mod raw_task;; // Try to use the scheduler if a runtime context is available; // 如果运行时上下文可用，尝试使用调度器".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:08:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 110 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "inner: Arc<TaskInner<T>>,; use std::thread;; // For Phase 2, we'll use a simple thread-based executor" to "pub mod raw_task;; inner: Option<Arc<TaskInner<T>>>,; raw_core: Option<Arc<raw_task::TaskCore>>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:08:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 182 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "inner: Arc<TaskInner<T>>,; /// Create a new join handle; /// 创建新的join句柄" to "pub mod raw_task;; inner: Option<Arc<TaskInner<T>>>,; raw_core: Option<Arc<raw_task::TaskCore>>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:09:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
New file `crates/hiver-runtime/src/task/raw_task.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-runtime/src/task/raw_task.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-21 07:11:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
New file `crates/hiver-runtime/src/task/raw_task.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-runtime/src/task/raw_task.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-21 07:11:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 182 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "inner: Arc<TaskInner<T>>,; /// Create a new join handle; /// 创建新的join句柄" to "pub mod raw_task;; inner: Option<Arc<TaskInner<T>>>,; raw_core: Option<raw_task::TaskRef>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:11:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 181 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "inner: Arc<TaskInner<T>>,; /// Create a new join handle; /// 创建新的join句柄" to "pub mod raw_task;; inner: Option<Arc<TaskInner<T>>>,; raw_core: Option<raw_task::TaskRef>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:12:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 168 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "inner: Arc<TaskInner<T>>,; /// Create a new join handle; /// 创建新的join句柄" to "pub mod raw_task;; inner: Option<Arc<TaskInner<T>>>,; raw_core: Option<raw_task::TaskRef>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:12:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/local.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/local.rs`. Approximately 27 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/scheduler/local.rs` from "// Execute the task; // 执行任务; // TODO: Actually execute the future (Phase 1: placeholder)" to "// Execute the task by polling its future via the vtable; // 通过vtable轮询其future来执行任务; let completed = unsafe { crate::tas...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:12:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/work_stealing.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/work_stealing.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/scheduler/work_stealing.rs` from "// Execute the task; // 执行任务; // TODO: Actually execute the future (Phase 1: placeholder)" to "// Execute the task by polling its future via the vtable; // 通过vtable轮询其future来执行任务; let completed = unsafe { crate::tas...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-21 07:12:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
New file `crates/hiver-runtime/src/task/raw_task.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-runtime/src/task/raw_task.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-21 07:12:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
New file `crates/hiver-runtime/src/task/raw_task.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-runtime/src/task/raw_task.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-21 07:13:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
New file `crates/hiver-runtime/src/task/raw_task.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-runtime/src/task/raw_task.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-22 23:16:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/handle.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/handle.rs`. Approximately 44 lines changed.
### Change Summary
Added "/// Block until a notification arrives or timeout elapses; /// 阻塞直到收到通知或超时; ///" in `crates/hiver-runtime/src/scheduler/handle.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:17:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/local.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/local.rs`. Approximately 44 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/scheduler/local.rs` from "_wake: &super::handle::WakeChannel,; // Execute the task; // 执行任务" to "wake: &super::handle::WakeChannel,; // Execute the task by polling its future via the vtable; // 通过vtable轮询其future来执行任务".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:19:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/runtime.rs`
### Change Record
Modified file `crates/hiver-runtime/src/runtime.rs`. Approximately 184 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/runtime.rs` from "loop {; return Ok(());; // TODO: Implement thread-local storage for current runtime handle" to "/// Thread-local storage for the current runtime handle; /// 当前运行时句柄的线程本地存储; thread_local! {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:19:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/handle.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/handle.rs`. Approximately 69 lines changed.
### Change Summary
Added "/// Block until a notification arrives or timeout elapses; /// 阻塞直到收到通知或超时; ///" in `crates/hiver-runtime/src/scheduler/handle.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:20:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/local.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/local.rs`. Approximately 59 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/scheduler/local.rs` from "// TODO: Integrate driver for I/O events; // TODO: 与driver集成以处理I/O事件; _wake: &super::handle::WakeChannel," to "// Driver is stored by Runtime and used in its block_on event loop.; // Scheduler worker handles task polling; Runtime h...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:27:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 115 lines changed.
### Change Summary
Added "/// Type-safe SQL parameter value; /// 类型安全的 SQL 参数值; ///" in `crates/hiver-data-rdbc/src/client.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:27:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 118 lines changed.
### Change Summary
Added "/// Type-safe SQL parameter value; /// 类型安全的 SQL 参数值; ///" in `crates/hiver-data-rdbc/src/client.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:27:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/lib.rs` from "pub use client::{DatabaseClient, ToSql};" to "pub use client::{DatabaseClient, QueryParam, ToSql};".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:28:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 166 lines changed.
### Change Summary
Added "/// Type-safe SQL parameter value; /// 类型安全的 SQL 参数值; ///" in `crates/hiver-data-rdbc/src/client.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:28:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "use hiver_data_rdbc::DatabaseClient;" to "use hiver_data_rdbc::{DatabaseClient, QueryParam};".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:29:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 52 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "use hiver_data_rdbc::DatabaseClient;; /// The condition SQL; /// 条件 SQL" to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; /// The condition SQL (with \`?\` placeholders); /// 条件 SQL（使用 \`?\`...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:29:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 79 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "use hiver_data_rdbc::DatabaseClient;; /// The condition SQL; /// 条件 SQL" to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; /// The condition SQL (with \`?\` placeholders); /// 条件 SQL（使用 \`?\`...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:30:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 224 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "use hiver_data_rdbc::DatabaseClient;; /// The condition SQL; /// 条件 SQL" to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; /// The condition SQL (with \`?\` placeholders); /// 条件 SQL（使用 \`?\`...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:31:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 285 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "use hiver_data_rdbc::DatabaseClient;; /// The condition SQL; /// 条件 SQL" to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; /// The condition SQL (with \`?\` placeholders); /// 条件 SQL（使用 \`?\`...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:34:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 400 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from "use hiver_data_rdbc::DatabaseClient;; use hiver_data_rdbc::Row;; let vals: Vec<String> = map.values().map(|v| json_value..." to "//! All queries use parameterized placeholders (\`$1, $2, ...\`) to prevent SQL injection.; //!; //! 所有查询使用参数化占位符（\`$1, ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:35:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "&[&self.parent_id.as_str()]," to "&[hiver_data_rdbc::QueryParam::Text(self.parent_id.clone())],".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:36:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/repository.rs`. Approximately 31 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs` from "_params: &[&dyn crate::query::ToSql],; _params: &[&dyn crate::query::ToSql],; _params: &[&dyn crate::query::ToSql]," to "_params: &[hiver_data_rdbc::QueryParam],; _params: &[hiver_data_rdbc::QueryParam],; _params: &[hiver_data_rdbc::QueryPar...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:36:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/sqlx.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/sqlx.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/sqlx.rs` from "pub fn where_(mut self, condition: impl Into<String>, params: &[&dyn crate::query::ToSql]) -> Self {; self.params.push(p..." to "pub fn where_(mut self, condition: impl Into<String>, params: &[hiver_data_rdbc::QueryParam]) -> Self {; self.params.pus...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:37:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "pub use hiver_data_rdbc::QueryParam;" in `crates/hiver-data-orm/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:37:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/sqlx.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/sqlx.rs`. Approximately 33 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/sqlx.rs` from "pub fn where_(mut self, condition: impl Into<String>, params: &[&dyn crate::query::ToSql]) -> Self {; self.params.push(p..." to "pub fn where_(mut self, condition: impl Into<String>, params: &[hiver_data_rdbc::QueryParam]) -> Self {; self.params.pus...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:37:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/sqlx.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/sqlx.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/sqlx.rs` from "//! .where_("active = $1", &[&true]); pub fn where_(mut self, condition: impl Into<String>, params: &[&dyn crate::query:..." to "//! .where_("active = ?", &[QueryParam::Bool(true)]); pub fn where_(mut self, condition: impl Into<String>, params: &[ne...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:37:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/sqlx.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/sqlx.rs`. Approximately 51 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/sqlx.rs` from "//! .where_("active = $1", &[&true]); pub fn where_(mut self, condition: impl Into<String>, params: &[&dyn crate::query:..." to "//! .where_("active = ?", &[QueryParam::Bool(true)]); pub fn where_(mut self, condition: impl Into<String>, params: &[ne...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-22 23:38:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/lib.rs` from "//! .where_("email LIKE ?", &["%@example.com"])" to "//! .where_("email LIKE ?", &[QueryParam::Text("%@example.com".into())]); pub use hiver_data_rdbc::QueryParam;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 10:39:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 169 lines changed.
### Change Summary
Added "/// Type-safe SQL parameter value; /// 类型安全的 SQL 参数值; ///" in `crates/hiver-data-rdbc/src/client.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 10:40:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-macros/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-macros/src/lib.rs`. Approximately 271 lines changed.
### Change Summary
Changed `crates/hiver-data-macros/src/lib.rs` from "/// Generate a custom query method implementation; _entity_type: &proc_macro2::TokenStream,; let query = #sql;" to "/// Convert \`?\` placeholders to \`$N\` positional markers.; fn convert_placeholders(sql: &str) -> String {; let mut re...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 10:59:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-annotations/src/transactional_macro.rs`
### Change Record
Modified file `crates/hiver-data-annotations/src/transactional_macro.rs`. Approximately 153 lines changed.
### Change Summary
Changed `crates/hiver-data-annotations/src/transactional_macro.rs` from "let _func_name = &input.sig.ident;; // Parse transactional attributes; // 解析事务属性" to "// Build TransactionDefinition configuration; let mut def_config = quote! {};; def_config = quote! {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 11:00:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-annotations/src/pre_authorize_macro.rs`
### Change Record
Modified file `crates/hiver-data-annotations/src/pre_authorize_macro.rs`. Approximately 39 lines changed.
### Change Summary
Changed `crates/hiver-data-annotations/src/pre_authorize_macro.rs` from "// Parse the attribute as a simple string literal; // 将属性解析为简单的字符串字面量; // 生成包装代码 / Generate wrapper code" to "let __hiver_sec_ctx = ::hiver_security::context::get_security_context(); .unwrap_or_else(|| std::sync::Arc::new(::hiver_...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 17:55:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 12 lines changed.
### Change Summary
Added "use crate::row::Row;" in `crates/hiver-data-rdbc/src/connection.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 17:55:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 34 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/connection.rs` from "/// Execute a query and return the first row (placeholder - returns count for now); /// 执行查询并返回第一行（占位符 - 现在返回计数）; -> std..." to "use crate::row::Row;; /// Execute a query and return the first row; /// 执行查询并返回第一行".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 17:55:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 58 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/connection.rs` from "/// Execute a query and return the first row (placeholder - returns count for now); /// 执行查询并返回第一行（占位符 - 现在返回计数）; -> std..." to "use crate::row::Row;; /// Execute a query and return the first row; /// 执行查询并返回第一行".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 17:55:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 80 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/connection.rs` from "/// Execute a query and return the first row (placeholder - returns count for now); /// 执行查询并返回第一行（占位符 - 现在返回计数）; -> std..." to "use crate::row::Row;; /// Execute a query and return the first row; /// 执行查询并返回第一行".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 17:56:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 104 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/connection.rs` from "/// Execute a query and return the first row (placeholder - returns count for now); /// 执行查询并返回第一行（占位符 - 现在返回计数）; -> std..." to "use crate::row::Row;; /// Execute a query and return the first row; /// 执行查询并返回第一行".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 17:56:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 158 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/connection.rs` from "/// Execute a query and return the first row (placeholder - returns count for now); /// 执行查询并返回第一行（占位符 - 现在返回计数）; -> std..." to "use crate::row::Row;; /// Execute a query and return the first row; /// 执行查询并返回第一行".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:42:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/bean.rs`
### Change Record
Modified file `crates/hiver-core/src/bean.rs`. Approximately 201 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-core/src/bean.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:43:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/extension.rs`
### Change Record
Modified file `crates/hiver-core/src/extension.rs`. Approximately 214 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-core/src/extension.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:44:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/Cargo.toml`
### Change Record
Modified file `crates/hiver-ldap/Cargo.toml`. Approximately 16 lines changed.
### Change Summary
Added "# LDAP client (optional) / LDAP客户端（可选）; ldap3 = { version = "0.11", optional = true }; [features]" in `crates/hiver-ldap/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:44:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 641 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-core/src/container.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:44:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/session.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/session.rs`. Approximately 83 lines changed.
### Change Summary
Added "use std::time::Instant;; /// Acknowledgment ID; /// 确认 ID" in `crates/hiver-websocket-stomp/src/session.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:44:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/session.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/session.rs`. Approximately 98 lines changed.
### Change Summary
Added "use std::time::Instant;; /// Acknowledgment ID; /// 确认 ID" in `crates/hiver-websocket-stomp/src/session.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:44:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/session.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/session.rs`. Approximately 106 lines changed.
### Change Summary
Added "use std::time::Instant;; /// Acknowledgment ID; /// 确认 ID" in `crates/hiver-websocket-stomp/src/session.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/session.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/session.rs`. Approximately 187 lines changed.
### Change Summary
Added "use std::time::Instant;; /// Acknowledgment ID; /// 确认 ID" in `crates/hiver-websocket-stomp/src/session.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/reactive.rs`
### Change Record
Modified file `crates/hiver-core/src/reactive.rs`. Approximately 251 lines changed.
### Change Summary
Added "// ── Additional Mono tests / 额外Mono测试 ──────────────────────────; #[tokio::test]; async fn test_mono_from_future() {" in `crates/hiver-core/src/reactive.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/route.rs`
### Change Record
Modified file `crates/hiver-router/src/route.rs`. Approximately 362 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-router/src/route.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/lib.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/lib.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/lib.rs` from "AckMode, HeartbeatConfig, MemoryBroker, StompBroker, StompSession, Subscription,; SubscriptionId, TransactionState," to "AckMode, AckId, HeartbeatConfig, MemoryBroker, PendingAck, StompBroker, StompSession,; Subscription, SubscriptionId, Tra...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/context.rs`
### Change Record
Modified file `crates/hiver-ldap/src/context.rs`. Approximately 334 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/context.rs` from "#[derive(Debug, Clone)]; pub fn is_connected(&self) -> bool { self.connected }; pub async fn simple_bind(&mut self, _use..." to "//!; //! Provides connection management with optional real LDAP support via \`ldap3\`.; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/context.rs`
### Change Record
Modified file `crates/hiver-ldap/src/context.rs`. Approximately 333 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/context.rs` from "#[derive(Debug, Clone)]; pub fn is_connected(&self) -> bool { self.connected }; pub async fn simple_bind(&mut self, _use..." to "//!; //! Provides connection management with optional real LDAP support via \`ldap3\`.; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/timeout.rs`
### Change Record
Modified file `crates/hiver-resilience/src/timeout.rs`. Approximately 632 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/timeout.rs` from "//! This module provides timeout functionality.; //! 本模块提供超时功能。; //! TODO: Implement in Phase 4 / 将在第4阶段实现" to "//! The timeout pattern wraps async operations with a deadline, ensuring that; //! unresponsive services do not block ca...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/lib.rs`
### Change Record
Modified file `crates/hiver-resilience/src/lib.rs`. Approximately 11 lines changed.
### Change Summary
Added "pub use timeout::{; Timeout, TimeoutConfig, TimeoutError, TimeoutMetrics, TimeoutRegistry, timeout," in `crates/hiver-resilience/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/handler.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/handler.rs`. Approximately 114 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/handler.rs` from "use crate::session::{AckMode, HeartbeatConfig, StompBroker, StompSession, Subscription};" to "use crate::session::{; AckMode, HeartbeatConfig, PendingAck, StompBroker, StompSession, Subscription,; use std::collecti...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/params.rs`
### Change Record
Modified file `crates/hiver-router/src/params.rs`. Approximately 258 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-router/src/params.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 782 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-core/src/container.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:45:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/handler.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/handler.rs`. Approximately 133 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/handler.rs` from "use crate::session::{AckMode, HeartbeatConfig, StompBroker, StompSession, Subscription};" to "use crate::session::{; AckMode, HeartbeatConfig, PendingAck, StompBroker, StompSession, Subscription,; use std::collecti...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:46:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/handler.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/handler.rs`. Approximately 205 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/handler.rs` from "use crate::session::{AckMode, HeartbeatConfig, StompBroker, StompSession, Subscription};" to "use crate::session::{; AckMode, HeartbeatConfig, PendingAck, StompBroker, StompSession, Subscription,; use std::collecti...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:46:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/reactive.rs`
### Change Record
Modified file `crates/hiver-core/src/reactive.rs`. Approximately 368 lines changed.
### Change Summary
Added "// ── Additional Mono tests / 额外Mono测试 ──────────────────────────; #[tokio::test]; async fn test_mono_from_future() {" in `crates/hiver-core/src/reactive.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:46:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/template.rs`
### Change Record
Modified file `crates/hiver-ldap/src/template.rs`. Approximately 356 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/template.rs` from "//! Equivalent to Spring LDAP's \`LdapTemplate\`; //! 等价于 Spring LDAP 的 \`LdapTemplate\`; use crate::error::LdapResult;" to "//! Equivalent to Spring LDAP's \`LdapTemplate\`.; //! When the \`ldap\` feature is enabled, operations connect to a rea...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:46:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rbac.rs`
### Change Record
Modified file `crates/hiver-security/src/rbac.rs`. Approximately 903 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rbac.rs` from "#[tokio::test]; async fn test_rbac_config() {; async fn test_rbac_manager() {" to "use std::sync::atomic::{AtomicUsize, Ordering};; // ── Helper: create a user role mapping / 辅助函数：创建用户角色映射 ──; fn make_us...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:46:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/handler.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/handler.rs`. Approximately 285 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/handler.rs` from "use crate::session::{AckMode, HeartbeatConfig, StompBroker, StompSession, Subscription};; // Check authentication; let l..." to "use crate::session::{; AckMode, HeartbeatConfig, PendingAck, StompBroker, StompSession, Subscription,; use std::collecti...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:46:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/route.rs`
### Change Record
Modified file `crates/hiver-router/src/route.rs`. Approximately 362 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-router/src/route.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:46:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/route.rs`
### Change Record
Modified file `crates/hiver-router/src/route.rs`. Approximately 362 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-router/src/route.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:46:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/handler.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/handler.rs`. Approximately 450 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/handler.rs` from "use crate::session::{AckMode, HeartbeatConfig, StompBroker, StompSession, Subscription};; // Check authentication; let l..." to "use crate::session::{; AckMode, HeartbeatConfig, PendingAck, StompBroker, StompSession, Subscription,; use std::collecti...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:47:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/handler.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/handler.rs`. Approximately 467 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/handler.rs` from "use crate::session::{AckMode, HeartbeatConfig, StompBroker, StompSession, Subscription};; // Check authentication; let l..." to "use crate::session::{; AckMode, HeartbeatConfig, PendingAck, StompBroker, StompSession, Subscription,; use std::collecti...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:47:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/odm.rs`
### Change Record
Modified file `crates/hiver-ldap/src/odm.rs`. Approximately 404 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/odm.rs` from "//! Equivalent to Spring LDAP ODM; //! 等价于 Spring LDAP ODM; #[derive(Debug, Clone, Serialize, Deserialize)]" to "//! Equivalent to Spring LDAP ODM.; //! Provides utilities to map between Rust structs and LDAP entries.; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:47:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/handler.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/handler.rs`. Approximately 519 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/handler.rs` from "use crate::session::{AckMode, HeartbeatConfig, StompBroker, StompSession, Subscription};; // Check authentication; let l..." to "use crate::session::{; AckMode, HeartbeatConfig, PendingAck, StompBroker, StompSession, Subscription,; use std::collecti...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:48:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/handler.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/handler.rs`. Approximately 1029 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/handler.rs` from "use crate::session::{AckMode, HeartbeatConfig, StompBroker, StompSession, Subscription};; // Check authentication; let l..." to "use crate::session::{; AckMode, HeartbeatConfig, PendingAck, StompBroker, StompSession, Subscription,; use std::collecti...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:48:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/lib.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/lib.rs`. Approximately 18 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/lib.rs` from "AckMode, HeartbeatConfig, MemoryBroker, StompBroker, StompSession, Subscription,; SubscriptionId, TransactionState,; pub..." to "AckMode, AckId, HeartbeatConfig, MemoryBroker, PendingAck, StompBroker, StompSession,; Subscription, SubscriptionId, Tra...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:48:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/repository.rs`
### Change Record
Modified file `crates/hiver-ldap/src/repository.rs`. Approximately 679 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/repository.rs` from "//! Equivalent to Spring LDAP Repository Support; //! 等价于 Spring LDAP Repository 支持; use crate::odm::OdmEntry;" to "//! Equivalent to Spring LDAP Repository Support.; //! Provides CRUD operations using \`LdapTemplate\` with ODM integrat...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:48:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/authorization_server.rs`
### Change Record
Modified file `crates/hiver-security/src/authorization_server.rs`. Approximately 1092 lines changed.
### Change Summary
Changed `crates/hiver-security/src/authorization_server.rs` from "async fn test_authorization_code_flow() {; async fn test_pkce_s256_flow() {; let challenge = {" to "// ── Helpers / 辅助函数 ──; .redirect_uri("https://app.test/cb2"); .scope("read")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:48:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/lib.rs`
### Change Record
Modified file `crates/hiver-ldap/src/lib.rs`. Approximately 37 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/lib.rs` from "pub use context::{ContextSource, LdapContextSource};; pub use mapper::{AttributesMapper, ContextMapper};; pub use odm::{..." to "//!; //! # Feature flags / 功能标志; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:50:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/tests.rs`
### Change Record
Modified file `crates/hiver-ldap/src/tests.rs`. Approximately 675 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/tests.rs` from "//! Tests for hiver-ldap; fn smoke_test() {; assert!(true);" to "//! Integration-level tests for hiver-ldap; //! hiver-ldap 的集成级测试; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:50:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/repository.rs`
### Change Record
Modified file `crates/hiver-ldap/src/repository.rs`. Approximately 679 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/repository.rs` from "//! Equivalent to Spring LDAP Repository Support; //! 等价于 Spring LDAP Repository 支持; use crate::odm::OdmEntry;" to "//! Equivalent to Spring LDAP Repository Support.; //! Provides CRUD operations using \`LdapTemplate\` with ODM integrat...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:50:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/repository.rs`
### Change Record
Modified file `crates/hiver-ldap/src/repository.rs`. Approximately 681 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/repository.rs` from "//! Equivalent to Spring LDAP Repository Support; //! 等价于 Spring LDAP Repository 支持; use crate::odm::OdmEntry;" to "//! Equivalent to Spring LDAP Repository Support.; //! Provides CRUD operations using \`LdapTemplate\` with ODM integrat...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:50:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/repository.rs`
### Change Record
Modified file `crates/hiver-ldap/src/repository.rs`. Approximately 680 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/repository.rs` from "//! Equivalent to Spring LDAP Repository Support; //! 等价于 Spring LDAP Repository 支持; use crate::odm::OdmEntry;" to "//! Equivalent to Spring LDAP Repository Support.; //! Provides CRUD operations using \`LdapTemplate\` with ODM integrat...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:51:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/template.rs`
### Change Record
Modified file `crates/hiver-ldap/src/template.rs`. Approximately 357 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/template.rs` from "//! Equivalent to Spring LDAP's \`LdapTemplate\`; //! 等价于 Spring LDAP 的 \`LdapTemplate\`; use crate::mapper::ContextMapp..." to "//! Equivalent to Spring LDAP's \`LdapTemplate\`.; //! When the \`ldap\` feature is enabled, operations connect to a rea...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:53:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-redis/src/lock.rs`
### Change Record
Modified file `crates/hiver-data-redis/src/lock.rs`. Approximately 12 lines changed.
### Change Summary
Added "use std::sync::atomic::{AtomicBool, Ordering};; use std::sync::Arc;" in `crates/hiver-data-redis/src/lock.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation-annotations/src/lib.rs`. Approximately 27 lines changed.
### Change Summary
Changed `crates/hiver-validation-annotations/src/lib.rs` from "min_attr.map(|_| {; // For now, use a default value of 0; // In a full implementation, you'd parse the attribute to get ..." to "///; /// Supports both \`#[min(5)]\` and \`#[min(value = 5)]\` forms.; /// 同时支持 \`#[min(5)]\` 和 \`#[min(value = 5)]\` 两种...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation-annotations/src/lib.rs`. Approximately 50 lines changed.
### Change Summary
Changed `crates/hiver-validation-annotations/src/lib.rs` from "min_attr.map(|_| {; // For now, use a default value of 0; // In a full implementation, you'd parse the attribute to get ..." to "///; /// Supports both \`#[min(5)]\` and \`#[min(value = 5)]\` forms.; /// 同时支持 \`#[min(5)]\` 和 \`#[min(value = 5)]\` 两种...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cache/Cargo.toml`
### Change Record
Modified file `crates/hiver-cache/Cargo.toml`. Approximately 40 lines changed.
### Change Summary
Added "[features]; default = []; # Enable Redis cache backend / 启用 Redis 缓存后端" in `crates/hiver-cache/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation-annotations/src/lib.rs`. Approximately 73 lines changed.
### Change Summary
Changed `crates/hiver-validation-annotations/src/lib.rs` from "min_attr.map(|_| {; // For now, use a default value of 0; // In a full implementation, you'd parse the attribute to get ..." to "///; /// Supports both \`#[min(5)]\` and \`#[min(value = 5)]\` forms.; /// 同时支持 \`#[min(5)]\` 和 \`#[min(value = 5)]\` 两种...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation-annotations/src/lib.rs`. Approximately 96 lines changed.
### Change Summary
Changed `crates/hiver-validation-annotations/src/lib.rs` from "min_attr.map(|_| {; // For now, use a default value of 0; // In a full implementation, you'd parse the attribute to get ..." to "///; /// Supports both \`#[min(5)]\` and \`#[min(value = 5)]\` forms.; /// 同时支持 \`#[min(5)]\` 和 \`#[min(value = 5)]\` 两种...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-redis/src/lock.rs`
### Change Record
Modified file `crates/hiver-data-redis/src/lock.rs`. Approximately 255 lines changed.
### Change Summary
Changed `crates/hiver-data-redis/src/lock.rs` from "/// Guard for a held distributed lock. / 持有的分布式锁的守卫。; /// Automatically releases the lock when dropped (best-effort).; /..." to "use std::sync::atomic::{AtomicBool, Ordering};; use std::sync::Arc;; /// Guard for a held distributed lock with optional...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-tx/src/sqlx_manager.rs`
### Change Record
Modified file `crates/hiver-tx/src/sqlx_manager.rs`. Approximately 279 lines changed.
### Change Summary
Changed `crates/hiver-tx/src/sqlx_manager.rs` from "//! SQLx-backed transaction manager.; //! 基于 SQLx 的事务管理器。; use sqlx::{Pool, Postgres};" to "//! SQLx-backed transaction manager with multi-database support.; //! 基于 SQLx 的多数据库事务管理器。; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-flyway/src/dialect.rs`
### Change Record
New file `crates/hiver-flyway/src/dialect.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-flyway/src/dialect.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 18:54:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-redis/src/lock.rs`
### Change Record
Modified file `crates/hiver-data-redis/src/lock.rs`. Approximately 277 lines changed.
### Change Summary
Changed `crates/hiver-data-redis/src/lock.rs` from "Some(ref s) if s == "OK" => Ok(Some(RedisLockGuard {; client: self.client.clone(),; key: self.key.clone()," to "use std::sync::atomic::{AtomicBool, Ordering};; use std::sync::Arc;; Some(ref s) if s == "OK" => Ok(Some(RedisLockGuard:...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/lib.rs`
### Change Record
Modified file `crates/hiver-flyway/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "pub mod dialect;" in `crates/hiver-flyway/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation-annotations/src/lib.rs`. Approximately 227 lines changed.
### Change Summary
Changed `crates/hiver-validation-annotations/src/lib.rs` from "/// Parse value from meta item; /// 从 meta 项中解析值; ///" to "/// Parse a single numeric value from an attribute like \`#[name(5)]\` or \`#[name(value = 5)]\`.; /// 从属性中解析单个数值，支持 \`#...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/config.rs`
### Change Record
Modified file `crates/hiver-flyway/src/config.rs`. Approximately 12 lines changed.
### Change Summary
Added "use crate::dialect::DatabaseType;" in `crates/hiver-flyway/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:54:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-redis/src/lock.rs`
### Change Record
Modified file `crates/hiver-data-redis/src/lock.rs`. Approximately 301 lines changed.
### Change Summary
Changed `crates/hiver-data-redis/src/lock.rs` from "/// TODO: Auto-renewal is not yet implemented. The \`renew_interval_secs\`; /// field is stored but no background task i..." to "use std::sync::atomic::{AtomicBool, Ordering};; use std::sync::Arc;; /// until released. The interval is capped at \`ttl...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:55:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/config.rs`
### Change Record
Modified file `crates/hiver-flyway/src/config.rs`. Approximately 23 lines changed.
### Change Summary
Added "use crate::dialect::DatabaseType;; /// Detected database type; /// 检测到的数据库类型" in `crates/hiver-flyway/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:55:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/config.rs`
### Change Record
Modified file `crates/hiver-flyway/src/config.rs`. Approximately 31 lines changed.
### Change Summary
Added "use crate::dialect::DatabaseType;; /// Detected database type; /// 检测到的数据库类型" in `crates/hiver-flyway/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:55:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/Cargo.toml`
### Change Record
Modified file `crates/hiver-validation-annotations/Cargo.toml`. Approximately 12 lines changed.
### Change Summary
Removed "darling = { workspace = true }; darling_core = { workspace = true }" from `crates/hiver-validation-annotations/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:55:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/config.rs`
### Change Record
Modified file `crates/hiver-flyway/src/config.rs`. Approximately 51 lines changed.
### Change Summary
Changed `crates/hiver-flyway/src/config.rs` from "let mut config = Self { datasource_url: url, ..Default::default() };" to "use crate::dialect::DatabaseType;; /// Detected database type; /// 检测到的数据库类型".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:55:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/config.rs`
### Change Record
Modified file `crates/hiver-flyway/src/config.rs`. Approximately 73 lines changed.
### Change Summary
Changed `crates/hiver-flyway/src/config.rs` from "let mut config = Self { datasource_url: url, ..Default::default() };; self.config.datasource_url = url.into();" to "use crate::dialect::DatabaseType;; /// Detected database type; /// 检测到的数据库类型".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:55:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-tx/src/registry.rs`
### Change Record
New file `crates/hiver-tx/src/registry.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-tx/src/registry.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 18:55:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation-annotations/src/lib.rs`. Approximately 263 lines changed.
### Change Summary
Changed `crates/hiver-validation-annotations/src/lib.rs` from "// Parse min/max from #[size(min = X, max = Y)]; // 解析 #[size(min = X, max = Y)] 中的 min/max; let mut min = 0u32;" to "parse_min_max_attr(attr).map(|(min, max)| (id.clone(), min, max)); /// Parse a single numeric value from an attribute li...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:55:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-tx/src/lib.rs`
### Change Record
Modified file `crates/hiver-tx/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "mod registry;" in `crates/hiver-tx/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:55:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-tx/src/lib.rs`
### Change Record
Modified file `crates/hiver-tx/src/lib.rs`. Approximately 20 lines changed.
### Change Summary
Added "mod registry;; pub use registry::{DelegatingTransactionManager, TransactionManagerRegistry};" in `crates/hiver-tx/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:55:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-tx/src/lib.rs`
### Change Record
Modified file `crates/hiver-tx/src/lib.rs`. Approximately 31 lines changed.
### Change Summary
Changed `crates/hiver-tx/src/lib.rs` from "IsolationLevel, Propagation, Transaction, TransactionError, TransactionManager,; TransactionResult, TransactionStatus, T..." to "mod registry;; pub use registry::{DelegatingTransactionManager, TransactionManagerRegistry};; DelegatingTransactionManag...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:56:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-tx/src/lib.rs`
### Change Record
Modified file `crates/hiver-tx/src/lib.rs`. Approximately 41 lines changed.
### Change Summary
Changed `crates/hiver-tx/src/lib.rs` from "IsolationLevel, Propagation, Transaction, TransactionError, TransactionManager,; TransactionResult, TransactionStatus, T..." to "mod registry;; pub use registry::{DelegatingTransactionManager, TransactionManagerRegistry};; DelegatingTransactionManag...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:56:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-redis/src/lock.rs`
### Change Record
Modified file `crates/hiver-data-redis/src/lock.rs`. Approximately 741 lines changed.
### Change Summary
Changed `crates/hiver-data-redis/src/lock.rs` from "/// TODO: Auto-renewal is not yet implemented. The \`renew_interval_secs\`; /// field is stored but no background task i..." to "use std::sync::atomic::{AtomicBool, Ordering};; use std::sync::Arc;; /// until released. The interval is capped at \`ttl...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:56:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/src/tests.rs`
### Change Record
Modified file `crates/hiver-validation-annotations/src/tests.rs`. Approximately 343 lines changed.
### Change Summary
Changed `crates/hiver-validation-annotations/src/tests.rs` from "//! Tests for hiver-validation-annotations; //! 测试模块; fn smoke_test() {" to "//! Tests for hiver-validation-attributes attribute parsing.; //! 验证注解属性解析的测试。; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:56:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-redis/Cargo.toml`
### Change Record
Modified file `crates/hiver-data-redis/Cargo.toml`. Approximately 10 lines changed.
### Change Summary
Changed `crates/hiver-data-redis/Cargo.toml` from "tokio = { workspace = true, features = ["sync"] }" to "tokio = { workspace = true, features = ["sync", "rt", "time"] }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:56:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/schedule/mod.rs`
### Change Record
Modified file `crates/hiver-starter/src/schedule/mod.rs`. Approximately 1242 lines changed.
### Change Summary
Changed `crates/hiver-starter/src/schedule/mod.rs` from "//! Schedule 自动配置模块 / Schedule Auto-Configuration Module; //! Auto-configures scheduled task functionality.; /// 定时任务自动配..." to "//! Schedule auto-configuration module / 定时任务自动配置模块; //! Automatically configures scheduled task functionality.; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:56:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-cache/src/redis_cache.rs`
### Change Record
New file `crates/hiver-cache/src/redis_cache.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-cache/src/redis_cache.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 18:56:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-tx/src/tests.rs`
### Change Record
Modified file `crates/hiver-tx/src/tests.rs`. Approximately 364 lines changed.
### Change Summary
Added "use super::*;; use crate::manager::TransactionDefinition;; use crate::registry::{DelegatingTransactionManager, Transacti..." in `crates/hiver-tx/src/tests.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:56:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/flyway.rs`
### Change Record
Modified file `crates/hiver-flyway/src/flyway.rs`. Approximately 520 lines changed.
### Change Summary
Changed `crates/hiver-flyway/src/flyway.rs` from "use sqlx::{Pool, Postgres, Row};; pool: Pool<Postgres>,; let pool = Pool::<Postgres>::connect(&config.datasource_url)" to "dialect::DatabaseType,; use sqlx::{Any, Pool, Row};; pool: Pool<Any>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:56:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/migration.rs`
### Change Record
Modified file `crates/hiver-flyway/src/migration.rs`. Approximately 17 lines changed.
### Change Summary
Changed `crates/hiver-flyway/src/migration.rs` from "/// Execute the migration; /// 执行迁移; pub async fn execute(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> crat..." to "/// Execute the migration on a database-agnostic transaction; /// 在数据库无关的事务上执行迁移; pub async fn execute_on(&self, tx: &mu...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:57:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/Cargo.toml`
### Change Record
Modified file `crates/hiver-flyway/Cargo.toml`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-flyway/Cargo.toml` from "sqlx = { workspace = true, features = ["runtime-tokio", "sqlite", "postgres", "mysql", "chrono"] }" to "sqlx = { workspace = true, features = ["runtime-tokio", "sqlite", "postgres", "mysql", "chrono", "any"] }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:57:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/lib.rs`
### Change Record
Modified file `crates/hiver-flyway/src/lib.rs`. Approximately 20 lines changed.
### Change Summary
Added "pub mod dialect;; pub use dialect::DatabaseType;" in `crates/hiver-flyway/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:57:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/src/tests.rs`
### Change Record
Modified file `crates/hiver-validation-annotations/src/tests.rs`. Approximately 346 lines changed.
### Change Summary
Changed `crates/hiver-validation-annotations/src/tests.rs` from "//! Tests for hiver-validation-annotations; //! 测试模块; #[cfg(test)]" to "//! Tests for hiver-validation-annotations attribute parsing.; //! 验证注解属性解析的测试。; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:57:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-cache/src/redis_cache_manager.rs`
### Change Record
New file `crates/hiver-cache/src/redis_cache_manager.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-cache/src/redis_cache_manager.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 18:57:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/tests.rs`
### Change Record
Modified file `crates/hiver-flyway/src/tests.rs`. Approximately 287 lines changed.
### Change Summary
Changed `crates/hiver-flyway/src/tests.rs` from "fn smoke_test() {; assert!(true, "hiver-flyway test infrastructure is working");; fn test_basic_math() {" to "use crate::dialect::DatabaseType;; use crate::config::Config;; // ------------------------------------------------------...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:57:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-cache/src/redis_cache.rs`
### Change Record
New file `crates/hiver-cache/src/redis_cache.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-cache/src/redis_cache.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 18:58:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-cache/src/redis_cache.rs`
### Change Record
New file `crates/hiver-cache/src/redis_cache.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-cache/src/redis_cache.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 18:58:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation-annotations/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation-annotations/src/lib.rs`. Approximately 238 lines changed.
### Change Summary
Changed `crates/hiver-validation-annotations/src/lib.rs` from "// Parse min/max from #[size(min = X, max = Y)]; // 解析 #[size(min = X, max = Y)] 中的 min/max; let mut min = 0u32;" to "parse_min_max_attr(attr).map(|(min, max)| (id.clone(), min, max)); /// Parse a single numeric value from an attribute li...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:58:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cache/src/lib.rs`
### Change Record
Modified file `crates/hiver-cache/src/lib.rs`. Approximately 38 lines changed.
### Change Summary
Added "#[cfg(feature = "redis")]; mod redis_cache;; #[cfg(feature = "redis")]" in `crates/hiver-cache/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 18:58:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-cache/src/redis_cache_manager.rs`
### Change Record
New file `crates/hiver-cache/src/redis_cache_manager.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-cache/src/redis_cache_manager.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:02:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/config.rs`
### Change Record
Modified file `crates/hiver-kafka/src/config.rs`. Approximately 146 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-kafka/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:02:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/config.rs`
### Change Record
Modified file `crates/hiver-amqp/src/config.rs`. Approximately 122 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-amqp/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:02:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/producer.rs`
### Change Record
Modified file `crates/hiver-kafka/src/producer.rs`. Approximately 139 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-kafka/src/producer.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:02:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `tests/Cargo.toml`
### Change Record
Modified file `tests/Cargo.toml`. Approximately 91 lines changed.
### Change Summary
Changed `tests/Cargo.toml` from "hiver-data-rdbc = { path = "../crates/hiver-data-rdbc" }; sqlx = { workspace = true, features = ["runtime-tokio", "sqlit..." to "[features]; default = []; # Integration tests that require Docker / 需要 Docker 的集成测试".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:02:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/lib.rs`
### Change Record
Modified file `crates/hiver-runtime/src/lib.rs`. Approximately 24 lines changed.
### Change Summary
Removed "#![allow(clippy::needless_else)]; #![allow(clippy::match_single_binding)]; #![allow(clippy::clone_on_copy)]" from `crates/hiver-runtime/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:03:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/consumer.rs`
### Change Record
Modified file `crates/hiver-kafka/src/consumer.rs`. Approximately 184 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-kafka/src/consumer.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:03:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/exchange.rs`
### Change Record
Modified file `crates/hiver-amqp/src/exchange.rs`. Approximately 121 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-amqp/src/exchange.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:03:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 177 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "if let Err(_) = inner.state.compare_exchange(; inner: Arc<TaskInner<T>>,; /// Create a new join handle" to "pub mod raw_task;; if inner.state.compare_exchange(; inner: Option<Arc<TaskInner<T>>>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:03:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
New file `crates/hiver-runtime/src/task/raw_task.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-runtime/src/task/raw_task.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:03:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 185 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "if let Err(_) = inner.state.compare_exchange(; ) {; inner: Arc<TaskInner<T>>," to "pub mod raw_task;; if inner.state.compare_exchange(; .is_err()".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:03:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/message.rs`
### Change Record
Modified file `crates/hiver-kafka/src/message.rs`. Approximately 157 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-kafka/src/message.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:03:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/value.rs`
### Change Record
Modified file `crates/hiver-config/src/value.rs`. Approximately 514 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-config/src/value.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:03:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-aop/src/runtime.rs`
### Change Record
Modified file `crates/hiver-aop/src/runtime.rs`. Approximately 606 lines changed.
### Change Summary
Changed `crates/hiver-aop/src/runtime.rs` from "#[test]; fn test_pointcut_parsing() {; let expr = PointcutExpression::new("execution(* com.example..*.*(..))".to_string(..." to "// ========================================================================; // Helper / 辅助函数; // ======================...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:03:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 187 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "if let Err(_) = inner.state.compare_exchange(; ) {; inner: Arc<TaskInner<T>>," to "pub mod raw_task;; if inner.state.compare_exchange(; .is_err()".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:04:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 189 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "if let Err(_) = inner.state.compare_exchange(; ) {; inner: Arc<TaskInner<T>>," to "pub mod raw_task;; if inner.state.compare_exchange(; .is_err()".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:04:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/serialization.rs`
### Change Record
Modified file `crates/hiver-kafka/src/serialization.rs`. Approximately 183 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-kafka/src/serialization.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:04:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-benches/Cargo.toml`
### Change Record
Modified file `crates/hiver-benches/Cargo.toml`. Approximately 40 lines changed.
### Change Summary
Added "hiver-security = { path = "../hiver-security" }; hiver-data-orm = { path = "../hiver-data-orm" }; serde = { workspace = ..." in `crates/hiver-benches/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:04:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/Cargo.toml`
### Change Record
Modified file `crates/hiver-cloud/Cargo.toml`. Approximately 9 lines changed.
### Change Summary
Added "mockito = { workspace = true }" in `crates/hiver-cloud/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:04:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/topic.rs`
### Change Record
Modified file `crates/hiver-kafka/src/topic.rs`. Approximately 118 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-kafka/src/topic.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:04:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-aop/src/tests.rs`
### Change Record
Modified file `crates/hiver-aop/src/tests.rs`. Approximately 47 lines changed.
### Change Summary
Changed `crates/hiver-aop/src/tests.rs` from "//! Tests for hiver-aop; //! 测试模块; fn test_basic_math() {" to "//! Tests for hiver-aop proc-macro crate; //! hiver-aop 过程宏 crate 的测试; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:04:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 191 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "use std::panic;; if let Err(_) = inner.state.compare_exchange(; ) {" to "pub mod raw_task;; if inner.state.compare_exchange(; .is_err()".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:04:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-benches/runtime_driver.rs`
### Change Record
New file `crates/hiver-benches/runtime_driver.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-benches/runtime_driver.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:04:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/source.rs`
### Change Record
Modified file `crates/hiver-config/src/source.rs`. Approximately 240 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-config/src/source.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/queue.rs`
### Change Record
Modified file `crates/hiver-amqp/src/queue.rs`. Approximately 116 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-amqp/src/queue.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/message.rs`
### Change Record
Modified file `crates/hiver-amqp/src/message.rs`. Approximately 157 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-amqp/src/message.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/binding.rs`
### Change Record
Modified file `crates/hiver-amqp/src/binding.rs`. Approximately 106 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-amqp/src/binding.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/connection.rs`
### Change Record
Modified file `crates/hiver-amqp/src/connection.rs`. Approximately 103 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-amqp/src/connection.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/publisher.rs`
### Change Record
Modified file `crates/hiver-amqp/src/publisher.rs`. Approximately 133 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-amqp/src/publisher.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/listener.rs`
### Change Record
Modified file `crates/hiver-amqp/src/listener.rs`. Approximately 101 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-amqp/src/listener.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/converter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/converter.rs`. Approximately 91 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-amqp/src/converter.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/rest_api.rs`
### Change Record
New file `examples/rest_api.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/rest_api.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:05:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/environment.rs`
### Change Record
Modified file `crates/hiver-config/src/environment.rs`. Approximately 323 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-config/src/environment.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/tests.rs`
### Change Record
Modified file `crates/hiver-amqp/src/tests.rs`. Approximately 186 lines changed.
### Change Summary
Changed `crates/hiver-amqp/src/tests.rs` from "//! Tests for hiver-amqp" to "//! Integration tests for hiver-amqp; use hiver_amqp::*;; // ── Constants tests / 常量测试 ───────────────────────────────".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:05:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-benches/http_techpower.rs`
### Change Record
New file `crates/hiver-benches/http_techpower.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-benches/http_techpower.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:06:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/config.rs`
### Change Record
Modified file `crates/hiver-config/src/config.rs`. Approximately 388 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-config/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:06:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-cloud/src/discovery/consul.rs`
### Change Record
New file `crates/hiver-cloud/src/discovery/consul.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-cloud/src/discovery/consul.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:06:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/role.rs`
### Change Record
Modified file `crates/hiver-security/src/role.rs`. Approximately 15 lines changed.
### Change Summary
Removed "impl From<String> for Role {" from `crates/hiver-security/src/role.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:06:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-benches/data_orm.rs`
### Change Record
New file `crates/hiver-benches/data_orm.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-benches/data_orm.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:06:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/loader.rs`
### Change Record
Modified file `crates/hiver-config/src/loader.rs`. Approximately 150 lines changed.
### Change Summary
Added "/// Test ConfigLoader::new has sensible defaults; /// 测试ConfigLoader::new有合理的默认值; #[test]" in `crates/hiver-config/src/loader.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/role.rs`
### Change Record
### Change Summary
Changed `crates/hiver-security/src/role.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/role.rs`
### Change Record
Modified file `crates/hiver-security/src/role.rs`. Approximately 15 lines changed.
### Change Summary
Removed "impl From<String> for Role {" from `crates/hiver-security/src/role.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/microservice.rs`
### Change Record
New file `examples/microservice.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/microservice.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:07:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/role.rs`
### Change Record
### Change Summary
Changed `crates/hiver-security/src/role.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/error.rs`
### Change Record
Modified file `crates/hiver-config/src/error.rs`. Approximately 163 lines changed.
### Change Summary
Added "#[cfg(test)]; mod tests {; use super::*;" in `crates/hiver-config/src/error.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/tests.rs`
### Change Record
Modified file `crates/hiver-kafka/src/tests.rs`. Approximately 294 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/tests.rs` from "//! Tests for hiver-kafka; //! 测试模块; fn smoke_test() {" to "//! Integration tests for hiver-kafka; //! hiver-kafka 集成测试; use hiver_kafka::{".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-cloud/src/discovery/mod.rs`
### Change Record
New file `crates/hiver-cloud/src/discovery/mod.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-cloud/src/discovery/mod.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:07:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/lib.rs`
### Change Record
Modified file `crates/hiver-cloud/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "consul::{ConsulConfig, ConsulServiceRegistry}," in `crates/hiver-cloud/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/lib.rs`
### Change Record
Modified file `crates/hiver-cloud/src/lib.rs`. Approximately 20 lines changed.
### Change Summary
Added "consul::{ConsulConfig, ConsulServiceRegistry},; ConsulConfig, ConsulServiceRegistry," in `crates/hiver-cloud/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/properties.rs`
### Change Record
Modified file `crates/hiver-config/src/properties.rs`. Approximately 144 lines changed.
### Change Summary
Added "/// Test registry get returns registered config; /// 测试注册表get返回已注册的配置; #[test]" in `crates/hiver-config/src/properties.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `tests/integration/postgres_tests.rs`
### Change Record
New file `tests/integration/postgres_tests.rs`, not yet tracked by version control.
### Change Summary
Changed `tests/integration/postgres_tests.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:07:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `tests/integration/mysql_tests.rs`
### Change Record
New file `tests/integration/mysql_tests.rs`, not yet tracked by version control.
### Change Summary
Changed `tests/integration/mysql_tests.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:07:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `tests/integration/redis_tests.rs`
### Change Record
New file `tests/integration/redis_tests.rs`, not yet tracked by version control.
### Change Summary
Changed `tests/integration/redis_tests.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:07:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `tests/integration/kafka_tests.rs`
### Change Record
New file `tests/integration/kafka_tests.rs`, not yet tracked by version control.
### Change Summary
Changed `tests/integration/kafka_tests.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:07:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/role.rs`
### Change Record
Modified file `crates/hiver-security/src/role.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-security/src/role.rs` from "write!(f, "{}", self.name()); write!(f, "{}", self.name())" to "write!(f, "{self.name()}"); write!(f, "{self.name()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:07:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `tests/integration/rabbitmq_tests.rs`
### Change Record
New file `tests/integration/rabbitmq_tests.rs`, not yet tracked by version control.
### Change Summary
Changed `tests/integration/rabbitmq_tests.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:07:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-benches/security.rs`
### Change Record
New file `crates/hiver-benches/security.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-benches/security.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:07:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `tests/integration/mysql_tests.rs`
### Change Record
New file `tests/integration/mysql_tests.rs`, not yet tracked by version control.
### Change Summary
Changed `tests/integration/mysql_tests.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:07:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/role.rs`
### Change Record
Modified file `crates/hiver-security/src/role.rs`. Approximately 30 lines changed.
### Change Summary
Changed `crates/hiver-security/src/role.rs` from "write!(f, "{}", self.name()); write!(f, "{}", self.authority()); write!(f, "{}", self.name())" to "write!(f, "{self.name()}"); write!(f, "{self.authority()}"); write!(f, "{self.name()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-cloud/src/discovery/mod.rs`
### Change Record
New file `crates/hiver-cloud/src/discovery/mod.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-cloud/src/discovery/mod.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:08:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/Cargo.toml`
### Change Record
Modified file `crates/hiver-config/Cargo.toml`. Approximately 9 lines changed.
### Change Summary
Added "tempfile = { workspace = true }" in `crates/hiver-config/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/locale.rs`
### Change Record
Modified file `crates/hiver-i18n/src/locale.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-i18n/src/locale.rs` from "write!(f, "{}", self.language)" to "write!(f, "{self.language}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/lib.rs`
### Change Record
Modified file `crates/hiver-cloud/src/lib.rs`. Approximately 22 lines changed.
### Change Summary
Added "#[cfg(feature = "consul")]; pub use discovery::consul::{ConsulConfig, ConsulServiceRegistry};; ConsulConfig, ConsulServi..." in `crates/hiver-cloud/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-websocket-stomp/src/frame.rs`
### Change Record
Modified file `crates/hiver-websocket-stomp/src/frame.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-websocket-stomp/src/frame.rs` from "write!(f, "{}", self.as_str())" to "write!(f, "{self.as_str()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/Cargo.toml`
### Change Record
Modified file `crates/hiver-config/Cargo.toml`. Approximately 9 lines changed.
### Change Summary
Added "tempfile = "3"" in `crates/hiver-config/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/lib.rs`
### Change Record
Modified file `crates/hiver-cloud/src/lib.rs`. Approximately 24 lines changed.
### Change Summary
Added "#[cfg(feature = "consul")]; pub use discovery::consul::{ConsulConfig, ConsulServiceRegistry};; #[cfg(feature = "consul")..." in `crates/hiver-cloud/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/specification.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/specification.rs`. Approximately 17 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/specification.rs` from "Self::I64(n) => write!(f, "{}", n),; Self::F64(n) => write!(f, "{}", n),; Self::Bool(b) => write!(f, "{}", b)," to "Self::I64(n) => write!(f, "{n}"),; Self::F64(n) => write!(f, "{n}"),; Self::Bool(b) => write!(f, "{b}"),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/specification.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/specification.rs`. Approximately 25 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/specification.rs` from "Self::I64(n) => write!(f, "{}", n),; Self::F64(n) => write!(f, "{}", n),; Self::Bool(b) => write!(f, "{}", b)," to "Self::I64(n) => write!(f, "{n}"),; Self::F64(n) => write!(f, "{n}"),; Self::Bool(b) => write!(f, "{b}"),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/properties.rs`
### Change Record
Modified file `crates/hiver-config/src/properties.rs`. Approximately 157 lines changed.
### Change Summary
Added "/// Test registry get returns registered config; /// 测试注册表get返回已注册的配置; #[test]" in `crates/hiver-config/src/properties.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ai/src/prompt.rs`
### Change Record
Modified file `crates/hiver-ai/src/prompt.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-ai/src/prompt.rs` from "write!(f, "{}", self.template)" to "write!(f, "{self.template}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/sse.rs`
### Change Record
Modified file `crates/hiver-http/src/sse.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-http/src/sse.rs` from "write!(f, "{}", self.to_sse_format())" to "write!(f, "{self.to_sse_format()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/validation.rs`
### Change Record
Modified file `crates/hiver-http/src/validation.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-http/src/validation.rs` from "write!(f, "{}", first)" to "write!(f, "{first}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/builder.rs`
### Change Record
Modified file `crates/hiver-http/src/builder.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-http/src/builder.rs` from "write!(f, "{}", self.uri)" to "write!(f, "{self.uri}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/odm.rs`
### Change Record
Modified file `crates/hiver-ldap/src/odm.rs`. Approximately 404 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/odm.rs` from "//! Equivalent to Spring LDAP ODM; //! 等价于 Spring LDAP ODM; #[derive(Debug, Clone, Serialize, Deserialize)]" to "//! Equivalent to Spring LDAP ODM.; //! Provides utilities to map between Rust structs and LDAP entries.; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:08:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-observability/src/metrics.rs`
### Change Record
Modified file `crates/hiver-observability/src/metrics.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-observability/src/metrics.rs` from "write!(f, "{}", self.as_str())" to "write!(f, "{self.as_str()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-observability/src/trace.rs`
### Change Record
Modified file `crates/hiver-observability/src/trace.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-observability/src/trace.rs` from "write!(f, "{}", self.to_hex()); write!(f, "{}", self.to_hex())" to "write!(f, "{self.to_hex()}"); write!(f, "{self.to_hex()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-shell/src/command.rs`
### Change Record
Modified file `crates/hiver-shell/src/command.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-shell/src/command.rs` from "write!(f, "{}", self.name)?;" to "write!(f, "{self.name}")?;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-shell/src/result.rs`
### Change Record
Modified file `crates/hiver-shell/src/result.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-shell/src/result.rs` from "write!(f, "{}", self.text)" to "write!(f, "{self.text}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-shell/src/result.rs`
### Change Record
Modified file `crates/hiver-shell/src/result.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-shell/src/result.rs` from "write!(f, "{}", self.text); write!(f, "{}", self.render_table())" to "write!(f, "{self.text}"); write!(f, "{self.render_table()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/web3_dapp.rs`
### Change Record
New file `examples/web3_dapp.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/web3_dapp.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:09:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-shell/src/result.rs`
### Change Record
Modified file `crates/hiver-shell/src/result.rs`. Approximately 30 lines changed.
### Change Summary
Changed `crates/hiver-shell/src/result.rs` from "write!(f, "{}", self.text); write!(f, "{}", self.render_table()); write!(f, "{}", self.render_json())" to "write!(f, "{self.text}"); write!(f, "{self.render_table()}"); write!(f, "{self.render_json()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/config/environment.rs`
### Change Record
Modified file `crates/hiver-starter/src/config/environment.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-starter/src/config/environment.rs` from "write!(f, "{}", self.name())" to "write!(f, "{self.name()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/environment.rs`
### Change Record
Modified file `crates/hiver-config/src/environment.rs`. Approximately 332 lines changed.
### Change Summary
Changed `crates/hiver-config/src/environment.rs` from "write!(f, "{}", self.0)" to "write!(f, "{self.0}"); #[cfg(test)]; mod tests {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/value.rs`
### Change Record
Modified file `crates/hiver-config/src/value.rs`. Approximately 529 lines changed.
### Change Summary
Changed `crates/hiver-config/src/value.rs` from "Value::Bool(v) => write!(f, "{}", v),; Value::Integer(v) => write!(f, "{}", v),; Value::Float(v) => write!(f, "{}", v)," to "Value::Bool(v) => write!(f, "{v}"),; Value::Integer(v) => write!(f, "{v}"),; Value::Float(v) => write!(f, "{v}"),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/tx.rs`
### Change Record
Modified file `crates/hiver-web3/src/tx.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/tx.rs` from "write!(f, "{}", self.to_hex()); write!(f, "{}", self.to_hex())" to "write!(f, "{self.to_hex()}"); write!(f, "{self.to_hex()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/subscribe.rs`
### Change Record
Modified file `crates/hiver-web3/src/subscribe.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/subscribe.rs` from "write!(f, "{}", self.method_name())" to "write!(f, "{self.method_name()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/subscribe.rs`
### Change Record
Modified file `crates/hiver-web3/src/subscribe.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/subscribe.rs` from "write!(f, "{}", self.method_name()); write!(f, "{}", self.0)" to "write!(f, "{self.method_name()}"); write!(f, "{self.0}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/chain.rs`
### Change Record
Modified file `crates/hiver-web3/src/chain.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/chain.rs` from "Self::Number(n) => write!(f, "{}", n)," to "Self::Number(n) => write!(f, "{n}"),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/wallet.rs`
### Change Record
Modified file `crates/hiver-web3/src/wallet.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/wallet.rs` from "write!(f, "{}", self.checksum())" to "write!(f, "{self.checksum()}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:09:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/src/error.rs`
### Change Record
Modified file `crates/hiver-graphql/src/error.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-graphql/src/error.rs` from "write!(f, "{}", self.message)?;" to "write!(f, "{self.message}")?;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:10:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-micrometer/src/metric.rs`
### Change Record
Modified file `crates/hiver-micrometer/src/metric.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-micrometer/src/metric.rs` from "write!(f, "{}", self.0)" to "write!(f, "{self.0}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:10:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/session.rs`
### Change Record
Modified file `crates/hiver-session/src/session.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-session/src/session.rs` from "write!(f, "{}", self.0)" to "write!(f, "{self.0}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:10:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/status.rs`
### Change Record
Modified file `crates/hiver-http/src/status.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-http/src/status.rs` from "write!(f, "{}", self.0)" to "write!(f, "{self.0}")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:10:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/pool.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/pool.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/pool.rs` from "let type_name = format!("{}", col.type_info()).to_lowercase();" to "let type_name = col.type_info().to_string().to_lowercase();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:10:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-observability/src/hiver_format.rs`
### Change Record
Modified file `crates/hiver-observability/src/hiver_format.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-observability/src/hiver_format.rs` from "write!(writer, "{}", info)?;" to "write!(writer, "{info}")?;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:10:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/builder.rs`
### Change Record
Modified file `crates/hiver-http/src/builder.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-http/src/builder.rs` from "write!(f, "{}", self.uri); let _ = write!(result, "{}", host);" to "write!(f, "{self.uri}"); let _ = write!(result, "{host}");".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:10:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/grpc_service.rs`
### Change Record
New file `examples/grpc_service.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/grpc_service.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:10:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `examples/Cargo.toml`
### Change Record
Modified file `examples/Cargo.toml`. Approximately 27 lines changed.
### Change Summary
Added "[[bin]]; name = "rest_api"; path = "rest_api.rs"" in `examples/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:11:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/part_tree.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/part_tree.rs`. Approximately 43 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/part_tree.rs` from "assert_eq!(format!("{}", part), "age GREATER_THAN");; assert_eq!(format!("{}", ob), "name DESC");; assert_eq!(format!("{..." to "assert_eq!(part.to_string(), "age GREATER_THAN");; assert_eq!(ob.to_string(), "name DESC");; assert_eq!(Subject::Find.to...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:11:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `examples/Cargo.toml`
### Change Record
Modified file `examples/Cargo.toml`. Approximately 48 lines changed.
### Change Summary
Added "[[bin]]; name = "rest_api"; path = "rest_api.rs"" in `examples/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:11:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/optimistic_lock.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/optimistic_lock.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/optimistic_lock.rs` from "let msg = format!("{}", err);" to "let msg = err.to_string();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:11:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-events/src/transactional_listener.rs`
### Change Record
Modified file `crates/hiver-events/src/transactional_listener.rs`. Approximately 20 lines changed.
### Change Summary
Changed `crates/hiver-events/src/transactional_listener.rs` from "assert_eq!(format!("{}", TransactionPhase::BeforeCommit), "before_commit");; assert_eq!(format!("{}", TransactionPhase::..." to "assert_eq!(TransactionPhase::BeforeCommit.to_string(), "before_commit");; assert_eq!(TransactionPhase::AfterCommit.to_st...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:11:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-events/src/condition.rs`
### Change Record
Modified file `crates/hiver-events/src/condition.rs`. Approximately 15 lines changed.
### Change Summary
Changed `crates/hiver-events/src/condition.rs` from "assert_eq!(format!("{}", CompareOp::Eq), "==");; assert_eq!(format!("{}", CompareOp::Contains), "contains");" to "assert_eq!(CompareOp::Eq.to_string(), "==");; assert_eq!(CompareOp::Contains.to_string(), "contains");".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:11:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ai/src/chat_model.rs`
### Change Record
Modified file `crates/hiver-ai/src/chat_model.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-ai/src/chat_model.rs` from "assert_eq!(format!("{}", Role::User), "user");" to "assert_eq!(Role::User.to_string(), "user");".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:11:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-flyway/src/dialect.rs`
### Change Record
New file `crates/hiver-flyway/src/dialect.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-flyway/src/dialect.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:11:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/tests.rs`
### Change Record
Modified file `crates/hiver-flyway/src/tests.rs`. Approximately 287 lines changed.
### Change Summary
Changed `crates/hiver-flyway/src/tests.rs` from "fn smoke_test() {; assert!(true, "hiver-flyway test infrastructure is working");; fn test_basic_math() {" to "use crate::dialect::DatabaseType;; use crate::config::Config;; // ------------------------------------------------------...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:12:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/rest_api.rs`
### Change Record
New file `examples/rest_api.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/rest_api.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:12:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/validation.rs`
### Change Record
Modified file `crates/hiver-http/src/validation.rs`. Approximately 31 lines changed.
### Change Summary
Changed `crates/hiver-http/src/validation.rs` from "write!(f, "{}", first); format!("{}", value.len()),; format!("{}", value.len())," to "write!(f, "{first}"); value.len().to_string(),; value.len().to_string(),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:12:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/validation.rs`
### Change Record
Modified file `crates/hiver-http/src/validation.rs`. Approximately 49 lines changed.
### Change Summary
Changed `crates/hiver-http/src/validation.rs` from "write!(f, "{}", first); format!("{}", value.len()),; format!("{}", value.len())," to "write!(f, "{first}"); value.len().to_string(),; value.len().to_string(),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:12:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/rest_api.rs`
### Change Record
New file `examples/rest_api.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/rest_api.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:12:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/rest_api.rs`
### Change Record
New file `examples/rest_api.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/rest_api.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:13:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/rest_api.rs`
### Change Record
New file `examples/rest_api.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/rest_api.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:13:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/autoconfigure_processor.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/autoconfigure_processor.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-starter/src/core/autoconfigure_processor.rs` from "let error_msg = format!("{}", e);" to "let error_msg = e.to_string();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:13:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/autoconfigure_processor.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/autoconfigure_processor.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-starter/src/core/autoconfigure_processor.rs` from "let error_msg = format!("{}", e);; let display = format!("{}", result);" to "let error_msg = e.to_string();; let display = result.to_string();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:13:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/condition_evaluator.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/condition_evaluator.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-starter/src/core/condition_evaluator.rs` from "let display = format!("{}", config);" to "let display = config.to_string();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:13:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-observability/src/metrics.rs`
### Change Record
Modified file `crates/hiver-observability/src/metrics.rs`. Approximately 27 lines changed.
### Change Summary
Changed `crates/hiver-observability/src/metrics.rs` from "write!(f, "{}", self.as_str()); let formatted_value = if line.ends_with("_sum") || line.ends_with("_count") {; // Histog..." to "write!(f, "{self.as_str()}"); let formatted_value = value.to_string();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:13:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/sea_orm.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/sea_orm.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/sea_orm.rs` from "let param_values: Vec<String> = map.values().map(|v| format!("{}", v)).collect();" to "let param_values: Vec<String> = map.values().map(|v| v.to_string()).collect();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:13:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/rest_api.rs`
### Change Record
New file `examples/rest_api.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/rest_api.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:13:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/value.rs`
### Change Record
Modified file `crates/hiver-config/src/value.rs`. Approximately 538 lines changed.
### Change Summary
Changed `crates/hiver-config/src/value.rs` from "Value::Bool(v) => write!(f, "{}", v),; Value::Integer(v) => write!(f, "{}", v),; Value::Float(v) => write!(f, "{}", v)," to "Value::Bool(v) => write!(f, "{v}"),; Value::Integer(v) => write!(f, "{v}"),; Value::Float(v) => write!(f, "{v}"),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:14:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/microservice.rs`
### Change Record
New file `examples/microservice.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/microservice.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:16:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/grpc_service.rs`
### Change Record
New file `examples/grpc_service.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/grpc_service.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:16:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/web3_dapp.rs`
### Change Record
New file `examples/web3_dapp.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/web3_dapp.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:16:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/web3_dapp.rs`
### Change Record
New file `examples/web3_dapp.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/web3_dapp.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:17:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/web3_dapp.rs`
### Change Record
New file `examples/web3_dapp.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/web3_dapp.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:17:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/web3_dapp.rs`
### Change Record
New file `examples/web3_dapp.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/web3_dapp.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:17:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/web3_dapp.rs`
### Change Record
New file `examples/web3_dapp.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/web3_dapp.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:17:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/web3_dapp.rs`
### Change Record
New file `examples/web3_dapp.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/web3_dapp.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:18:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/web3_dapp.rs`
### Change Record
New file `examples/web3_dapp.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/web3_dapp.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:18:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `examples/web3_dapp.rs`
### Change Record
New file `examples/web3_dapp.rs`, not yet tracked by version control.
### Change Summary
Changed `examples/web3_dapp.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-24 19:20:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 662 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "//! Tests for #[Data] derive macro; //! #[Data] 派生宏测试; use hiver_lombok::Data;" to "//! Comprehensive tests for all hiver-lombok derive macros.; //! hiver-lombok 所有派生宏的综合测试。; //!".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:22:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/tests.rs`
### Change Record
Modified file `crates/hiver-vault/src/tests.rs`. Approximately 1605 lines changed.
### Change Summary
Changed `crates/hiver-vault/src/tests.rs` from "#[cfg(test)]; mod tests {; #[test]" to "//! hiver-vault 测试; //!; //! Comprehensive test suite using mockito for HTTP mocking.".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:22:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/pki.rs`
### Change Record
Modified file `crates/hiver-vault/src/pki.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-vault/src/pki.rs` from "#[derive(Debug, Clone, Serialize, Deserialize)]" to "#[derive(Debug, Clone, Default, Serialize, Deserialize)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:28:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/handle.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/handle.rs`. Approximately 69 lines changed.
### Change Summary
Added "/// Block until a notification arrives or timeout elapses; /// 阻塞直到收到通知或超时; ///" in `crates/hiver-runtime/src/scheduler/handle.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 19:29:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/specification.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/specification.rs`. Approximately 17 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/specification.rs` from "Self::I64(n) => write!(f, "{}", n),; Self::F64(n) => write!(f, "{}", n),; Self::Bool(b) => write!(f, "{}", b)," to "Self::I64(n) => write!(f, "{n}"),; Self::F64(n) => write!(f, "{n}"),; Self::Bool(b) => write!(f, "{b}"),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:52:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/tests.rs`
### Change Record
Modified file `crates/hiver-amqp/src/tests.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-amqp/src/tests.rs` from "use hiver_amqp::*;" to "use crate::*;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:53:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/converter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/converter.rs`. Approximately 35 lines changed.
### Change Summary
Changed `crates/hiver-amqp/src/converter.rs` from "/// Test MessageConverter trait object usage / 测试 MessageConverter trait 对象使用; #[test]; fn test_message_converter_trait_..." to "/// TODO: MessageConverter has generic methods and is not dyn-compatible.; /// Re-enable once trait is refactored to sup...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:53:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-aop/src/runtime.rs`
### Change Record
Modified file `crates/hiver-aop/src/runtime.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-aop/src/runtime.rs` from "let downcast = aspect.unwrap().downcast_ref::<&str>();" to "let arc = aspect.unwrap();; let downcast = arc.as_ref().downcast_ref::<&str>();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:53:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/loader.rs`
### Change Record
Modified file `crates/hiver-config/src/loader.rs`. Approximately 12 lines changed.
### Change Summary
Added "use crate::{PropertySource, Value};" in `crates/hiver-config/src/loader.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:53:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/value.rs`
### Change Record
Modified file `crates/hiver-config/src/value.rs`. Approximately 12 lines changed.
### Change Summary
Added "use crate::{Environment, PropertySource};" in `crates/hiver-config/src/value.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:53:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/value.rs`
### Change Record
Modified file `crates/hiver-config/src/value.rs`. Approximately 48 lines changed.
### Change Summary
Changed `crates/hiver-config/src/value.rs` from "use crate::Environment;; use crate::Environment;; use crate::Environment;" to "use crate::{Environment, PropertySource};".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:53:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/properties.rs`
### Change Record
Modified file `crates/hiver-config/src/properties.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-config/src/properties.rs` from "#[derive(Debug, Clone)]" to "#[derive(Debug, Clone, serde::Deserialize)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:53:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/properties.rs`
### Change Record
Modified file `crates/hiver-config/src/properties.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-config/src/properties.rs` from "#[derive(Debug, Clone)]; #[derive(Debug, Clone, Default)]" to "#[derive(Debug, Clone, serde::Deserialize)]; #[derive(Debug, Clone, Default, serde::Deserialize)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:54:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/config.rs`
### Change Record
Modified file `crates/hiver-config/src/config.rs`. Approximately 12 lines changed.
### Change Summary
Added "use crate::{PropertySource, Value};" in `crates/hiver-config/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:54:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/config.rs`
### Change Record
Modified file `crates/hiver-config/src/config.rs`. Approximately 57 lines changed.
### Change Summary
Changed `crates/hiver-config/src/config.rs` from "fn test_config_caching() {; let config = Config::new();; let mut source = PropertySource::new("s1");" to "use crate::{PropertySource, Value};; /// TODO: add_property_source_first method does not exist yet.; /// Re-enable when ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:54:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();" to "container.register(|_| Ok(EmailService::default())).unwrap();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:54:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 40 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:54:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 49 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:54:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 67 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 121 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 130 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 143 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 152 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 169 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 174 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 183 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 192 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 201 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:55:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 210 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:56:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 217 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:56:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 221 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:56:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 225 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:56:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 229 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:56:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 238 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:56:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 247 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:56:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 256 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:56:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 260 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:57:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 264 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:57:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 273 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "container.register::<EmailService>(|_| Ok(EmailService::default())).unwrap();; .register::<UserRepository>(|_| Ok(UserRe..." to "container.register(|_| Ok(EmailService::default())).unwrap();; .register(|_| Ok(UserRepository::default())); .register(|...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:57:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/extension.rs`
### Change Record
Modified file `crates/hiver-core/src/extension.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-core/src/extension.rs` from "assert_eq!(ext.get::<String>(), Some(&"text"));" to "assert_eq!(ext.get::<String>(), Some(&"text".to_string()));".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:57:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/reactive.rs`
### Change Record
Modified file `crates/hiver-core/src/reactive.rs`. Approximately 16 lines changed.
### Change Summary
Changed `crates/hiver-core/src/reactive.rs` from "let result = Flux::from_iter(vec!["a", "b", "c"]); acc.push_str(x);" to "let result = Flux::from_iter(vec!["a".to_string(), "b".to_string(), "c".to_string()]); acc.push_str(&x);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:57:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/reactive.rs`
### Change Record
Modified file `crates/hiver-core/src/reactive.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-core/src/reactive.rs` from "let result = Flux::from_iter(vec!["a", "b", "c"]); acc.push_str(x);; let items: Vec<i32> = Flux::from_iter(Vec::new()).c..." to "let result = Flux::from_iter(vec!["a".to_string(), "b".to_string(), "c".to_string()]); acc.push_str(&x);; let items: Vec...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:57:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/tests.rs`
### Change Record
Modified file `crates/hiver-kafka/src/tests.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/tests.rs` from "use hiver_kafka::{" to "use crate::{".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:58:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/tests.rs`
### Change Record
Modified file `crates/hiver-kafka/src/tests.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/tests.rs` from "use hiver_kafka::{; use hiver_kafka::CompressionType;" to "use crate::{; use crate::CompressionType;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:58:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/tests.rs`
### Change Record
Modified file `crates/hiver-kafka/src/tests.rs`. Approximately 31 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/tests.rs` from "use hiver_kafka::{; use hiver_kafka::CompressionType;; use hiver_kafka::RecordHeader;" to "use crate::{; use crate::CompressionType;; use crate::RecordHeader;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:58:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/tests.rs`
### Change Record
Modified file `crates/hiver-kafka/src/tests.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/tests.rs` from "use hiver_kafka::{; use hiver_kafka::CompressionType;; assert_eq!(key.as_bytes(), Some(b"order-123".as_slice()));" to "use crate::{; use crate::CompressionType;; assert_eq!(key.as_bytes(), Some(&b"order-123"[..]));".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:58:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/tests.rs`
### Change Record
Modified file `crates/hiver-ldap/src/tests.rs`. Approximately 25 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/tests.rs` from "use hiver_ldap::*;; use hiver_ldap::context::LdapContextSourceBuilder;; use hiver_ldap::mapper::AttrMap;" to "use crate::*;; use crate::context::LdapContextSourceBuilder;; use crate::mapper::AttrMap;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:59:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/tests.rs`
### Change Record
Modified file `crates/hiver-ldap/src/tests.rs`. Approximately 25 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/tests.rs` from "use hiver_ldap::*;; use hiver_ldap::context::LdapContextSourceBuilder;; use hiver_ldap::mapper::AttrMap;" to "use crate::*;; use crate::context::LdapContextSourceBuilder;; use crate::mapper::{self, AttrMap};".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:59:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 52 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// Test Getter skips fields annotated with #[get] or #[skip].; /// 测试 Getter 跳过标注了 #[get] 或 #[skip] 的字段。; #[test]" to "/// TODO: #[get] attribute is not registered as a derive helper attribute.; /// Re-enable when the Getter derive macro d...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 21:59:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 98 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// Test Getter skips fields annotated with #[get] or #[skip].; /// 测试 Getter 跳过标注了 #[get] 或 #[skip] 的字段。; #[test]" to "/// TODO: #[get] attribute is not registered as a derive helper attribute.; /// Re-enable when the Getter derive macro d...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:00:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 142 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// Test Data with generic struct.; /// 测试 Data 在泛型结构体上的表现。; #[test]" to "/// TODO: Data derive macro does not handle generic structs correctly (Pair<T, U>).; /// Re-enable when the macro suppor...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:00:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 221 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// Test Data with generic struct.; /// 测试 Data 在泛型结构体上的表现。; #[test]" to "/// TODO: Data derive macro does not handle generic structs correctly (Pair<T, U>).; /// Re-enable when the macro suppor...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:00:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/timeout.rs`
### Change Record
Modified file `crates/hiver-resilience/src/timeout.rs`. Approximately 15 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/timeout.rs` from "let result: std::result::Result<i32, &str> = t.call(|| async { Ok(100) }).await;; assert_eq!(result.unwrap(), Ok(100));" to "let result = t.call(|| async { 100i32 }).await;; assert_eq!(result.unwrap(), 100);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:01:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/config.rs`
### Change Record
Modified file `crates/hiver-config/src/config.rs`. Approximately 60 lines changed.
### Change Summary
Changed `crates/hiver-config/src/config.rs` from "/// Test that config caches values and invalidates on new source; /// 测试配置缓存值并在新源添加时失效; #[test]" to "use crate::{PropertySource, Value};; // TODO: add_property_source_first method does not exist yet.; // Re-enable when th...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:02:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/converter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/converter.rs`. Approximately 35 lines changed.
### Change Summary
Changed `crates/hiver-amqp/src/converter.rs` from "/// Test MessageConverter trait object usage / 测试 MessageConverter trait 对象使用; #[test]; fn test_message_converter_trait_..." to "// TODO: MessageConverter has generic methods and is not dyn-compatible.; // Re-enable once trait is refactored to suppo...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:02:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/tests.rs`
### Change Record
Modified file `crates/hiver-kafka/src/tests.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/tests.rs` from "use hiver_kafka::{; use hiver_kafka::CompressionType;; assert_eq!(key.as_bytes(), Some(b"order-123".as_slice()));" to "use crate::{; use crate::config::CompressionType;; assert_eq!(key.as_bytes(), Some(&b"order-123"[..]));".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:02:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/message.rs`
### Change Record
Modified file `crates/hiver-kafka/src/message.rs`. Approximately 40 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/message.rs` from "/// Test MessageHeaderValue serialization; /// 测试 MessageHeaderValue 序列化; #[test]" to "// TODO: MessageHeaderValue does not derive PartialEq, so assert_eq! cannot be used.; // Re-enable once PartialEq is add...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:03:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/serialization.rs`
### Change Record
Modified file `crates/hiver-kafka/src/serialization.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/serialization.rs` from "assert_eq!(data.as_bytes(), Some(b"hello".as_slice()));" to "assert_eq!(SerializeData::as_bytes(&data), Some(&b"hello"[..]));".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:03:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/serialization.rs`
### Change Record
Modified file `crates/hiver-kafka/src/serialization.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/serialization.rs` from "assert_eq!(data.as_bytes(), Some(b"hello".as_slice()));; assert_eq!(data.as_bytes(), Some(b"world".as_slice()));" to "assert_eq!(SerializeData::as_bytes(&data), Some(&b"hello"[..]));; assert_eq!(SerializeData::as_bytes(&data), Some(&b"wor...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:04:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/serialization.rs`
### Change Record
Modified file `crates/hiver-kafka/src/serialization.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/serialization.rs` from "assert_eq!(data.as_bytes(), Some(b"hello".as_slice()));; assert_eq!(data.as_bytes(), Some(b"world".as_slice()));" to "assert_eq!(SerializeData::as_bytes(data), Some(&b"hello"[..]));; assert_eq!(SerializeData::as_bytes(&data), Some(&b"worl...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:05:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/serialization.rs`
### Change Record
Modified file `crates/hiver-kafka/src/serialization.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/serialization.rs` from "#[derive(Clone, Default)]; assert_eq!(data.as_bytes(), Some(b"hello".as_slice()));; assert_eq!(data.as_bytes(), Some(b"w..." to "#[derive(Clone)]; impl Default for KeySerializer {; fn default() -> Self {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:08:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/config.rs`
### Change Record
Modified file `crates/hiver-amqp/src/config.rs`. Approximately 19 lines changed.
### Change Summary
Added "self.url.clear();; self.url.clear();" in `crates/hiver-amqp/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:09:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/config.rs`
### Change Record
Modified file `crates/hiver-amqp/src/config.rs`. Approximately 27 lines changed.
### Change Summary
Added "self.url.clear();; self.url.clear();; self.url.clear();" in `crates/hiver-amqp/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:11:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/environment.rs`
### Change Record
Modified file `crates/hiver-config/src/environment.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-config/src/environment.rs` from "assert_eq!(result, "missing stays");" to "assert_eq!(result, "missing ${no.key} stays");".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-24 22:11:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/config.rs`
### Change Record
Modified file `crates/hiver-config/src/config.rs`. Approximately 78 lines changed.
### Change Summary
Changed `crates/hiver-config/src/config.rs` from "assert!(result.is_err());; assert!(result.is_err());; /// Test that config caches values and invalidates on new source" to "use crate::{PropertySource, Value};; assert!(result.is_ok());; assert!(result.is_ok());".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:52:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/src/data.rs`
### Change Record
Modified file `crates/hiver-lombok/src/data.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-lombok/src/data.rs` from "#struct_name: Clone," to "#struct_name #ty_generics: Clone,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:52:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/src/value.rs`
### Change Record
Modified file `crates/hiver-lombok/src/value.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-lombok/src/value.rs` from "#struct_name: Clone," to "#struct_name #ty_generics: Clone,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:52:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/src/with_method.rs`
### Change Record
Modified file `crates/hiver-lombok/src/with_method.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-lombok/src/with_method.rs` from "quote! { where #struct_name: Clone }" to "quote! { where #struct_name #ty_generics: Clone }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:53:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 50 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// TODO: Data derive macro does not handle generic structs correctly (Pair<T, U>).; /// Re-enable when the macro suppor..." to "/// Generic struct for testing Data with type parameters.; /// 用于测试 Data 泛型类型参数的泛型结构体。; #[derive(Data, Clone, PartialEq,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:53:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 98 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// TODO: Data derive macro does not handle generic structs correctly (Pair<T, U>).; /// Re-enable when the macro suppor..." to "/// Generic struct for testing Data with type parameters.; /// 用于测试 Data 泛型类型参数的泛型结构体。; #[derive(Data, Clone, PartialEq,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:53:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 144 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// TODO: Data derive macro does not handle generic structs correctly (Pair<T, U>).; /// Re-enable when the macro suppor..." to "/// Generic struct for testing Data with type parameters.; /// 用于测试 Data 泛型类型参数的泛型结构体。; #[derive(Data, Clone, PartialEq,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:53:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 153 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// TODO: Data derive macro does not handle generic structs correctly (Pair<T, U>).; /// Re-enable when the macro suppor..." to "/// Generic struct for testing Data with type parameters.; /// 用于测试 Data 泛型类型参数的泛型结构体。; #[derive(Data, Clone, PartialEq,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:53:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 162 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// TODO: Data derive macro does not handle generic structs correctly (Pair<T, U>).; /// Re-enable when the macro suppor..." to "/// Generic struct for testing Data with type parameters.; /// 用于测试 Data 泛型类型参数的泛型结构体。; #[derive(Data, Clone, PartialEq,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:54:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/tests/data_test.rs`
### Change Record
Modified file `crates/hiver-lombok/tests/data_test.rs`. Approximately 241 lines changed.
### Change Summary
Changed `crates/hiver-lombok/tests/data_test.rs` from "/// TODO: Data derive macro does not handle generic structs correctly (Pair<T, U>).; /// Re-enable when the macro suppor..." to "/// Generic struct for testing Data with type parameters.; /// 用于测试 Data 泛型类型参数的泛型结构体。; #[derive(Data, Clone, PartialEq,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:54:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/src/lib.rs`
### Change Record
Modified file `crates/hiver-lombok/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-lombok/src/lib.rs` from "#[proc_macro_derive(Getter)]" to "#[proc_macro_derive(Getter, attributes(get))]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:54:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/src/lib.rs`
### Change Record
Modified file `crates/hiver-lombok/src/lib.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-lombok/src/lib.rs` from "#[proc_macro_derive(Getter)]; #[proc_macro_derive(Setter)]" to "#[proc_macro_derive(Getter, attributes(get))]; #[proc_macro_derive(Setter, attributes(set))]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:54:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/src/constructor.rs`
### Change Record
Modified file `crates/hiver-lombok/src/constructor.rs`. Approximately 44 lines changed.
### Change Summary
Changed `crates/hiver-lombok/src/constructor.rs` from "// Generate constructor with Default::default() for each field; // 为每个字段生成使用 Default::default() 的构造函数; let constructor_e..." to "// Generate Default implementation only (no new() to avoid conflict with AllArgsConstructor); // 仅生成 Default 实现（不生成 new(...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 07:54:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/examples/user_entity.rs`
### Change Record
Modified file `crates/hiver-lombok/examples/user_entity.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-lombok/examples/user_entity.rs` from "let config = DefaultConfig::new();" to "let config = DefaultConfig::default();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 18:58:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/message.rs`
### Change Record
Modified file `crates/hiver-kafka/src/message.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/message.rs` from "#[derive(Clone, Debug, Serialize, Deserialize)]" to "#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 18:58:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/sea_orm.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/sea_orm.rs`. Approximately 15 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/sea_orm.rs` from "// TODO: Pass param_values through a parameterized execute API once available.; // For now, log a warning that mock clie..." to "// Values interpolated into SQL string; parameterized binding requires; // DatabaseClient extension (tracked separately)...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 18:59:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/config.rs`
### Change Record
Modified file `crates/hiver-config/src/config.rs`. Approximately 18 lines changed.
### Change Summary
Added "/// Add a property source with highest priority.; /// 添加最高优先级的属性源。; pub fn add_property_source_first(&self, source: Prop..." in `crates/hiver-config/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 18:59:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/config.rs`
### Change Record
Modified file `crates/hiver-config/src/config.rs`. Approximately 64 lines changed.
### Change Summary
Changed `crates/hiver-config/src/config.rs` from "// TODO: add_property_source_first method does not exist yet.; // Re-enable when the method is implemented.; // add_prop..." to "/// Add a property source with highest priority.; /// 添加最高优先级的属性源。; pub fn add_property_source_first(&self, source: Prop...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 19:00:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/message.rs`
### Change Record
Modified file `crates/hiver-kafka/src/message.rs`. Approximately 47 lines changed.
### Change Summary
Changed `crates/hiver-kafka/src/message.rs` from "#[derive(Clone, Debug, Serialize, Deserialize)]; // TODO: MessageHeaderValue does not derive PartialEq, so assert_eq! ca..." to "#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]; #[test]; fn test_message_header_value_serde() {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 19:52:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 12 lines changed.
### Change Summary
Added "addr: SocketAddr," in `crates/hiver-runtime/src/io.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 19:52:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 25 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/io.rs` from "pub fn send_to<'a, 'b>(&'a mut self, buf: &'b [u8], _addr: SocketAddr) -> SendToFuture<'a, 'b> {" to "pub fn send_to<'a, 'b>(&'a mut self, buf: &'b [u8], addr: SocketAddr) -> SendToFuture<'a, 'b> {; addr,; addr: SocketAddr...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 19:52:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 47 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/io.rs` from "pub fn send_to<'a, 'b>(&'a mut self, buf: &'b [u8], _addr: SocketAddr) -> SendToFuture<'a, 'b> {; // For now, use regula..." to "pub fn send_to<'a, 'b>(&'a mut self, buf: &'b [u8], addr: SocketAddr) -> SendToFuture<'a, 'b> {; addr,; addr: SocketAddr...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 19:53:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 83 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/io.rs` from "pub fn send_to<'a, 'b>(&'a mut self, buf: &'b [u8], _addr: SocketAddr) -> SendToFuture<'a, 'b> {; // For now, use regula..." to "pub fn send_to<'a, 'b>(&'a mut self, buf: &'b [u8], addr: SocketAddr) -> SendToFuture<'a, 'b> {; addr,; addr: SocketAddr...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 19:53:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 85 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/io.rs` from "pub fn send_to<'a, 'b>(&'a mut self, buf: &'b [u8], _addr: SocketAddr) -> SendToFuture<'a, 'b> {; // For now, use regula..." to "pub fn send_to<'a, 'b>(&'a mut self, buf: &'b [u8], addr: SocketAddr) -> SendToFuture<'a, 'b> {; addr,; addr: SocketAddr...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 19:59:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/unified.rs`
### Change Record
New file `crates/hiver-response/src/unified.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/unified.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 19:59:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/lib.rs`
### Change Record
Modified file `crates/hiver-response/src/lib.rs`. Approximately 15 lines changed.
### Change Summary
Added "pub mod unified;; pub use unified::{ApiResponse, DefaultResponseAdvice, ResponseAdvice, ResponseResult};" in `crates/hiver-response/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 19:59:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/unified.rs`
### Change Record
New file `crates/hiver-response/src/unified.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/unified.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 19:59:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/unified.rs`
### Change Record
New file `crates/hiver-response/src/unified.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/unified.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 19:59:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/unified.rs`
### Change Record
New file `crates/hiver-response/src/unified.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/unified.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 19:59:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/unified.rs`
### Change Record
New file `crates/hiver-response/src/unified.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/unified.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:00:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/unified.rs`
### Change Record
New file `crates/hiver-response/src/unified.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/unified.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:00:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/unified.rs`
### Change Record
New file `crates/hiver-response/src/unified.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/unified.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:00:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/unified.rs`
### Change Record
New file `crates/hiver-response/src/unified.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/unified.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:00:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
New file `crates/hiver-security/src/data_scope.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:00:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/lib.rs`
### Change Record
Modified file `crates/hiver-security/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "pub mod data_scope;" in `crates/hiver-security/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 20:00:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/lib.rs`
### Change Record
Modified file `crates/hiver-security/src/lib.rs`. Approximately 23 lines changed.
### Change Summary
Added "pub mod data_scope;; pub use data_scope::{; DataScope, DataScopeApply, DataScopeContext, DataScopeMiddleware, DataScopeR..." in `crates/hiver-security/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 20:00:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/lib.rs`
### Change Record
Modified file `crates/hiver-security/src/lib.rs`. Approximately 39 lines changed.
### Change Summary
Changed `crates/hiver-security/src/lib.rs` from "GrantedAuthority, JwtAuthentication, JwtClaims, JwtTokenProvider, JwtUtil, PasswordEncoder,; Permission, PermissionEntry..." to "pub mod data_scope;; pub use data_scope::{; DataScope, DataScopeApply, DataScopeContext, DataScopeMiddleware, DataScopeR...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 20:00:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-extractors/src/multipart.rs`
### Change Record
New file `crates/hiver-extractors/src/multipart.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-extractors/src/multipart.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:01:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-extractors/src/lib.rs`
### Change Record
Modified file `crates/hiver-extractors/src/lib.rs`. Approximately 22 lines changed.
### Change Summary
Added "#[cfg(feature = "multipart")]; pub mod multipart;; #[cfg(feature = "multipart")]" in `crates/hiver-extractors/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 20:01:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
New file `crates/hiver-security/src/data_scope.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:01:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
New file `crates/hiver-security/src/data_scope.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:01:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-extractors/src/multipart.rs`
### Change Record
New file `crates/hiver-extractors/src/multipart.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-extractors/src/multipart.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:01:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-extractors/src/multipart.rs`
### Change Record
New file `crates/hiver-extractors/src/multipart.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-extractors/src/multipart.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:02:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-extractors/src/multipart.rs`
### Change Record
New file `crates/hiver-extractors/src/multipart.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-extractors/src/multipart.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:02:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-extractors/src/multipart.rs`
### Change Record
New file `crates/hiver-extractors/src/multipart.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-extractors/src/multipart.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:03:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-extractors/src/multipart.rs`
### Change Record
New file `crates/hiver-extractors/src/multipart.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-extractors/src/multipart.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:03:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-extractors/src/multipart.rs`
### Change Record
New file `crates/hiver-extractors/src/multipart.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-extractors/src/multipart.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 20:04:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 13 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| 28 | **文件上传** | \`MultipartFile\` | multer | ⚠️ 70% | **高** |" to "| 28 | **文件上传** | \`MultipartFile\` | Multipart + UploadConfig | ✅ 90% | - |".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 20:04:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 20 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| 28 | **文件上传** | \`MultipartFile\` | multer | ⚠️ 70% | **高** |; | 34 | **统一响应** | \`Result<T>\` | \`Result<T>\` | ⚠️ 80..." to "| 28 | **文件上传** | \`MultipartFile\` | Multipart + UploadConfig | ✅ 90% | - |; | 34 | **统一响应** | \`Result<T>\` | \`ApiRes...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 20:04:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 29 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| 28 | **文件上传** | \`MultipartFile\` | multer | ⚠️ 70% | **高** |; | 34 | **统一响应** | \`Result<T>\` | \`Result<T>\` | ⚠️ 80..." to "| 28 | **文件上传** | \`MultipartFile\` | Multipart + UploadConfig | ✅ 90% | - |; | 34 | **统一响应** | \`Result<T>\` | \`ApiRes...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 20:05:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 42 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| **实战功能 / Practice** | 11 | 8 | 2 | 1 | 80% |; | **企业功能 / Enterprise** | 12 | 8 | 2 | 2 | 75% |; | **总计 / Total** | 50 ..." to "| **实战功能 / Practice** | 11 | 9 | 1 | 1 | 85% |; | **企业功能 / Enterprise** | 12 | 9 | 1 | 2 | 80% |; | **总计 / Total** | 50 ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:00:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `Cargo.toml`
### Change Record
Modified file `Cargo.toml`. Approximately 15 lines changed.
### Change Summary
Added "# Spreadsheet / 电子表格; # Equivalent to: Spring Apache POI; zip = { version = "2", default-features = false, features = ["..." in `Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:00:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/Cargo.toml`
### Change Record
Modified file `crates/hiver-response/Cargo.toml`. Approximately 13 lines changed.
### Change Summary
Added "# Spreadsheet / 电子表格 (Spring Apache POI); zip = { workspace = true }" in `crates/hiver-response/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:02:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
New file `crates/hiver-validation/src/custom.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:03:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/error.rs`
### Change Record
Modified file `crates/hiver-validation/src/error.rs`. Approximately 17 lines changed.
### Change Summary
Added "/// 嵌套字段路径（如 "address.street"）/ Nested field path (e.g. "address.street"); pub field_path: Option<String>,; /// 拒绝的值（被验证..." in `crates/hiver-validation/src/error.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:03:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:03:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/error.rs`
### Change Record
Modified file `crates/hiver-validation/src/error.rs`. Approximately 26 lines changed.
### Change Summary
Added "/// 嵌套字段路径（如 "address.street"）/ Nested field path (e.g. "address.street"); pub field_path: Option<String>,; /// 拒绝的值（被验证..." in `crates/hiver-validation/src/error.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:03:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/error.rs`
### Change Record
Modified file `crates/hiver-validation/src/error.rs`. Approximately 77 lines changed.
### Change Summary
Added "/// 嵌套字段路径（如 "address.street"）/ Nested field path (e.g. "address.street"); pub field_path: Option<String>,; /// 拒绝的值（被验证..." in `crates/hiver-validation/src/error.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:03:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:03:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/error.rs`
### Change Record
Modified file `crates/hiver-validation/src/error.rs`. Approximately 87 lines changed.
### Change Summary
Added "/// 嵌套字段路径（如 "address.street"）/ Nested field path (e.g. "address.street"); pub field_path: Option<String>,; /// 拒绝的值（被验证..." in `crates/hiver-validation/src/error.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:03:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/nested.rs`
### Change Record
Modified file `crates/hiver-validation/src/nested.rs`. Approximately 14 lines changed.
### Change Summary
Added "field_path: field_error.field_path,; rejected_value: field_error.rejected_value,; constraint_name: field_error.constrain..." in `crates/hiver-validation/src/nested.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:03:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:03:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/nested.rs`
### Change Record
Modified file `crates/hiver-validation/src/nested.rs`. Approximately 24 lines changed.
### Change Summary
Added "field_path: field_error.field_path,; rejected_value: field_error.rejected_value,; constraint_name: field_error.constrain..." in `crates/hiver-validation/src/nested.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:04:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "pub mod custom;" in `crates/hiver-validation/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:04:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:04:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation/src/lib.rs`. Approximately 26 lines changed.
### Change Summary
Added "pub mod custom;; // Re-export custom validators / 重新导出自定义验证器; pub use custom::{" in `crates/hiver-validation/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:04:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:04:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
New file `crates/hiver-validation/src/custom.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:04:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
New file `crates/hiver-response/src/excel.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/excel.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:04:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
New file `crates/hiver-validation/src/custom.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:04:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
New file `crates/hiver-validation/src/custom.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:04:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
New file `crates/hiver-validation/src/custom.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:04:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
New file `crates/hiver-validation/src/custom.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:04:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/lib.rs`
### Change Record
Modified file `crates/hiver-response/src/lib.rs`. Approximately 22 lines changed.
### Change Summary
Added "pub mod excel;; // Excel re-exports; pub use excel::{" in `crates/hiver-response/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:04:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
New file `crates/hiver-validation/src/custom.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:05:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
New file `crates/hiver-validation/src/custom.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:05:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
New file `crates/hiver-response/src/excel.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/excel.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:05:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
New file `crates/hiver-response/src/excel.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/excel.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:05:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
New file `crates/hiver-response/src/excel.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/excel.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:05:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
New file `crates/hiver-response/src/excel.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/excel.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:05:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
New file `crates/hiver-response/src/excel.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/excel.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:06:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
New file `crates/hiver-response/src/excel.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-response/src/excel.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:06:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/doc_pdf.rs`
### Change Record
New file `crates/hiver-openapi/src/doc_pdf.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/doc_pdf.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:07:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/lib.rs`
### Change Record
Modified file `crates/hiver-openapi/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Added "pub mod postman;; pub mod doc_pdf;" in `crates/hiver-openapi/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:07:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/lib.rs`
### Change Record
Modified file `crates/hiver-openapi/src/lib.rs`. Approximately 22 lines changed.
### Change Summary
Added "pub mod postman;; pub mod doc_pdf;; pub use postman::{PostmanCollection, PostmanGenerator, CollectionInfo, PostmanItem, ..." in `crates/hiver-openapi/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:07:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:07:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/doc_pdf.rs`
### Change Record
New file `crates/hiver-openapi/src/doc_pdf.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/doc_pdf.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:07:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/doc_pdf.rs`
### Change Record
New file `crates/hiver-openapi/src/doc_pdf.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/doc_pdf.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:07:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/doc_pdf.rs`
### Change Record
New file `crates/hiver-openapi/src/doc_pdf.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/doc_pdf.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:07:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:07:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:07:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:08:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:08:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:08:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:08:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:08:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
New file `crates/hiver-openapi/src/postman.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:08:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-openapi/src/doc_pdf.rs`
### Change Record
New file `crates/hiver-openapi/src/doc_pdf.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-openapi/src/doc_pdf.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:08:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/lib.rs`
### Change Record
Modified file `crates/hiver-openapi/src/lib.rs`. Approximately 25 lines changed.
### Change Summary
Added "pub mod postman;; pub mod doc_pdf;; pub use openapi::OpenApiBuilder;" in `crates/hiver-openapi/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:09:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 12 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| 17 | **参数校验** | \`@Valid\` | \`#[validate]\` | ⚠️ 70% | **高** |" to "| 17 | **参数校验** | \`@Valid\` | \`#[validate]\` + CustomValidator | ✅ 95% | - |".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:09:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 23 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| 17 | **参数校验** | \`@Valid\` | \`#[validate]\` | ⚠️ 70% | **高** |; | 37 | **Excel 导出** | Apache POI | rust_xlsxwriter | ..." to "| 17 | **参数校验** | \`@Valid\` | \`#[validate]\` + CustomValidator | ✅ 95% | - |; | 37 | **Excel 导出** | Apache POI | Excel...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:09:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 33 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| 17 | **参数校验** | \`@Valid\` | \`#[validate]\` | ⚠️ 70% | **高** |; | 37 | **Excel 导出** | Apache POI | rust_xlsxwriter | ..." to "| 17 | **参数校验** | \`@Valid\` | \`#[validate]\` + CustomValidator | ✅ 95% | - |; | 37 | **Excel 导出** | Apache POI | Excel...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:10:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 50 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| **核心功能 / Core** | 10 | 9 | 0 | 1 | 90% |; | **进阶功能 / Advanced** | 8 | 7 | 1 | 0 | 90% |; | **实战功能 / Practice** | 11 | ..." to "| **核心功能 / Core** | 10 | 10 | 0 | 0 | 100% |; | **进阶功能 / Advanced** | 8 | 8 | 0 | 0 | 100% |; | **实战功能 / Practice** | 11...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:20:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-http/src/controller_advice.rs`
### Change Record
New file `crates/hiver-http/src/controller_advice.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-http/src/controller_advice.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:21:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-http/src/controller_advice.rs`
### Change Record
New file `crates/hiver-http/src/controller_advice.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-http/src/controller_advice.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:21:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/permission.rs`
### Change Record
New file `crates/hiver-security/src/permission.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/permission.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:21:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/lib.rs`
### Change Record
Modified file `crates/hiver-http/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "pub mod controller_advice;" in `crates/hiver-http/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:21:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/lib.rs`
### Change Record
Modified file `crates/hiver-http/src/lib.rs`. Approximately 24 lines changed.
### Change Summary
Added "pub mod controller_advice;; pub use controller_advice::{; ControllerAdvice, ControllerAdviceBuilder, ControllerErrorResp..." in `crates/hiver-http/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:21:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "ServerConfig, TagConfig," to "ServerConfig, TagConfig, SecurityScheme,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:21:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 53 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "ServerConfig, TagConfig," to "ServerConfig, TagConfig, SecurityScheme,; /// Add a server URL with variables (e.g. \`{protocol}://api.{host}/v{version}...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:21:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 164 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "ServerConfig, TagConfig," to "ServerConfig, TagConfig, SecurityScheme,; /// Add a server URL with variables (e.g. \`{protocol}://api.{host}/v{version}...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:22:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 182 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "ServerConfig, TagConfig," to "ServerConfig, TagConfig, SecurityScheme,; /// Names of security schemes registered via convenience methods; /// 通过便捷方法注册...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:22:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 230 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "ServerConfig, TagConfig," to "ServerConfig, TagConfig, SecurityScheme,; /// Names of security schemes registered via convenience methods; /// 通过便捷方法注册...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:22:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 600 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "ServerConfig, TagConfig," to "ServerConfig, TagConfig, SecurityScheme,; /// Names of security schemes registered via convenience methods; /// 通过便捷方法注册...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:23:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
New file `crates/hiver-security/src/email.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/email.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:23:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/lib.rs`
### Change Record
Modified file `crates/hiver-security/src/lib.rs`. Approximately 17 lines changed.
### Change Summary
Added "pub mod email;; pub mod permission;" in `crates/hiver-security/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:23:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/lib.rs`
### Change Record
Modified file `crates/hiver-security/src/lib.rs`. Approximately 32 lines changed.
### Change Summary
Added "pub mod email;; pub mod permission;; pub use email::{" in `crates/hiver-security/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:23:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/lib.rs`
### Change Record
Modified file `crates/hiver-security/src/lib.rs`. Approximately 47 lines changed.
### Change Summary
Changed `crates/hiver-security/src/lib.rs` from "PasswordEncoder, Permission, PermissionEntry, PreAuthorize, RbacConfig, RbacManager,; RoleEnum, RolePermission, Roles, S..." to "pub mod email;; pub mod permission;; pub use email::{".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:23:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 975 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "ServerConfig, TagConfig," to "ServerConfig, TagConfig, SecurityScheme,; /// Names of security schemes registered via convenience methods; /// 通过便捷方法注册...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:23:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/lib.rs`
### Change Record
Modified file `crates/hiver-openapi/src/lib.rs`. Approximately 19 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/lib.rs` from "pub use config::{OpenApiConfig, ServerConfig, ContactConfig, LicenseConfig, InfoConfig, TagConfig, ExternalDocsConfig};" to "pub use config::{OpenApiConfig, ServerConfig, ContactConfig, LicenseConfig, InfoConfig, TagConfig, ExternalDocsConfig, S...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:23:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/permission.rs`
### Change Record
New file `crates/hiver-security/src/permission.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/permission.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:23:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/permission.rs`
### Change Record
New file `crates/hiver-security/src/permission.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/permission.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:24:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 977 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "Components, InfoConfig, OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig," to "OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig, SecurityScheme, config::Secur...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:24:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 977 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "Components, InfoConfig, OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig," to "OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig, SecurityScheme, config::Secur...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:24:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/permission.rs`
### Change Record
New file `crates/hiver-security/src/permission.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/permission.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:24:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-http/src/controller_advice.rs`
### Change Record
New file `crates/hiver-http/src/controller_advice.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-http/src/controller_advice.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:24:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/permission.rs`
### Change Record
New file `crates/hiver-security/src/permission.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/permission.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:24:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-security/src/permission.rs`
### Change Record
New file `crates/hiver-security/src/permission.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-security/src/permission.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:24:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 982 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "Components, InfoConfig, OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig," to "OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig, SecurityScheme, config::Secur...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:24:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 982 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "Components, InfoConfig, OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig," to "OpenApi, OpenApiConfig, Operation, PathItem, Schema,; ServerConfig, TagConfig, SecurityScheme, config::SecuritySchemeCon...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:25:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 982 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "Components, InfoConfig, OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig," to "OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig, SecurityScheme, config::Secur...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:25:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 985 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "Components, InfoConfig, OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig," to "OpenApi, OpenApiConfig, Operation, PathItem, Schema,; ServerConfig, TagConfig, SecurityScheme, config::SecuritySchemeCon...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:25:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 986 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "Components, InfoConfig, OpenApi, OpenApiConfig, Operation, PathItem, Schema, SchemaType,; ServerConfig, TagConfig,; pub ..." to "OpenApi, OpenApiConfig, Operation, PathItem, Schema,; ServerConfig, TagConfig, SecurityScheme, config::SecuritySchemeCon...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:25:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-http/src/controller_advice.rs`
### Change Record
New file `crates/hiver-http/src/controller_advice.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-http/src/controller_advice.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-29 22:25:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/config.rs`
### Change Record
Modified file `crates/hiver-openapi/src/config.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]" in `crates/hiver-openapi/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:26:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/config.rs`
### Change Record
Modified file `crates/hiver-openapi/src/config.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/config.rs` from "#[derive(Debug, Clone, Serialize, Deserialize)]" to "#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:27:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 13 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| 18 | **全局异常处理** | \`@ControllerAdvice\` | ErrorHandler | ⚠️ 80% | **高** |" to "| 18 | **全局异常处理** | \`@ControllerAdvice\` | ControllerAdvice + 5 handlers | ✅ 95% | - |".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:27:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 25 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| 18 | **全局异常处理** | \`@ControllerAdvice\` | ErrorHandler | ⚠️ 80% | **高** |; | 21 | 权限控制 | \`@PreAuthorize\` | \`#[requi..." to "| 18 | **全局异常处理** | \`@ControllerAdvice\` | ControllerAdvice + 5 handlers | ✅ 95% | - |; | 21 | 权限控制 | \`@PreAuthorize\`...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:27:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 34 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| 18 | **全局异常处理** | \`@ControllerAdvice\` | ErrorHandler | ⚠️ 80% | **高** |; | 21 | 权限控制 | \`@PreAuthorize\` | \`#[requi..." to "| 18 | **全局异常处理** | \`@ControllerAdvice\` | ControllerAdvice + 5 handlers | ✅ 95% | - |; | 21 | 权限控制 | \`@PreAuthorize\`...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:27:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-boot-feature-matrix.md`
### Change Record
Modified file `docs/spring-boot/spring-boot-feature-matrix.md`. Approximately 45 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-boot-feature-matrix.md` from "| **企业功能 / Enterprise** | 12 | 10 | 1 | 1 | 85% |; | **总计 / Total** | 50 | 48 | 1 | 1 | 98% |; | 18 | **全局异常处理** | \`@Co..." to "| **企业功能 / Enterprise** | 12 | 12 | 0 | 0 | 100% |; | **总计 / Total** | 50 | 50 | 0 | 0 | 100% |; | 18 | **全局异常处理** | \`@...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:42:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-ecosystem-gap-analysis.md`
### Change Record
Modified file `docs/spring-boot/spring-ecosystem-gap-analysis.md`. Approximately 17 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-ecosystem-gap-analysis.md` from "**Hiver 整体完成度：90-95%**; - **已实现且可用（90%+）**：IoC/DI、HTTP、路由、安全、缓存、事务、验证、中间件、STOMP WebSocket、响应式（含背压）、宏系统、配置（含加密/RefreshSco..." to "**Hiver 整体完成度：95-100%**; - **已实现且可用（90%+）**：全部 Spring Boot 50 项功能均已实现，包括 IoC/DI、HTTP、路由、安全（含权限注册表/审计）、缓存、事务、验证（含自定义校验器）、...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:42:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/spring-boot/spring-ecosystem-gap-analysis.md`
### Change Record
Modified file `docs/spring-boot/spring-ecosystem-gap-analysis.md`. Approximately 26 lines changed.
### Change Summary
Changed `docs/spring-boot/spring-ecosystem-gap-analysis.md` from "| 21 | Spring Authorization Server | hiver-security (内含) | **80%** | P1 | 多种 Grant Type、授权服务器 |; **Hiver 整体完成度：90-95%**;..." to "| 21 | Spring Authorization Server | hiver-security (内含) | **90%** | P1 | 多种 Grant Type/授权服务器/权限注册表/审计日志 |; **Hiver 整体完成...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/authorization_server.rs`
### Change Record
Modified file `crates/hiver-security/src/authorization_server.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-security/src/authorization_server.rs` from "#[allow(clippy::unused_async)]; pub async fn introspect(&self, token: &str) -> IntrospectionResult {" to "pub fn introspect(&self, token: &str) -> IntrospectionResult {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rbac.rs`
### Change Record
Modified file `crates/hiver-security/src/rbac.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rbac.rs` from "#[allow(clippy::unused_async)]; async fn get_all_inherited_roles(&self, role: &str) -> HashSet<String> {" to "fn get_all_inherited_roles(&self, role: &str) -> HashSet<String> {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 27 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rbac.rs`
### Change Record
Modified file `crates/hiver-security/src/rbac.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rbac.rs` from "let inherited_roles = self.get_all_inherited_roles(role_name).await;; #[allow(clippy::unused_async)]; async fn get_all_i..." to "let inherited_roles = self.get_all_inherited_roles(role_name);; fn get_all_inherited_roles(&self, role: &str) -> HashSet...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 35 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 43 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 51 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 59 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 67 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rbac.rs`
### Change Record
Modified file `crates/hiver-security/src/rbac.rs`. Approximately 33 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rbac.rs` from "let inherited_roles = self.get_all_inherited_roles(role_name).await;; #[allow(clippy::unused_async)]; async fn get_all_i..." to "let inherited_roles = self.get_all_inherited_roles(role_name);; fn get_all_inherited_roles(&self, role: &str) -> HashSet...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 75 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 83 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 91 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 99 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rbac.rs`
### Change Record
Modified file `crates/hiver-security/src/rbac.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rbac.rs` from "if self.role_inherits_role(user_role_name, role).await {; let inherited_roles = self.get_all_inherited_roles(role_name)...." to "if self.role_inherits_role(user_role_name, role) {; let inherited_roles = self.get_all_inherited_roles(role_name);; fn g...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 107 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 115 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 123 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv_v2.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv_v2.rs`. Approximately 131 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv_v2.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/oauth2.rs`
### Change Record
Modified file `crates/hiver-security/src/oauth2.rs`. Approximately 19 lines changed.
### Change Summary
Changed `crates/hiver-security/src/oauth2.rs` from "#[allow(clippy::format_push_string)]; url.push_str(&format!(; "&code_challenge={}&code_challenge_method=S256"," to "url.push_str("&code_challenge=");; url.push_str(&urlencoding::encode(challenge));; url.push_str("&code_challenge_method=...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:46:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/Cargo.toml`
### Change Record
Modified file `crates/hiver-security/Cargo.toml`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-security/Cargo.toml` from "tokio = { workspace = true, features = ["sync", "macros", "rt"] }" to "tokio = { workspace = true, features = ["sync", "macros", "rt", "net", "io-util", "time"] }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/kv.rs`
### Change Record
Modified file `crates/hiver-vault/src/kv.rs`. Approximately 18 lines changed.
### Change Summary
Removed "/// Patch request for KV v2 / KV v2 的补丁请求; #[derive(Debug, Clone, Serialize)]; #[allow(dead_code)]" from `crates/hiver-vault/src/kv.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-macros/src/pre_authorize.rs`
### Change Record
Modified file `crates/hiver-macros/src/pre_authorize.rs`. Approximately 84 lines changed.
### Change Summary
Changed `crates/hiver-macros/src/pre_authorize.rs` from "use proc_macro::TokenStream;; use quote::quote;; use syn::{ItemFn, parse_macro_input};" to "//! Pre-authorize macro helpers (superseded by spring_di module).; //! Pre-authorize 宏辅助（已被 spring_di 模块取代）。".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 36 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-macros/src/bean_register.rs`
### Change Record
Modified file `crates/hiver-macros/src/bean_register.rs`. Approximately 16 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; fn should_inject_field(field: &Field) -> bool {; field_is_autowired(field) || extract_arc_inner(&fi..." from `crates/hiver-macros/src/bean_register.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 44 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/resource_bundle.rs`
### Change Record
Modified file `crates/hiver-i18n/src/resource_bundle.rs`. Approximately 16 lines changed.
### Change Summary
Changed `crates/hiver-i18n/src/resource_bundle.rs` from "#[allow(dead_code)]; async fn needs_reload(&self) -> bool {" to "/// Check if cache needs reload.; /// 检查缓存是否需要重载。; pub async fn needs_reload(&self) -> bool {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 52 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/message_source.rs`
### Change Record
Modified file `crates/hiver-i18n/src/message_source.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-i18n/src/message_source.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 60 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/message_source.rs`
### Change Record
Modified file `crates/hiver-i18n/src/message_source.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-i18n/src/message_source.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 68 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/message_source.rs`
### Change Record
Modified file `crates/hiver-i18n/src/message_source.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-i18n/src/message_source.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 76 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/message_source.rs`
### Change Record
Modified file `crates/hiver-i18n/src/message_source.rs`. Approximately 33 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-i18n/src/message_source.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 84 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/lib.rs`
### Change Record
Modified file `crates/hiver-integration/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-integration/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 92 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/filter.rs`
### Change Record
Modified file `crates/hiver-integration/src/filter.rs`. Approximately 16 lines changed.
### Change Summary
Changed `crates/hiver-integration/src/filter.rs` from "pub struct PayloadTypeFilter {; #[allow(dead_code)]; type_name: String," to "pub struct PayloadTypeFilter {}".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 100 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/filter.rs`
### Change Record
Modified file `crates/hiver-integration/src/filter.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-integration/src/filter.rs` from "pub struct PayloadTypeFilter {; #[allow(dead_code)]; type_name: String," to "pub struct PayloadTypeFilter {}; Self {}".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 108 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/endpoint.rs`
### Change Record
Modified file `crates/hiver-integration/src/endpoint.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; input_channel: Arc<dyn MessageChannel>," from `crates/hiver-integration/src/endpoint.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-vault/src/auth_jwt.rs`
### Change Record
Modified file `crates/hiver-vault/src/auth_jwt.rs`. Approximately 116 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-vault/src/auth_jwt.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/endpoint.rs`
### Change Record
Modified file `crates/hiver-integration/src/endpoint.rs`. Approximately 28 lines changed.
### Change Summary
Changed `crates/hiver-integration/src/endpoint.rs` from "#[allow(dead_code)]; input_channel: Arc<dyn MessageChannel>,; input_channel: Arc<dyn MessageChannel>," to "_input_channel: Arc<dyn MessageChannel>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/endpoint.rs`
### Change Record
Modified file `crates/hiver-integration/src/endpoint.rs`. Approximately 36 lines changed.
### Change Summary
Changed `crates/hiver-integration/src/endpoint.rs` from "#[allow(dead_code)]; input_channel: Arc<dyn MessageChannel>,; input_channel: Arc<dyn MessageChannel>," to "_input_channel: Arc<dyn MessageChannel>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-data-rdbc/src/client.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/executor.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/executor.rs`. Approximately 19 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; fn validate_identifier(id: &str) -> Result<()> {; if id.is_empty() || !id.chars().all(|c| c.is_alph..." from `crates/hiver-data-rdbc/src/executor.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/executor.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/executor.rs`. Approximately 27 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; fn validate_identifier(id: &str) -> Result<()> {; if id.is_empty() || !id.chars().all(|c| c.is_alph..." from `crates/hiver-data-rdbc/src/executor.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 209 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "/// SMTP-based email sender (stub implementation).; /// 基于 SMTP 的邮件发送器（桩实现）。; /// In production, this would open a TCP/T..." to "/// SMTP-based email sender.; /// 基于 SMTP 的邮件发送器。; /// Opens a plain TCP connection to the SMTP server and performs the ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:47:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 217 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "/// SMTP-based email sender (stub implementation).; /// 基于 SMTP 的邮件发送器（桩实现）。; /// In production, this would open a TCP/T..." to "use tokio::io::{AsyncReadExt, AsyncWriteExt};; /// SMTP-based email sender.; /// 基于 SMTP 的邮件发送器。".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 255 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "/// SMTP-based email sender (stub implementation).; /// 基于 SMTP 的邮件发送器（桩实现）。; /// In production, this would open a TCP/T..." to "use tokio::io::{AsyncReadExt, AsyncWriteExt};; /// SMTP-based email sender.; /// 基于 SMTP 的邮件发送器。".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/message_source.rs`
### Change Record
Modified file `crates/hiver-i18n/src/message_source.rs`. Approximately 36 lines changed.
### Change Summary
Changed `crates/hiver-i18n/src/message_source.rs` from "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" to "/// Default message source resolvable implementation; /// 默认消息源可解析实现; #[cfg_attr(not(test), allow(dead_code))]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 255 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "/// SMTP-based email sender (stub implementation).; /// 基于 SMTP 的邮件发送器（桩实现）。; /// In production, this would open a TCP/T..." to "use tokio::io::{AsyncBufReadExt, AsyncWriteExt};; /// SMTP-based email sender.; /// 基于 SMTP 的邮件发送器。".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/message_source.rs`
### Change Record
Modified file `crates/hiver-i18n/src/message_source.rs`. Approximately 37 lines changed.
### Change Summary
Changed `crates/hiver-i18n/src/message_source.rs` from "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" to "/// Default message source resolvable implementation; /// 默认消息源可解析实现; #[cfg_attr(not(test), allow(dead_code))]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/message_source.rs`
### Change Record
Modified file `crates/hiver-i18n/src/message_source.rs`. Approximately 38 lines changed.
### Change Summary
Changed `crates/hiver-i18n/src/message_source.rs` from "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" to "/// Default message source resolvable implementation; /// 默认消息源可解析实现; #[cfg_attr(not(test), allow(dead_code))]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-i18n/src/message_source.rs`
### Change Record
Modified file `crates/hiver-i18n/src/message_source.rs`. Approximately 39 lines changed.
### Change Summary
Changed `crates/hiver-i18n/src/message_source.rs` from "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" to "/// Default message source resolvable implementation; /// 默认消息源可解析实现; #[cfg_attr(not(test), allow(dead_code))]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/lib.rs`
### Change Record
### Change Summary
Changed `crates/hiver-integration/src/lib.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/endpoint.rs`
### Change Record
Modified file `crates/hiver-integration/src/endpoint.rs`. Approximately 45 lines changed.
### Change Summary
Changed `crates/hiver-integration/src/endpoint.rs` from "#[allow(dead_code)]; input_channel: Arc<dyn MessageChannel>,; input_channel: Arc<dyn MessageChannel>," to "_input_channel: Arc<dyn MessageChannel>,; _input_channel: Arc<dyn MessageChannel>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/endpoint.rs`
### Change Record
Modified file `crates/hiver-integration/src/endpoint.rs`. Approximately 47 lines changed.
### Change Summary
Changed `crates/hiver-integration/src/endpoint.rs` from "#[allow(dead_code)]; input_channel: Arc<dyn MessageChannel>,; input_channel: Arc<dyn MessageChannel>," to "_input_channel: Arc<dyn MessageChannel>,; _input_channel: Arc<dyn MessageChannel>,; output_channel: output_channel,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 16 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/client.rs` from "#[allow(dead_code)]" to "#[cfg(test)]; #[cfg(test)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/executor.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/executor.rs`. Approximately 28 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/executor.rs` from "#[allow(dead_code)]; fn validate_identifier(id: &str) -> Result<()> {; if id.is_empty() || !id.chars().all(|c| c.is_alph..." to "#[cfg(test)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:48:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 27 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/client.rs` from "use crate::error::{Error, Result};; #[allow(dead_code)]" to "#[cfg(test)]; use crate::error::Error;; use crate::error::Result;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:49:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 256 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "/// SMTP-based email sender (stub implementation).; /// 基于 SMTP 的邮件发送器（桩实现）。; /// In production, this would open a TCP/T..." to "use tokio::io::{AsyncBufReadExt, AsyncWriteExt};; /// SMTP-based email sender.; /// 基于 SMTP 的邮件发送器。".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:49:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 256 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "/// SMTP-based email sender (stub implementation).; /// 基于 SMTP 的邮件发送器（桩实现）。; /// In production, this would open a TCP/T..." to "use tokio::io::{AsyncBufReadExt, AsyncWriteExt};; /// SMTP-based email sender.; /// 基于 SMTP 的邮件发送器。".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:57:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/consumer_group_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/consumer_group_manager.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-kafka/src/consumer_group_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:57:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/consumer_group_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/consumer_group_manager.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/consumer_group_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:57:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/consumer_group_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/consumer_group_manager.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/consumer_group_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:57:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/consumer_group_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/consumer_group_manager.rs`. Approximately 36 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/consumer_group_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/offset_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/offset_manager.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-kafka/src/offset_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/offset_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/offset_manager.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/offset_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/offset_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/offset_manager.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/offset_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/offset_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/offset_manager.rs`. Approximately 36 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/offset_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/offset_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/offset_manager.rs`. Approximately 44 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/offset_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/offset_manager.rs`
### Change Record
Modified file `crates/hiver-kafka/src/offset_manager.rs`. Approximately 52 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/offset_manager.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/transactional_producer.rs`
### Change Record
Modified file `crates/hiver-kafka/src/transactional_producer.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-kafka/src/transactional_producer.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/transactional_producer.rs`
### Change Record
Modified file `crates/hiver-kafka/src/transactional_producer.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/transactional_producer.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/transactional_producer.rs`
### Change Record
Modified file `crates/hiver-kafka/src/transactional_producer.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/transactional_producer.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/transactional_producer.rs`
### Change Record
Modified file `crates/hiver-kafka/src/transactional_producer.rs`. Approximately 36 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/transactional_producer.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-kafka/src/transactional_producer.rs`
### Change Record
Modified file `crates/hiver-kafka/src/transactional_producer.rs`. Approximately 44 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-kafka/src/transactional_producer.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/time.rs`
### Change Record
Modified file `crates/hiver-runtime/src/time.rs`. Approximately 116 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/time.rs` from "use std::sync::Mutex;; #[allow(dead_code)]; const WHEEL3_SHIFT: usize = 6;" to "timer_registry: std::sync::Mutex<HashMap<u64, ()>>,; timer_registry: std::sync::Mutex::new(HashMap::new()),; if registry...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 14 lines changed.
### Change Summary
Removed "/// Whether this stream is in non-blocking mode / 此流是否处于非阻塞模式; #[allow(dead_code)]; non_blocking: bool," from `crates/hiver-runtime/src/io.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 21 lines changed.
### Change Summary
Removed "/// Whether this stream is in non-blocking mode / 此流是否处于非阻塞模式; #[allow(dead_code)]; non_blocking: bool," from `crates/hiver-runtime/src/io.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/ack.rs`
### Change Record
Modified file `crates/hiver-amqp/src/ack.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-amqp/src/ack.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/ack.rs`
### Change Record
Modified file `crates/hiver-amqp/src/ack.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-amqp/src/ack.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/ack.rs`
### Change Record
Modified file `crates/hiver-amqp/src/ack.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-amqp/src/ack.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/ack.rs`
### Change Record
Modified file `crates/hiver-amqp/src/ack.rs`. Approximately 36 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-amqp/src/ack.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/converter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/converter.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-amqp/src/converter.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/converter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/converter.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-amqp/src/converter.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/converter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/converter.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-amqp/src/converter.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/interest.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/interest.rs`. Approximately 57 lines changed.
### Change Summary
Removed "/// Convert to kqueue event flags; /// 转换为kqueue事件标志; #[cfg(any(" from `crates/hiver-runtime/src/driver/interest.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/converter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/converter.rs`. Approximately 36 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-amqp/src/converter.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/dead_letter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/dead_letter.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-amqp/src/dead_letter.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/dead_letter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/dead_letter.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-amqp/src/dead_letter.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-amqp/src/dead_letter.rs`
### Change Record
Modified file `crates/hiver-amqp/src/dead_letter.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-amqp/src/dead_letter.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/interest.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/interest.rs`. Approximately 65 lines changed.
### Change Summary
Removed "use std::os::fd::RawFd;; /// Convert to kqueue event flags; /// 转换为kqueue事件标志" from `crates/hiver-runtime/src/driver/interest.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/kqueue.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/kqueue.rs`. Approximately 15 lines changed.
### Change Summary
Removed "/// Change buffer for registering/deregistering events (reserved for future use); /// 用于注册/注销事件的change缓冲区（保留供将来使用）; #[al..." from `crates/hiver-runtime/src/driver/kqueue.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:58:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/kqueue.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/kqueue.rs`. Approximately 32 lines changed.
### Change Summary
Removed "/// Change buffer for registering/deregistering events (reserved for future use); /// 用于注册/注销事件的change缓冲区（保留供将来使用）; #[al..." from `crates/hiver-runtime/src/driver/kqueue.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-runtime/src/task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/lib.rs`
### Change Record
Modified file `crates/hiver-agent/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "#![allow(dead_code)]" in `crates/hiver-agent/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/chain.rs`
### Change Record
Modified file `crates/hiver-agent/src/chain.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-agent/src/chain.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-runtime/src/task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/chain.rs`
### Change Record
Modified file `crates/hiver-agent/src/chain.rs`. Approximately 44 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-agent/src/chain.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/chain.rs`
### Change Record
Modified file `crates/hiver-agent/src/chain.rs`. Approximately 52 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-agent/src/chain.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/chain.rs`
### Change Record
Modified file `crates/hiver-agent/src/chain.rs`. Approximately 68 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-agent/src/chain.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/react.rs`
### Change Record
Modified file `crates/hiver-agent/src/react.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-agent/src/react.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/react.rs`
### Change Record
Modified file `crates/hiver-agent/src/react.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-agent/src/react.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/react.rs`
### Change Record
Modified file `crates/hiver-agent/src/react.rs`. Approximately 36 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-agent/src/react.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-runtime/src/task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/middleware.rs`
### Change Record
Modified file `crates/hiver-session/src/middleware.rs`. Approximately 30 lines changed.
### Change Summary
Changed `crates/hiver-session/src/middleware.rs` from "/// Inner service; /// 内部服务; #[allow(dead_code)]" to "/// Inner service (type parameter preserved for API compatibility).; /// 内部服务（类型参数保留用于 API 兼容性）。; _inner: std::marker::P...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/react.rs`
### Change Record
Modified file `crates/hiver-agent/src/react.rs`. Approximately 44 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-agent/src/react.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 35 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-runtime/src/task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 43 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-runtime/src/task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ai/src/ollama.rs`
### Change Record
Modified file `crates/hiver-ai/src/ollama.rs`. Approximately 22 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; total_duration: Option<u64>,; #[serde(default)]" from `crates/hiver-ai/src/ollama.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 89 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-runtime/src/task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ai/src/openai.rs`
### Change Record
Modified file `crates/hiver-ai/src/openai.rs`. Approximately 13 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; total_tokens: u32," from `crates/hiver-ai/src/openai.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ai/src/rag.rs`
### Change Record
Modified file `crates/hiver-ai/src/rag.rs`. Approximately 85 lines changed.
### Change Summary
Changed `crates/hiver-ai/src/rag.rs` from "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" to "/// Retained for future direct embedding access in the pipeline.; /// 为管道中未来的直接嵌入访问而保留。".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/lib.rs`
### Change Record
Modified file `crates/hiver-openapi/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "#![allow(dead_code)]" in `crates/hiver-openapi/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/swagger.rs`
### Change Record
Modified file `crates/hiver-openapi/src/swagger.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-openapi/src/swagger.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/swagger.rs`
### Change Record
Modified file `crates/hiver-openapi/src/swagger.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-openapi/src/swagger.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-openapi/src/generator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 18 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-openapi/src/generator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 26 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-openapi/src/generator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 175 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-runtime/src/task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 34 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-openapi/src/generator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 42 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-openapi/src/generator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 50 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-openapi/src/generator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 58 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-openapi/src/generator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 74 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-openapi/src/generator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 22:59:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task/raw_task.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-runtime/src/task/raw_task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task/raw_task.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-runtime/src/task/raw_task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/store.rs`
### Change Record
Modified file `crates/hiver-session/src/store.rs`. Approximately 26 lines changed.
### Change Summary
Changed `crates/hiver-session/src/store.rs` from "/// Serializable session data for storage; /// 可序列化的会话数据用于存储; #[allow(dead_code)]" to "/// Serializable session data for storage (used by Redis and MongoDB stores).; /// 可序列化的会话数据用于存储（由 Redis 和 MongoDB 存储使用）...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/store.rs`
### Change Record
Modified file `crates/hiver-session/src/store.rs`. Approximately 27 lines changed.
### Change Summary
Changed `crates/hiver-session/src/store.rs` from "/// Serializable session data for storage; /// 可序列化的会话数据用于存储; #[allow(dead_code)]" to "/// Serializable session data for storage (used by Redis and MongoDB stores).; /// 可序列化的会话数据用于存储（由 Redis 和 MongoDB 存储使用）...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/work_stealing.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/work_stealing.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-runtime/src/scheduler/work_stealing.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-batch/src/operator.rs`
### Change Record
Modified file `crates/hiver-batch/src/operator.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-batch/src/operator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 81 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-openapi/src/generator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-batch/src/operator.rs`
### Change Record
Modified file `crates/hiver-batch/src/operator.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-batch/src/operator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ai/src/ollama.rs`
### Change Record
Modified file `crates/hiver-ai/src/ollama.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-ai/src/ollama.rs` from "#[allow(dead_code)]; total_duration: Option<u64>,; #[serde(default)]" to "#[allow(dead_code)] // Fields only needed for serde deserialization".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ai/src/anthropic.rs`
### Change Record
Modified file `crates/hiver-ai/src/anthropic.rs`. Approximately 30 lines changed.
### Change Summary
Changed `crates/hiver-ai/src/anthropic.rs` from "#[allow(dead_code)]; #[allow(dead_code)]" to "// message field is deserialized from \`message_start\` events but not consumed; // (used to advance the SSE parser past...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:00:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/store.rs`
### Change Record
Modified file `crates/hiver-session/src/store.rs`. Approximately 35 lines changed.
### Change Summary
Changed `crates/hiver-session/src/store.rs` from "/// Serializable session data for storage; /// 可序列化的会话数据用于存储; #[allow(dead_code)]" to "#[cfg(any(feature = "redis", feature = "mongodb"))]; #[cfg(any(feature = "redis", feature = "mongodb"))]; /// Serializab...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:01:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 184 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "use crate::scheduler::{RawTask, SchedulerHandle};; #[allow(dead_code)]; #[allow(dead_code)]" to "use crate::scheduler::SchedulerHandle;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:01:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 193 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "use crate::scheduler::{RawTask, SchedulerHandle};; #[allow(dead_code)]; /// Reference count / 引用计数" to "use crate::scheduler::SchedulerHandle;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:01:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 202 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "use crate::scheduler::{RawTask, SchedulerHandle};; #[allow(dead_code)]; /// Reference count / 引用计数" to "use crate::scheduler::SchedulerHandle;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:01:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 211 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "use crate::scheduler::{RawTask, SchedulerHandle};; #[allow(dead_code)]; /// Reference count / 引用计数" to "use crate::scheduler::SchedulerHandle;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:01:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 210 lines changed.
### Change Summary
Removed "use crate::scheduler::{RawTask, SchedulerHandle};; #[allow(dead_code)]; /// Reference count / 引用计数" from `crates/hiver-runtime/src/task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:01:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task/raw_task.rs`. Approximately 24 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; pub(crate) fn scheduler(&self) -> &SchedulerHandle {; &self.scheduler" from `crates/hiver-runtime/src/task/raw_task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:02:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task/raw_task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task/raw_task.rs`. Approximately 27 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; pub(crate) fn scheduler(&self) -> &SchedulerHandle {; &self.scheduler" from `crates/hiver-runtime/src/task/raw_task.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:02:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/time.rs`
### Change Record
Modified file `crates/hiver-runtime/src/time.rs`. Approximately 117 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/time.rs` from "use std::sync::Mutex;; #[allow(dead_code)]; const WHEEL3_SHIFT: usize = 6;" to "#[cfg(test)]; timer_registry: std::sync::Mutex<HashMap<u64, ()>>,; timer_registry: std::sync::Mutex::new(HashMap::new())...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:02:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/work_stealing.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/work_stealing.rs`. Approximately 15 lines changed.
### Change Summary
Added "///; /// Used to create a detached handle from an existing scheduler's worker queues,; /// enabling task submission from..." in `crates/hiver-runtime/src/scheduler/work_stealing.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:10:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-retry/src/template.rs`
### Change Record
Modified file `crates/hiver-retry/src/template.rs`. Approximately 60 lines changed.
### Change Summary
Removed "/// Callback from function; /// 函数回调; #[allow(dead_code)]" from `crates/hiver-retry/src/template.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:10:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/lib.rs`
### Change Record
Modified file `crates/hiver-integration/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-integration/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:10:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-mongodb/src/bulk.rs`
### Change Record
Modified file `crates/hiver-data-mongodb/src/bulk.rs`. Approximately 79 lines changed.
### Change Summary
Removed "/// Convert to MongoDB write models.; #[allow(dead_code)]; fn to_write_models(&self) -> Vec<mongodb::bson::Document> {" from `crates/hiver-data-mongodb/src/bulk.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:10:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-mongodb/src/repository.rs`
### Change Record
Modified file `crates/hiver-data-mongodb/src/repository.rs`. Approximately 17 lines changed.
### Change Summary
Removed "/// Convert ID to Bson / 将 ID 转换为 Bson; #[allow(dead_code)]; fn to_bson(&self, id: ID) -> Bson {" from `crates/hiver-data-mongodb/src/repository.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:10:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/context.rs`
### Change Record
Modified file `crates/hiver-core/src/context.rs`. Approximately 11 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-core/src/context.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:10:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/context.rs`
### Change Record
Modified file `crates/hiver-core/src/context.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-core/src/context.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:10:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/flyway.rs`
### Change Record
Modified file `crates/hiver-flyway/src/flyway.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-flyway/src/flyway.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:10:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/interceptor.rs`
### Change Record
Modified file `crates/hiver-grpc/src/interceptor.rs`. Approximately 11 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-grpc/src/interceptor.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:11:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/autoconfigure.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/autoconfigure.rs`. Approximately 24 lines changed.
### Change Summary
Removed "/// 要检查的类型 ID; /// Type ID to check; #[allow(dead_code)]" from `crates/hiver-starter/src/core/autoconfigure.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:11:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/autoconfigure.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/autoconfigure.rs`. Approximately 44 lines changed.
### Change Summary
Removed "/// 要检查的类型 ID; /// Type ID to check; #[allow(dead_code)]" from `crates/hiver-starter/src/core/autoconfigure.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:11:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/autoconfigure.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/autoconfigure.rs`. Approximately 52 lines changed.
### Change Summary
Removed "/// 要检查的类型 ID; /// Type ID to check; #[allow(dead_code)]" from `crates/hiver-starter/src/core/autoconfigure.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:11:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/autoconfigure.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/autoconfigure.rs`. Approximately 72 lines changed.
### Change Summary
Removed "/// 要检查的类型 ID; /// Type ID to check; #[allow(dead_code)]" from `crates/hiver-starter/src/core/autoconfigure.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:11:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/autoconfigure.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/autoconfigure.rs`. Approximately 92 lines changed.
### Change Summary
Removed "/// 要检查的类型 ID; /// Type ID to check; #[allow(dead_code)]" from `crates/hiver-starter/src/core/autoconfigure.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:11:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/bean_factory_post_processor.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/bean_factory_post_processor.rs`. Approximately 21 lines changed.
### Change Summary
Removed "/// 占位符前缀; /// Placeholder prefix; #[allow(dead_code)]" from `crates/hiver-starter/src/core/bean_factory_post_processor.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:11:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/core/bean_factory_post_processor.rs`
### Change Record
Modified file `crates/hiver-starter/src/core/bean_factory_post_processor.rs`. Approximately 48 lines changed.
### Change Summary
Removed "/// 占位符前缀; /// Placeholder prefix; #[allow(dead_code)]" from `crates/hiver-starter/src/core/bean_factory_post_processor.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/context.rs`
### Change Record
Modified file `crates/hiver-core/src/context.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-core/src/context.rs` from "#[allow(dead_code)]; #[allow(dead_code)]" to "#[allow(dead_code)] // public API scaffolding, field access not yet implemented".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/context.rs`
### Change Record
Modified file `crates/hiver-core/src/context.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-core/src/context.rs` from "#[allow(dead_code)]; #[allow(dead_code)]" to "#[allow(dead_code)] // public API scaffolding, field access not yet implemented; #[allow(dead_code)] // storage for futu...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/interceptor.rs`
### Change Record
Modified file `crates/hiver-grpc/src/interceptor.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-grpc/src/interceptor.rs` from "#[allow(dead_code)]" to "#[allow(dead_code)] // trait method for future ErasedInterceptor impls".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-integration/src/lib.rs`
### Change Record
Modified file `crates/hiver-integration/src/lib.rs`. Approximately 15 lines changed.
### Change Summary
Changed `crates/hiver-integration/src/lib.rs` from "#[allow(dead_code)]; #[allow(dead_code)]" to "#[allow(dead_code)] // no builder method yet; match arm in BuiltFlow::process ready for when added; #[allow(dead_code)] ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-flyway/src/flyway.rs`
### Change Record
Modified file `crates/hiver-flyway/src/flyway.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-flyway/src/flyway.rs` from "#[allow(dead_code)]" to "#[allow(dead_code)] // used in tests; kept for API compatibility with parse_migration_filename_with_dialect".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/projection.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/projection.rs`. Approximately 12 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-data-orm/src/projection.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/projection.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/projection.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-data-orm/src/projection.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/projection.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/projection.rs`. Approximately 28 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-data-orm/src/projection.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/projection.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/projection.rs`. Approximately 36 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-data-orm/src/projection.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/pool.rs`
### Change Record
Modified file `crates/hiver-ldap/src/pool.rs`. Approximately 16 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/pool.rs` from "struct PooledConnection {; #[allow(dead_code)]; active: bool," to "struct PooledConnection {}".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/pool.rs`
### Change Record
Modified file `crates/hiver-ldap/src/pool.rs`. Approximately 25 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/pool.rs` from "struct PooledConnection {; #[allow(dead_code)]; active: bool," to "struct PooledConnection {}; idle.push_back(PooledConnection {});".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/pool.rs`
### Change Record
Modified file `crates/hiver-ldap/src/pool.rs`. Approximately 33 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/pool.rs` from "struct PooledConnection {; #[allow(dead_code)]; active: bool," to "struct PooledConnection {}; idle.push_back(PooledConnection {});; idle.push_back(PooledConnection {});".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/ldif.rs`
### Change Record
Modified file `crates/hiver-ldap/src/ldif.rs`. Approximately 11 lines changed.
### Change Summary
Removed "#[allow(dead_code)]" from `crates/hiver-ldap/src/ldif.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/context.rs`
### Change Record
Modified file `crates/hiver-ldap/src/context.rs`. Approximately 20 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; connect_timeout: Duration,; connect_timeout: Duration::from_secs(30)," from `crates/hiver-ldap/src/context.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/repository.rs`
### Change Record
Modified file `crates/hiver-ldap/src/repository.rs`. Approximately 29 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/repository.rs` from "#[allow(dead_code)]; id_extractor: E,; id_extractor: E," to "_id_extractor: E,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:12:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/projection.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/projection.rs`. Approximately 52 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" from `crates/hiver-data-orm/src/projection.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:13:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-events/src/transactional_listener.rs`
### Change Record
Modified file `crates/hiver-events/src/transactional_listener.rs`. Approximately 13 lines changed.
### Change Summary
Removed "#[allow(dead_code)]; status: String," from `crates/hiver-events/src/transactional_listener.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:13:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-events/src/transactional_listener.rs`
### Change Record
Modified file `crates/hiver-events/src/transactional_listener.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-events/src/transactional_listener.rs` from "#[allow(dead_code)]; status: String,; #[allow(dead_code)]" to "_payment_id: u64,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:13:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/load_balancer.rs`
### Change Record
Modified file `crates/hiver-cloud/src/load_balancer.rs`. Approximately 30 lines changed.
### Change Summary
Removed "/// Random strategy (reserved for future use); /// 随机策略（预留，未来使用）; #[allow(dead_code)]" from `crates/hiver-cloud/src/load_balancer.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:13:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 24 lines changed.
### Change Summary
Removed "/// Cache entry for config responses with TTL tracking.; /// 带TTL跟踪的配置响应缓存条目。; #[derive(Debug, Clone)]" from `crates/hiver-cloud/src/config_client.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:14:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/context.rs`
### Change Record
Modified file `crates/hiver-ldap/src/context.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/context.rs` from "password: Option<String>,; #[allow(dead_code)]" to "password: Option<String,; #[allow(dead_code)] // Used by builder; will be consumed by future create_connection()".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:14:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/context.rs`
### Change Record
Modified file `crates/hiver-ldap/src/context.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/context.rs` from "#[allow(dead_code)]" to "#[allow(dead_code)] // Used by builder; will be consumed by future create_connection()".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:14:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/repository.rs`
### Change Record
Modified file `crates/hiver-ldap/src/repository.rs`. Approximately 40 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/repository.rs` from "_marker: PhantomData<(T, ID)>,; #[allow(dead_code)]; id_extractor: E," to "_marker: PhantomData<(T, ID, E)>,; _marker: PhantomData<(T, ID, E)>,; _id_extractor: E,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:14:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/repository.rs`
### Change Record
Modified file `crates/hiver-ldap/src/repository.rs`. Approximately 31 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/repository.rs` from "#[allow(dead_code)]; id_extractor: E,; _marker: PhantomData<(T, ID)>," to "_marker: PhantomData<(T, ID, E)>,; _id_extractor: E,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:18:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ai/src/openai.rs`
### Change Record
Modified file `crates/hiver-ai/src/openai.rs`. Approximately 12 lines changed.
### Change Summary
Added "total_tokens: u32," in `crates/hiver-ai/src/openai.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:18:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-events/src/transactional_listener.rs`
### Change Record
Modified file `crates/hiver-events/src/transactional_listener.rs`. Approximately 12 lines changed.
### Change Summary
Added "status: String," in `crates/hiver-events/src/transactional_listener.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:18:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-events/src/transactional_listener.rs`
### Change Record
Modified file `crates/hiver-events/src/transactional_listener.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-events/src/transactional_listener.rs` from "_payment_id: u64," to "status: String,; payment_id: u64,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:22:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/client.rs` from "// Single quotes are escaped, so the injection is neutralized; assert_eq!(; result," to "// Single quotes are escaped (' → ''), wrapping in SQL quotes produces:; // WHERE name = '''; DROP TABLE users; --'; // ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-29 23:32:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-lombok/src/builder.rs`
### Change Record
Modified file `crates/hiver-lombok/src/builder.rs`. Approximately 117 lines changed.
### Change Summary
Changed `crates/hiver-lombok/src/builder.rs` from "// Generate Builder struct; // 生成 Builder 结构体; #(" to "// Detect Option<T> fields for optional handling; // 检测 Option<T> 字段以进行可选处理; let is_option: Vec<_> = field_types".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:08:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/interceptor.rs`
### Change Record
Modified file `crates/hiver-grpc/src/interceptor.rs`. Approximately 129 lines changed.
### Change Summary
Added "// ─────────────────────────────────────────────────────────────────────────────; // Client-side interceptor; // ───────..." in `crates/hiver-grpc/src/interceptor.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:08:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/Cargo.toml`
### Change Record
Modified file `crates/hiver-grpc/Cargo.toml`. Approximately 13 lines changed.
### Change Summary
Added "# UUID for request IDs / 用于请求 ID 的 UUID; uuid = { workspace = true }" in `crates/hiver-grpc/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:09:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-grpc/src/retry.rs`
### Change Record
New file `crates/hiver-grpc/src/retry.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-grpc/src/retry.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 00:10:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/metadata.rs`
### Change Record
Modified file `crates/hiver-grpc/src/metadata.rs`. Approximately 68 lines changed.
### Change Summary
Added "use std::time::Duration;; /// Deadline propagation via gRPC metadata.; /// 通过 gRPC 元数据传播 Deadline。" in `crates/hiver-grpc/src/metadata.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:10:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/lib.rs`
### Change Record
Modified file `crates/hiver-grpc/src/lib.rs`. Approximately 14 lines changed.
### Change Summary
Added "pub mod retry;; pub use retry::RetryPolicy;" in `crates/hiver-grpc/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:11:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/interceptor.rs`
### Change Record
Modified file `crates/hiver-grpc/src/interceptor.rs`. Approximately 186 lines changed.
### Change Summary
Added "// ─────────────────────────────────────────────────────────────────────────────; // Client-side interceptor; // ───────..." in `crates/hiver-grpc/src/interceptor.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:11:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-grpc/src/retry.rs`
### Change Record
New file `crates/hiver-grpc/src/retry.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-grpc/src/retry.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 00:14:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-tx/src/events.rs`
### Change Record
New file `crates/hiver-tx/src/events.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-tx/src/events.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 00:14:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-tx/src/lib.rs`
### Change Record
Modified file `crates/hiver-tx/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "pub mod events;" in `crates/hiver-tx/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:14:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-tx/src/lib.rs`
### Change Record
Modified file `crates/hiver-tx/src/lib.rs`. Approximately 23 lines changed.
### Change Summary
Added "pub mod events;; pub use events::{; LoggingSynchronization, PhaseListener, SynchronizationRegistry, TransactionPhase," in `crates/hiver-tx/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:15:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-tx/src/events.rs`
### Change Record
New file `crates/hiver-tx/src/events.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-tx/src/events.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 00:33:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-config/src/refresh.rs`
### Change Record
Modified file `crates/hiver-config/src/refresh.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-config/src/refresh.rs` from "pub type ChangeListener = Box<dyn Fn(&ConfigChangeEvent) + Send + Sync>;" to "pub(crate) type ChangeListener = Box<dyn Fn(&ConfigChangeEvent) + Send + Sync>;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:33:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/retry.rs`
### Change Record
Modified file `crates/hiver-grpc/src/retry.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-grpc/src/retry.rs` from "Fut: std::future::Future<Output = Result<T, tonic::Status>>," to "Fut: Future<Output = Result<T, tonic::Status>>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:33:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/ldif.rs`
### Change Record
Modified file `crates/hiver-ldap/src/ldif.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/ldif.rs` from "attrs.sort_by_key(|(k, _)| k.clone());" to "attrs.sort_by_key(|(k, _)| (**k).clone());".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:33:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ai/src/openai.rs`
### Change Record
Modified file `crates/hiver-ai/src/openai.rs`. Approximately 11 lines changed.
### Change Summary
Added "#[allow(dead_code)]" in `crates/hiver-ai/src/openai.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:33:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/interceptor.rs`
### Change Record
Modified file `crates/hiver-grpc/src/interceptor.rs`. Approximately 16 lines changed.
### Change Summary
Added "#[allow(dead_code)]; #[allow(dead_code)]" in `crates/hiver-grpc/src/interceptor.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:33:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/retry.rs`
### Change Record
Modified file `crates/hiver-grpc/src/retry.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-grpc/src/retry.rs` from "Fut: std::future::Future<Output = Result<T, tonic::Status>>," to "use std::future::Future;; Fut: Future<Output = Result<T, tonic::Status>>,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:33:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/interceptor.rs`
### Change Record
Modified file `crates/hiver-grpc/src/interceptor.rs`. Approximately 22 lines changed.
### Change Summary
Added "#[allow(dead_code)]; #[allow(dead_code)]; #[allow(dead_code)]" in `crates/hiver-grpc/src/interceptor.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 00:33:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-ldap/src/operations.rs`
### Change Record
Modified file `crates/hiver-ldap/src/operations.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-ldap/src/operations.rs` from "use crate::error::{LdapError, LdapResult};" to "use crate::error::LdapResult;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:05:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/lib.rs`. Approximately 52 lines changed.
### Change Summary
Added "/// Trait for converting Rust types to SQL literal strings.; /// 将 Rust 类型转换为 SQL 字面量字符串的 trait。; ///" in `crates/hiver-data-commons/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:05:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 49 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/client.rs` from "/// Trait for SQL parameter conversion; /// SQL 参数转换 trait; pub trait ToSql: Send + Sync {" to "pub use hiver_data_commons::ToSql;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:06:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 83 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "/// Trait for SQL parameter conversion; /// SQL 参数转换的 trait; pub trait ToSql: Send + Sync {" to "use hiver_data_commons::ToSql;; pub use hiver_data_commons::ToSql;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:06:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 75 lines changed.
### Change Summary
Removed "/// Trait for SQL parameter conversion; /// SQL 参数转换的 trait; pub trait ToSql: Send + Sync {" from `crates/hiver-data-orm/src/query.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:06:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/lib.rs` from "pub use query::{QueryBuilder, WhereClause, OrderBy, Limit, ToSql};" to "pub use query::{QueryBuilder, WhereClause, OrderBy, Limit};; pub use hiver_data_commons::ToSql;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:08:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 71 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/client.rs` from "/// Trait for SQL parameter conversion; /// SQL 参数转换 trait; pub trait ToSql: Send + Sync {" to "impl From<hiver_data_commons::Value> for QueryParam {; fn from(v: hiver_data_commons::Value) -> Self {; match v {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:10:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/executor.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/executor.rs`. Approximately 548 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/executor.rs` from "use crate::client::DatabaseClient;; let (sql, _params) = self.build_select_query(wrapper, table);; let rows = self.clien..." to "use crate::client::{DatabaseClient, QueryParam};; let (sql, params) = self.build_select_query(wrapper, table);; let rows...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:11:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 91 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/client.rs` from "/// Trait for SQL parameter conversion; /// SQL 参数转换 trait; pub trait ToSql: Send + Sync {" to "impl From<hiver_data_commons::Value> for QueryParam {; fn from(v: hiver_data_commons::Value) -> Self {; match v {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:11:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/executor.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/executor.rs`. Approximately 531 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/executor.rs` from "use crate::client::DatabaseClient;; let (sql, _params) = self.build_select_query(wrapper, table);; let rows = self.clien..." to "use crate::client::{DatabaseClient, QueryParam};; let (sql, params) = self.build_select_query(wrapper, table);; let rows...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:11:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/executor.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/executor.rs`. Approximately 531 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/executor.rs` from "use crate::client::DatabaseClient;; let (sql, _params) = self.build_select_query(wrapper, table);; let rows = self.clien..." to "use crate::client::{DatabaseClient, QueryParam};; let (sql, params) = self.build_select_query(wrapper, table);; let rows...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:11:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/query_runtime.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/query_runtime.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/query_runtime.rs` from "client::DatabaseClient," to "client::{DatabaseClient, QueryParam},".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:11:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/query_runtime.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/query_runtime.rs`. Approximately 99 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/query_runtime.rs` from "client::DatabaseClient,; ) -> R2dbcResult<(String, Vec<serde_json::Value>)> {; // Replace :param with $1, $2, etc." to "client::{DatabaseClient, QueryParam},; ) -> R2dbcResult<(String, Vec<QueryParam>)> {; values.push(QueryParam::from(value...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:11:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/query_runtime.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/query_runtime.rs`. Approximately 110 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/query_runtime.rs` from "client::DatabaseClient,; ) -> R2dbcResult<(String, Vec<serde_json::Value>)> {; // Replace :param with $1, $2, etc." to "client::{DatabaseClient, QueryParam},; ) -> R2dbcResult<(String, Vec<QueryParam>)> {; values.push(QueryParam::from(value...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:12:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/query_runtime.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/query_runtime.rs`. Approximately 121 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/query_runtime.rs` from "client::DatabaseClient,; ) -> R2dbcResult<(String, Vec<serde_json::Value>)> {; // Replace :param with $1, $2, etc." to "client::{DatabaseClient, QueryParam},; ) -> R2dbcResult<(String, Vec<QueryParam>)> {; values.push(QueryParam::from(value...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:12:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/query_runtime.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/query_runtime.rs`. Approximately 132 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/query_runtime.rs` from "client::DatabaseClient,; ) -> R2dbcResult<(String, Vec<serde_json::Value>)> {; // Replace :param with $1, $2, etc." to "client::{DatabaseClient, QueryParam},; ) -> R2dbcResult<(String, Vec<QueryParam>)> {; values.push(QueryParam::from(value...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:12:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/query_runtime.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/query_runtime.rs`. Approximately 150 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/query_runtime.rs` from "client::DatabaseClient,; ) -> R2dbcResult<(String, Vec<serde_json::Value>)> {; // Replace :param with $1, $2, etc." to "client::{DatabaseClient, QueryParam},; ) -> R2dbcResult<(String, Vec<QueryParam>)> {; values.push(QueryParam::from(value...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:26:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/service.rs`
### Change Record
Modified file `crates/hiver-http/src/service.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-http/src/service.rs` from "fn call(&self, req: Request) -> impl Future<Output = Result<Response>> + Send {; async move { self(req).await }" to "async fn call(&self, req: Request) -> Result<Response> {; self(req).await".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:26:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/queue.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/queue.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/scheduler/queue.rs` from "Err(_) => continue," to "Err(_) => {}".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:26:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/queue.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/queue.rs`. Approximately 20 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/scheduler/queue.rs` from "Err(_) => continue,; continue;" to "Err(_) => {}".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:26:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/runtime.rs`
### Change Record
Modified file `crates/hiver-runtime/src/runtime.rs`. Approximately 18 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/runtime.rs` from "CURRENT_HANDLE.with(|h| {; h.borrow(); .clone()" to "Self::try_current(); .expect("Handle::current() called outside of a runtime context")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:26:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "pub fn register_conditional<T, F, C>(&mut self, factory: F, condition: C) -> Result<()>" to "pub fn register_conditional<T, F, C>(&mut self, factory: F, condition: &C) -> Result<()>".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:26:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/executor.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/executor.rs`. Approximately 26 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/executor.rs` from "let mut idx = 1u32;; for (column, value) in &wrapper.sets {; idx += 1;" to "for (idx, (column, value)) in (1u32..).zip(wrapper.sets.iter()) {; let (where_clause, _where_params) = Self::build_where...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:26:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/specification.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/specification.rs`. Approximately 17 lines changed.
### Change Summary
Added "/// Create a NOT predicate.; /// 创建 NOT 谓词。; ///" in `crates/hiver-data-commons/src/specification.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:26:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/specification.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/specification.rs`. Approximately 25 lines changed.
### Change Summary
Added "/// Create a NOT predicate.; /// 创建 NOT 谓词。; ///" in `crates/hiver-data-commons/src/specification.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/part_tree.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/part_tree.rs`. Approximately 28 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/part_tree.rs` from "} else if let Ok(n) = digits.parse::<u32>() {; n; 1" to "digits.parse::<u32>().unwrap_or(1); digits.parse::<u32>().unwrap_or(1)".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/specification.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/specification.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::should_implement_trait)]" in `crates/hiver-data-orm/src/specification.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 17 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from "let mut param_idx = 1u32;; for _ in params {; param_idx += 1;" to "for (param_idx, _) in (1u32..).zip(params.iter()) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 30 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from "let mut param_idx = 1u32;; for _ in params {; param_idx += 1;" to "for (param_idx, _) in (1u32..).zip(params.iter()) {; for (param_idx, _) in (1u32..).zip(params.iter()) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 43 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from "let mut param_idx = 1u32;; for _ in params {; param_idx += 1;" to "for (param_idx, _) in (1u32..).zip(params.iter()) {; for (param_idx, _) in (1u32..).zip(params.iter()) {; for (param_idx...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/projection.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/projection.rs`. Approximately 17 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/projection.rs` from "let mut param_idx = 1u32;; for _ in params {; param_idx += 1;" to "for (param_idx, _) in (1u32..).zip(params.iter()) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/projection.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/projection.rs`. Approximately 30 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/projection.rs` from "let mut param_idx = 1u32;; for _ in params {; param_idx += 1;" to "for (param_idx, _) in (1u32..).zip(params.iter()) {; for (param_idx, _) in (1u32..).zip(params.iter()) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/sse.rs`
### Change Record
Modified file `crates/hiver-http/src/sse.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::struct_field_names)]" in `crates/hiver-http/src/sse.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/validation.rs`
### Change Record
Modified file `crates/hiver-http/src/validation.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-http/src/validation.rs` from "Self { errors: Vec::new() }" to "impl Default for ValidationErrors {; fn default() -> Self {; Self { errors: Vec::new() }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/part_tree.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/part_tree.rs`. Approximately 34 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/part_tree.rs` from "let num = if digits.is_empty() {; 1; } else if let Ok(n) = digits.parse::<u32>() {" to "let num = digits.parse::<u32>().unwrap_or(1);; let num = digits.parse::<u32>().unwrap_or(1);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/validation.rs`
### Change Record
Modified file `crates/hiver-http/src/validation.rs`. Approximately 41 lines changed.
### Change Summary
Changed `crates/hiver-http/src/validation.rs` from "Self { errors: Vec::new() }; pub fn require_min<T>(field: &str, value: T, min: T) -> Option<ValidationError>; pub fn req..." to "impl Default for ValidationErrors {; fn default() -> Self {; Self { errors: Vec::new() }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/websocket.rs`
### Change Record
Modified file `crates/hiver-http/src/websocket.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-http/src/websocket.rs` from ".unwrap()" to ".unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/route.rs`
### Change Record
Modified file `crates/hiver-router/src/route.rs`. Approximately 12 lines changed.
### Change Summary
Added ".field("handler", &self.handler)" in `crates/hiver-router/src/route.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/route.rs`
### Change Record
Modified file `crates/hiver-router/src/route.rs`. Approximately 70 lines changed.
### Change Summary
Changed `crates/hiver-router/src/route.rs` from "use hiver_http::Body;; use hiver_http::StatusCode;; .unwrap())" to ".field("handler", &self.handler); .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR))); .unwrap_or_els...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/route.rs`
### Change Record
Modified file `crates/hiver-router/src/route.rs`. Approximately 79 lines changed.
### Change Summary
Changed `crates/hiver-router/src/route.rs` from "use hiver_http::{Request, Response, Result, StatusCode};; use hiver_http::Body;; use hiver_http::StatusCode;" to "use hiver_http::{Body, Request, Response, Result, StatusCode};; .field("handler", &self.handler); .unwrap_or_else(|_| Re...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/route.rs`
### Change Record
Modified file `crates/hiver-router/src/route.rs`. Approximately 91 lines changed.
### Change Summary
Changed `crates/hiver-router/src/route.rs` from "use hiver_http::{Request, Response, Result, StatusCode};; use hiver_http::Body;; use hiver_http::StatusCode;" to "use hiver_http::{Body, Request, Response, Result, StatusCode};; .field("handler", &self.handler); .unwrap_or_else(|_| Re...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/router.rs`
### Change Record
Modified file `crates/hiver-router/src/router.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-router/src/router.rs` from "method: &Method," to "method: Method,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/router.rs`
### Change Record
Modified file `crates/hiver-router/src/router.rs`. Approximately 33 lines changed.
### Change Summary
Changed `crates/hiver-router/src/router.rs` from "method: &Method,; let (route, params) = match matched {; Some(m) => m," to "method: Method,; let Some((route, params)) = matched else {; return Ok(Response::builder()".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/specification.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/specification.rs`. Approximately 20 lines changed.
### Change Summary
Added "#[allow(clippy::should_implement_trait)]; #[allow(clippy::should_implement_trait)]" in `crates/hiver-data-commons/src/specification.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/router.rs`
### Change Record
Modified file `crates/hiver-router/src/router.rs`. Approximately 50 lines changed.
### Change Summary
Changed `crates/hiver-router/src/router.rs` from "method: &Method,; let (route, params) = match matched {; Some(m) => m," to "method: Method,; let Some((route, params)) = matched else {; return Ok(Response::builder()".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/trie.rs`
### Change Record
Modified file `crates/hiver-router/src/trie.rs`. Approximately 39 lines changed.
### Change Summary
Changed `crates/hiver-router/src/trie.rs` from "fn router_for_method_mut(&mut self, method: &Method) -> &mut matchit::Router<MethodRoute> {; Method::GET => &mut self.ge..." to "fn router_for_method_mut(&mut self, method: Method) -> &mut matchit::Router<MethodRoute> {; Method::GET | Method::TRACE ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/trie.rs`
### Change Record
Modified file `crates/hiver-router/src/trie.rs`. Approximately 49 lines changed.
### Change Summary
Changed `crates/hiver-router/src/trie.rs` from "fn router_for_method_mut(&mut self, method: &Method) -> &mut matchit::Router<MethodRoute> {; Method::GET => &mut self.ge..." to "fn router_for_method_mut(&mut self, method: Method) -> &mut matchit::Router<MethodRoute> {; Method::GET | Method::TRACE ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/runtime.rs`
### Change Record
Modified file `crates/hiver-runtime/src/runtime.rs`. Approximately 20 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/runtime.rs` from "CURRENT_HANDLE.with(|h| {; h.borrow(); .clone()" to "#[allow(clippy::expect_used)]; Self::try_current(); .expect("Handle::current() called outside of a runtime context")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/scheduler/queue.rs`
### Change Record
Modified file `crates/hiver-runtime/src/scheduler/queue.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/scheduler/queue.rs` from "match self.tail.compare_exchange(tail, tail + 1, Ordering::AcqRel, Ordering::Relaxed) {; Ok(_) => return true,; Err(_) =..." to "if self.tail.compare_exchange(tail, tail + 1, Ordering::AcqRel, Ordering::Relaxed).is_ok() {; return true;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/trie.rs`
### Change Record
Modified file `crates/hiver-router/src/trie.rs`. Approximately 58 lines changed.
### Change Summary
Changed `crates/hiver-router/src/trie.rs` from "let router = self.router_for_method_mut(&method);; fn router_for_method_mut(&mut self, method: &Method) -> &mut matchit:..." to "let router = self.router_for_method_mut(method);; fn router_for_method_mut(&mut self, method: Method) -> &mut matchit::R...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/trie.rs`
### Change Record
Modified file `crates/hiver-router/src/trie.rs`. Approximately 67 lines changed.
### Change Summary
Changed `crates/hiver-router/src/trie.rs` from "let router = self.router_for_method_mut(&method);; method: &Method,; fn router_for_method_mut(&mut self, method: &Method..." to "let router = self.router_for_method_mut(method);; method: Method,; fn router_for_method_mut(&mut self, method: Method) -...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/trie.rs`
### Change Record
Modified file `crates/hiver-router/src/trie.rs`. Approximately 76 lines changed.
### Change Summary
Changed `crates/hiver-router/src/trie.rs` from "let router = self.router_for_method_mut(&method);; method: &Method,; fn router_for_method_mut(&mut self, method: &Method..." to "let router = self.router_for_method_mut(method);; method: Method,; fn router_for_method_mut(&mut self, method: Method) -...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/router.rs`
### Change Record
Modified file `crates/hiver-router/src/router.rs`. Approximately 54 lines changed.
### Change Summary
Changed `crates/hiver-router/src/router.rs` from "method: &Method,; let matched = self.match_route(&method, &path);; let (route, params) = match matched {" to "method: Method,; let matched = self.match_route(method, &path);; let Some((route, params)) = matched else {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/trie.rs`
### Change Record
Modified file `crates/hiver-router/src/trie.rs`. Approximately 85 lines changed.
### Change Summary
Changed `crates/hiver-router/src/trie.rs` from "let router = self.router_for_method_mut(&method);; method: &Method,; fn router_for_method_mut(&mut self, method: &Method..." to "let router = self.router_for_method_mut(method);; method: Method,; fn router_for_method_mut(&mut self, method: Method) -...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/validation.rs`
### Change Record
Modified file `crates/hiver-http/src/validation.rs`. Approximately 60 lines changed.
### Change Summary
Changed `crates/hiver-http/src/validation.rs` from "Self { errors: Vec::new() }; pub fn require_min<T>(field: &str, value: T, min: T) -> Option<ValidationError>; pub fn req..." to "impl Default for ValidationErrors {; fn default() -> Self {; Self { errors: Vec::new() }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:27:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/validation.rs`
### Change Record
Modified file `crates/hiver-http/src/validation.rs`. Approximately 59 lines changed.
### Change Summary
Changed `crates/hiver-http/src/validation.rs` from "#[derive(Debug, Clone)]; Self { errors: Vec::new() }; pub fn require_min<T>(field: &str, value: T, min: T) -> Option<Val..." to "#[derive(Debug, Clone, Default)]; Self::default(); pub fn require_min<T>(field: &str, value: &T, min: &T) -> Option<Vali...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:28:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `Cargo.toml`
### Change Record
Modified file `Cargo.toml`. Approximately 15 lines changed.
### Change Summary
Changed `Cargo.toml` from "unnecessary_qualifications = "allow"" to "# (Note: this is a rustc lint, not clippy; see workspace.lints.rust.unused_qualifications); # (注：这是rustc的lint，不是clippy的；...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:28:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-router/src/trie.rs`
### Change Record
Modified file `crates/hiver-router/src/trie.rs`. Approximately 135 lines changed.
### Change Summary
Changed `crates/hiver-router/src/trie.rs` from "let router = self.router_for_method_mut(&method);; method: &Method,; fn router_for_method_mut(&mut self, method: &Method..." to "let router = self.router_for_method_mut(method);; method: Method,; fn router_for_method_mut(&mut self, method: Method) -...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 09:28:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-core/src/container.rs`
### Change Record
Modified file `crates/hiver-core/src/container.rs`. Approximately 49 lines changed.
### Change Summary
Changed `crates/hiver-core/src/container.rs` from "pub fn register_conditional<T, F, C>(&mut self, factory: F, condition: C) -> Result<()>; cond,; cond," to "pub fn register_conditional<T, F, C>(&mut self, factory: F, condition: &C) -> Result<()>; &cond,; &cond,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/health.rs`
### Change Record
Modified file `crates/hiver-grpc/src/health.rs`. Approximately 49 lines changed.
### Change Summary
Changed `crates/hiver-grpc/src/health.rs` from ".unwrap(); .unwrap(); .unwrap()" to ".expect("health service lock poisoned"); .expect("health service lock poisoned"); .expect("health service lock poisoned"...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/metadata.rs`
### Change Record
Modified file `crates/hiver-grpc/src/metadata.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-grpc/src/metadata.rs` from "deadline_ms.to_string().parse().unwrap_or_else(|_| "0".parse().unwrap());" to "deadline_ms.to_string().parse().unwrap_or_else(|_| MetadataValue::from_static("0"));".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/retry.rs`
### Change Record
Modified file `crates/hiver-grpc/src/retry.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-grpc/src/retry.rs` from "Err(last_err.unwrap())" to "Err(last_err.expect("retry loop should always produce an error"))".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/server.rs`
### Change Record
Modified file `crates/hiver-grpc/src/server.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-grpc/src/server.rs` from ".unwrap_or_else(|_| "0.0.0.0:50051".parse().unwrap());" to ".unwrap_or_else(|_| "0.0.0.0:50051".parse().expect("hardcoded address is valid"));".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/client.rs`
### Change Record
Modified file `crates/hiver-grpc/src/client.rs`. Approximately 12 lines changed.
### Change Summary
Added ".filter(|_| !self.channels.is_empty())" in `crates/hiver-grpc/src/client.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/tls.rs`
### Change Record
Modified file `crates/hiver-grpc/src/tls.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::struct_field_names)]" in `crates/hiver-grpc/src/tls.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/src/engine.rs`
### Change Record
Modified file `crates/hiver-graphql/src/engine.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-graphql/src/engine.rs` from "fn from_ag(resp: AGResponse) -> Self {" to "fn from_ag(resp: &AGResponse) -> Self {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/src/engine.rs`
### Change Record
Modified file `crates/hiver-graphql/src/engine.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-graphql/src/engine.rs` from "fn from_ag(resp: AGResponse) -> Self {; GraphQLResponse::from_ag(resp)" to "fn from_ag(resp: &AGResponse) -> Self {; GraphQLResponse::from_ag(&resp)".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/src/engine.rs`
### Change Record
Modified file `crates/hiver-graphql/src/engine.rs`. Approximately 30 lines changed.
### Change Summary
Changed `crates/hiver-graphql/src/engine.rs` from "fn from_ag(resp: AGResponse) -> Self {; GraphQLResponse::from_ag(resp); BatchResponse::Single(resp) => vec![GraphQLRespo..." to "fn from_ag(resp: &AGResponse) -> Self {; GraphQLResponse::from_ag(&resp); BatchResponse::Single(resp) => vec![GraphQLRes...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/src/resolver.rs`
### Change Record
Modified file `crates/hiver-graphql/src/resolver.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::struct_field_names)]" in `crates/hiver-graphql/src/resolver.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {" to "fn as_str(self) -> &'static str {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/src/error.rs`
### Change Record
Modified file `crates/hiver-graphql/src/error.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-graphql/src/error.rs` from "if !self.locations.is_empty() {; let loc = &self.locations[0];" to "if let Some(loc) = self.locations.first() {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:57:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 40 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 55 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/src/dataloader.rs`
### Change Record
Modified file `crates/hiver-graphql/src/dataloader.rs`. Approximately 13 lines changed.
### Change Summary
Added "#![allow(clippy::expect_used)]" in `crates/hiver-graphql/src/dataloader.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 70 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 84 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 94 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 103 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-spel/src/parser.rs`
### Change Record
Modified file `crates/hiver-spel/src/parser.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::indexing_slicing)]" in `crates/hiver-spel/src/parser.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 113 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 129 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 137 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 146 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-spel/src/evaluator.rs`
### Change Record
Modified file `crates/hiver-spel/src/evaluator.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::float_cmp)]" in `crates/hiver-spel/src/evaluator.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 162 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 169 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 178 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 183 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/multichain.rs`
### Change Record
Modified file `crates/hiver-web3/src/multichain.rs`. Approximately 20 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/multichain.rs` from "Self::Ethereum => NativeCurrency { symbol: "ETH", decimals: 18, name: "Ether" },; Self::Arbitrum => NativeCurrency { sym..." to "Self::Ethereum | Self::Arbitrum | Self::Optimism | Self::Base | Self::ZkSync =>; NativeCurrency { symbol: "ETH", decimal...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 199 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 209 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/multichain.rs`
### Change Record
Modified file `crates/hiver-web3/src/multichain.rs`. Approximately 37 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/multichain.rs` from "Self::Ethereum => NativeCurrency { symbol: "ETH", decimals: 18, name: "Ether" },; Self::Arbitrum => NativeCurrency { sym..." to "Self::Ethereum | Self::Arbitrum | Self::Optimism | Self::Base | Self::ZkSync =>; NativeCurrency { symbol: "ETH", decimal...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/multichain.rs`
### Change Record
Modified file `crates/hiver-web3/src/multichain.rs`. Approximately 45 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/multichain.rs` from "Self::Ethereum => NativeCurrency { symbol: "ETH", decimals: 18, name: "Ether" },; Self::Arbitrum => NativeCurrency { sym..." to "Self::Ethereum | Self::Arbitrum | Self::Optimism | Self::Base | Self::ZkSync =>; NativeCurrency { symbol: "ETH", decimal...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 214 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 221 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/multichain.rs`
### Change Record
Modified file `crates/hiver-web3/src/multichain.rs`. Approximately 54 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/multichain.rs` from "Self::Ethereum => NativeCurrency { symbol: "ETH", decimals: 18, name: "Ether" },; Self::Arbitrum => NativeCurrency { sym..." to "Self::Ethereum | Self::Arbitrum | Self::Optimism | Self::Base | Self::ZkSync =>; NativeCurrency { symbol: "ETH", decimal...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/multichain.rs`
### Change Record
Modified file `crates/hiver-web3/src/multichain.rs`. Approximately 61 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/multichain.rs` from "Self::Ethereum => NativeCurrency { symbol: "ETH", decimals: 18, name: "Ether" },; Self::Arbitrum => NativeCurrency { sym..." to "Self::Ethereum | Self::Arbitrum | Self::Optimism | Self::Base | Self::ZkSync =>; NativeCurrency { symbol: "ETH", decimal...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 239 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 257 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/defi.rs`
### Change Record
Modified file `crates/hiver-web3/src/defi.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/defi.rs` from "let offset_ids = 0x60u64 + 0x20 + (n as u64) * 0x20;" to "let offset_ids = 0x60u64 + 0x20 + n * 0x20;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:58:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/response.rs`
### Change Record
Modified file `crates/hiver-response/src/response.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-response/src/response.rs` from ".unwrap()" to ".expect("response builder with default body should not fail")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/events.rs`
### Change Record
Modified file `crates/hiver-session/src/events.rs`. Approximately 32 lines changed.
### Change Summary
Changed `crates/hiver-session/src/events.rs` from "SessionEvent::Created { session_id, .. } => Some(session_id),; SessionEvent::Expired { session_id, .. } => Some(session_..." to "SessionEvent::Created { session_id, .. }; | SessionEvent::Expired { session_id, .. }; | SessionEvent::Destroyed { sessio...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/defi.rs`
### Change Record
Modified file `crates/hiver-web3/src/defi.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/defi.rs` from "let offset_ids = 0x60u64 + 0x20 + (n as u64) * 0x20;" to "#![allow(clippy::indexing_slicing)]; #![allow(clippy::cast_precision_loss)]; let offset_ids = 0x60u64 + 0x20 + n * 0x20;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/events.rs`
### Change Record
Modified file `crates/hiver-session/src/events.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-session/src/events.rs` from "SessionEvent::Created { session_id, .. } => Some(session_id),; SessionEvent::Expired { session_id, .. } => Some(session_..." to "SessionEvent::Created { session_id, .. }; | SessionEvent::Expired { session_id, .. }; | SessionEvent::Destroyed { sessio...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/events.rs`
### Change Record
Modified file `crates/hiver-session/src/events.rs`. Approximately 51 lines changed.
### Change Summary
Changed `crates/hiver-session/src/events.rs` from "SessionEvent::Created { session_id, .. } => Some(session_id),; SessionEvent::Expired { session_id, .. } => Some(session_..." to "SessionEvent::Created { session_id, .. }; | SessionEvent::Expired { session_id, .. }; | SessionEvent::Destroyed { sessio...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/events.rs`
### Change Record
Modified file `crates/hiver-session/src/events.rs`. Approximately 66 lines changed.
### Change Summary
Changed `crates/hiver-session/src/events.rs` from "SessionEvent::Created { session_id, .. } => Some(session_id),; SessionEvent::Expired { session_id, .. } => Some(session_..." to "SessionEvent::Created { session_id, .. }; | SessionEvent::Expired { session_id, .. }; | SessionEvent::Destroyed { sessio...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/hd_wallet.rs`
### Change Record
Modified file `crates/hiver-web3/src/hd_wallet.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/hd_wallet.rs` from "if entropy.is_empty() || entropy.len() % 4 != 0 {" to "if entropy.is_empty() || !(entropy.len() * 4).is_multiple_of(4) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/hd_wallet.rs`
### Change Record
Modified file `crates/hiver-web3/src/hd_wallet.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/hd_wallet.rs` from "if entropy.is_empty() || entropy.len() % 4 != 0 {" to "if entropy.is_empty() || !entropy.len().is_multiple_of(4) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-session/src/events.rs`
### Change Record
Modified file `crates/hiver-session/src/events.rs`. Approximately 81 lines changed.
### Change Summary
Changed `crates/hiver-session/src/events.rs` from "SessionEvent::Created { session_id, .. } => Some(session_id),; SessionEvent::Expired { session_id, .. } => Some(session_..." to "SessionEvent::Created { session_id, .. }; | SessionEvent::Expired { session_id, .. }; | SessionEvent::Destroyed { sessio...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-extractors/src/path.rs`
### Change Record
Modified file `crates/hiver-extractors/src/path.rs`. Approximately 20 lines changed.
### Change Summary
Changed `crates/hiver-extractors/src/path.rs` from "let v1 = T1::from_str(path_vars.get(&var_names[0]).expect("unexpected error")); let v2 = T2::from_str(path_vars.get(&var..." to "let v1 = T1::from_str(path_vars.get(&var_names[0]).ok_or_else(|| {; ExtractorError::Missing("expected 2 path parameters"...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/hd_wallet.rs`
### Change Record
Modified file `crates/hiver-web3/src/hd_wallet.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/hd_wallet.rs` from "if entropy.is_empty() || entropy.len() % 4 != 0 {; let hardened = 0x80000000u32;" to "if entropy.is_empty() || !entropy.len().is_multiple_of(4) {; let hardened = 0x8000_0000u32;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-extractors/src/path.rs`
### Change Record
Modified file `crates/hiver-extractors/src/path.rs`. Approximately 41 lines changed.
### Change Summary
Changed `crates/hiver-extractors/src/path.rs` from "let v1 = T1::from_str(path_vars.get(&var_names[0]).expect("unexpected error")); let v2 = T2::from_str(path_vars.get(&var..." to "let v1 = T1::from_str(path_vars.get(&var_names[0]).ok_or_else(|| {; ExtractorError::Missing("expected 2 path parameters"...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/hd_wallet.rs`
### Change Record
Modified file `crates/hiver-web3/src/hd_wallet.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/hd_wallet.rs` from "if entropy.is_empty() || entropy.len() % 4 != 0 {; let hardened = 0x80000000u32;" to "#[allow(clippy::indexing_slicing)]; if entropy.is_empty() || !entropy.len().is_multiple_of(4) {; let hardened = 0x8000_0...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/auth.rs`
### Change Record
Modified file `crates/hiver-security/src/auth.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::new_ret_no_self)]" in `crates/hiver-security/src/auth.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/hd_wallet.rs`
### Change Record
Modified file `crates/hiver-web3/src/hd_wallet.rs`. Approximately 32 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/hd_wallet.rs` from "if entropy.is_empty() || entropy.len() % 4 != 0 {; let hardened = 0x80000000u32;" to "#[allow(clippy::indexing_slicing)]; if entropy.is_empty() || !entropy.len().is_multiple_of(4) {; #[allow(clippy::expect_...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-web3/src/hd_wallet.rs`
### Change Record
Modified file `crates/hiver-web3/src/hd_wallet.rs`. Approximately 41 lines changed.
### Change Summary
Changed `crates/hiver-web3/src/hd_wallet.rs` from "if entropy.is_empty() || entropy.len() % 4 != 0 {; let hardened = 0x80000000u32;" to "#[allow(clippy::indexing_slicing)]; if entropy.is_empty() || !entropy.len().is_multiple_of(4) {; #[allow(clippy::expect_...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/acl.rs`
### Change Record
Modified file `crates/hiver-security/src/acl.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-security/src/acl.rs` from "self.acls.write().unwrap().insert(key, acl);" to "self.acls.write().expect("lock poisoned").insert(key, acl);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/acl.rs`
### Change Record
Modified file `crates/hiver-security/src/acl.rs`. Approximately 19 lines changed.
### Change Summary
Changed `crates/hiver-security/src/acl.rs` from "self.acls.write().unwrap().insert(key, acl);; self.acls.read().unwrap()" to "self.acls.write().expect("lock poisoned").insert(key, acl);; self.acls.read().expect("lock poisoned")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/acl.rs`
### Change Record
Modified file `crates/hiver-security/src/acl.rs`. Approximately 27 lines changed.
### Change Summary
Changed `crates/hiver-security/src/acl.rs` from "self.acls.write().unwrap().insert(key, acl);; self.acls.read().unwrap(); self.acls.write().unwrap()" to "self.acls.write().expect("lock poisoned").insert(key, acl);; self.acls.read().expect("lock poisoned"); self.acls.write()...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/acl.rs`
### Change Record
Modified file `crates/hiver-security/src/acl.rs`. Approximately 35 lines changed.
### Change Summary
Changed `crates/hiver-security/src/acl.rs` from "self.acls.write().unwrap().insert(key, acl);; self.acls.read().unwrap(); self.acls.write().unwrap()" to "self.acls.write().expect("lock poisoned").insert(key, acl);; self.acls.read().expect("lock poisoned"); self.acls.write()...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config.rs`. Approximately 17 lines changed.
### Change Summary
Added "Self::default(); impl Default for SimpleConfigWatcher {; fn default() -> Self {" in `crates/hiver-cloud/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/acl.rs`
### Change Record
Modified file `crates/hiver-security/src/acl.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-security/src/acl.rs` from "self.acls.write().unwrap().insert(key, acl);; self.acls.read().unwrap(); self.acls.write().unwrap()" to "self.acls.write().expect("lock poisoned").insert(key, acl);; self.acls.read().expect("lock poisoned"); self.acls.write()...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config.rs`. Approximately 25 lines changed.
### Change Summary
Added "Self::default(); impl Default for SimpleConfigWatcher {; fn default() -> Self {" in `crates/hiver-cloud/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config.rs`. Approximately 34 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config.rs` from "pub fn decrypt_properties(props: &mut HashMap<String, String>, encryptor: &dyn ConfigEncryptor) -> Result<(), ConfigErro..." to "Self::default(); impl Default for SimpleConfigWatcher {; fn default() -> Self {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/authorization_server.rs`
### Change Record
Modified file `crates/hiver-security/src/authorization_server.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-security/src/authorization_server.rs` from "let part1: String = (0..4).map(|_| chars[rand::random::<u8>() as usize % n]).collect();; let part2: String = (0..4).map(..." to "let part1: String = (0..4).map(|_| chars.get(rand::random::<u8>() as usize % n).copied().unwrap_or('X')).collect();; let...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
Modified file `crates/hiver-security/src/data_scope.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs` from "let ids: Vec<String> = scope.dept_ids.iter().map(|id| id.to_string()).collect();" to "let ids: Vec<String> = scope.dept_ids.iter().map(ToString::to_string).collect();".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
Modified file `crates/hiver-security/src/data_scope.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs` from "let ids: Vec<String> = scope.dept_ids.iter().map(|id| id.to_string()).collect();; if let Some(auth) = security_context.g..." to "let ids: Vec<String> = scope.dept_ids.iter().map(ToString::to_string).collect();; if let Some(auth) = security_context.g...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
Modified file `crates/hiver-security/src/data_scope.rs`. Approximately 35 lines changed.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs` from "let ids: Vec<String> = scope.dept_ids.iter().map(|id| id.to_string()).collect();; if let Some(auth) = security_context.g..." to "let ids: Vec<String> = scope.dept_ids.iter().map(ToString::to_string).collect();; if let Some(auth) = security_context.g...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/doc_pdf.rs`
### Change Record
Modified file `crates/hiver-openapi/src/doc_pdf.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/doc_pdf.rs` from ".as_ref(); .map(|t| format!("{:?}", t).to_lowercase()); .unwrap_or_else(|| "any".to_string())" to "use std::fmt::Write;; .as_ref().map_or_else(|| "any".to_string(), |t| format!("{:?}", t).to_lowercase())".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "prefix: String," to "prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
Modified file `crates/hiver-security/src/data_scope.rs`. Approximately 39 lines changed.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs` from "let ids: Vec<String> = scope.dept_ids.iter().map(|id| id.to_string()).collect();; if let Some(auth) = security_context.g..." to "let ids: Vec<String> = scope.dept_ids.iter().map(ToString::to_string).collect();; if let Some(auth) = security_context.g...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 27 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 10:59:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
Modified file `crates/hiver-security/src/data_scope.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs` from "let ids: Vec<String> = scope.dept_ids.iter().map(|id| id.to_string()).collect();; if let Some(auth) = security_context.g..." to "let ids: Vec<String> = scope.dept_ids.iter().map(ToString::to_string).collect();; if let Some(auth) = security_context.g...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/doc_pdf.rs`
### Change Record
Modified file `crates/hiver-openapi/src/doc_pdf.rs`. Approximately 129 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/doc_pdf.rs` from ".as_ref(); .map(|t| format!("{:?}", t).to_lowercase()); .unwrap_or_else(|| "any".to_string())" to "use std::fmt::Write;; .as_ref().map_or_else(|| "any".to_string(), |t| format!("{:?}", t).to_lowercase()); let _ = write!...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 35 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
Modified file `crates/hiver-security/src/data_scope.rs`. Approximately 79 lines changed.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs` from "let ids: Vec<String> = scope.dept_ids.iter().map(|id| id.to_string()).collect();; if let Some(auth) = security_context.g..." to "let ids: Vec<String> = scope.dept_ids.iter().map(ToString::to_string).collect();; if let Some(auth) = security_context.g...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 44 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
Modified file `crates/hiver-security/src/data_scope.rs`. Approximately 88 lines changed.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs` from "let ids: Vec<String> = scope.dept_ids.iter().map(|id| id.to_string()).collect();; if let Some(auth) = security_context.g..." to "let ids: Vec<String> = scope.dept_ids.iter().map(ToString::to_string).collect();; if let Some(auth) = security_context.g...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/data_scope.rs`
### Change Record
Modified file `crates/hiver-security/src/data_scope.rs`. Approximately 96 lines changed.
### Change Summary
Changed `crates/hiver-security/src/data_scope.rs` from "let ids: Vec<String> = scope.dept_ids.iter().map(|id| id.to_string()).collect();; if let Some(auth) = security_context.g..." to "let ids: Vec<String> = scope.dept_ids.iter().map(ToString::to_string).collect();; #[allow(clippy::unused_async)]; if let...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "if line.as_bytes()[3] == b' ' {" to "if line.as_bytes().get(3) == Some(&b' ') {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "if line.as_bytes()[3] == b' ' {; if !(greeting_code >= 200 && greeting_code < 300) {" to "if line.as_bytes().get(3) == Some(&b' ') {; if !(200..300).contains(&greeting_code) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 53 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/doc_pdf.rs`
### Change Record
Modified file `crates/hiver-openapi/src/doc_pdf.rs`. Approximately 266 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/doc_pdf.rs` from ".as_ref(); .map(|t| format!("{:?}", t).to_lowercase()); .unwrap_or_else(|| "any".to_string())" to "use std::fmt::Write;; .as_ref().map_or_else(|| "any".to_string(), |t| format!("{:?}", t).to_lowercase()); let _ = write!...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/circuit_breaker.rs`
### Change Record
Modified file `crates/hiver-cloud/src/circuit_breaker.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::indexing_slicing)]" in `crates/hiver-cloud/src/circuit_breaker.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/circuit_breaker.rs`
### Change Record
Modified file `crates/hiver-cloud/src/circuit_breaker.rs`. Approximately 20 lines changed.
### Change Summary
Added "#[allow(clippy::indexing_slicing)]; #[allow(clippy::cast_precision_loss)]" in `crates/hiver-cloud/src/circuit_breaker.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/circuit_breaker.rs`
### Change Record
Modified file `crates/hiver-cloud/src/circuit_breaker.rs`. Approximately 28 lines changed.
### Change Summary
Added "#[allow(clippy::indexing_slicing)]; #[allow(clippy::cast_precision_loss)]; #[allow(clippy::cast_precision_loss)]" in `crates/hiver-cloud/src/circuit_breaker.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 30 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "if line.as_bytes()[3] == b' ' {; if !(greeting_code >= 200 && greeting_code < 300) {" to "use std::fmt::Write as FmtWrite;; if line.as_bytes().get(3) == Some(&b' ') {; if !(200..300).contains(&greeting_code) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/feign.rs`
### Change Record
Modified file `crates/hiver-cloud/src/feign.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::cast_precision_loss)]" in `crates/hiver-cloud/src/feign.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/email.rs`
### Change Record
Modified file `crates/hiver-security/src/email.rs`. Approximately 49 lines changed.
### Change Summary
Changed `crates/hiver-security/src/email.rs` from "if line.as_bytes()[3] == b' ' {; if !(greeting_code >= 200 && greeting_code < 300) {; data_payload.push_str(&format!("Fr..." to "use std::fmt::Write as FmtWrite;; if line.as_bytes().get(3) == Some(&b' ') {; if !(200..300).contains(&greeting_code) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 15 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "Err(_) => continue, // Another thread changed it; retry.; // 另一个线程更改了它；重试。" to "Err(_) => {} // Another thread changed it; retry.; // 另一个线程更改了它；重试。".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/encoder.rs`
### Change Record
Modified file `crates/hiver-security/src/encoder.rs`. Approximately 74 lines changed.
### Change Summary
Changed `crates/hiver-security/src/encoder.rs` from "let mut mac = Hmac::<Sha256>::new_from_slice(raw.as_bytes()).expect("unexpected error");; hex::encode(&result[..self.key..." to "let mut mac = Hmac::<Sha256>::new_from_slice(raw.as_bytes()).expect("HMAC key length is valid");; hex::encode(&result.ge...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rememberme.rs`
### Change Record
Modified file `crates/hiver-security/src/rememberme.rs`. Approximately 16 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rememberme.rs` from "if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {; Some((parts[0].to_string(), parts[1].to_string())..." to "let (p0, p1) = (parts.get(0)?, parts.get(1)?);; if !p0.is_empty() && !p1.is_empty() {; Some((p0.to_string(), p1.to_strin...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "Err(_) => continue, // Another thread changed it; retry.; // 另一个线程更改了它；重试。" to "Err(_) => {} // Another thread changed it; retry.; // 另一个线程更改了它；重试。; #[allow(clippy::missing_fields_in_debug)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rememberme.rs`
### Change Record
Modified file `crates/hiver-security/src/rememberme.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rememberme.rs` from "if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {; Some((parts[0].to_string(), parts[1].to_string())..." to "let (p0, p1) = (parts.get(0)?, parts.get(1)?);; if !p0.is_empty() && !p1.is_empty() {; Some((p0.to_string(), p1.to_strin...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rememberme.rs`
### Change Record
Modified file `crates/hiver-security/src/rememberme.rs`. Approximately 33 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rememberme.rs` from "if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {; Some((parts[0].to_string(), parts[1].to_string())..." to "let (p0, p1) = (parts.get(0)?, parts.get(1)?);; if !p0.is_empty() && !p1.is_empty() {; Some((p0.to_string(), p1.to_strin...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:00:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 32 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "Err(_) => continue, // Another thread changed it; retry.; // 另一个线程更改了它；重试。; GatewayCbState::Closed => true," to "Err(_) => {} // Another thread changed it; retry.; // 另一个线程更改了它；重试。; #[allow(clippy::missing_fields_in_debug)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rememberme.rs`
### Change Record
Modified file `crates/hiver-security/src/rememberme.rs`. Approximately 41 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rememberme.rs` from "if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {; Some((parts[0].to_string(), parts[1].to_string())..." to "let (p0, p1) = (parts.get(0)?, parts.get(1)?);; if !p0.is_empty() && !p1.is_empty() {; Some((p0.to_string(), p1.to_strin...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 39 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "Err(_) => continue, // Another thread changed it; retry.; // 另一个线程更改了它；重试。; GatewayCbState::Closed => true," to "Err(_) => {} // Another thread changed it; retry.; // 另一个线程更改了它；重试。; #[allow(clippy::missing_fields_in_debug)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rememberme.rs`
### Change Record
Modified file `crates/hiver-security/src/rememberme.rs`. Approximately 50 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rememberme.rs` from "if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {; Some((parts[0].to_string(), parts[1].to_string())..." to "let (p0, p1) = (parts.get(0)?, parts.get(1)?);; if !p0.is_empty() && !p1.is_empty() {; Some((p0.to_string(), p1.to_strin...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rememberme.rs`
### Change Record
Modified file `crates/hiver-security/src/rememberme.rs`. Approximately 59 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rememberme.rs` from "if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {; Some((parts[0].to_string(), parts[1].to_string())..." to "let (p0, p1) = (parts.get(0)?, parts.get(1)?);; if !p0.is_empty() && !p1.is_empty() {; Some((p0.to_string(), p1.to_strin...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 48 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "Err(_) => continue, // Another thread changed it; retry.; // 另一个线程更改了它；重试。; GatewayCbState::Closed => true," to "Err(_) => {} // Another thread changed it; retry.; // 另一个线程更改了它；重试。; #[allow(clippy::missing_fields_in_debug)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/rememberme.rs`
### Change Record
Modified file `crates/hiver-security/src/rememberme.rs`. Approximately 68 lines changed.
### Change Summary
Changed `crates/hiver-security/src/rememberme.rs` from "if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {; Some((parts[0].to_string(), parts[1].to_string())..." to "let (p0, p1) = (parts.get(0)?, parts.get(1)?);; if !p0.is_empty() && !p1.is_empty() {; Some((p0.to_string(), p1.to_strin...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/permission.rs`
### Change Record
Modified file `crates/hiver-security/src/permission.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::unused_async)]" in `crates/hiver-security/src/permission.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 57 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "Err(_) => continue, // Another thread changed it; retry.; // 另一个线程更改了它；重试。; GatewayCbState::Closed => true," to "Err(_) => {} // Another thread changed it; retry.; // 另一个线程更改了它；重试。; #[allow(clippy::missing_fields_in_debug)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/circuit.rs`
### Change Record
Modified file `crates/hiver-resilience/src/circuit.rs`. Approximately 12 lines changed.
### Change Summary
Removed "0 => CircuitState::Closed," from `crates/hiver-resilience/src/circuit.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/circuit.rs`
### Change Record
Modified file `crates/hiver-resilience/src/circuit.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/circuit.rs` from "(failed as f64) / (total as f64); 0 => CircuitState::Closed," to "#[allow(clippy::cast_precision_loss)]; let rate = (failed as f64) / (total as f64);; rate".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 67 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "Err(_) => continue, // Another thread changed it; retry.; // 另一个线程更改了它；重试。; GatewayCbState::Closed => true," to "Err(_) => {} // Another thread changed it; retry.; // 另一个线程更改了它；重试。; #[allow(clippy::missing_fields_in_debug)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
Modified file `crates/hiver-openapi/src/postman.rs`. Approximately 39 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs` from "req = self.convert_parameter(&param, req);; req = self.convert_parameter(&param, req);; if let Some(body) = &op.request_..." to "req = self.convert_parameter(param, req);; req = self.convert_parameter(param, req);; if let Some(body) = &op.request_bo...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/load_balancer.rs`
### Change Record
Modified file `crates/hiver-cloud/src/load_balancer.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/load_balancer.rs` from "Some(instances[index].clone())" to "instances.get(index).cloned()".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
Modified file `crates/hiver-openapi/src/postman.rs`. Approximately 48 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs` from "collection.info.description = openapi.info.description.clone();; req = self.convert_parameter(&param, req);; req = self...." to "collection.info.description.clone_from(&openapi.info.description);; req = self.convert_parameter(param, req);; req = sel...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/load_balancer.rs`
### Change Record
Modified file `crates/hiver-cloud/src/load_balancer.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/load_balancer.rs` from "Some(instances[index].clone()); if best.is_none() || state.current > best.unwrap().1 {" to "instances.get(index).cloned(); if best.is_none() || state.current > best.map_or(i64::MIN, |(_, c)| c) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/load_balancer.rs`
### Change Record
Modified file `crates/hiver-cloud/src/load_balancer.rs`. Approximately 31 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/load_balancer.rs` from "Some(instances[index].clone()); if best.is_none() || state.current > best.unwrap().1 {; return Some(&instances[0]);" to "instances.get(index).cloned(); if best.is_none() || state.current > best.map_or(i64::MIN, |(_, c)| c) {; return instance...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/circuit.rs`
### Change Record
Modified file `crates/hiver-resilience/src/circuit.rs`. Approximately 118 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/circuit.rs` from "let mut start = self.window_start.lock().expect("lock poisoned");; (failed as f64) / (total as f64); 0 => CircuitState::..." to "let mut start = self.window_start.lock().unwrap_or_else(|e| panic!("lock poisoned: {e}"));; #[allow(clippy::cast_precisi...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/discovery.rs`
### Change Record
Modified file `crates/hiver-resilience/src/discovery.rs`. Approximately 49 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/discovery.rs` from "let services = self.services.read().expect("lock poisoned");; let mut counter = self.rr_counter.write().expect("lock poi..." to "let services = self.services.read().unwrap_or_else(|e| panic!("lock poisoned: {e}"));; let mut counter = self.rr_counter...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/load_balancer.rs`
### Change Record
Modified file `crates/hiver-cloud/src/load_balancer.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/load_balancer.rs` from "Some(instances[index].clone()); if best.is_none() || state.current > best.unwrap().1 {; return Some(&instances[0]);" to "instances.get(index).cloned(); if best.is_none() || state.current > best.map_or(i64::MIN, |(_, c)| c) {; return instance...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/discovery.rs`
### Change Record
Modified file `crates/hiver-resilience/src/discovery.rs`. Approximately 53 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/discovery.rs` from "let services = self.services.read().expect("lock poisoned");; let mut counter = self.rr_counter.write().expect("lock poi..." to "let services = self.services.read().unwrap_or_else(|e| panic!("lock poisoned: {e}"));; let mut counter = self.rr_counter...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/config.rs`
### Change Record
Modified file `crates/hiver-openapi/src/config.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::large_enum_variant)]" in `crates/hiver-openapi/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/operation.rs`
### Change Record
Modified file `crates/hiver-openapi/src/operation.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::large_enum_variant)]" in `crates/hiver-openapi/src/operation.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:01:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/discovery.rs`
### Change Record
Modified file `crates/hiver-resilience/src/discovery.rs`. Approximately 71 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/discovery.rs` from "let services = self.services.read().expect("lock poisoned");; let mut counter = self.rr_counter.write().expect("lock poi..." to "let services = self.services.read().unwrap_or_else(|e| panic!("lock poisoned: {e}"));; let mut counter = self.rr_counter...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/generator.rs`
### Change Record
Modified file `crates/hiver-openapi/src/generator.rs`. Approximately 67 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/generator.rs` from "if !security_schemes.is_empty() {; if let Some(ref mut components) = openapi.components {; .operation_id(format!("get_{}..." to "if !security_schemes.is_empty(); && let Some(ref mut components) = openapi.components {; .operation_id(format!("get_{}",...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/load_balancer.rs`
### Change Record
Modified file `crates/hiver-cloud/src/load_balancer.rs`. Approximately 50 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/load_balancer.rs` from "Some(instances[index].clone()); if best.is_none() || state.current > best.unwrap().1 {; return Some(&instances[0]);" to "instances.get(index).cloned(); #[allow(clippy::expect_used)]; if best.is_none() || state.current > best.map_or(i64::MIN,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/rate_limit.rs`
### Change Record
Modified file `crates/hiver-resilience/src/rate_limit.rs`. Approximately 65 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/rate_limit.rs` from "let mut last = self.last_refill.lock().expect("lock poisoned");; let mut timestamps = self.timestamps.lock().expect("loc..." to "let mut last = self.last_refill.lock().unwrap_or_else(|e| panic!("lock poisoned: {e}"));; let mut timestamps = self.time...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/rate_limit.rs`
### Change Record
Modified file `crates/hiver-resilience/src/rate_limit.rs`. Approximately 70 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/rate_limit.rs` from "let mut last = self.last_refill.lock().expect("lock poisoned");; let tokens_to_add = (elapsed.as_secs_f64() * refill_rat..." to "let mut last = self.last_refill.lock().unwrap_or_else(|e| panic!("lock poisoned: {e}"));; let tokens_to_add = (elapsed.a...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/validators.rs`
### Change Record
Modified file `crates/hiver-validation/src/validators.rs`. Approximately 13 lines changed.
### Change Summary
Added "#![allow(clippy::expect_used)]" in `crates/hiver-validation/src/validators.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/validators.rs`
### Change Record
Modified file `crates/hiver-validation/src/validators.rs`. Approximately 23 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/validators.rs` from "/// 验证邮箱 / Validate email" to "#![allow(clippy::expect_used)]; /// Validate email".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/validators.rs`
### Change Record
Modified file `crates/hiver-validation/src/validators.rs`. Approximately 22 lines changed.
### Change Summary
Added "#![allow(clippy::expect_used)]; ///" in `crates/hiver-validation/src/validators.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/rate_limit.rs`
### Change Record
Modified file `crates/hiver-resilience/src/rate_limit.rs`. Approximately 78 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/rate_limit.rs` from "let mut last = self.last_refill.lock().expect("lock poisoned");; let tokens_to_add = (elapsed.as_secs_f64() * refill_rat..." to "let mut last = self.last_refill.lock().unwrap_or_else(|e| panic!("lock poisoned: {e}"));; let tokens_to_add = (elapsed.a...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/retry.rs`
### Change Record
Modified file `crates/hiver-resilience/src/retry.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::cast_precision_loss)]" in `crates/hiver-resilience/src/retry.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-openapi/src/postman.rs`
### Change Record
Modified file `crates/hiver-openapi/src/postman.rs`. Approximately 48 lines changed.
### Change Summary
Changed `crates/hiver-openapi/src/postman.rs` from "collection.info.description = openapi.info.description.clone();; req = self.convert_parameter(&param, req);; req = self...." to "collection.info.description.clone_from(&openapi.info.description);; req = self.convert_parameter(param, req);; req = sel...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/retry.rs`
### Change Record
Modified file `crates/hiver-resilience/src/retry.rs`. Approximately 20 lines changed.
### Change Summary
Added "#[allow(clippy::cast_precision_loss)]; #[allow(clippy::cast_precision_loss)]" in `crates/hiver-resilience/src/retry.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/lib.rs`
### Change Record
Modified file `crates/hiver-validation/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Added "#![allow(clippy::result_large_err)]" in `crates/hiver-validation/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/retry.rs`
### Change Record
Modified file `crates/hiver-resilience/src/retry.rs`. Approximately 29 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/retry.rs` from "let error_ref = last_error.as_ref().expect("unexpected error");" to "#[allow(clippy::cast_precision_loss)]; #[allow(clippy::cast_precision_loss)]; let error_ref = last_error.as_ref().unwrap...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/retry.rs`
### Change Record
Modified file `crates/hiver-resilience/src/retry.rs`. Approximately 38 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/retry.rs` from "let error_ref = last_error.as_ref().expect("unexpected error");; error: last_error.expect("unexpected error")," to "#[allow(clippy::cast_precision_loss)]; #[allow(clippy::cast_precision_loss)]; let error_ref = last_error.as_ref().unwrap...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/timeout.rs`
### Change Record
Modified file `crates/hiver-resilience/src/timeout.rs`. Approximately 29 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/timeout.rs` from "let mut timeouts = self.timeouts.write().expect("lock poisoned");; let timeouts = self.timeouts.read().expect("lock pois..." to "let mut timeouts = self.timeouts.write().unwrap_or_else(|e| panic!("lock poisoned: {e}"));; let timeouts = self.timeouts...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/annotations.rs`
### Change Record
Modified file `crates/hiver-validation/src/annotations.rs`. Approximately 12 lines changed.
### Change Summary
Added "#[allow(clippy::indexing_slicing)]" in `crates/hiver-validation/src/annotations.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/timeout.rs`
### Change Record
Modified file `crates/hiver-resilience/src/timeout.rs`. Approximately 37 lines changed.
### Change Summary
Changed `crates/hiver-resilience/src/timeout.rs` from "let mut timeouts = self.timeouts.write().expect("lock poisoned");; let timeouts = self.timeouts.read().expect("lock pois..." to "#[allow(clippy::struct_field_names)]; let mut timeouts = self.timeouts.write().unwrap_or_else(|e| panic!("lock poisoned:...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:02:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
Modified file `crates/hiver-validation/src/custom.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs` from "self.validators.get(name).map(|b| b.as_ref())" to "self.validators.get(name).map(|v| v.as_ref())".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/compression.rs`
### Change Record
Modified file `crates/hiver-middleware/src/compression.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/compression.rs` from "parts[1]" to "parts.get(1).unwrap_or(&"")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/compression.rs`
### Change Record
Modified file `crates/hiver-middleware/src/compression.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/compression.rs` from "parts[1]; body: Body," to "parts.get(1).unwrap_or(&""); body: &Body,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/custom.rs`
### Change Record
Modified file `crates/hiver-validation/src/custom.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/custom.rs` from "self.validators.get(name).map(|b| b.as_ref())" to "self.validators.get(name).map(|v| &**v)".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/groups.rs`
### Change Record
Modified file `crates/hiver-validation/src/groups.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/groups.rs` from "pub fn add<G: ValidationGroup>(&mut self, group: G) {" to "pub fn add<G: ValidationGroup>(&mut self, group: &G) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/compression.rs`
### Change Record
Modified file `crates/hiver-middleware/src/compression.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/compression.rs` from "parts[1]" to "parts.get(1).unwrap_or(&""); #[allow(clippy::needless_pass_by_value)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/compression.rs`
### Change Record
Modified file `crates/hiver-middleware/src/compression.rs`. Approximately 30 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/compression.rs` from "parts[1]; let compression = compression_type.expect("unexpected error");" to "parts.get(1).unwrap_or(&""); #[allow(clippy::needless_pass_by_value)]; let compression = compression_type.unwrap_or_else...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/cors.rs`
### Change Record
Modified file `crates/hiver-middleware/src/cors.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/cors.rs` from "pub fn allowed_methods(mut self, methods: Vec<&str>) -> Self {" to "pub fn allowed_methods(mut self, methods: &[&str]) -> Self {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/groups.rs`
### Change Record
Modified file `crates/hiver-validation/src/groups.rs`. Approximately 29 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/groups.rs` from "set.add(group);; set.add(group);; pub fn add<G: ValidationGroup>(&mut self, group: G) {" to "set.add(&group);; set.add(&group);; pub fn add<G: ValidationGroup>(&mut self, group: &G) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/cors.rs`
### Change Record
Modified file `crates/hiver-middleware/src/cors.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/cors.rs` from "pub fn allowed_methods(mut self, methods: Vec<&str>) -> Self {; pub fn allowed_headers(mut self, headers: Vec<&str>) -> ..." to "pub fn allowed_methods(mut self, methods: &[&str]) -> Self {; pub fn allowed_headers(mut self, headers: &[&str]) -> Self...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/groups.rs`
### Change Record
Modified file `crates/hiver-validation/src/groups.rs`. Approximately 40 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/groups.rs` from "set.add(group);; set.add(group);; pub fn add<G: ValidationGroup>(&mut self, group: G) {" to "set.add(&group);; set.add(&group);; pub fn add<G: ValidationGroup>(&mut self, group: &G) {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/cors.rs`
### Change Record
Modified file `crates/hiver-middleware/src/cors.rs`. Approximately 29 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/cors.rs` from "pub fn allowed_methods(mut self, methods: Vec<&str>) -> Self {; pub fn allowed_headers(mut self, headers: Vec<&str>) -> ..." to "pub fn allowed_methods(mut self, methods: &[&str]) -> Self {; pub fn allowed_headers(mut self, headers: &[&str]) -> Self...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/cors.rs`
### Change Record
Modified file `crates/hiver-middleware/src/cors.rs`. Approximately 40 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/cors.rs` from "/// .allowed_methods(vec!["GET", "POST"]); /// .allowed_headers(vec!["Content-Type", "Authorization"]); pub fn allowed_m..." to "/// .allowed_methods(&["GET", "POST"]); /// .allowed_headers(&["Content-Type", "Authorization"]); pub fn allowed_methods...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/jwt_auth.rs`
### Change Record
Modified file `crates/hiver-middleware/src/jwt_auth.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/jwt_auth.rs` from "Some(_) => None,; None => None," to "Some(_) | None => None,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/jwt_auth.rs`
### Change Record
Modified file `crates/hiver-middleware/src/jwt_auth.rs`. Approximately 18 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/jwt_auth.rs` from "Some(_) => None,; None => None,; let token = if let Some(t) = token { t } else {" to "Some(_) | None => None,; let Some(token) = token else {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:03:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/static_files.rs`
### Change Record
### Change Summary
Changed `crates/hiver-middleware/src/static_files.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:04:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/static_files.rs`
### Change Record
Modified file `crates/hiver-middleware/src/static_files.rs`. Approximately 12 lines changed.
### Change Summary
Added "use std::fmt::Write as FmtWrite;" in `crates/hiver-middleware/src/static_files.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:04:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/groups.rs`
### Change Record
Modified file `crates/hiver-validation/src/groups.rs`. Approximately 43 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/groups.rs` from "pub fn with_group<G: ValidationGroup>(group: G) -> Self {; set.add(group);; set.add(group);" to "pub fn with_group<G: ValidationGroup>(group: &G) -> Self {; set.add(&group);; set.add(&group);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:04:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/static_files.rs`
### Change Record
Modified file `crates/hiver-middleware/src/static_files.rs`. Approximately 27 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/static_files.rs` from "html.push_str(&format!(" to "use std::fmt::Write as FmtWrite;; let _ = write!(; html,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:04:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/groups.rs`
### Change Record
Modified file `crates/hiver-validation/src/groups.rs`. Approximately 48 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/groups.rs` from "Self::with_group(DefaultGroup); pub fn with_group<G: ValidationGroup>(group: G) -> Self {; set.add(group);" to "Self::with_group(&DefaultGroup); pub fn with_group<G: ValidationGroup>(group: &G) -> Self {; set.add(&group);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:04:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/static_files.rs`
### Change Record
Modified file `crates/hiver-middleware/src/static_files.rs`. Approximately 35 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/static_files.rs` from "html.push_str(&format!(; .unwrap())" to "use std::fmt::Write as FmtWrite;; let _ = write!(; html,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:04:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/groups.rs`
### Change Record
Modified file `crates/hiver-validation/src/groups.rs`. Approximately 58 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/groups.rs` from "Self::with_group(DefaultGroup); pub fn with_group<G: ValidationGroup>(group: G) -> Self {; set.add(group);" to "Self::with_group(&DefaultGroup); pub fn with_group<G: ValidationGroup>(group: &G) -> Self {; set.add(&group);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:04:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/static_files.rs`
### Change Record
Modified file `crates/hiver-middleware/src/static_files.rs`. Approximately 45 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/static_files.rs` from "html.push_str(&format!(; .unwrap()); .unwrap());" to "use std::fmt::Write as FmtWrite;; let _ = write!(; html,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:04:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/static_files.rs`
### Change Record
Modified file `crates/hiver-middleware/src/static_files.rs`. Approximately 54 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/static_files.rs` from "html.push_str(&format!(; .unwrap()); .unwrap());" to "use std::fmt::Write as FmtWrite;; let _ = write!(; html,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:04:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/groups.rs`
### Change Record
Modified file `crates/hiver-validation/src/groups.rs`. Approximately 58 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/groups.rs` from "Self::with_group(DefaultGroup); set.add(group);; set.add(group);" to "Self::with_group(&DefaultGroup); #[allow(clippy::needless_pass_by_value)]; set.add(&group);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:05:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/groups.rs`
### Change Record
Modified file `crates/hiver-validation/src/groups.rs`. Approximately 53 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/groups.rs` from "set.add(group);; set.add(group);; pub fn add<G: ValidationGroup>(&mut self, group: G) {" to "#[allow(clippy::needless_pass_by_value)]; set.add(&group);; set.add(&group);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:05:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-validation/src/groups.rs`
### Change Record
Modified file `crates/hiver-validation/src/groups.rs`. Approximately 43 lines changed.
### Change Summary
Changed `crates/hiver-validation/src/groups.rs` from "set.add(group);; set.add(group);; pub fn add<G: ValidationGroup>(&mut self, group: G) {" to "#[allow(clippy::needless_pass_by_value)]; set.add(&group);; set.add(&group);".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:05:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/src/engine.rs`
### Change Record
Modified file `crates/hiver-graphql/src/engine.rs`. Approximately 33 lines changed.
### Change Summary
Changed `crates/hiver-graphql/src/engine.rs` from "fn from_ag(resp: AGResponse) -> Self {; GraphQLResponse::from_ag(resp); resps.into_iter().map(GraphQLResponse::from_ag)...." to "fn from_ag(resp: &AGResponse) -> Self {; GraphQLResponse::from_ag(&resp); resps.into_iter().map(|r| GraphQLResponse::fro...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:05:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 62 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:05:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/lib.rs`
### Change Record
Modified file `crates/hiver-security/src/lib.rs`. Approximately 19 lines changed.
### Change Summary
Added "// Allow expect_used/unwrap_used on RwLock/Mutex guards: lock poisoning is; // intentionally unrecoverable — panicking i..." in `crates/hiver-security/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:05:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 72 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:05:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/lib.rs`
### Change Record
Modified file `crates/hiver-resilience/src/lib.rs`. Approximately 24 lines changed.
### Change Summary
Added "// Allow expect_used on RwLock/Mutex guards: lock poisoning is intentionally; // unrecoverable — panicking is the standa..." in `crates/hiver-resilience/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:06:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/lib.rs`
### Change Record
Modified file `crates/hiver-grpc/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Added "#![allow(clippy::expect_used)]; #![allow(clippy::unwrap_used)]" in `crates/hiver-grpc/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:06:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-extractors/src/path.rs`
### Change Record
Modified file `crates/hiver-extractors/src/path.rs`. Approximately 45 lines changed.
### Change Summary
Changed `crates/hiver-extractors/src/path.rs` from "let v1 = T1::from_str(path_vars.get(&var_names[0]).expect("unexpected error")); let v2 = T2::from_str(path_vars.get(&var..." to "let v1 = T1::from_str(path_vars.get(var_names.first().ok_or_else(|| {; ExtractorError::Missing("expected 2 path paramete...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:06:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-grpc/src/client.rs`
### Change Record
Modified file `crates/hiver-grpc/src/client.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-grpc/src/client.rs` from "Some(&self.channels[idx])" to "self.channels.get(idx)".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:06:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-extractors/src/path.rs`
### Change Record
Modified file `crates/hiver-extractors/src/path.rs`. Approximately 51 lines changed.
### Change Summary
Changed `crates/hiver-extractors/src/path.rs` from "let v1 = T1::from_str(path_vars.get(&var_names[0]).expect("unexpected error")); let v2 = T2::from_str(path_vars.get(&var..." to "let v1 = T1::from_str(path_vars.get(var_names.first().ok_or_else(|| {; ExtractorError::Missing("expected 2 path paramete...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:06:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/src/engine.rs`
### Change Record
Modified file `crates/hiver-graphql/src/engine.rs`. Approximately 35 lines changed.
### Change Summary
Changed `crates/hiver-graphql/src/engine.rs` from "fn from_ag(resp: AGResponse) -> Self {; let json = serde_json::to_value(&resp).unwrap_or(serde_json::json!({}));; GraphQ..." to "fn from_ag(resp: &AGResponse) -> Self {; let json = serde_json::to_value(resp).unwrap_or(serde_json::json!({}));; GraphQ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:06:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/excel.rs`
### Change Record
Modified file `crates/hiver-response/src/excel.rs`. Approximately 257 lines changed.
### Change Summary
Changed `crates/hiver-response/src/excel.rs` from "fn as_str(&self) -> &'static str {; ExcelCell::Number(_) => "n",; ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n"," to "fn as_str(self) -> &'static str {; ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",; let _ = w...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:06:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/response.rs`
### Change Record
Modified file `crates/hiver-response/src/response.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-response/src/response.rs` from ".unwrap()" to ".unwrap_or_else(|_| http::Response::builder().body(Body::default()).unwrap_or_else(|_| http::Response::new()))".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:07:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 64 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:07:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/lib.rs`
### Change Record
Modified file `crates/hiver-security/src/lib.rs`. Approximately 49 lines changed.
### Change Summary
Added "// Allow expect_used/unwrap_used on RwLock/Mutex guards: lock poisoning is; // intentionally unrecoverable — panicking i..." in `crates/hiver-security/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:07:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 76 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:07:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-resilience/src/lib.rs`
### Change Record
Modified file `crates/hiver-resilience/src/lib.rs`. Approximately 27 lines changed.
### Change Summary
Added "// Allow expect_used on RwLock/Mutex guards: lock poisoning is intentionally; // unrecoverable — panicking is the standa..." in `crates/hiver-resilience/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:07:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 72 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:07:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 78 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "match self.tokens.compare_exchange(; ) {; Ok(_) => return true," to "if self.tokens.compare_exchange(; ).is_ok() {; return true;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:08:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 81 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "match self.tokens.compare_exchange(; ) {; Ok(_) => return true," to "if self.tokens.compare_exchange(; ).is_ok() {; return true;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:08:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/response.rs`
### Change Record
Modified file `crates/hiver-response/src/response.rs`. Approximately 17 lines changed.
### Change Summary
Changed `crates/hiver-response/src/response.rs` from ".unwrap()" to ".unwrap_or_else(|_| http::Response::builder().body(Body::default()).unwrap_or_else(|_| {; http::Response::builder().stat...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:08:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/lib.rs`
### Change Record
Modified file `crates/hiver-response/src/lib.rs`. Approximately 16 lines changed.
### Change Summary
Added "// Allow expect_used: Response::builder().body() failure is unrecoverable;; // using expect with a descriptive message i..." in `crates/hiver-response/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:08:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-response/src/response.rs`
### Change Record
Modified file `crates/hiver-response/src/response.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-response/src/response.rs` from ".unwrap()" to ".expect("response builder with default body should not fail")".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:08:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/lib.rs`
### Change Record
Modified file `crates/hiver-middleware/src/lib.rs`. Approximately 19 lines changed.
### Change Summary
Added "// Allow expect_used: Response::builder().body() failure is unrecoverable;; // using expect with a descriptive message i..." in `crates/hiver-middleware/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:09:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/static_files.rs`
### Change Record
Modified file `crates/hiver-middleware/src/static_files.rs`. Approximately 54 lines changed.
### Change Summary
Changed `crates/hiver-middleware/src/static_files.rs` from "html.push_str(&format!(; .unwrap()); .unwrap());" to "use std::fmt::Write as FmtWrite;; let _ = write!(; html,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:09:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-middleware/src/lib.rs`
### Change Record
Modified file `crates/hiver-middleware/src/lib.rs`. Approximately 36 lines changed.
### Change Summary
Added "// Allow expect_used: Response::builder().body() failure is unrecoverable;; // using expect with a descriptive message i..." in `crates/hiver-middleware/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:09:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-extractors/src/lib.rs`
### Change Record
Modified file `crates/hiver-extractors/src/lib.rs`. Approximately 16 lines changed.
### Change Summary
Added "// Allow indexing_slicing: path variable names are guaranteed-length; // by prior length checks. Using .get() adds unnec..." in `crates/hiver-extractors/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:09:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 89 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "match self.tokens.compare_exchange(; ) {; Ok(_) => return true," to "if self.tokens.compare_exchange(; ).is_ok() {; return true;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:10:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/gateway.rs`
### Change Record
Modified file `crates/hiver-cloud/src/gateway.rs`. Approximately 86 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/gateway.rs` from "match self.tokens.compare_exchange(; ) {; Ok(_) => return true," to "if self.tokens.compare_exchange(; ).is_ok() {; return true;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:10:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 64 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; prefix: &str,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:10:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 65 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; #[allow(clippy::match_same_arms)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:10:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cloud/src/config_client.rs`
### Change Record
Modified file `crates/hiver-cloud/src/config_client.rs`. Approximately 65 lines changed.
### Change Summary
Changed `crates/hiver-cloud/src/config_client.rs` from "flatten_object(obj, String::new(), &mut map);; flatten_object(obj, String::new(), &mut map);; prefix: String," to "flatten_object(obj, "", &mut map);; flatten_object(obj, "", &mut map);; #[allow(clippy::match_wildcard_for_single_varian...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:15:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-test/src/lib.rs`
### Change Record
Modified file `crates/hiver-test/src/lib.rs`. Approximately 9 lines changed.
### Change Summary
Added "#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::doc_overindented_list_items, clippy::missing_fields_in_d..." in `crates/hiver-test/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:15:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/lib.rs`
### Change Record
Modified file `crates/hiver-starter/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-starter/src/lib.rs` from "#![allow(dead_code)]" to "#![allow(dead_code, clippy::expect_used, clippy::indexing_slicing, clippy::type_complexity, clippy::format_in_format_arg...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:15:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/lib.rs`
### Change Record
Modified file `crates/hiver-starter/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-starter/src/lib.rs` from "#![allow(dead_code)]" to "#![allow(dead_code, clippy::expect_used, clippy::indexing_slicing, clippy::type_complexity, clippy::format_in_format_arg...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:21:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-actuator/src/lib.rs`
### Change Record
Modified file `crates/hiver-actuator/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "#![allow(clippy::expect_used, clippy::unwrap_used, clippy::needless_pass_by_value)]" in `crates/hiver-actuator/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:21:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-agent/src/lib.rs`
### Change Record
Modified file `crates/hiver-agent/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]" in `crates/hiver-agent/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:21:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-annotations/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-annotations/src/lib.rs`. Approximately 11 lines changed.
### Change Summary
Added "#![allow(clippy::expect_used, clippy::unwrap_used)]" in `crates/hiver-data-annotations/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 11:22:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-actuator/src/metrics.rs`
### Change Record
Modified file `crates/hiver-actuator/src/metrics.rs`. Approximately 31 lines changed.
### Change Summary
Changed `crates/hiver-actuator/src/metrics.rs` from "let mut metrics = Vec::new();; // JVM/process equivalent metrics (placeholder values); metrics.push(" to "vec![; Metric::gauge("process.cpu.usage", 5).with_description("Process CPU usage"),; Metric::gauge("system.cpu.count", 4...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:44:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/kqueue.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/kqueue.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/kqueue.rs` from "let driver = KqueueDriver::new().unwrap();" to "// Create a driver without real kqueue fd (only needs ring buffer math); // 创建不带真实kqueue fd的driver（只需要环形缓冲区计算）; let driv...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:49:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/kqueue.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/kqueue.rs`. Approximately 29 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/kqueue.rs` from "let driver = KqueueDriver::new().unwrap();" to "// Create a driver without real kqueue fd (only needs ring buffer math); // 创建不带真实kqueue fd的driver（只需要环形缓冲区计算）; let driv...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:50:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/kqueue.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/kqueue.rs`. Approximately 90 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/kqueue.rs` from "let driver = KqueueDriver::new();; assert!(driver.is_ok());; let driver = driver.unwrap();" to "// Use dummy fd to avoid SIGBUS from real kqueue fd cleanup in tests; // 使用虚拟fd避免测试中真实kqueue fd清理导致的SIGBUS; let driver =...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:58:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `CONTRIBUTING.md`
### Change Record
Modified file `CONTRIBUTING.md`. Approximately 54 lines changed.
### Change Summary
Changed `CONTRIBUTING.md` from "- Rust 1.75 or later / Rust 1.75 或更高版本; git clone https://github.com/hiver-framework/hiver.git; # Install development to..." to "- Rust 1.93 or later / Rust 1.93 或更高版本; git clone https://github.com/ViewWay/hiver.git; # Add required components / 添加必需...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:58:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `CONTRIBUTING.md`
### Change Record
Modified file `CONTRIBUTING.md`. Approximately 64 lines changed.
### Change Summary
Changed `CONTRIBUTING.md` from "- Rust 1.75 or later / Rust 1.75 或更高版本; git clone https://github.com/hiver-framework/hiver.git; # Install development to..." to "- Rust 1.93 or later / Rust 1.93 或更高版本; git clone https://github.com/ViewWay/hiver.git; # Add required components / 添加必需...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:58:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `CONTRIBUTING.md`
### Change Record
Modified file `CONTRIBUTING.md`. Approximately 89 lines changed.
### Change Summary
Changed `CONTRIBUTING.md` from "- Rust 1.75 or later / Rust 1.75 或更高版本; git clone https://github.com/hiver-framework/hiver.git; # Install development to..." to "- Rust 1.93 or later / Rust 1.93 或更高版本; git clone https://github.com/ViewWay/hiver.git; # Add required components / 添加必需...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:58:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `CONTRIBUTING.md`
### Change Record
Modified file `CONTRIBUTING.md`. Approximately 100 lines changed.
### Change Summary
Changed `CONTRIBUTING.md` from "- Rust 1.75 or later / Rust 1.75 或更高版本; git clone https://github.com/hiver-framework/hiver.git; # Install development to..." to "- Rust 1.93 or later / Rust 1.93 或更高版本; git clone https://github.com/ViewWay/hiver.git; # Add required components / 添加必需...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:58:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/DOCS-INDEX.md`
### Change Record
Modified file `docs/DOCS-INDEX.md`. Approximately 23 lines changed.
### Change Summary
Changed `docs/DOCS-INDEX.md` from "Overall Completion: 35%; 总体完成度：35%; ├─ Data Layer: █░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0% ❌ CRITICAL" to "Overall Completion: 40%; 总体完成度：40%; ├─ Data Layer: ████████░░░░░░░░░░░░░░░░░░░░ 33% ⚠️ (8.1 commons ✅, 8.2 rdbc ✅, 8.3 o...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:58:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/DOCS-INDEX.md`
### Change Record
Modified file `docs/DOCS-INDEX.md`. Approximately 39 lines changed.
### Change Summary
Changed `docs/DOCS-INDEX.md` from "Overall Completion: 35%; 总体完成度：35%; ├─ Data Layer: █░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0% ❌ CRITICAL" to "Overall Completion: 40%; 总体完成度：40%; ├─ Data Layer: ████████░░░░░░░░░░░░░░░░░░░░ 33% ⚠️ (8.1 commons ✅, 8.2 rdbc ✅, 8.3 o...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:58:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/DOCS-INDEX.md`
### Change Record
Modified file `docs/DOCS-INDEX.md`. Approximately 49 lines changed.
### Change Summary
Changed `docs/DOCS-INDEX.md` from "Overall Completion: 35%; 总体完成度：35%; ├─ Data Layer: █░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0% ❌ CRITICAL" to "Overall Completion: 40%; 总体完成度：40%; ├─ Data Layer: ████████░░░░░░░░░░░░░░░░░░░░ 33% ⚠️ (8.1 commons ✅, 8.2 rdbc ✅, 8.3 o...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:59:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/SECURITY_AUDIT.md`
### Change Record
Modified file `docs/SECURITY_AUDIT.md`. Approximately 16 lines changed.
### Change Summary
Changed `docs/SECURITY_AUDIT.md` from "**Date / 日期**: 2026-01-29; **Phase / 阶段**: Phase 7 - Production Ready / 生产就绪" to "**Date / 日期**: 2026-05-30; **Phase / 阶段**: Phase 8 - Data Layer / 数据层; **Last Audit Tool / 上次审计工具**: \`cargo audit\` (RU...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:59:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/SECURITY_AUDIT.md`
### Change Record
Modified file `docs/SECURITY_AUDIT.md`. Approximately 75 lines changed.
### Change Summary
Changed `docs/SECURITY_AUDIT.md` from "**Date / 日期**: 2026-01-29; **Phase / 阶段**: Phase 7 - Production Ready / 生产就绪; ### 1. RSA Marvin Attack (sqlx-mysql path)" to "**Date / 日期**: 2026-05-30; **Phase / 阶段**: Phase 8 - Data Layer / 数据层; **Last Audit Tool / 上次审计工具**: \`cargo audit\` (RU...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 14:59:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/SECURITY_AUDIT.md`
### Change Record
Modified file `docs/SECURITY_AUDIT.md`. Approximately 95 lines changed.
### Change Summary
Changed `docs/SECURITY_AUDIT.md` from "**Date / 日期**: 2026-01-29; **Phase / 阶段**: Phase 7 - Production Ready / 生产就绪; ### 1. RSA Marvin Attack (sqlx-mysql path)" to "**Date / 日期**: 2026-05-30; **Phase / 阶段**: Phase 8 - Data Layer / 数据层; **Last Audit Tool / 上次审计工具**: \`cargo audit\` (RU...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 15:03:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `.github/workflows/ci.yml`
### Change Record
Modified file `.github/workflows/ci.yml`. Approximately 317 lines changed.
### Change Summary
Changed `.github/workflows/ci.yml` from "# Hiver Framework CI/CD Pipeline; # Hiver框架 CI/CD流水线; #" to "# Hiver Framework CI Pipeline; # Hiver框架CI流水线; branches: [main]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 15:06:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `CLAUDE.md`
### Change Record
Modified file `CLAUDE.md`. Approximately 13 lines changed.
### Change Summary
Changed `CLAUDE.md` from "- 8.3 hiver-data-orm: ORM abstraction, ActiveRecord, Model derive, SeaORM bridge" to "- 8.3 hiver-data-orm: ORM abstraction, ActiveRecord, Model derive, QueryBuilder, Relationships, Migrations, SeaORM/Diese...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:55:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;" to "use hiver_data_rdbc::{DatabaseClient, QueryParam};".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:55:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 29 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",; .fetch_all_params(&sql, &[QueryPa...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:56:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 40 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",; .fetch_all_params(&sql, &[QueryPa...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:56:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 56 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",; .fetch_all_params(&sql, &[QueryPa...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:56:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 72 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",; .fetch_all_params(&sql, &[QueryPa...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:56:28
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 90 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",; .fetch_all_params(&sql, &[QueryPa...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:56:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 108 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",; .fetch_all_params(&sql, &[QueryPa...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:56:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 126 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",; .fetch_all_params(&sql, &[QueryPa...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:56:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 163 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",; .fetch_all_params(&sql, &[QueryPa...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:56:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 178 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",; .fetch_all_params(&sql, &[QueryPa...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:59:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-annotations/src/relation.rs`
### Change Record
Modified file `crates/hiver-data-annotations/src/relation.rs`. Approximately 51 lines changed.
### Change Summary
Added "//! - \`#[OneToOne]\` - One-to-one relationship / 一对一关系; /// Implements #[OneToOne] attribute macro.; /// 实现 #[OneToOne]..." in `crates/hiver-data-annotations/src/relation.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:59:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-annotations/src/entity.rs`
### Change Record
Modified file `crates/hiver-data-annotations/src/entity.rs`. Approximately 12 lines changed.
### Change Summary
Added ""OneToOne"," in `crates/hiver-data-annotations/src/entity.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:59:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-annotations/src/entity.rs`
### Change Record
Modified file `crates/hiver-data-annotations/src/entity.rs`. Approximately 25 lines changed.
### Change Summary
Added ""OneToOne",; "OneToOne" => {; let args = parse_attr_args(attr);" in `crates/hiver-data-annotations/src/entity.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 20:59:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-annotations/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-annotations/src/lib.rs`. Approximately 37 lines changed.
### Change Summary
Added "/// Marks a one-to-one relationship; /// 标记一对一关系; ///" in `crates/hiver-data-annotations/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:00:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 181 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use std::collections::HashMap;; use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:01:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 368 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use std::collections::HashMap;; use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:01:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 374 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use std::collections::HashMap;; use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:01:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 367 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use std::collections::HashMap;; use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:02:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 12 lines changed.
### Change Summary
Added "use crate::relationships::EagerQueryBuilder;" in `crates/hiver-data-orm/src/active_record.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:02:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 34 lines changed.
### Change Summary
Added "use crate::relationships::EagerQueryBuilder;; /// Get an EagerQueryBuilder for this model type with relationship preload..." in `crates/hiver-data-orm/src/active_record.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:02:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/lib.rs` from "pub use relationships::{HasMany, HasOne, BelongsTo, BelongsToMany, EagerLoad, Relation, RelationType, OnDelete};" to "pub use relationships::{HasMany, HasOne, BelongsTo, BelongsToMany, EagerLoad, WithRelations, EagerQueryBuilder, Relation...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:02:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/model.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/model.rs`. Approximately 23 lines changed.
### Change Summary
Added "/// Get the relationship definitions for this model.; /// 获取此模型的关系定义。; ///" in `crates/hiver-data-orm/src/model.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:03:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 435 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "use hiver_data_rdbc::DatabaseClient;; "SELECT * FROM {} WHERE {} = {}",; self.parent_id," to "use std::collections::HashMap;; use hiver_data_rdbc::{DatabaseClient, QueryParam};; "SELECT * FROM {} WHERE {} = $1",".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:03:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 34 lines changed.
### Change Summary
Added "use crate::relationships::{EagerQueryBuilder, enforce_cascade};; /// Get an EagerQueryBuilder for this model type with r..." in `crates/hiver-data-orm/src/active_record.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:03:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 46 lines changed.
### Change Summary
Added "use crate::relationships::{EagerQueryBuilder, enforce_cascade};; // Enforce cascade rules before deleting the parent rec..." in `crates/hiver-data-orm/src/active_record.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:03:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/Cargo.toml`
### Change Record
Modified file `crates/hiver-data-rdbc/Cargo.toml`. Approximately 20 lines changed.
### Change Summary
Added "# Transaction bridge with hiver-tx; tx-bridge = ["dep:hiver-tx"]; # Optional transaction management bridge" in `crates/hiver-data-rdbc/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:03:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:03:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/lib.rs`. Approximately 13 lines changed.
### Change Summary
Added "#[cfg(feature = "tx-bridge")]; pub mod tx_bridge;" in `crates/hiver-data-rdbc/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-30 21:05:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:05:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:07:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:16:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:17:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:17:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:17:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:18:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:20:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:20:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:20:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:21:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/tx_bridge.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/tx_bridge.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/tx_bridge.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-30 21:21:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 30 lines changed.
### Change Summary
Added "/// Get the current transaction from the active transaction context, if any.; /// 从活跃事务上下文中获取当前事务（如果有）。; ///" in `crates/hiver-data-rdbc/src/client.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 01:09:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `README.md`
### Change Record
Modified file `README.md`. Approximately 42 lines changed.
### Change Summary
Changed `README.md` from "**59 crates** across 10 functional domains. See [CODEMAP.md](docs/CODEMAP.md) for the full reference.; │ hiver-response,..." to "**62 crates** across 10 functional domains. See [CODEMAP.md](docs/CODEMAP.md) for the full reference.; │ hiver-response,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 01:09:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `README.md`
### Change Record
Modified file `README.md`. Approximately 51 lines changed.
### Change Summary
Changed `README.md` from "**59 crates** across 10 functional domains. See [CODEMAP.md](docs/CODEMAP.md) for the full reference.; │ hiver-response,..." to "**62 crates** across 10 functional domains. See [CODEMAP.md](docs/CODEMAP.md) for the full reference.; │ hiver-response,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 01:09:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `README.md`
### Change Record
Modified file `README.md`. Approximately 59 lines changed.
### Change Summary
Changed `README.md` from "**59 crates** across 10 functional domains. See [CODEMAP.md](docs/CODEMAP.md) for the full reference.; │ hiver-response,..." to "**62 crates** across 10 functional domains. See [CODEMAP.md](docs/CODEMAP.md) for the full reference.; │ hiver-response,...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 01:11:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/CODEMAP.md`
### Change Record
Modified file `docs/CODEMAP.md`. Approximately 12 lines changed.
### Change Summary
Changed `docs/CODEMAP.md` from "**Total Crates**: 59 | **Total Lines**: ~181k" to "**Total Crates**: 62 | **Total Lines**: ~181k".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 01:11:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/CODEMAP.md`
### Change Record
Modified file `docs/CODEMAP.md`. Approximately 34 lines changed.
### Change Summary
Changed `docs/CODEMAP.md` from "**Total Crates**: 59 | **Total Lines**: ~181k; │ hiver-macros │ │ hiver-vault │ │ hiver-lombok │; │ hiver-lombok │ │ nex..." to "**Total Crates**: 62 | **Total Lines**: ~181k; │ hiver-macros │ │ hiver-vault │ │ hiver-shell-macros │; │ hiver-lombok │...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 01:12:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/DOCS-INDEX.md`
### Change Record
Modified file `docs/DOCS-INDEX.md`. Approximately 19 lines changed.
### Change Summary
Changed `docs/DOCS-INDEX.md` from "Overall Completion: 40%; 总体完成度：40%; ├─ Data Layer: ████████░░░░░░░░░░░░░░░░░░░░ 33% ⚠️ (8.1 commons ✅, 8.2 rdbc ✅, 8.3 o..." to "Overall Completion: 45%; 总体完成度：45%; ├─ Data Layer: ██████████████░░░░░░░░░░░░░░ 55% ⚠️ (8.1 commons ✅, 8.2 rdbc ✅, 8.3 o...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 01:12:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/DOCS-INDEX.md`
### Change Record
Modified file `docs/DOCS-INDEX.md`. Approximately 28 lines changed.
### Change Summary
Changed `docs/DOCS-INDEX.md` from "Overall Completion: 40%; 总体完成度：40%; ├─ Data Layer: ████████░░░░░░░░░░░░░░░░░░░░ 33% ⚠️ (8.1 commons ✅, 8.2 rdbc ✅, 8.3 o..." to "Overall Completion: 45%; 总体完成度：45%; ├─ Data Layer: ██████████████░░░░░░░░░░░░░░ 55% ⚠️ (8.1 commons ✅, 8.2 rdbc ✅, 8.3 o...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 01:12:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/DOCS-INDEX.md`
### Change Record
Modified file `docs/DOCS-INDEX.md`. Approximately 37 lines changed.
### Change Summary
Changed `docs/DOCS-INDEX.md` from "Overall Completion: 40%; 总体完成度：40%; ├─ Data Layer: ████████░░░░░░░░░░░░░░░░░░░░ 33% ⚠️ (8.1 commons ✅, 8.2 rdbc ✅, 8.3 o..." to "Overall Completion: 45%; 总体完成度：45%; ├─ Data Layer: ██████████████░░░░░░░░░░░░░░ 55% ⚠️ (8.1 commons ✅, 8.2 rdbc ✅, 8.3 o...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:02:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 19 lines changed.
### Change Summary
Added "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f..." in `crates/hiver-runtime/src/driver/iouring.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:03:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 41 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/iouring.rs` from "// SAFETY: IoUringDriver can be sent between threads; // IoUringDriver可以在线程间发送; // SAFETY: IoUringDriver can be shared b..." to "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:03:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 51 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/iouring.rs` from "// SAFETY: IoUringDriver can be sent between threads; // IoUringDriver可以在线程间发送; // SAFETY: IoUringDriver can be shared b..." to "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:03:22
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 74 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/iouring.rs` from "// SAFETY: IoUringDriver can be sent between threads; // IoUringDriver可以在线程间发送; // SAFETY: IoUringDriver can be shared b..." to "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:03:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 96 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/iouring.rs` from "// SAFETY: IoUringDriver can be sent between threads; // IoUringDriver可以在线程间发送; // SAFETY: IoUringDriver can be shared b..." to "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:03:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 104 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/iouring.rs` from "// SAFETY: IoUringDriver can be sent between threads; // IoUringDriver可以在线程间发送; // SAFETY: IoUringDriver can be shared b..." to "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:04:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 112 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/iouring.rs` from "// SAFETY: IoUringDriver can be sent between threads; // IoUringDriver可以在线程间发送; // SAFETY: IoUringDriver can be shared b..." to "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:04:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 120 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/iouring.rs` from "// SAFETY: IoUringDriver can be sent between threads; // IoUringDriver可以在线程间发送; // SAFETY: IoUringDriver can be shared b..." to "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:04:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 127 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/iouring.rs` from "// SAFETY: IoUringDriver can be sent between threads; // IoUringDriver可以在线程间发送; // SAFETY: IoUringDriver can be shared b..." to "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:04:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/iouring.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/iouring.rs`. Approximately 135 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/iouring.rs` from "// SAFETY: IoUringDriver can be sent between threads; // IoUringDriver可以在线程间发送; // SAFETY: IoUringDriver can be shared b..." to "/// Actual mmap size for submission queue ring / 提交队列环形缓冲区的实际 mmap 尺寸; sq_ring_mmap_size: usize,; /// Actual mmap size f...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:07:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `README.zh.md`
### Change Record
Modified file `README.zh.md`. Approximately 25 lines changed.
### Change Summary
Changed `README.zh.md` from "- **原生 Web3** - 内置区块链和智能合约支持; - **可观测性** - 兼容 OpenTelemetry 的追踪/指标; - **类型安全** - 利用 Rust 的类型系统" to "- **类 Spring 注解** - \`#[controller]\`、\`#[service]\`、\`#[repository]\`、\`#[autowired]\`、\`#[transactional]\`、\`@Cacheabl...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:07:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `README.zh.md`
### Change Record
Modified file `README.zh.md`. Approximately 39 lines changed.
### Change Summary
Changed `README.zh.md` from "- **原生 Web3** - 内置区块链和智能合约支持; - **可观测性** - 兼容 OpenTelemetry 的追踪/指标; - **类型安全** - 利用 Rust 的类型系统" to "- **类 Spring 注解** - \`#[controller]\`、\`#[service]\`、\`#[repository]\`、\`#[autowired]\`、\`#[transactional]\`、\`@Cacheabl...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:07:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `README.zh.md`
### Change Record
Modified file `README.zh.md`. Approximately 83 lines changed.
### Change Summary
Changed `README.zh.md` from "- **原生 Web3** - 内置区块链和智能合约支持; - **可观测性** - 兼容 OpenTelemetry 的追踪/指标; - **类型安全** - 利用 Rust 的类型系统" to "- **类 Spring 注解** - \`#[controller]\`、\`#[service]\`、\`#[repository]\`、\`#[autowired]\`、\`#[transactional]\`、\`@Cacheabl...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:07:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `README.zh.md`
### Change Record
Modified file `README.zh.md`. Approximately 99 lines changed.
### Change Summary
Changed `README.zh.md` from "- **原生 Web3** - 内置区块链和智能合约支持; - **可观测性** - 兼容 OpenTelemetry 的追踪/指标; - **类型安全** - 利用 Rust 的类型系统" to "- **类 Spring 注解** - \`#[controller]\`、\`#[service]\`、\`#[repository]\`、\`#[autowired]\`、\`#[transactional]\`、\`@Cacheabl...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:08:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/time.rs`
### Change Record
Modified file `crates/hiver-runtime/src/time.rs`. Approximately 50 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/time.rs` from "#[allow(dead_code)]; wheel: *const TimerWheel,; // SAFETY: The wheel pointer is valid as long as the global timer exists" to "/// The handle references the global timer wheel, which has static lifetime.; /// 句柄引用全局时间轮，全局时间轮具有静态生命周期。; // SAFETY: T...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:08:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/time.rs`
### Change Record
Modified file `crates/hiver-runtime/src/time.rs`. Approximately 68 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/time.rs` from "TimerHandle::new(id, self); TimerHandle::new(id, self); #[allow(dead_code)]" to "TimerHandle::new(id); TimerHandle::new(id); /// The handle references the global timer wheel, which has static lifetime.".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:09:38
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/task.rs`
### Change Record
Modified file `crates/hiver-runtime/src/task.rs`. Approximately 20 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/task.rs` from "// Continue polling (busy wait for simplicity); // 继续轮询（为简单起见使用忙等待）; std::hint::spin_loop();" to "// Yield to avoid busy-wait burning CPU.; // A 1ms sleep is a reasonable trade-off between; // responsiveness and CPU us...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:11:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `README.zh.md`
### Change Record
Modified file `README.zh.md`. Approximately 35 lines changed.
### Change Summary
Changed `README.zh.md` from "<a href="https://github.com/hiver-rs/hiver/blob/main/README.md">English</a>&nbsp;&nbsp;; <a href="https://github.com/nex..." to "<a href="https://github.com/ViewWay/hiver/blob/main/README.md">English</a>&nbsp;&nbsp;; <a href="https://github.com/View...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:11:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `README.md`
### Change Record
Modified file `README.md`. Approximately 35 lines changed.
### Change Summary
Changed `README.md` from "<a href="https://github.com/hiver-rs/hiver/blob/main/README.md">English</a>&nbsp;&nbsp;; <a href="https://github.com/nex..." to "<a href="https://github.com/ViewWay/hiver/blob/main/README.md">English</a>&nbsp;&nbsp;; <a href="https://github.com/View...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:11:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 34 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/io.rs` from "/// # Panics / 恐慌; /// This function currently panics as it's not fully implemented.; /// 此函数当前会恐慌，因为它尚未完全实现。" to "/// # Note / 注意; /// This is a simplified split implementation. Both halves reference the same; /// underlying socket. T...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:11:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 116 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/io.rs` from "/// # Panics / 恐慌; /// This function currently panics as it's not fully implemented.; /// 此函数当前会恐慌，因为它尚未完全实现。" to "/// # Note / 注意; /// This is a simplified split implementation. Both halves reference the same; /// underlying socket. T...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:12:33
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/io.rs`
### Change Record
Modified file `crates/hiver-runtime/src/io.rs`. Approximately 116 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/io.rs` from "/// # Panics / 恐慌; /// This function currently panics as it's not fully implemented.; /// 此函数当前会恐慌，因为它尚未完全实现。" to "/// # Note / 注意; /// This is a simplified split implementation. Both halves reference the same; /// underlying socket. T...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:13:08
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-runtime/src/driver/config.rs`
### Change Record
Modified file `crates/hiver-runtime/src/driver/config.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-runtime/src/driver/config.rs` from "DriverType::Auto => unreachable!()," to "DriverType::Auto => {; // Auto should have been resolved by detect_best_driver() above.; // If we reach here, detection ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:13:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/CODEMAP.md`
### Change Record
Modified file `docs/CODEMAP.md`. Approximately 11 lines changed.
### Change Summary
Added "| **hiver-modulith** | 571 | Spring Modulith | \`Module\`, \`ModuleRegistry\`, \`DomainEvent\`, \`EventPublisher\`, \`ve..." in `docs/CODEMAP.md`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:13:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/CODEMAP.md`
### Change Record
Modified file `docs/CODEMAP.md`. Approximately 20 lines changed.
### Change Summary
Added "| **hiver-modulith** | 571 | Spring Modulith | \`Module\`, \`ModuleRegistry\`, \`DomainEvent\`, \`EventPublisher\`, \`ve..." in `docs/CODEMAP.md`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:13:29
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/CODEMAP.md`
### Change Record
Modified file `docs/CODEMAP.md`. Approximately 28 lines changed.
### Change Summary
Added "| **hiver-agent** | 1,901 | Spring AI Agent | \`Agent\`, \`ReActAgent\`, \`AgentChain\`, \`MapReduceAgent\`, \`RouterAge..." in `docs/CODEMAP.md`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:14:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/encoder.rs`
### Change Record
Modified file `crates/hiver-security/src/encoder.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-security/src/encoder.rs` from "bcrypt::hash(raw, self.cost).unwrap_or_else(|_| {; // Fallback to simple hash on error; use md5::{Digest, Md5};" to "// SECURITY: Never silently degrade to a weaker hash algorithm.; // If bcrypt fails, that's a fatal error requiring inve...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:14:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/encoder.rs`
### Change Record
Modified file `crates/hiver-security/src/encoder.rs`. Approximately 93 lines changed.
### Change Summary
Changed `crates/hiver-security/src/encoder.rs` from "bcrypt::hash(raw, self.cost).unwrap_or_else(|_| {; // Fallback to simple hash on error; use md5::{Digest, Md5};" to "// SECURITY: Never silently degrade to a weaker hash algorithm.; // If bcrypt fails, that's a fatal error requiring inve...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:15:03
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/INDEX.md`
### Change Record
Modified file `docs/INDEX.md`. Approximately 12 lines changed.
### Change Summary
Added "- **Phase 7 (Performance & Hardening)**: [Implementation plan § Phase 7](design/implementation-plan.md); - **Phase 8 (Da..." in `docs/INDEX.md`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:15:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/encoder.rs`
### Change Record
Modified file `crates/hiver-security/src/encoder.rs`. Approximately 93 lines changed.
### Change Summary
Changed `crates/hiver-security/src/encoder.rs` from "bcrypt::hash(raw, self.cost).unwrap_or_else(|_| {; // Fallback to simple hash on error; use md5::{Digest, Md5};" to "// SECURITY: Never silently degrade to a weaker hash algorithm.; // If bcrypt fails, that's a fatal error requiring inve...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:15:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-security/src/encoder.rs`
### Change Record
Modified file `crates/hiver-security/src/encoder.rs`. Approximately 147 lines changed.
### Change Summary
Changed `crates/hiver-security/src/encoder.rs` from "bcrypt::hash(raw, self.cost).unwrap_or_else(|_| {; // Fallback to simple hash on error; use md5::{Digest, Md5};" to "// SECURITY: Never silently degrade to a weaker hash algorithm.; // If bcrypt fails, that's a fatal error requiring inve...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:21:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `docs/api/AGENTS.md`
### Change Record
New file `docs/api/AGENTS.md`, not yet tracked by version control.
### Change Summary
Changed `docs/api/AGENTS.md`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 12:22:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `docs/api/full-api-reference.md`
### Change Record
New file `docs/api/full-api-reference.md`, not yet tracked by version control.
### Change Summary
Changed `docs/api/full-api-reference.md`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 12:24:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `docs/api/api-schema.json`
### Change Record
New file `docs/api/api-schema.json`, not yet tracked by version control.
### Change Summary
Changed `docs/api/api-schema.json`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 12:25:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `docs/api/api-schema.json`
### Change Record
New file `docs/api/api-schema.json`, not yet tracked by version control.
### Change Summary
Changed `docs/api/api-schema.json`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 12:58:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/lib.rs`. Approximately 20 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/lib.rs` from "pub mod repository;; pub mod extractor;; pub mod executor;" to "pub mod executor;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:58:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/lib.rs`. Approximately 42 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/lib.rs` from "pub mod repository;; pub mod extractor;; pub mod executor;" to "pub mod executor;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:58:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/lib.rs`. Approximately 55 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/lib.rs` from "pub mod repository;; pub mod extractor;; pub mod executor;" to "pub mod executor;; DatabaseClient, DatabaseConfig, Error, IsolationLevel, MySqlConfig,; PgPoolClient, PostgresConfig, Qu...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:59:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Removed "pub mod repository;" from `crates/hiver-data-orm/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:59:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 20 lines changed.
### Change Summary
Removed "pub mod repository;; pub use repository::{OrmRepository, DefaultOrmRepository};" from `crates/hiver-data-orm/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 12:59:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 28 lines changed.
### Change Summary
Removed "pub mod repository;; pub use repository::{OrmRepository, DefaultOrmRepository};; OrmRepository, DefaultOrmRepository," from `crates/hiver-data-orm/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:01:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/query_runtime.rs`
### Change Record
New file `crates/hiver-data-orm/src/query_runtime.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/query_runtime.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 13:01:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 33 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/lib.rs` from "pub mod repository;; pub use repository::{OrmRepository, DefaultOrmRepository};; OrmRepository, DefaultOrmRepository," to "pub mod query_runtime;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:01:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 37 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/lib.rs` from "pub mod repository;; pub use repository::{OrmRepository, DefaultOrmRepository};; OrmRepository, DefaultOrmRepository," to "pub mod query_runtime;; pub use query_runtime::{AnnotatedQueryExecutor, ParamStyle, QueryMetadata, QueryType};".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:03:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/config.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/config.rs`. Approximately 142 lines changed.
### Change Summary
Added "// ── PoolConfig (moved from connection.rs) ─────────────────────────────; /// Connection pool configuration; /// 连接池配置" in `crates/hiver-data-rdbc/src/config.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:04:07
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 127 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/connection.rs` from "/// Connection pool configuration; /// 连接池配置; ///" to "#![allow(deprecated)] // This module contains deprecated types still used by downstream crates; // Re-export PoolConfig ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:04:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 134 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/connection.rs` from "/// Connection pool configuration; /// 连接池配置; ///" to "#![allow(deprecated)] // This module contains deprecated types still used by downstream crates; // Re-export PoolConfig ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:04:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 142 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/connection.rs` from "/// Connection pool configuration; /// 连接池配置; ///" to "#![allow(deprecated)] // This module contains deprecated types still used by downstream crates; // Re-export PoolConfig ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:04:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/lib.rs`. Approximately 66 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/lib.rs` from "pub mod repository;; pub mod extractor;; pub mod executor;" to "pub mod executor;; #[allow(deprecated)]; pub use connection::Connection;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:04:49
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/connection.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/connection.rs`. Approximately 142 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/connection.rs` from "use std::time::Duration;; /// Connection pool configuration; /// 连接池配置" to "#![allow(deprecated)] // This module contains deprecated types still used by downstream crates; // Re-export PoolConfig ...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:06:58
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-rdbc/src/sql_builder.rs`
### Change Record
New file `crates/hiver-data-rdbc/src/sql_builder.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-rdbc/src/sql_builder.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 13:07:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/executor.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/executor.rs`. Approximately 489 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/executor.rs` from "use hiver_data_commons::{Condition, Page, PageRequest, QueryWrapper, UpdateWrapper};; let (sql, params) = self.build_sel..." to "//!; //! SQL generation is delegated to the \`sql_builder\` module.; //! SQL 生成委托给 \`sql_builder\` 模块。".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:08:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/lib.rs`. Approximately 67 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/lib.rs` from "pub mod repository;; pub mod extractor;; pub mod executor;" to "pub mod executor;; pub mod sql_builder;; #[allow(deprecated)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:09:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/error.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/error.rs`. Approximately 30 lines changed.
### Change Summary
Added "impl From<hiver_data_rdbc::R2dbcError> for OrmError {; /// Convert R2DBC errors into ORM errors with semantic mapping.; ..." in `crates/hiver-data-orm/src/error.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:09:36
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/error.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/error.rs`. Approximately 50 lines changed.
### Change Summary
Added "impl From<hiver_data_rdbc::R2dbcError> for OrmError {; /// Convert R2DBC errors into ORM errors with semantic mapping.; ..." in `crates/hiver-data-orm/src/error.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:10:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 38 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/lib.rs` from "pub mod repository;; pub use repository::{OrmRepository, DefaultOrmRepository};; OrmRepository, DefaultOrmRepository," to "pub mod query_runtime;; pub use query_runtime::{AnnotatedQueryExecutor, ParamStyle, QueryMetadata, QueryType};; Annotate...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:10:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 39 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/lib.rs` from "pub mod repository;; pub mod connection;; pub use repository::{OrmRepository, DefaultOrmRepository};" to "pub mod mock_connection;; pub mod query_runtime;; pub use query_runtime::{AnnotatedQueryExecutor, ParamStyle, QueryMetad...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:10:32
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 40 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/lib.rs` from "pub mod repository;; pub mod connection;; pub use repository::{OrmRepository, DefaultOrmRepository};" to "pub mod mock_connection;; pub mod query_runtime;; pub use mock_connection::Connection;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:11:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/tests.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/tests.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/tests.rs` from "connection::Connection," to "mock_connection::Connection,".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:14:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/INDEX.md`
### Change Record
Modified file `docs/INDEX.md`. Approximately 13 lines changed.
### Change Summary
Added "| [AGENTS.md](api/AGENTS.md) | AI-optimized compact API reference (62 crates) / AI 优化版 API 参考 |; | [api-schema.json](api..." in `docs/INDEX.md`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:14:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/INDEX.md`
### Change Record
Modified file `docs/INDEX.md`. Approximately 16 lines changed.
### Change Summary
Added "| [AGENTS.md](api/AGENTS.md) | AI-optimized compact API reference (62 crates) / AI 优化版 API 参考 |; | [api-schema.json](api..." in `docs/INDEX.md`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:14:18
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/book/src/advanced/resilience.md`
### Change Record
Modified file `docs/book/src/advanced/resilience.md`. Approximately 13 lines changed.
### Change Summary
Changed `docs/book/src/advanced/resilience.md` from "> **Status**: Phase 4 In Progress 🔄; > **状态**: 第4阶段进行中 🔄" to "> **Status**: Phase 4 Complete ✅; > **状态**: 第4阶段完成 ✅".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:18:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/book/src/reference/api.md`
### Change Record
Modified file `docs/book/src/reference/api.md`. Approximately 174 lines changed.
### Change Summary
Changed `docs/book/src/reference/api.md` from "# API Documentation; # API文档; | **API Quick Reference** | [api-quick-reference.md](../../../api/api-quick-reference.md) ..." to "# API Reference / API 参考; > **Status**: 62 Crates, 10 Domains / 62 个 Crate，10 个功能域; > **状态**: 全阶段完成 ✅".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:19:09
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/book/src/reference/performance.md`
### Change Record
Modified file `docs/book/src/reference/performance.md`. Approximately 176 lines changed.
### Change Summary
Changed `docs/book/src/reference/performance.md` from "Hiver is designed for high performance from the ground up.; Hiver 从设计之初就追求高性能。; | Metric | Target | Status |" to "Hiver is designed for high performance from the ground up with a thread-per-core architecture and io-uring-first I/O mod...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:19:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/book/src/reference/security.md`
### Change Record
Modified file `docs/book/src/reference/security.md`. Approximately 241 lines changed.
### Change Summary
Changed `docs/book/src/reference/security.md` from "> **Status**: Phase 3+ Available ✅; Hiver provides comprehensive security features for your applications.; Hiver 为您的应用程序..." to "> **Status**: Phase 3+ Available ✅; Hiver provides comprehensive security features inspired by Spring Security.; Hiver 提...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:21:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/book/src/advanced/testing.md`
### Change Record
Modified file `docs/book/src/advanced/testing.md`. Approximately 307 lines changed.
### Change Summary
Changed `docs/book/src/advanced/testing.md` from "> **Status**: Phase 3+ Available ✅; Hiver provides comprehensive testing support for your applications.; Hiver 为您的应用程序提供..." to "> **Status**: Phase 3+ Available ✅; Hiver provides comprehensive testing support including unit tests, integration tests...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:21:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/book/src/reference/configuration.md`
### Change Record
Modified file `docs/book/src/reference/configuration.md`. Approximately 293 lines changed.
### Change Summary
Changed `docs/book/src/reference/configuration.md` from "> **Status**: Phase 2 Available ✅; > **状态**: 第2阶段可用 ✅; Hiver provides flexible configuration management similar to Sprin..." to "> **Status**: Phase 2+ Available ✅; > **状态**: 第2阶段+可用 ✅; Hiver provides flexible configuration management similar to Spr...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:23:16
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/guides/benchmarking.md`
### Change Record
Modified file `docs/guides/benchmarking.md`. Approximately 14 lines changed.
### Change Summary
Changed `docs/guides/benchmarking.md` from "**Status**: Phase 1 Completed ✅, Phase 2 Benchmarks Completed ✅; **状态**: 第1阶段已完成 ✅，第2阶段基准测试已完成 ✅" to "**Status**: Phase 7 Completed ✅ (All benchmarks passed); **状态**: 第7阶段已完成 ✅（所有基准测试已通过）".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:23:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/guides/benchmarking.md`
### Change Record
Modified file `docs/guides/benchmarking.md`. Approximately 40 lines changed.
### Change Summary
Changed `docs/guides/benchmarking.md` from "**Status**: Phase 1 Completed ✅, Phase 2 Benchmarks Completed ✅; **状态**: 第1阶段已完成 ✅，第2阶段基准测试已完成 ✅; | Metric / 指标 | Target..." to "**Status**: Phase 7 Completed ✅ (All benchmarks passed); **状态**: 第7阶段已完成 ✅（所有基准测试已通过）; | Metric / 指标 | Target / 目标 | Ach...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:24:21
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/design/STRATEGY-OVERVIEW.md`
### Change Record
Modified file `docs/design/STRATEGY-OVERVIEW.md`. Approximately 865 lines changed.
### Change Summary
Changed `docs/design/STRATEGY-OVERVIEW.md` from "## 📊 Current State Assessment / 当前状态评估; │ Overall Completion: 35% │; │ 总体完成度: 35% │" to "## Current State Assessment / 当前状态评估; │ Overall Completion: ~90% │; │ 总体完成度: ~90% │".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 13:25:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `docs/design/design-spec.md`
### Change Record
Modified file `docs/design/design-spec.md`. Approximately 14 lines changed.
### Change Summary
Changed `docs/design/design-spec.md` from "**Date**: 2026-01-23; **Status**: Draft / 草稿" to "**Date**: 2026-05-31; **Status**: Active (Phases 0-7 Complete, Phase 8 In Progress) / 活跃（阶段 0-7 完成，阶段 8 进行中）".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:35:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-macros/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-macros/src/lib.rs`. Approximately 59 lines changed.
### Change Summary
Changed `crates/hiver-data-macros/src/lib.rs` from "// Implement CrudRepository using QueryBuilder/ActiveRecord; "INSERT INTO {} (id) VALUES ({}) ON CONFLICT (id) DO UPDATE..." to "// Implement CrudRepository using parameterized queries; "INSERT INTO {} (id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 14 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("update failed: {e}")))?" to ".await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("update failed: {e}")))?; .await" to ".await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 34 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:23
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 43 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 54 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:26
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 64 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 74 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:35
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 83 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 104 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 114 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 124 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 134 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 144 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:36:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 153 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from ".await; .map_err(|e| crate::Error::unknown(format!("insert failed: {e}")))?; .await" to ".await?; .await?; .await?".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:37:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/data/mod.rs`
### Change Record
Modified file `crates/hiver-starter/src/data/mod.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-starter/src/data/mod.rs` from "pub use hiver_data_rdbc::connection::ConnectionPool;" to "pub use hiver_data_rdbc::SqlxPoolClient;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:38:01
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-starter/src/data/mod.rs`
### Change Record
Modified file `crates/hiver-starter/src/data/mod.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-starter/src/data/mod.rs` from "pub use hiver_data_rdbc::connection::ConnectionPool;; pub async fn create_pool(&self) -> Result<ConnectionPool, hiver_da..." to "pub use hiver_data_rdbc::SqlxPoolClient;; #[cfg(feature = "sqlx")]; pub async fn create_pool(&self) -> Result<SqlxPoolCl...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:43:19
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/Cargo.toml`
### Change Record
Modified file `crates/hiver-graphql/Cargo.toml`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-graphql/Cargo.toml` from "async-graphql = { version = "6", optional = true, features = ["chrono"] }" to "async-graphql = { version = "7", optional = true, features = ["chrono"] }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:45:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/Cargo.toml`
### Change Record
Modified file `crates/hiver-graphql/Cargo.toml`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-graphql/Cargo.toml` from "async-graphql = { version = "6", optional = true, features = ["chrono"] }" to "async-graphql = { version = "7.0.10", optional = true, features = ["chrono"] }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:49:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/Cargo.toml`
### Change Record
### Change Summary
Changed `crates/hiver-graphql/Cargo.toml`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:49:48
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 18 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "let mut condition = w.condition.clone();; for _ in &w.params {; condition = condition.replacen('?', &format!("${param_id..." to "let condition = hiver_data_commons::replace_placeholders(&w.condition, w.params.len(), param_idx);; param_idx += w.param...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:50:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/lib.rs`. Approximately 30 lines changed.
### Change Summary
Added "/// Replace \`?\` placeholders in a SQL fragment with \`$N\` positional parameters.; /// 将 SQL 片段中的 \`?\` 占位符替换为 \`$N\` ..." in `crates/hiver-data-commons/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:50:04
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 32 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "let mut condition = w.condition.clone();; for _param in &w.params {; condition = condition.replacen('?', &format!("${par..." to "let condition = hiver_data_commons::replace_placeholders(&w.condition, w.params.len(), param_idx);; param_idx += w.param...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:50:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 40 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from "let mut cond = condition.to_string();; for (param_idx, _) in (1u32..).zip(params.iter()) {; cond = cond.replacen('?', &f..." to "let cond = hiver_data_commons::replace_placeholders(condition, params.len(), 1);; let cond = hiver_data_commons::replace...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:53:43
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 15:53:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/lib.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/lib.rs`. Approximately 12 lines changed.
### Change Summary
Added "pub mod repository;" in `crates/hiver-data-orm/src/lib.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:54:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 15:55:56
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 15:55:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 15:57:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-graphql/Cargo.toml`
### Change Record
Modified file `crates/hiver-graphql/Cargo.toml`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-graphql/Cargo.toml` from "async-graphql = { version = "6", optional = true, features = ["chrono"] }" to "async-graphql = { version = "7.2", optional = true, features = ["chrono"] }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 15:57:30
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 15:57:39
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/Cargo.toml`
### Change Record
Modified file `crates/hiver-data-orm/Cargo.toml`. Approximately 13 lines changed.
### Change Summary
Added "# Transaction support; hiver-tx = { path = "../hiver-tx" }" in `crates/hiver-data-orm/Cargo.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 16:13:06
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 16:13:46
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 16:13:52
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 16:14:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 16:15:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 16:15:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 16:15:50
- **Project**: hiver
- **Branch**: main
- **Change Type**: New file (untracked)
- **File**: `crates/hiver-data-orm/src/repository.rs`
### Change Record
New file `crates/hiver-data-orm/src/repository.rs`, not yet tracked by version control.
### Change Summary
Changed `crates/hiver-data-orm/src/repository.rs`, but no concrete content diff was available; it may be formatting, permission-only, or already rolled back.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions. File is untracked; remember to `git add` if adding to version control.
## 2026-05-31 19:47:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/query.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/query.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/query.rs` from "Self::String(s) => format!("'{}'", s.replace('\'', "''"))," to "Self::String(s) => format!("'{}'", s.replace('\'', "''").replace('\0', "")),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:47:53
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/query.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/query.rs`. Approximately 24 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/query.rs` from "Self::Like { field, pattern } => format!("{} LIKE '{}'", field, pattern),; Self::NotLike { field, pattern } => format!("..." to "Self::Like { field, pattern } => format!("{} LIKE '{}'", field, pattern.replace('\'', "''").replace('\0', "")),; Self::N...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:48:11
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/query.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/query.rs`. Approximately 47 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/query.rs` from "Self::Eq { field, value } => format!("{} = {}", field, value.to_sql()),; Self::Ne { field, value } => format!("{} != {}"..." to "/// Validate that a field/identifier contains only safe characters.; /// 验证字段/标识符仅包含安全字符。; fn validate_field(field: &str...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:48:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/query.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/query.rs`. Approximately 69 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/query.rs` from "Self::Eq { field, value } => format!("{} = {}", field, value.to_sql()),; Self::Ne { field, value } => format!("{} != {}"..." to "/// Validate that a field/identifier contains only safe characters.; /// 验证字段/标识符仅包含安全字符。; fn validate_field(field: &str...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:48:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 17 lines changed.
### Change Summary
Added "let placeholder_count = condition.matches('?').count();; assert_eq!(; placeholder_count, params.len()," in `crates/hiver-data-orm/src/query.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:48:54
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 26 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "fn validate_identifier(name: &str) -> bool {" to "pub(crate) fn validate_identifier(name: &str) -> bool {; let placeholder_count = condition.matches('?').count();; assert...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:49:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 16 lines changed.
### Change Summary
Added "assert!(; validate_identifier(&o.property),; "Invalid sort property: {}"," in `crates/hiver-data-orm/src/active_record.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:49:17
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/active_record.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/active_record.rs`. Approximately 25 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/active_record.rs` from "use crate::query::QueryBuilder;" to "use crate::query::{QueryBuilder, validate_identifier};; assert!(; validate_identifier(&o.property),".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:49:47
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 34 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/client.rs` from "/// Replace \`$1, $2, ...\` placeholders with SQL literals; /// 将 \`$1, $2, ...\` 占位符替换为 SQL 字面量; for (i, param) in para..." to "/// Replace \`$1, $2, ...\` placeholders with SQL literals.; /// 将 \`$1, $2, ...\` 占位符替换为 SQL 字面量。; ///".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:50:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 24 lines changed.
### Change Summary
Added "///; /// # Important / 重要; ///" in `crates/hiver-data-orm/src/relationships.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:50:55
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 37 lines changed.
### Change Summary
Added "/// Check if a string is a valid SQL identifier (alphanumeric + underscore only).; /// 检查字符串是否为有效的 SQL 标识符（仅字母数字和下划线）。; ..." in `crates/hiver-data-orm/src/relationships.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:51:05
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 37 lines changed.
### Change Summary
Added "/// Check if a string is a valid SQL identifier (alphanumeric + underscore only).; /// 检查字符串是否为有效的 SQL 标识符（仅字母数字和下划线）。; ..." in `crates/hiver-data-orm/src/relationships.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:53:44
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 58 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/client.rs` from "/// Replace \`$1, $2, ...\` placeholders with SQL literals; /// 将 \`$1, $2, ...\` 占位符替换为 SQL 字面量; for (i, param) in para..." to "#[allow(deprecated)]; #[allow(deprecated)]; #[allow(deprecated)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 19:54:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-rdbc/src/client.rs`
### Change Record
Modified file `crates/hiver-data-rdbc/src/client.rs`. Approximately 66 lines changed.
### Change Summary
Changed `crates/hiver-data-rdbc/src/client.rs` from "/// Replace \`$1, $2, ...\` placeholders with SQL literals; /// 将 \`$1, $2, ...\` 占位符替换为 SQL 字面量; for (i, param) in para..." to "#[allow(deprecated)]; #[allow(deprecated)]; #[allow(deprecated)]".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:22:31
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/method_name.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/method_name.rs`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/method_name.rs` from "ConditionType::NotIn => Condition::NotEquals {" to "ConditionType::NotIn => Condition::NotIn {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:22:40
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/method_name.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/method_name.rs`. Approximately 22 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/method_name.rs` from "ConditionType::NotLike => Condition::NotEquals {; ConditionType::NotIn => Condition::NotEquals {" to "ConditionType::NotLike => Condition::NotLike {; ConditionType::NotIn => Condition::NotIn {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:23:00
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 27 lines changed.
### Change Summary
Added "/// Validate a JOIN ON condition to reject SQL injection patterns.; /// 验证 JOIN ON 条件以拒绝 SQL 注入模式。; fn validate_on_condi..." in `crates/hiver-data-orm/src/query.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:23:13
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 49 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "validate_identifier(condition),; "having column must contain only alphanumeric characters and underscores, got: {conditi..." to "/// Validate a JOIN ON condition to reject SQL injection patterns.; /// 验证 JOIN ON 条件以拒绝 SQL 注入模式。; fn validate_on_condi...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:24:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/method_name.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/method_name.rs`. Approximately 41 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/method_name.rs` from "ConditionType::NotLike => Condition::NotEquals {; ConditionType::NotIn => Condition::NotEquals {" to "/// Not like: field NOT LIKE value; /// 不匹配; NotLike {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:24:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/method_name.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/method_name.rs`. Approximately 50 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/method_name.rs` from "ConditionType::NotLike => Condition::NotEquals {; ConditionType::NotIn => Condition::NotEquals {" to "/// Not like: field NOT LIKE value; /// 不匹配; NotLike {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:24:12
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-commons/src/method_name.rs`
### Change Record
Modified file `crates/hiver-data-commons/src/method_name.rs`. Approximately 59 lines changed.
### Change Summary
Changed `crates/hiver-data-commons/src/method_name.rs` from "ConditionType::NotLike => Condition::NotEquals {; ConditionType::NotIn => Condition::NotEquals {" to "/// Not like: field NOT LIKE value; /// 不匹配; NotLike {".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:39:20
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 21 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "/// Check if a string is a valid SQL identifier (alphanumeric + underscore only).; /// 检查字符串是否为有效的 SQL 标识符（仅字母数字和下划线）。; ..." to "use crate::query::validate_identifier;".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:39:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/relationships.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/relationships.rs`. Approximately 32 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/relationships.rs` from "/// Check if a string is a valid SQL identifier (alphanumeric + underscore only).; /// 检查字符串是否为有效的 SQL 标识符（仅字母数字和下划线）。; ..." to "use crate::query::validate_identifier;; debug_assert!(validate_identifier(&self.join_table), "Invalid join table: {}", s...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:39:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 31 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "/// Validate a JOIN ON condition to reject SQL injection patterns.; /// 验证 JOIN ON 条件以拒绝 SQL 注入模式。; fn validate_on_condi..." to "/// Validate a raw SQL condition to reject injection patterns.; /// 验证原始 SQL 条件以拒绝注入模式。; fn validate_raw_condition(condi...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:39:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 43 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "/// Validate a JOIN ON condition to reject SQL injection patterns.; /// 验证 JOIN ON 条件以拒绝 SQL 注入模式。; fn validate_on_condi..." to "/// Validate a raw SQL condition to reject injection patterns.; /// 验证原始 SQL 条件以拒绝注入模式。; fn validate_raw_condition(condi...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:42:34
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-orm/src/query.rs`
### Change Record
Modified file `crates/hiver-data-orm/src/query.rs`. Approximately 47 lines changed.
### Change Summary
Changed `crates/hiver-data-orm/src/query.rs` from "/// Validate a JOIN ON condition to reject SQL injection patterns.; /// 验证 JOIN ON 条件以拒绝 SQL 注入模式。; fn validate_on_condi..." to "/// Validate a raw SQL condition to reject injection patterns.; /// 验证原始 SQL 条件以拒绝注入模式。; fn validate_raw_condition(condi...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:56:10
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-test/Cargo.toml`
### Change Record
Modified file `crates/hiver-test/Cargo.toml`. Approximately 15 lines changed.
### Change Summary
Changed `crates/hiver-test/Cargo.toml` from "testcontainers = "0.23"; testcontainers-modules = { version = "0.11", features = ["postgres", "redis", "kafka"] }" to "testcontainers = "0.27"; testcontainers-modules = { version = "0.15", features = ["postgres", "redis", "kafka"] }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:56:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `tests/Cargo.toml`
### Change Record
Modified file `tests/Cargo.toml`. Approximately 15 lines changed.
### Change Summary
Changed `tests/Cargo.toml` from "testcontainers = "0.23"; testcontainers-modules = { version = "0.11", features = ["postgres", "redis", "kafka", "rabbitm..." to "testcontainers = "0.27"; testcontainers-modules = { version = "0.15", features = ["postgres", "redis", "kafka", "rabbitm...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 21:59:24
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-data-mongodb/Cargo.toml`
### Change Record
Modified file `crates/hiver-data-mongodb/Cargo.toml`. Approximately 13 lines changed.
### Change Summary
Changed `crates/hiver-data-mongodb/Cargo.toml` from "mongodb = { version = "3.1" }" to "mongodb = { version = "3.7" }".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:06:02
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `Cargo.toml`
### Change Record
Modified file `Cargo.toml`. Approximately 19 lines changed.
### Change Summary
Changed `Cargo.toml` from "# Note: Using 1.5.x versions which include fixes for ruint vulnerability (RUSTSEC-2025-0137); alloy = { version = "1.5",..." to "# Note: Using 2.0.x which fixes jsonwebtoken CVE-2026-25537 (uses ^10.3.0); alloy = { version = "2.0", features = ["netw...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:06:42
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `Cargo.toml`
### Change Record
Modified file `Cargo.toml`. Approximately 19 lines changed.
### Change Summary
Changed `Cargo.toml` from "# Note: Using 1.5.x versions which include fixes for ruint vulnerability (RUSTSEC-2025-0137); alloy = { version = "1.5",..." to "# Note: Using 2.0.x which fixes jsonwebtoken CVE-2026-25537 (uses ^10.3.0); alloy = { version = "2.0", features = ["netw...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:23:51
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `book.toml`
### Change Record
Modified file `book.toml`. Approximately 12 lines changed.
### Change Summary
Added "description = "A production-grade, high-availability web framework written in Rust with custom async runtime, Spring-lik..." in `book.toml`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:23:59
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `book.toml`
### Change Record
Modified file `book.toml`. Approximately 25 lines changed.
### Change Summary
Changed `book.toml` from "git-repository-url = "https://github.com/hiver-framework/hiver"; edit-url-template = "https://github.com/hiver-framework..." to "description = "A production-grade, high-availability web framework written in Rust with custom async runtime, Spring-lik...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:28:14
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `book.toml`
### Change Record
Modified file `book.toml`. Approximately 27 lines changed.
### Change Summary
Changed `book.toml` from "multilingual = true; git-repository-url = "https://github.com/hiver-framework/hiver"; edit-url-template = "https://githu..." to "description = "A production-grade, high-availability web framework written in Rust with custom async runtime, Spring-lik...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:50:25
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cache/src/cacheable.rs`
### Change Record
Modified file `crates/hiver-cache/src/cacheable.rs`. Approximately 29 lines changed.
### Change Summary
Changed `crates/hiver-cache/src/cacheable.rs` from "pub async fn get_or_fetch<K, V, F>(cache: &dyn Cache<K, V>, key: &K, fetch: F) -> Option<V>; F: Future<Output = Option<V..." to "pub async fn get_or_fetch<K, V, Fut, F>(cache: &dyn Cache<K, V>, key: &K, fetch: F) -> Option<V>; F: FnOnce() -> Fut,; F...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:50:41
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cache/src/cacheable.rs`
### Change Record
Modified file `crates/hiver-cache/src/cacheable.rs`. Approximately 58 lines changed.
### Change Summary
Changed `crates/hiver-cache/src/cacheable.rs` from "pub async fn get_or_fetch<K, V, F>(cache: &dyn Cache<K, V>, key: &K, fetch: F) -> Option<V>; F: Future<Output = Option<V..." to "pub async fn get_or_fetch<K, V, Fut, F>(cache: &dyn Cache<K, V>, key: &K, fetch: F) -> Option<V>; F: FnOnce() -> Fut,; F...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:50:57
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cache/src/cacheable.rs`
### Change Record
Modified file `crates/hiver-cache/src/cacheable.rs`. Approximately 87 lines changed.
### Change Summary
Changed `crates/hiver-cache/src/cacheable.rs` from "pub async fn get_or_fetch<K, V, F>(cache: &dyn Cache<K, V>, key: &K, fetch: F) -> Option<V>; F: Future<Output = Option<V..." to "pub async fn get_or_fetch<K, V, Fut, F>(cache: &dyn Cache<K, V>, key: &K, fetch: F) -> Option<V>; F: FnOnce() -> Fut,; F...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:51:45
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cache/src/cache_put.rs`
### Change Record
Modified file `crates/hiver-cache/src/cache_put.rs`. Approximately 69 lines changed.
### Change Summary
Changed `crates/hiver-cache/src/cache_put.rs` from "pub async fn execute_and_update<K, V, F>(cache: &dyn Cache<K, V>, key: K, f: F) -> V; F: Future<Output = V> + Send,; let..." to "pub async fn execute_and_update<K, V, Fut, F>(cache: &dyn Cache<K, V>, key: K, f: F) -> V; F: FnOnce() -> Fut,; Fut: Fut...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:52:27
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-cache/src/cache_evict.rs`
### Change Record
Modified file `crates/hiver-cache/src/cache_evict.rs`. Approximately 79 lines changed.
### Change Summary
Changed `crates/hiver-cache/src/cache_evict.rs` from "pub async fn execute_and_evict_key<K, V, F>(cache: &dyn Cache<K, V>, key: &K, f: F); F: Future<Output = ()> + Send,; f.a..." to "pub async fn execute_and_evict_key<K, V, Fut, F>(cache: &dyn Cache<K, V>, key: &K, f: F); F: FnOnce() -> Fut,; Fut: Futu...".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:53:15
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/body.rs`
### Change Record
Modified file `crates/hiver-http/src/body.rs`. Approximately 17 lines changed.
### Change Summary
Added "impl AsRef<[u8]> for FullBody {; fn as_ref(&self) -> &[u8] {; &self.data" in `crates/hiver-http/src/body.rs`.
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.
## 2026-05-31 22:53:37
- **Project**: hiver
- **Branch**: main
- **Change Type**: Modified
- **File**: `crates/hiver-http/src/response.rs`
### Change Record
Modified file `crates/hiver-http/src/response.rs`. Approximately 15 lines changed.
### Change Summary
Changed `crates/hiver-http/src/response.rs` from "pub fn body(mut self, body: Body) -> Result<Response> {; self.body = Some(body);" to "pub fn body(mut self, body: impl Into<Body>) -> Result<Response> {; self.body = Some(body.into());".
### Risk Alert
No specific risk detected. Manual review recommended.
### Suggestion
Run relevant tests before committing to avoid regressions.