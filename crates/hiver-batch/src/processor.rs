//! Item processor for batch processing
//! 批处理项目处理器

use crate::error::{BatchError, BatchResult};
use async_trait::async_trait;

/// Item processor trait
/// 项目处理器trait
///
/// Transforms items between reading and writing.
/// 在读取和写入之间转换项目。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface ItemProcessor<I, O> {
///     O process(I item) throws Exception;
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_batch::prelude::*;
/// use async_trait::async_trait;
///
/// struct UpperCaseProcessor;
///
/// #[async_trait]
/// impl ItemProcessor for UpperCaseProcessor {
///     type Input = String;
///     type Output = String;
///
///     async fn process(&self, item: String) -> BatchResult<Option<String>> {
///         Ok(Some(item.to_uppercase()))
///     }
/// }
/// ```
///
/// Returning `None` from process filters the item (it won't be written).
/// 从 process 返回 `None` 会过滤该项目（它不会被写入）。
#[async_trait]
pub trait ItemProcessor: Send + Sync {
    /// Input item type
    /// 输入项目类型
    type Input: Send + Sync;

    /// Output item type
    /// 输出项目类型
    type Output: Send + Sync;

    /// Process an item
    /// 处理项目
    ///
    /// Returns `None` to filter the item.
    /// 返回 `None` 以过滤该项目。
    async fn process(&self, item: Self::Input) -> BatchResult<Option<Self::Output>>;

    /// Called before processing starts
    /// 处理开始前调用
    async fn before_process(&self) -> BatchResult<()> {
        Ok(())
    }

    /// Called after processing completes
    /// 处理完成后调用
    async fn after_process(&self) -> BatchResult<()> {
        Ok(())
    }
}

/// Pass-through processor
/// 直通处理器
///
/// Passes items through without modification.
/// 无修改传递项目。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // No processor defined means pass-through
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PassThroughProcessor<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for PassThroughProcessor<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> PassThroughProcessor<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> ItemProcessor for PassThroughProcessor<T>
where
    T: Send + Sync,
{
    type Input = T;
    type Output = T;

    async fn process(&self, item: T) -> BatchResult<Option<T>> {
        Ok(Some(item))
    }
}

/// Function-based processor
/// 基于函数的处理器
///
/// Creates a processor from a function.
/// 从函数创建处理器。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_batch::prelude::*;
///
/// let processor = FunctionProcessor::new(|item: i32| async {
///     Ok(Some(item * 2))
/// });
/// ```
pub struct FunctionProcessor<I, O, F>
where
    F: Fn(I) -> BatchResult<Option<O>> + Send + Sync,
{
    func: F,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O, F> FunctionProcessor<I, O, F>
where
    F: Fn(I) -> BatchResult<Option<O>> + Send + Sync,
{
    /// Create new function processor
    /// 创建新函数处理器
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I, O, F> Clone for FunctionProcessor<I, O, F>
where
    F: Fn(I) -> BatchResult<Option<O>> + Send + Sync + Clone,
{
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<I, O, F> ItemProcessor for FunctionProcessor<I, O, F>
where
    I: Send + Sync,
    O: Send + Sync,
    F: Fn(I) -> BatchResult<Option<O>> + Send + Sync,
{
    type Input = I;
    type Output = O;

    async fn process(&self, item: I) -> BatchResult<Option<O>> {
        (self.func)(item)
    }
}

/// Filter processor
/// 过滤处理器
///
/// Filters items based on a predicate.
/// 基于谓词过滤项目。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_batch::prelude::*;
///
/// // Only process positive numbers
/// let processor = FilterProcessor::new(|item: &i32| *item > 0);
/// ```
pub struct FilterProcessor<T, F>
where
    F: Fn(&T) -> bool + Send + Sync,
{
    predicate: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> FilterProcessor<T, F>
where
    F: Fn(&T) -> bool + Send + Sync,
{
    /// Create new filter processor
    /// 创建新过滤处理器
    pub fn new(predicate: F) -> Self {
        Self {
            predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> Clone for FilterProcessor<T, F>
where
    F: Fn(&T) -> bool + Send + Sync + Clone,
{
    fn clone(&self) -> Self {
        Self {
            predicate: self.predicate.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> ItemProcessor for FilterProcessor<T, Box<dyn Fn(&T) -> bool + Send + Sync>>
where
    T: Send + Sync + Clone,
{
    type Input = T;
    type Output = T;

    async fn process(&self, item: T) -> BatchResult<Option<T>> {
        if (self.predicate)(&item) {
            Ok(Some(item))
        } else {
            Ok(None) // Filter out
        }
    }
}

/// Map processor
/// 映射处理器
///
/// Transforms items using a function.
/// 使用函数转换项目。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_batch::prelude::*;
///
/// // Double each number
/// let processor = MapProcessor::new(|item: i32| item * 2);
/// ```
pub struct MapProcessor<T, U, F>
where
    F: Fn(T) -> U + Send + Sync,
{
    func: F,
    _phantom: std::marker::PhantomData<(T, U)>,
}

impl<T, U, F> MapProcessor<T, U, F>
where
    F: Fn(T) -> U + Send + Sync,
{
    /// Create new map processor
    /// 创建新映射处理器
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, U, F> Clone for MapProcessor<T, U, F>
where
    F: Fn(T) -> U + Send + Sync + Clone,
{
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T, U, F> ItemProcessor for MapProcessor<T, U, F>
where
    T: Send + Sync,
    U: Send + Sync,
    F: Fn(T) -> U + Send + Sync,
{
    type Input = T;
    type Output = U;

    async fn process(&self, item: T) -> BatchResult<Option<U>> {
        Ok(Some((self.func)(item)))
    }
}

/// Validating processor
/// 验证处理器
///
/// Validates items and returns errors for invalid ones.
/// 验证项目并对无效项目返回错误。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_batch::prelude::*;
///
/// let processor = ValidatingProcessor::new(|item: &String| {
///     if item.is_empty() {
///         Err("String cannot be empty".to_string())
///     } else {
///         Ok(())
///     }
/// });
/// ```
pub struct ValidatingProcessor<T, F>
where
    F: Fn(&T) -> Result<(), String> + Send + Sync,
{
    validator: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> ValidatingProcessor<T, F>
where
    F: Fn(&T) -> Result<(), String> + Send + Sync,
{
    /// Create new validating processor
    /// 创建新验证处理器
    pub fn new(validator: F) -> Self {
        Self {
            validator,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> Clone for ValidatingProcessor<T, F>
where
    F: Fn(&T) -> Result<(), String> + Send + Sync + Clone,
{
    fn clone(&self) -> Self {
        Self {
            validator: self.validator.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T, F> ItemProcessor for ValidatingProcessor<T, F>
where
    T: Send + Sync + Clone,
    F: Fn(&T) -> Result<(), String> + Send + Sync,
{
    type Input = T;
    type Output = T;

    async fn process(&self, item: T) -> BatchResult<Option<T>> {
        (self.validator)(&item).map_err(|msg| BatchError::ValidationError {
            message: msg,
        })?;
        Ok(Some(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pass_through_processor() {
        let processor = PassThroughProcessor::<i32>::new();

        assert_eq!(processor.process(42).await.unwrap(), Some(42));
        assert_eq!(processor.process(0).await.unwrap(), Some(0));
    }

    #[tokio::test]
    async fn test_pass_through_processor_str() {
        let processor = PassThroughProcessor::<&str>::new();

        assert_eq!(processor.process("hello").await.unwrap(), Some("hello"));
        assert_eq!(processor.process("world").await.unwrap(), Some("world"));
    }

    #[tokio::test]
    async fn test_function_processor() {
        let processor = FunctionProcessor::new(|item: i32| {
            Ok(Some(item * 2))
        });

        assert_eq!(processor.process(5).await.unwrap(), Some(10));
        assert_eq!(processor.process(-3).await.unwrap(), Some(-6));
    }

    #[tokio::test]
    async fn test_map_processor() {
        let processor = MapProcessor::new(|item: i32| item * 2);

        assert_eq!(processor.process(5).await.unwrap(), Some(10));
        assert_eq!(processor.process(0).await.unwrap(), Some(0));
    }

    #[tokio::test]
    async fn test_map_processor_string() {
        let processor = MapProcessor::new(|item: String| item.to_uppercase());

        assert_eq!(
            processor.process("hello".to_string()).await.unwrap(),
            Some("HELLO".to_string())
        );
    }

    #[tokio::test]
    async fn test_validating_processor() {
        let validator = |item: &i32| {
            if *item > 0 {
                Ok(())
            } else {
                Err("Must be positive".to_string())
            }
        };

        let processor = ValidatingProcessor::new(validator);

        assert!(processor.process(5).await.is_ok());
        assert!(processor.process(0).await.is_err());
        assert!(processor.process(-1).await.is_err());
    }
}
