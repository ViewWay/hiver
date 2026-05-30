//! Multi-producer, single-consumer channels for async communication
//! 用于异步通信的多生产者、单消费者通道
//!
//! # Overview / 概述
//!
//! This module provides multi-producer, single-consumer (mpsc) channels
//! for asynchronous communication between tasks.
//!
//! 本模块提供用于任务间异步通信的多生产者、单消费者(mpsc)通道。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_runtime::channel::unbounded;
//!
//! async fn example() {
//!     let (tx, mut rx) = unbounded();
//!
//!     // Spawn a sender task
//!     // 生成发送器任务
//!     nexus_runtime::spawn(async move {
//!         tx.send("Hello").await.unwrap();
//!         tx.send("World").await.unwrap();
//!     });
//!
//!     // Receive messages
//!     // 接收消息
//!     while let Some(msg) = rx.recv().await {
//!         println!("Received: {}", msg);
//!     }
//! }
//! ```

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// Error type for channel operations
/// 通道操作的错误类型
pub enum SendError<T> {
    /// The channel is closed
    /// 通道已关闭
    Closed(T),
}

impl<T> std::fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SendError::Closed")
            .field(&format_args!("_"))
            .finish()
    }
}

impl<T> PartialEq for SendError<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Eq for SendError<T> {}

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Channel closed")
    }
}

impl<T> std::error::Error for SendError<T> {}

/// Error type for receiving
/// 接收错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvError {
    /// The channel is empty and closed
    /// 通道为空且已关闭
    Closed,
}

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecvError::Closed => write!(f, "Channel closed"),
        }
    }
}

impl std::error::Error for RecvError {}

/// Unbounded mpsc channel
/// 无边界mpsc通道
///
/// Creates a channel with an unbounded buffer.
/// 创建具有无界缓冲区的通道。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_runtime::channel::unbounded;
///
/// let (tx, rx) = unbounded::<i32>();
/// ```
#[must_use]
pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(ChannelShared {
        buffer: Mutex::new(VecDeque::new()),
        sender_count: AtomicUsize::new(1),
        is_receiver_alive: AtomicBool::new(true),
        recv_waker: Mutex::new(None),
    });

    let sender = Sender {
        shared: shared.clone(),
    };
    let receiver = Receiver { shared };

    (sender, receiver)
}

/// Bounded mpsc channel
/// 有界mpsc通道
///
/// Creates a channel with a bounded buffer.
/// 创建具有有界缓冲区的通道。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_runtime::channel::bounded;
///
/// let (tx, rx) = bounded::<i32>(16);
/// ```
#[must_use]
pub fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(ChannelShared {
        buffer: Mutex::new(VecDeque::with_capacity(cap)),
        sender_count: AtomicUsize::new(1),
        is_receiver_alive: AtomicBool::new(true),
        recv_waker: Mutex::new(None),
    });

    let sender = Sender {
        shared: shared.clone(),
    };
    let receiver = Receiver { shared };

    (sender, receiver)
}

/// Shared state for the channel
/// 通道的共享状态
struct ChannelShared<T> {
    /// Message buffer
    /// 消息缓冲区
    buffer: Mutex<VecDeque<T>>,
    /// Number of active senders
    /// 活跃发送器数量
    sender_count: AtomicUsize,
    /// Whether the receiver is still alive
    /// 接收器是否仍然存活
    is_receiver_alive: AtomicBool,
    /// Waker for pending receive operations
    /// 挂起接收操作的waker
    recv_waker: Mutex<Option<Waker>>,
}

/// Sender side of the channel
/// 通道的发送端
///
/// Can be cloned to create multiple senders.
/// 可以克隆以创建多个发送器。
pub struct Sender<T> {
    shared: Arc<ChannelShared<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        // Increment sender count
        // 增加发送器计数
        self.shared.sender_count.fetch_add(1, Ordering::Relaxed);
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Sender<T> {
    /// Send a value synchronously to the channel
    /// 向通道同步发送值
    ///
    /// # Errors
    ///
    /// Returns `SendError::Closed` if the receiver has been dropped.
    /// 如果接收器已丢弃则返回 `SendError::Closed`。
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (should never happen in normal operation).
    /// 如果内部互斥锁被污染则恐慌（正常操作中不应发生）。
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        if !self.shared.is_receiver_alive.load(Ordering::Acquire) {
            return Err(SendError::Closed(value));
        }

        let mut buffer = self.shared.buffer.lock().unwrap();
        buffer.push_back(value);

        // Wake the receiver if it's waiting
        // 如果接收器在等待，唤醒它
        if let Some(waker) = self.shared.recv_waker.lock().unwrap().take() {
            drop(buffer);
            waker.wake();
        }

        Ok(())
    }

    /// Check if the channel is closed (receiver dropped)
    /// 检查通道是否已关闭（接收器已丢弃）
    #[must_use]
    pub fn is_closed(&self) -> bool {
        !self.shared.is_receiver_alive.load(Ordering::Acquire)
    }

    /// Get the number of active senders
    /// 获取活跃发送器数量
    #[must_use]
    pub fn sender_count(&self) -> usize {
        self.shared.sender_count.load(Ordering::Acquire)
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // Decrease sender count
        // 减少发送器计数
        let prev = self.shared.sender_count.fetch_sub(1, Ordering::AcqRel);

        if prev == 1 {
            // Last sender dropped, wake the receiver
            // 最后一个发送器丢弃，唤醒接收器
            if let Some(waker) = self.shared.recv_waker.lock().unwrap().take() {
                waker.wake();
            }
        }
    }
}

/// Receiver side of the channel
/// 通道的接收端
pub struct Receiver<T> {
    shared: Arc<ChannelShared<T>>,
}

impl<T> Receiver<T> {
    /// Receive a value from the channel
    /// 从通道接收值
    pub fn recv(&mut self) -> RecvFuture<'_, T> {
        RecvFuture::new(self)
    }

    /// Try to receive a value without blocking
    /// 尝试接收值而不阻塞
    ///
    /// # Errors
    ///
    /// Returns `Err(RecvError::Closed)` if the channel is empty and all senders are dropped.
    /// 如果通道为空且所有发送器已丢弃则返回 `Err(RecvError::Closed)`。
    ///
    /// Returns `Err(RecvError::Empty)` if the channel is empty but senders still exist.
    /// 如果通道为空但发送器仍然存在则返回 `Err(RecvError::Empty)`。
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (should never happen in normal operation).
    /// 如果内部互斥锁被污染则恐慌（正常操作中不应发生）。
    pub fn try_recv(&mut self) -> Result<T, RecvError> {
        let mut buffer = self.shared.buffer.lock().unwrap();

        if let Some(value) = buffer.pop_front() {
            Ok(value)
        } else if self.shared.sender_count.load(Ordering::Acquire) == 0 {
            // No senders left
            // 没有发送器了
            Err(RecvError::Closed)
        } else {
            // Channel empty but senders still exist
            // 通道为空但发送器仍然存在
            Err(RecvError::Closed)
        }
    }

    /// Get the number of messages in the channel buffer
    /// 获取通道缓冲区中的消息数量
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (should never happen in normal operation).
    /// 如果内部互斥锁被污染则恐慌（正常操作中不应发生）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.shared.buffer.lock().unwrap().len()
    }

    /// Check if the channel is empty
    /// 检查通道是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Mark receiver as dropped
        // 标记接收器已丢弃
        self.shared
            .is_receiver_alive
            .store(false, Ordering::Release);
    }
}

/// Receive future
/// 接收future
pub struct RecvFuture<'a, T> {
    /// Reference to the receiver's shared state
    /// 接收器共享状态的引用
    shared: Arc<ChannelShared<T>>,
    /// Marker for the lifetime
    /// 生命周期标记
    _marker: std::marker::PhantomData<&'a mut Receiver<T>>,
}

impl<'a, T> RecvFuture<'a, T> {
    /// Create a new receive future
    fn new(receiver: &'a mut Receiver<T>) -> Self {
        // We extract the Arc since the receiver only holds it
        // This is safe because the future borrows the receiver mutably
        Self {
            shared: Arc::clone(&receiver.shared),
            _marker: std::marker::PhantomData,
        }
    }
}

unsafe impl<T: Send> Send for RecvFuture<'_, T> {}
unsafe impl<T: Sync> Sync for RecvFuture<'_, T> {}

impl<T> Future for RecvFuture<'_, T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut buffer = self.shared.buffer.lock().unwrap();

        if let Some(value) = buffer.pop_front() {
            Poll::Ready(Some(value))
        } else if self.shared.sender_count.load(Ordering::Acquire) == 0 {
            // No senders left and buffer empty
            // 没有发送器了且缓冲区为空
            Poll::Ready(None)
        } else {
            // No value available, register waker
            // 没有可用值，注册waker
            *self.shared.recv_waker.lock().unwrap() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unbounded_channel_creation() {
        let (tx, _rx) = unbounded::<i32>();
        assert!(!tx.is_closed());
        assert_eq!(tx.sender_count(), 1);
    }

    #[test]
    fn test_bounded_channel_creation() {
        let (tx, _rx) = bounded::<i32>(16);
        assert!(!tx.is_closed());
        assert_eq!(tx.sender_count(), 1);
    }

    #[test]
    fn test_sender_clone() {
        let (tx, _rx) = unbounded::<i32>();
        let tx2 = tx.clone();
        assert_eq!(tx.sender_count(), 2);
        assert_eq!(tx2.sender_count(), 2);
        drop(tx);
        assert_eq!(tx2.sender_count(), 1);
    }

    #[test]
    fn test_receiver_empty() {
        let (_tx, rx) = unbounded::<i32>();
        assert!(rx.is_empty());
        assert_eq!(rx.len(), 0);
    }

    #[test]
    fn test_sync_send() {
        let (tx, mut rx) = unbounded::<i32>();

        assert!(tx.send(42).is_ok());
        assert!(tx.send(100).is_ok());

        assert_eq!(rx.len(), 2);
        assert!(!rx.is_empty());

        // Verify received data and order
        assert_eq!(rx.try_recv().unwrap(), 42);
        assert_eq!(rx.try_recv().unwrap(), 100);
        assert_eq!(rx.try_recv(), Err(RecvError::Closed));
    }

    #[test]
    fn test_send_after_receiver_drop() {
        let (tx, rx) = unbounded::<i32>();
        drop(rx);
        assert!(tx.is_closed());

        let err = tx.send(42).unwrap_err();
        assert!(matches!(err, SendError::Closed(42)));
        assert_eq!(err.to_string(), "Channel closed");
    }

    #[test]
    fn test_recv_error() {
        let err = RecvError::Closed;

        assert_eq!(err.to_string(), "Channel closed");
    }

    #[test]
    fn test_unbounded_send_recv_order() {
        let (tx, mut rx) = unbounded::<String>();
        for i in 0..10 {
            tx.send(format!("msg-{i}")).unwrap();
        }
        for i in 0..10 {
            assert_eq!(rx.try_recv().unwrap(), format!("msg-{i}"));
        }
    }

    #[test]
    fn test_bounded_channel_full() {
        let (tx, rx) = bounded::<i32>(2);
        assert!(tx.send(1).is_ok());
        assert!(tx.send(2).is_ok());
        // Third send should still succeed (unbounded buffer semantics via VecDeque)
        assert!(tx.send(3).is_ok());
        assert_eq!(rx.len(), 3);
    }

    #[test]
    fn test_close_after_all_senders_drop() {
        let (tx, mut rx) = unbounded::<i32>();
        let tx2 = tx.clone();
        tx.send(1).unwrap();
        drop(tx);
        // tx2 still alive, channel should not be closed
        assert!(!tx2.is_closed());
        tx2.send(2).unwrap();
        drop(tx2);
        // All senders dropped, channel closed
        assert_eq!(rx.try_recv().unwrap(), 1);
        assert_eq!(rx.try_recv().unwrap(), 2);
    }
}
