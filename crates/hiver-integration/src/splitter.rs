//! Message splitting patterns
//! 消息拆分模式

use async_trait::async_trait;

use crate::{
    error::{IntegrationError, Result},
    message::Message,
};

/// Message splitter
/// 消息拆分器
#[async_trait]
pub trait MessageSplitter: Send + Sync {
    /// Split a message into multiple messages
    /// 将消息拆分为多个消息
    async fn split(&self, message: Message) -> Result<Vec<Message>>;
}

/// Iterator-based splitter
/// 基于迭代器的拆分器
pub struct IteratorSplitter<T>
where
    T: Send + Sync + 'static,
{
    _phantom: std::marker::PhantomData<T>,
}

impl<T> IteratorSplitter<T>
where
    T: Send + Sync + 'static,
{
    /// Create a new iterator splitter
    /// 创建新的迭代器拆分器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Default for IteratorSplitter<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> MessageSplitter for IteratorSplitter<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn split(&self, message: Message) -> Result<Vec<Message>> {
        let items = message
            .get_payload::<Vec<T>>()
            .ok_or_else(|| IntegrationError::Payload("Payload is not a Vec".to_string()))?;

        Ok(items
            .into_iter()
            .map(|item| Message::clone(&message).clone_with_payload(item))
            .collect())
    }
}

/// Function-based splitter
/// 基于函数的拆分器
pub struct FunctionSplitter<F>
where
    F: Fn(Message) -> Result<Vec<Message>> + Send + Sync,
{
    f: F,
}

impl<F> FunctionSplitter<F>
where
    F: Fn(Message) -> Result<Vec<Message>> + Send + Sync,
{
    /// Create a new function splitter
    /// 创建新的函数拆分器
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

#[async_trait]
impl<F> MessageSplitter for FunctionSplitter<F>
where
    F: Fn(Message) -> Result<Vec<Message>> + Send + Sync,
{
    async fn split(&self, message: Message) -> Result<Vec<Message>> {
        (self.f)(message)
    }
}

/// Async function-based splitter
/// 异步函数拆分器
pub struct AsyncFunctionSplitter<F, Fut>
where
    F: Fn(Message) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Vec<Message>>> + Send,
{
    f: F,
}

impl<F, Fut> AsyncFunctionSplitter<F, Fut>
where
    F: Fn(Message) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Vec<Message>>> + Send,
{
    /// Create a new async function splitter
    /// 创建新的异步函数拆分器
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

#[async_trait]
impl<F, Fut> MessageSplitter for AsyncFunctionSplitter<F, Fut>
where
    F: Fn(Message) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Vec<Message>>> + Send,
{
    async fn split(&self, message: Message) -> Result<Vec<Message>> {
        (self.f)(message).await
    }
}

/// Delimiter-based splitter for strings
/// 基于分隔符的字符串拆分器
pub struct DelimiterSplitter {
    delimiter: String,
    trim: bool,
}

impl DelimiterSplitter {
    /// Create a new delimiter splitter
    /// 创建新的分隔符拆分器
    pub fn new(delimiter: impl Into<String>) -> Self {
        Self {
            delimiter: delimiter.into(),
            trim: false,
        }
    }

    /// Enable trimming of split parts
    /// 启用分割部分的修剪
    pub fn with_trim(mut self) -> Self {
        self.trim = true;
        self
    }
}

#[async_trait]
impl MessageSplitter for DelimiterSplitter {
    async fn split(&self, message: Message) -> Result<Vec<Message>> {
        let text = message
            .get_payload::<String>()
            .ok_or_else(|| IntegrationError::Payload("Payload is not a String".to_string()))?;

        let parts: Vec<String> = if self.trim {
            text.split(&self.delimiter)
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            text.split(&self.delimiter).map(|s| s.to_string()).collect()
        };

        Ok(parts
            .into_iter()
            .filter(|s| !s.is_empty())
            .map(|part| Message::clone(&message).clone_with_payload(part))
            .collect())
    }
}

/// Size-based splitter - splits collections into chunks
/// 基于大小的拆分器 - 将集合拆分为块
pub struct SizeSplitter {
    chunk_size: usize,
}

impl SizeSplitter {
    /// Create a new size splitter
    /// 创建新的大小拆分器
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }
}

#[async_trait]
impl MessageSplitter for SizeSplitter {
    async fn split(&self, message: Message) -> Result<Vec<Message>> {
        let items = message
            .get_payload::<Vec<String>>()
            .ok_or_else(|| IntegrationError::Payload("Payload is not a Vec<String>".to_string()))?;

        Ok(items
            .chunks(self.chunk_size)
            .map(|chunk| Message::clone(&message).clone_with_payload(chunk.to_vec()))
            .collect())
    }
}

/// JSON array splitter
/// JSON 数组拆分器
pub struct JsonArraySplitter;

impl JsonArraySplitter {
    /// Create a new JSON array splitter
    /// 创建新的 JSON 数组拆分器
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonArraySplitter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageSplitter for JsonArraySplitter {
    async fn split(&self, message: Message) -> Result<Vec<Message>> {
        let json_str = message
            .get_payload::<String>()
            .ok_or_else(|| IntegrationError::Payload("Payload is not a String".to_string()))?;

        let value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| IntegrationError::Transformation(format!("JSON parse error: {}", e)))?;

        let array = value
            .as_array()
            .ok_or_else(|| IntegrationError::Transformation("Not a JSON array".to_string()))?;

        Ok(array
            .iter()
            .map(|item| {
                let json_string = serde_json::to_string(item).map_err(|e| {
                    IntegrationError::Transformation(format!("JSON serialize error: {}", e))
                })?;
                Ok::<_, IntegrationError>(Message::clone(&message).clone_with_payload(json_string))
            })
            .collect::<Result<Vec<_>>>()?)
    }
}

/// Line splitter - splits by newlines
/// 行拆分器 - 按换行符拆分
pub struct LineSplitter {
    skip_empty: bool,
}

impl LineSplitter {
    /// Create a new line splitter
    /// 创建新的行拆分器
    pub fn new() -> Self {
        Self { skip_empty: true }
    }

    /// Configure whether to skip empty lines
    /// 配置是否跳过空行
    pub fn skip_empty(mut self, skip: bool) -> Self {
        self.skip_empty = skip;
        self
    }
}

impl Default for LineSplitter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageSplitter for LineSplitter {
    async fn split(&self, message: Message) -> Result<Vec<Message>> {
        let text = message
            .get_payload::<String>()
            .ok_or_else(|| IntegrationError::Payload("Payload is not a String".to_string()))?;

        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();

        let filtered = if self.skip_empty {
            lines.into_iter().filter(|l| !l.is_empty()).collect()
        } else {
            lines
        };

        Ok(filtered
            .into_iter()
            .map(|line| Message::clone(&message).clone_with_payload(line))
            .collect())
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
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_iterator_splitter() {
        let splitter = IteratorSplitter::<i32>::new();

        let data = vec![1, 2, 3, 4, 5];
        let message = Message::new(data);

        let result = splitter.split(message).await.unwrap();

        assert_eq!(result.len(), 5);
        assert_eq!(result[0].get_payload::<i32>(), Some(1));
        assert_eq!(result[4].get_payload::<i32>(), Some(5));
    }

    #[tokio::test]
    async fn test_delimiter_splitter() {
        let splitter = DelimiterSplitter::new(",").with_trim();

        let message = Message::new("apple, banana, cherry, date".to_string());

        let result = splitter.split(message).await.unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].get_payload::<String>(), Some("apple".to_string()));
        assert_eq!(result[1].get_payload::<String>(), Some("banana".to_string()));
        assert_eq!(result[2].get_payload::<String>(), Some("cherry".to_string()));
        assert_eq!(result[3].get_payload::<String>(), Some("date".to_string()));
    }

    #[tokio::test]
    async fn test_size_splitter() {
        let splitter = SizeSplitter::new(2);

        let data = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let message = Message::new(data);

        let result = splitter.split(message).await.unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].get_payload::<Vec<String>>().unwrap().len(), 2);
        assert_eq!(result[1].get_payload::<Vec<String>>().unwrap().len(), 2);
        assert_eq!(result[2].get_payload::<Vec<String>>().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_json_array_splitter() {
        let splitter = JsonArraySplitter::new();

        let json = r#"["a", "b", "c"]"#;
        let message = Message::new(json.to_string());

        let result = splitter.split(message).await.unwrap();

        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_line_splitter() {
        let splitter = LineSplitter::new();

        let text = "line1\nline2\n\nline3\n\n\nline4";
        let message = Message::new(text.to_string());

        let result = splitter.split(message).await.unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].get_payload::<String>(), Some("line1".to_string()));
        assert_eq!(result[1].get_payload::<String>(), Some("line2".to_string()));
        assert_eq!(result[2].get_payload::<String>(), Some("line3".to_string()));
        assert_eq!(result[3].get_payload::<String>(), Some("line4".to_string()));
    }

    #[tokio::test]
    async fn test_function_splitter() {
        let splitter = FunctionSplitter::new(|msg| {
            let text = msg.get_payload::<String>().unwrap_or_default();
            Ok(text
                .chars()
                .map(|c| Message::clone(&msg).clone_with_payload(c.to_string()))
                .collect())
        });

        let message = Message::new("abc".to_string());
        let result = splitter.split(message).await.unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].get_payload::<String>(), Some("a".to_string()));
        assert_eq!(result[1].get_payload::<String>(), Some("b".to_string()));
        assert_eq!(result[2].get_payload::<String>(), Some("c".to_string()));
    }
}
