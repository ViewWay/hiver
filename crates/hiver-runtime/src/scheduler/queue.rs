//! Local task queue for thread-per-core scheduler
//! thread-per-core调度器的本地任务队列

use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::RawTask;

/// Local queue for thread-per-core scheduler
/// thread-per-core调度器的本地队列
///
/// Uses a bounded ring buffer optimized for single consumer (the scheduler thread)
/// with support for external producers (work stealing injectors).
/// Uses interior mutability for thread-safe operations.
///
/// 使用为单个消费者（调度器线程）优化的有界环形缓冲区，
/// 支持外部生产者（工作窃取注入器）。
/// 使用内部可变性实现线程安全操作。
pub struct LocalQueue
{
    /// Ring buffer for task pointers / 任务指针的环形缓冲区
    buffer: Box<[UnsafeCell<MaybeUninit<RawTask>>]>,
    /// Queue capacity (must be power of 2) / 队列容量（必须是2的幂）
    capacity: usize,
    /// Capacity mask for fast modulo / 快速取模的容量掩码
    mask: usize,
    /// Head index (consumer) / 头索引（消费者）
    head: AtomicUsize,
    /// Tail index (producer) / 尾索引（生产者）
    tail: AtomicUsize,
}

// Safety: The queue uses atomic operations for thread safety
// and UnsafeCell for interior mutability
// 队列使用原子操作和UnsafeCell实现线程安全
unsafe impl Send for LocalQueue {}
unsafe impl Sync for LocalQueue {}

impl LocalQueue
{
    /// Create a new local queue with the specified capacity
    /// 创建具有指定容量的新本地队列
    ///
    /// The capacity will be rounded up to the next power of 2.
    /// 容量将向上舍入到下一个2的幂。
    #[must_use]
    pub fn new(capacity: usize) -> Self
    {
        let capacity = capacity.next_power_of_two().max(2);
        let mask = capacity - 1;

        // Initialize buffer with MaybeUninit (more efficient than Vec<Option>)
        // 使用MaybeUninit初始化缓冲区（比Vec<Option>更高效）
        let buffer = (0..capacity)
            .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
            .collect();

        Self {
            buffer,
            capacity,
            mask,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Push a task to the back of the queue
    /// 将任务推入队列后部
    ///
    /// Returns `true` if successful, `false` if the queue is full.
    /// 成功返回 `true`，队列已满返回 `false`。
    #[inline]
    pub fn push(&self, task: RawTask) -> bool
    {
        loop
        {
            let tail = self.tail.load(Ordering::Relaxed);
            let head = self.head.load(Ordering::Acquire);

            // Check if queue is full
            // 检查队列是否已满
            if tail - head >= self.capacity
            {
                return false;
            }

            let pos = tail & self.mask;
            // SAFETY: pos is within bounds and we have exclusive access to this slot
            // 通过环形缓冲区规则对此位置拥有独占访问权
            unsafe {
                self.buffer[pos].get().write(MaybeUninit::new(task));
            }

            if self
                .tail
                .compare_exchange(tail, tail + 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Pop a task from the front of the queue
    /// 从队列前部弹出一个任务
    ///
    /// Returns `Some(task)` if available, `None` if the queue is empty.
    /// 有可用任务时返回 `Some(task)`，队列为空时返回 `None`。
    #[inline]
    pub fn pop(&self) -> Option<RawTask>
    {
        loop
        {
            let head = self.head.load(Ordering::Relaxed);
            let tail = self.tail.load(Ordering::Acquire);

            if head == tail
            {
                return None;
            }

            let pos = head & self.mask;
            // SAFETY: pos is within bounds and we have exclusive access to this slot
            // The value was initialized by push, so assume_init is safe
            // 通过环形缓冲区规则对此位置拥有独占访问权
            // 该值由push初始化，因此assume_init是安全的
            let task = unsafe { self.buffer[pos].get().read().assume_init() };

            if let Ok(_) =
                self.head
                    .compare_exchange(head, head + 1, Ordering::AcqRel, Ordering::Relaxed)
            {
                return Some(task);
            }
            // Put the task back — another thread claimed this slot
            unsafe {
                self.buffer[pos].get().write(MaybeUninit::new(task));
            }
        }
    }

    /// Get the current length of the queue
    /// 获取队列当前长度
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize
    {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Relaxed);
        tail.saturating_sub(head)
    }

    /// Check if the queue is empty
    /// 检查队列是否为空
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool
    {
        self.len() == 0
    }

    /// Get the queue capacity
    /// 获取队列容量
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize
    {
        self.capacity
    }

    /// Steal half of the tasks from this queue
    /// 从此队列窃取一半任务
    ///
    /// Used for work stealing between scheduler threads.
    /// 用于调度器线程间的工作窃取。
    ///
    /// Returns the number of tasks stolen.
    /// 返回被窃取的任务数量。
    pub fn steal(&self, dest: &LocalQueue) -> usize
    {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        let len = tail.saturating_sub(head);
        if len == 0
        {
            return 0;
        }

        // Steal half (rounding down)
        // 窃取一半（向下取整）
        let steal_count = len / 2;
        if steal_count == 0
        {
            return 0;
        }

        let mut stolen = 0;
        for i in 0..steal_count
        {
            let pos = (head + i) & self.mask;
            // SAFETY: pos is within bounds and value was initialized by push
            // SAFETY: pos在范围内且值由push初始化
            let task = unsafe { self.buffer[pos].get().read().assume_init() };

            if dest.push(task)
            {
                stolen += 1;
                // Update head to reflect stolen tasks
                // 更新head以反映被窃取的任务
                self.head.store(head + i + 1, Ordering::Release);
            }
            else
            {
                // Destination full, put back remaining
                // 目标已满，放回剩余任务
                break;
            }
        }

        stolen
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_queue_push_pop()
    {
        let queue = LocalQueue::new(16);

        let task1 = 0x1000 as RawTask;
        let task2 = 0x2000 as RawTask;

        assert!(queue.push(task1));
        assert!(queue.push(task2));

        assert_eq!(queue.pop(), Some(task1));
        assert_eq!(queue.pop(), Some(task2));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_queue_empty_full()
    {
        let queue = LocalQueue::new(4);

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        // Fill the queue
        // 填满队列
        for i in 0..4
        {
            assert!(queue.push(i as RawTask));
        }

        // Queue should be full now
        // 队列现在应该满了
        assert!(!queue.push(99 as RawTask));
        assert_eq!(queue.len(), 4);

        // Empty the queue
        // 清空队列
        for i in 0..4
        {
            assert_eq!(queue.pop(), Some(i as RawTask));
        }

        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_wrap_around()
    {
        let queue = LocalQueue::new(4);

        // Fill and drain multiple times to test wrap-around
        // 多次填充和排空以测试包装
        for round in 0..3
        {
            for i in 0..4
            {
                assert!(queue.push((round * 4 + i) as RawTask));
            }

            for i in 0..4
            {
                assert_eq!(queue.pop(), Some((round * 4 + i) as RawTask));
            }
        }
    }

    #[test]
    fn test_queue_capacity_power_of_two()
    {
        // Capacity should be rounded to next power of 2
        // 容量应向上舍入到下一个2的幂
        let q = LocalQueue::new(5);
        assert_eq!(q.capacity(), 8);

        let q = LocalQueue::new(100);
        assert_eq!(q.capacity(), 128);
    }
}
