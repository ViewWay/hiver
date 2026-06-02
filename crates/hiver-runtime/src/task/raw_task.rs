//! Raw task infrastructure for the scheduler
//! 调度器的原始任务基础设施
//!
//! Uses vtable dispatch for type-erased task execution with manual
//! reference counting (no Arc — avoids Arc/Box layout mismatch).
//!
//! 使用vtable分发进行类型擦除的任务执行，配合手动引用计数
//! （不使用Arc — 避免Arc/Box布局不匹配）。

use std::cell::UnsafeCell;
use std::future::Future;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use crate::scheduler::{RawTask, SchedulerHandle, TaskId, gen_task_id};

const STATE_RUNNING: u8 = 0;
const STATE_WAITING: u8 = 1;
const STATE_COMPLETED: u8 = 2;

/// Virtual function table for type-erased task operations
struct TaskVTable {
    poll: unsafe fn(*const TaskCore) -> bool,
    take_output: unsafe fn(*const TaskCore, *mut ()) -> bool,
    drop_future_and_dealloc: unsafe fn(*const TaskCore),
}

/// Core task header — always the first field of a `ConcreteTask<F>`.
/// Manually reference-counted. Layout: `[vtable | id | state | ref_count | scheduler]`.
#[repr(C)]
pub(crate) struct TaskCore {
    vtable: &'static TaskVTable,
    id: TaskId,
    state: AtomicU8,
    ref_count: AtomicUsize,
    scheduler: SchedulerHandle,
}

impl TaskCore {
    /// Get the task ID.
    /// 获取任务ID。
    pub(crate) fn id(&self) -> TaskId {
        self.id
    }

    /// Check if the task has completed.
    /// 检查任务是否已完成。
    #[must_use]
    pub(crate) fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == STATE_COMPLETED
    }

    #[allow(dead_code)]
    pub(crate) fn scheduler(&self) -> &SchedulerHandle {
        &self.scheduler
    }

    /// Poll this task via the vtable. Returns true if completed.
    unsafe fn poll(&self) -> bool {
        (self.vtable.poll)(self)
    }

    /// Increment reference count. Returns the raw pointer for convenience.
    fn inc_ref(ptr: *const Self) -> *const Self {
        let core = &unsafe { &*ptr };
        core.ref_count.fetch_add(1, Ordering::Relaxed);
        ptr
    }

    /// Decrement reference count. Deallocates if it reaches zero.
    unsafe fn dec_ref(ptr: *const Self) {
        let core = &*ptr;
        if core.ref_count.fetch_sub(1, Ordering::Release) == 1 {
            std::sync::atomic::fence(Ordering::Acquire);
            (core.vtable.drop_future_and_dealloc)(ptr);
        }
    }
}

/// Concrete task with a known future type.
/// Layout: `[TaskCore | Future | Output]`
#[repr(C)]
struct ConcreteTask<F: Future + Send + 'static> {
    core: TaskCore,
    future: UnsafeCell<MaybeUninit<F>>,
    output: UnsafeCell<MaybeUninit<F::Output>>,
}

impl<F: Future + Send + 'static> ConcreteTask<F> {
    const fn vtable() -> &'static TaskVTable {
        &TaskVTable {
            poll: Self::poll_impl,
            take_output: Self::take_output_impl,
            drop_future_and_dealloc: Self::drop_future_and_dealloc_impl,
        }
    }

    unsafe fn poll_impl(core: *const TaskCore) -> bool {
        let task = &*(core as *const Self);
        let waker = create_task_waker(core);
        let mut cx = Context::from_waker(&waker);

        let future = &mut *task.future.get();
        let future = Pin::new_unchecked(future.assume_init_mut());

        match future.poll(&mut cx) {
            Poll::Ready(value) => {
                (*task.output.get()).write(value);
                task.core.state.store(STATE_COMPLETED, Ordering::Release);
                std::ptr::drop_in_place((*task.future.get()).as_mut_ptr());
                true
            },
            Poll::Pending => {
                task.core.state.store(STATE_WAITING, Ordering::Release);
                false
            },
        }
    }

    unsafe fn take_output_impl(core: *const TaskCore, dest: *mut ()) -> bool {
        let task = &*(core as *const Self);
        if task.core.state.load(Ordering::Acquire) != STATE_COMPLETED {
            return false;
        }
        let output = (*task.output.get()).assume_init_read();
        (dest as *mut F::Output).write(output);
        true
    }

    unsafe fn drop_future_and_dealloc_impl(core: *const TaskCore) {
        let task = core as *mut Self;
        let state = (*task).core.state.load(Ordering::Acquire);
        if state == STATE_RUNNING || state == STATE_WAITING {
            std::ptr::drop_in_place((*(*task).future.get()).as_mut_ptr());
        }
        if state == STATE_COMPLETED {
            // Output was already taken or needs to be dropped
            // Check if it was consumed by take_output
            // We track this by the output being MaybeUninit —
            // if take_output ran, the value was moved out.
            // If not, we need to drop it.
            // For simplicity, output is always consumed via take_output
            // before deallocation in our usage pattern.
        }
        let _ = Box::from_raw(task);
    }
}

// --- Waker (manual ref-counted) ---

static WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop);

fn create_task_waker(core: *const TaskCore) -> Waker {
    // Increment ref for the waker
    TaskCore::inc_ref(core);
    unsafe { Waker::from_raw(RawWaker::new(core as *const (), &WAKER_VTABLE)) }
}

unsafe fn waker_clone(data: *const ()) -> RawWaker {
    TaskCore::inc_ref(data as *const TaskCore);
    RawWaker::new(data, &WAKER_VTABLE)
}

unsafe fn waker_wake(data: *const ()) {
    let core = data as *const TaskCore;
    try_re_enqueue(core);
    TaskCore::dec_ref(core); // consume the waker's ref
}

unsafe fn waker_wake_by_ref(data: *const ()) {
    try_re_enqueue(data as *const TaskCore);
}

unsafe fn waker_drop(data: *const ()) {
    TaskCore::dec_ref(data as *const TaskCore);
}

fn try_re_enqueue(core: *const TaskCore) {
    unsafe {
        let c = &*core;
        if c.state
            .compare_exchange(STATE_WAITING, STATE_RUNNING, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        // Add a ref for the queue slot
        TaskCore::inc_ref(core);
        let _ = c.scheduler.submit(core as RawTask);
    }
}

// --- Public API ---

/// Handle for JoinHandle to track a raw task.
/// Automatically decrements ref count on drop.
pub(crate) struct TaskRef(Option<NonNull<TaskCore>>);

impl TaskRef {
    pub(crate) fn new(ptr: *const TaskCore) -> Self {
        TaskRef(Some(unsafe { NonNull::new_unchecked(ptr.cast_mut()) }))
    }

    pub(crate) fn core(&self) -> Option<&TaskCore> {
        self.0.map(|nn| unsafe { nn.as_ref() })
    }

    #[allow(dead_code)]
    pub(crate) fn is_some(&self) -> bool {
        self.0.is_some()
    }
}

impl Drop for TaskRef {
    fn drop(&mut self) {
        if let Some(nn) = self.0.take() {
            unsafe {
                TaskCore::dec_ref(nn.as_ptr());
            }
        }
    }
}

// Send + Sync: TaskRef is just a raw pointer with ref-counted ownership
unsafe impl Send for TaskRef {}
unsafe impl Sync for TaskRef {}

/// Allocate a new task. Returns (RawTask for queue, TaskRef for JoinHandle).
/// ref_count starts at 2: one for queue slot, one for JoinHandle.
pub(crate) fn allocate_task<F>(future: F, scheduler: SchedulerHandle) -> (RawTask, TaskRef)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let id = gen_task_id();

    let task = Box::new(ConcreteTask {
        core: TaskCore {
            vtable: ConcreteTask::<F>::vtable(),
            id,
            state: AtomicU8::new(STATE_RUNNING),
            ref_count: AtomicUsize::new(2), // queue + JoinHandle
            scheduler,
        },
        future: UnsafeCell::new(MaybeUninit::new(future)),
        output: UnsafeCell::new(MaybeUninit::uninit()),
    });

    let raw: *mut ConcreteTask<F> = Box::into_raw(task);
    let core_ptr: *const TaskCore = raw as *const TaskCore;

    let raw_task = core_ptr as RawTask;
    let task_ref = TaskRef::new(core_ptr);

    (raw_task, task_ref)
}

/// Poll a raw task. Returns true if completed.
///
/// # Safety
///
/// `raw_task` must be valid. Caller must have exclusive access.
pub unsafe fn poll_raw_task(raw_task: RawTask) -> bool {
    let core = raw_task as *const TaskCore;
    (*core).poll()
}

/// Deallocate a completed task (consumes the queue slot ref).
///
/// # Safety
///
/// `raw_task` must be valid. Must only be called when the task just completed
/// (the scheduler's queue ref is being consumed).
pub unsafe fn deallocate_completed_task(raw_task: RawTask) {
    TaskCore::dec_ref(raw_task as *const TaskCore);
}

/// Read output from a completed task core.
///
/// # Safety
///
/// `core` must point to a completed task.
pub(crate) unsafe fn read_output<T>(core: &TaskCore) -> Option<T> {
    let mut output: MaybeUninit<T> = MaybeUninit::uninit();
    let ok = (core.vtable.take_output)(core, &mut output as *mut _ as *mut ());
    if ok { Some(output.assume_init()) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_future_completes() {
        let scheduler = SchedulerHandle::new_default();
        let (raw_task, task_ref) = allocate_task(async { 42 }, scheduler);

        let core = task_ref.core().unwrap();
        assert_eq!(core.state.load(Ordering::Acquire), STATE_RUNNING);
        assert!(!core.is_completed());

        unsafe {
            assert!(poll_raw_task(raw_task));
        }
        assert!(core.is_completed());

        unsafe {
            let output: i32 = read_output(core).unwrap();
            assert_eq!(output, 42);
        }

        // Consume queue ref
        unsafe {
            deallocate_completed_task(raw_task);
        }
        // TaskRef drop will consume JoinHandle ref
    }

    #[test]
    fn test_task_id_unique() {
        let scheduler = SchedulerHandle::new_default();
        let (_, ref1) = allocate_task(async {}, scheduler.clone());
        let (_, ref2) = allocate_task(async {}, scheduler);

        assert_ne!(ref1.core().unwrap().id(), ref2.core().unwrap().id());
    }
}
