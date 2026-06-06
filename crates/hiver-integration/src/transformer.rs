//! Message transformation patterns
//! 消息转换模式

use std::sync::Arc;

use crate::{
    error::{IntegrationError, Result},
    message::Message,
};

/// Transformer for converting messages
/// 消息转换器
#[async_trait::async_trait]
pub trait Transformer: Send + Sync
{
    /// Transform the message
    /// 转换消息
    async fn transform(&self, message: Message) -> Result<Message>;
}

/// Generic transformer with typed input and output
/// 通用转换器带有类型化输入和输出
#[async_trait::async_trait]
pub trait GenericTransformer<I, O>: Send + Sync
where
    I: Send + 'static,
    O: Send + 'static,
{
    /// Transform input to output
    /// 转换输入为输出
    async fn transform_generic(&self, input: I) -> Result<O>;
}

/// Function-based transformer
/// 基于函数的转换器
pub struct FunctionTransformer<F>
where
    F: Fn(Message) -> Result<Message> + Send + Sync,
{
    f: F,
}

impl<F> FunctionTransformer<F>
where
    F: Fn(Message) -> Result<Message> + Send + Sync,
{
    /// Create a new function transformer
    /// 创建新的函数转换器
    pub fn new(f: F) -> Self
    {
        Self { f }
    }
}

#[async_trait::async_trait]
impl<F> Transformer for FunctionTransformer<F>
where
    F: Fn(Message) -> Result<Message> + Send + Sync,
{
    async fn transform(&self, message: Message) -> Result<Message>
    {
        // For sync functions, just call directly
        // 对于同步函数，直接调用
        (self.f)(message)
    }
}

/// Async function-based transformer
/// 异步函数转换器
pub struct AsyncFunctionTransformer<F, Fut>
where
    F: Fn(Message) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Message>> + Send,
{
    f: F,
}

impl<F, Fut> AsyncFunctionTransformer<F, Fut>
where
    F: Fn(Message) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Message>> + Send,
{
    /// Create a new async function transformer
    /// 创建新的异步函数转换器
    pub fn new(f: F) -> Self
    {
        Self { f }
    }
}

#[async_trait::async_trait]
impl<F, Fut> Transformer for AsyncFunctionTransformer<F, Fut>
where
    F: Fn(Message) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Message>> + Send,
{
    async fn transform(&self, message: Message) -> Result<Message>
    {
        (self.f)(message).await
    }
}

/// Generic function transformer
/// 通用函数转换器
pub struct GenericFunctionTransformer<I, O, F>
where
    I: Send + 'static,
    O: Send + 'static,
    F: Fn(I) -> Result<O> + Send + Sync,
{
    f: F,
    _phantom: std::marker::PhantomData<fn(I) -> O>,
}

impl<I, O, F> GenericFunctionTransformer<I, O, F>
where
    I: Send + 'static,
    O: Send + 'static,
    F: Fn(I) -> Result<O> + Send + Sync,
{
    /// Create a new generic function transformer
    /// 创建新的通用函数转换器
    pub fn new(f: F) -> Self
    {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<I, O, F> GenericTransformer<I, O> for GenericFunctionTransformer<I, O, F>
where
    I: Send + 'static,
    O: Send + 'static,
    F: Fn(I) -> Result<O> + Send + Sync,
{
    async fn transform_generic(&self, input: I) -> Result<O>
    {
        (self.f)(input)
    }
}

/// Header transformer - adds/removes/ modifies headers
/// 头部转换器 - 添加/删除/修改头部
#[derive(Clone)]
pub struct HeaderTransformer
{
    /// Headers to add
    /// 要添加的头部
    add: std::collections::HashMap<String, String>,

    /// Headers to remove
    /// 要删除的头部
    remove: Vec<String>,
}

impl HeaderTransformer
{
    /// Create a new header transformer
    /// 创建新的头部转换器
    pub fn new() -> Self
    {
        Self {
            add: std::collections::HashMap::new(),
            remove: Vec::new(),
        }
    }

    /// Add a header to be set
    /// 添加要设置的头部
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.add.insert(key.into(), value.into());
        self
    }

    /// Add a header to be removed
    /// 添加要删除的头部
    pub fn remove_header(mut self, key: impl Into<String>) -> Self
    {
        self.remove.push(key.into());
        self
    }
}

impl Default for HeaderTransformer
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Transformer for HeaderTransformer
{
    async fn transform(&self, mut message: Message) -> Result<Message>
    {
        // Add headers
        // 添加头部
        for (key, value) in &self.add
        {
            message.set_header(key, value.as_str());
        }

        // Remove headers
        // 删除头部
        for key in &self.remove
        {
            message.headers_mut().remove(key);
        }

        Ok(message)
    }
}

/// Payload transformer - transforms the payload using a function
/// 载荷转换器 - 使用函数转换载荷
pub struct PayloadTransformer<I, O, F>
where
    I: Send + 'static,
    O: Send + 'static,
    F: Fn(I) -> Result<O> + Send + Sync,
{
    f: F,
    _phantom: std::marker::PhantomData<fn(I) -> O>,
}

impl<I, O, F> PayloadTransformer<I, O, F>
where
    I: Send + 'static,
    O: Send + 'static,
    F: Fn(I) -> Result<O> + Send + Sync,
{
    /// Create a new payload transformer
    /// 创建新的载荷转换器
    pub fn new(f: F) -> Self
    {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<I, O, F> Transformer for PayloadTransformer<I, O, F>
where
    I: Clone + Send + 'static,
    O: Send + Sync + Clone + 'static,
    F: Fn(I) -> Result<O> + Send + Sync,
{
    async fn transform(&self, message: Message) -> Result<Message>
    {
        let input = message.get_payload::<I>().ok_or_else(|| {
            IntegrationError::Transformation(format!(
                "Payload is not of type {}",
                std::any::type_name::<I>()
            ))
        })?;

        let output = (self.f)(input)?;
        Ok(message.clone_with_payload(output))
    }
}

/// Content type transformer - transforms based on content type
/// 内容类型转换器 - 基于内容类型转换
pub struct ContentTypeTransformer
{
    transformers: std::collections::HashMap<String, Arc<dyn Transformer>>,
}

impl ContentTypeTransformer
{
    /// Create a new content type transformer
    /// 创建新的内容类型转换器
    pub fn new() -> Self
    {
        Self {
            transformers: std::collections::HashMap::new(),
        }
    }

    /// Register a transformer for a content type
    /// 为内容类型注册转换器
    pub fn register(&mut self, content_type: impl Into<String>, transformer: Arc<dyn Transformer>)
    {
        self.transformers.insert(content_type.into(), transformer);
    }

    /// Get default content type from message
    /// 从消息获取默认内容类型
    fn get_content_type(&self, message: &Message) -> Option<String>
    {
        message
            .header("content_type")
            .and_then(|h| h.as_str().map(|s| s.to_string()))
    }
}

impl Default for ContentTypeTransformer
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Transformer for ContentTypeTransformer
{
    async fn transform(&self, message: Message) -> Result<Message>
    {
        let content_type = self.get_content_type(&message).ok_or_else(|| {
            IntegrationError::Transformation("No content-type header".to_string())
        })?;

        let transformer = self.transformers.get(&content_type).ok_or_else(|| {
            IntegrationError::Transformation(format!(
                "No transformer for content-type: {}",
                content_type
            ))
        })?;

        transformer.transform(message).await
    }
}

/// Chain transformer - applies transformers in sequence
/// 链式转换器 - 依次应用转换器
pub struct ChainTransformer
{
    transformers: Vec<Arc<dyn Transformer>>,
}

impl ChainTransformer
{
    /// Create a new chain transformer
    /// 创建新的链式转换器
    pub fn new() -> Self
    {
        Self {
            transformers: Vec::new(),
        }
    }

    /// Add a transformer to the chain
    /// 添加转换器到链
    pub fn add(mut self, transformer: Arc<dyn Transformer>) -> Self
    {
        self.transformers.push(transformer);
        self
    }
}

impl Default for ChainTransformer
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Transformer for ChainTransformer
{
    async fn transform(&self, message: Message) -> Result<Message>
    {
        let mut current = message;
        for transformer in &self.transformers
        {
            current = transformer.transform(current).await?;
        }
        Ok(current)
    }
}

/// JSON transformer - serializes/deserializes JSON payloads
/// JSON 转换器 - 序列化/反序列化 JSON 载荷
pub struct JsonTransformer;

impl JsonTransformer
{
    /// Create a new JSON transformer
    /// 创建新的 JSON 转换器
    pub fn new() -> Self
    {
        Self
    }

    /// Transform JSON string to typed value
    /// 将 JSON 字符串转换为类型化值
    pub fn from_json<T: serde::de::DeserializeOwned + Send + Sync + Clone + 'static>(
        message: Message,
    ) -> Result<Message>
    {
        let json_str = message.get_payload::<String>().ok_or_else(|| {
            IntegrationError::Transformation("Payload is not a string".to_string())
        })?;

        let value: T = serde_json::from_str(&json_str)
            .map_err(|e| IntegrationError::Transformation(format!("JSON parse error: {}", e)))?;

        Ok(message.clone_with_payload(value))
    }

    /// Transform typed value to JSON string
    /// 将类型化值转换为 JSON 字符串
    pub fn to_json<T: serde::Serialize + Send + Sync + Clone + 'static>(
        message: Message,
    ) -> Result<Message>
    {
        let value = message.get_payload::<T>().ok_or_else(|| {
            IntegrationError::Transformation(format!(
                "Payload is not of type {}",
                std::any::type_name::<T>()
            ))
        })?;

        let json_str = serde_json::to_string(&value).map_err(|e| {
            IntegrationError::Transformation(format!("JSON serialize error: {}", e))
        })?;

        Ok(message.clone_with_payload(json_str))
    }
}

impl Default for JsonTransformer
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Transformer for JsonTransformer
{
    async fn transform(&self, message: Message) -> Result<Message>
    {
        // Default implementation just passes through
        // 默认实现只是传递
        Ok(message)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_function_transformer()
    {
        let transformer = FunctionTransformer::new(|msg| {
            let payload = msg.get_payload::<i32>().unwrap_or(0);
            Ok(msg.clone_with_payload(payload * 2))
        });

        let msg = Message::new(21);
        let result = transformer.transform(msg).await.unwrap();

        assert_eq!(result.get_payload::<i32>(), Some(42));
    }

    #[tokio::test]
    async fn test_header_transformer()
    {
        let transformer = HeaderTransformer::new()
            .add_header("new_header", "new_value")
            .remove_header("old_header");

        let mut msg = Message::new("test");
        msg.set_header("old_header", "old_value");
        msg.set_header("keep_header", "keep_value");

        let result = transformer.transform(msg).await.unwrap();

        assert_eq!(result.header("new_header").and_then(|h| h.as_str()), Some("new_value"));
        assert!(result.header("old_header").is_none());
        assert_eq!(result.header("keep_header").and_then(|h| h.as_str()), Some("keep_value"));
    }

    #[tokio::test]
    async fn test_chain_transformer()
    {
        let transformer = ChainTransformer::new()
            .add(Arc::new(HeaderTransformer::new().add_header("step1", "done")))
            .add(Arc::new(HeaderTransformer::new().add_header("step2", "done")));

        let msg = Message::new("test");
        let result = transformer.transform(msg).await.unwrap();

        assert_eq!(result.header("step1").and_then(|h| h.as_str()), Some("done"));
        assert_eq!(result.header("step2").and_then(|h| h.as_str()), Some("done"));
    }

    #[tokio::test]
    async fn test_payload_transformer()
    {
        let transformer = PayloadTransformer::new(|s: String| Ok(s.to_uppercase()));

        let msg = Message::new("hello".to_string());
        let result = transformer.transform(msg).await.unwrap();

        assert_eq!(result.get_payload::<String>(), Some("HELLO".to_string()));
    }

    #[tokio::test]
    async fn test_json_transformer()
    {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
        struct TestData
        {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let msg = Message::new(data.clone());
        let json_msg = JsonTransformer::to_json::<TestData>(msg).unwrap();

        let json_str = json_msg.get_payload::<String>().unwrap();
        assert!(json_str.contains("\"name\":\"test\""));

        let result = JsonTransformer::from_json::<TestData>(json_msg).unwrap();
        assert_eq!(result.get_payload::<TestData>(), Some(data));
    }
}
