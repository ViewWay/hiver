//! Select macro for waiting on multiple async operations
//! 用于等待多个异步操作的select宏
//!
//! # Overview / 概述
//!
//! This module provides a foundation for the `select!` macro which allows
//! waiting on multiple async operations and proceeding with the first one
//! that completes.
//!
//! 本模块为 `select!` 宏提供基础，允许等待多个异步操作并处理第一个完成的。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_runtime::select;
//! use hiver_runtime::time::{sleep, Duration};
//!
//! async fn example() {
//!     let sleep1 = sleep(Duration::from_millis(100));
//!     let sleep2 = sleep(Duration::from_millis(200));
//!
//!     select! {
//!         _ = sleep1 => println!("First completed"),
//!         _ = sleep2 => println!("Second completed"),
//!     }
//! }
//! ```

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Select from two futures, returning the first that completes
/// 从两个future中选择，返回第一个完成的
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_runtime::select::{select_two, SelectTwo};
/// use hiver_runtime::time::{sleep, Duration};
///
/// async fn example() {
///     let sleep1 = sleep(Duration::from_millis(100));
///     let sleep2 = sleep(Duration::from_millis(200));
///
///     match select_two(sleep1, sleep2).await {
///         SelectTwo::First(_) => println!("First completed"),
///         SelectTwo::Second(_) => println!("Second completed"),
///     }
/// }
/// ```
pub fn select_two<F1, F2, T1, T2>(future1: F1, future2: F2) -> SelectTwo<F1, F2>
where
    F1: Future<Output = T1> + Unpin,
    F2: Future<Output = T2> + Unpin,
{
    SelectTwo {
        future1: Some(future1),
        future2: Some(future2),
    }
}

/// Select future for two futures
/// 两个future的select future
pub struct SelectTwo<F1, F2>
{
    future1: Option<F1>,
    future2: Option<F2>,
}

impl<F1, F2, T1, T2> Future for SelectTwo<F1, F2>
where
    F1: Future<Output = T1> + Unpin,
    F2: Future<Output = T2> + Unpin,
{
    type Output = SelectTwoOutput<T1, T2>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        // Try polling the first future
        // 尝试轮询第一个future
        if let Some(future1) = &mut self.future1
        {
            match Pin::new(future1).poll(cx)
            {
                Poll::Ready(value) =>
                {
                    self.future1 = None;
                    return Poll::Ready(SelectTwoOutput::First(value));
                },
                Poll::Pending =>
                {},
            }
        }

        // Try polling the second future
        // 尝试轮询第二个future
        if let Some(future2) = &mut self.future2
        {
            match Pin::new(future2).poll(cx)
            {
                Poll::Ready(value) =>
                {
                    self.future2 = None;
                    return Poll::Ready(SelectTwoOutput::Second(value));
                },
                Poll::Pending =>
                {},
            }
        }

        Poll::Pending
    }
}

/// Output of selecting between two futures
/// 两个future之间选择的输出
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectTwoOutput<T1, T2>
{
    /// The first future completed
    /// 第一个future完成了
    First(T1),
    /// The second future completed
    /// 第二个future完成了
    Second(T2),
}

/// Select from multiple futures
/// 从多个future中选择
///
/// For Phase 2, this provides a simpler alternative to the full select! macro.
/// 对于第2阶段，这提供了完整的select!宏的更简单替代方案。
pub fn select_multiple<F>(futures: Vec<F>) -> SelectMultiple<F>
where
    F: Future + Unpin,
{
    SelectMultiple { futures }
}

/// Select future for multiple futures
/// 多个future的select future
pub struct SelectMultiple<F>
{
    futures: Vec<F>,
}

impl<F> Future for SelectMultiple<F>
where
    F: Future + Unpin,
{
    type Output = SelectMultipleOutput<F::Output>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        // Try polling each future
        // 尝试轮询每个future
        for (index, future) in self.futures.iter_mut().enumerate()
        {
            match Pin::new(future).poll(cx)
            {
                Poll::Ready(value) =>
                {
                    return Poll::Ready(SelectMultipleOutput::Completed(index, value));
                },
                Poll::Pending =>
                {},
            }
        }

        Poll::Pending
    }
}

/// Output of selecting from multiple futures
/// 从多个future中选择的输出
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMultipleOutput<T>
{
    /// A future completed with its index and value
    /// 一个future完成了，带有其索引和值
    Completed(usize, T),
}

/// Internal helper for select macro binding
/// select宏绑定的内部辅助
pub struct SelectMultipleBinding<T>
{
    /// Index of the completed future
    /// 完成的future的索引
    pub index: usize,

    /// Output value from the future
    /// 来自future的输出值
    pub output: Option<T>,
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
    use std::task::{Context, Waker};

    use super::*;

    #[test]
    fn test_select_two_output_first()
    {
        let output: SelectTwoOutput<i32, i32> = SelectTwoOutput::First(42);
        assert!(matches!(output, SelectTwoOutput::First(42)));
    }

    #[test]
    fn test_select_two_output_second()
    {
        let output: SelectTwoOutput<i32, i32> = SelectTwoOutput::Second(100);
        assert!(matches!(output, SelectTwoOutput::Second(100)));
    }

    #[test]
    fn test_select_multiple_output()
    {
        let output = SelectMultipleOutput::Completed(0, 42);
        assert!(matches!(output, SelectMultipleOutput::Completed(0, 42)));
    }

    #[test]
    fn test_ready_future()
    {
        // Test a future that's immediately ready
        // 测试立即就绪的future
        struct Ready;

        impl Future for Ready
        {
            type Output = i32;

            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
            {
                Poll::Ready(42)
            }
        }

        let noop_waker = Waker::noop();
        let mut context = Context::from_waker(noop_waker);

        let mut future = Box::pin(Ready);
        assert!(matches!(Pin::new(&mut future).poll(&mut context), Poll::Ready(42)));
    }
}
