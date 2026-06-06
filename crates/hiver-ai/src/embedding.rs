//! Embedding model abstractions for converting text to vector representations.
//! 用于将文本转换为向量表示的嵌入模型抽象。
//!
//! This module defines the core types for text embedding operations,
//! enabling semantic search, similarity comparison, and vector-based
//! retrieval augmented generation (RAG).
//!
//! 本模块定义了文本嵌入操作的核心类型，支持语义搜索、相似性比较
//! 和基于向量的检索增强生成 (RAG)。

use serde::{Deserialize, Serialize};

use crate::chat_model::{ModelError, TokenUsage};

/// A request to generate embeddings for text.
/// 生成文本嵌入的请求。
#[derive(Debug, Clone)]
pub struct EmbeddingRequest
{
    /// The input texts to embed.
    /// 要嵌入的输入文本。
    pub inputs: Vec<String>,
    /// The model identifier to use.
    /// 使用的模型标识符。
    pub model: Option<String>,
}

impl EmbeddingRequest
{
    /// Creates a new embedding request for a single text.
    /// 为单个文本创建新的嵌入请求。
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self
    {
        Self {
            inputs: vec![text.into()],
            model: None,
        }
    }

    /// Creates a new embedding request for multiple texts.
    /// 为多个文本创建新的嵌入请求。
    #[must_use]
    pub fn batch(inputs: Vec<String>) -> Self
    {
        Self {
            inputs,
            model: None,
        }
    }

    /// Sets the model identifier.
    /// 设置模型标识符。
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self
    {
        self.model = Some(model.into());
        self
    }
}

/// A response from an embedding model.
/// 嵌入模型的响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse
{
    /// The generated embedding vectors, one per input text.
    /// 生成的嵌入向量，每个输入文本一个。
    pub embeddings: Vec<Vec<f32>>,
    /// The model that generated these embeddings.
    /// 生成这些嵌入的模型。
    pub model: String,
    /// Token usage statistics.
    /// Token 使用统计。
    pub usage: TokenUsage,
}

impl EmbeddingResponse
{
    /// Creates a new embedding response.
    /// 创建新的嵌入响应。
    #[must_use]
    pub fn new(embeddings: Vec<Vec<f32>>, model: impl Into<String>) -> Self
    {
        Self {
            embeddings,
            model: model.into(),
            usage: TokenUsage::default(),
        }
    }

    /// Sets the token usage for this response.
    /// 设置此响应的 token 使用统计。
    #[must_use]
    pub fn usage(mut self, usage: TokenUsage) -> Self
    {
        self.usage = usage;
        self
    }

    /// Returns the number of embeddings in this response.
    /// 返回此响应中的嵌入数量。
    #[must_use]
    pub fn len(&self) -> usize
    {
        self.embeddings.len()
    }

    /// Returns true if there are no embeddings.
    /// 如果没有嵌入则返回 true。
    #[must_use]
    pub fn is_empty(&self) -> bool
    {
        self.embeddings.is_empty()
    }

    /// Returns the first embedding (for single-input requests).
    /// 返回第一个嵌入（用于单输入请求）。
    #[must_use]
    pub fn first(&self) -> Option<&Vec<f32>>
    {
        self.embeddings.first()
    }
}

/// Trait for embedding model implementations.
/// 嵌入模型实现的 trait。
///
/// This trait defines the interface for generating vector embeddings
/// from text, supporting various backends (OpenAI, local models, etc.).
///
/// 此 trait 定义了从文本生成向量嵌入的接口，支持各种后端。
#[async_trait::async_trait]
pub trait EmbeddingModel: Send + Sync
{
    /// Generates embeddings for the given texts.
    /// 为给定文本生成嵌入。
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, ModelError>;

    /// Generates an embedding for a single text, returning the vector directly.
    /// 为单个文本生成嵌入，直接返回向量。
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, ModelError>
    {
        let request = EmbeddingRequest::new(text);
        let response = self.embed(request).await?;
        response
            .first()
            .cloned()
            .ok_or_else(|| ModelError::ParseError("No embedding returned".to_string()))
    }
}

/// Computes the cosine similarity between two vectors.
/// 计算两个向量之间的余弦相似度。
///
/// Returns a value between -1.0 and 1.0, where 1.0 means identical
/// direction and 0.0 means orthogonal.
///
/// 返回 -1.0 到 1.0 之间的值，1.0 表示方向相同，0.0 表示正交。
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32
{
    if a.len() != b.len() || a.is_empty()
    {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0
    {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// Computes the Euclidean distance between two vectors.
/// 计算两个向量之间的欧几里得距离。
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32
{
    if a.len() != b.len()
    {
        return f32::MAX;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Normalizes a vector in place to unit length (L2 norm = 1.0).
/// 原地向量化为单位长度（L2 范数 = 1.0）。
///
/// If the vector has zero magnitude, it is left unchanged.
/// 如果向量为零向量，则保持不变。
///
/// # Example / 示例
/// ```rust
/// use hiver_ai::embedding::normalize;
/// let mut v = vec![3.0, 4.0];
/// normalize(&mut v);
/// assert!((v[0] - 0.6).abs() < 1e-6);
/// assert!((v[1] - 0.8).abs() < 1e-6);
/// ```
pub fn normalize(vec: &mut [f32])
{
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0
    {
        for x in vec.iter_mut()
        {
            *x /= norm;
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests
{
    use super::*;

    #[test]
    fn test_embedding_request_new()
    {
        let req = EmbeddingRequest::new("hello");
        assert_eq!(req.inputs.len(), 1);
        assert_eq!(req.inputs[0], "hello");
        assert!(req.model.is_none());
    }

    #[test]
    fn test_embedding_request_batch()
    {
        let req = EmbeddingRequest::batch(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(req.inputs.len(), 2);
    }

    #[test]
    fn test_embedding_request_model()
    {
        let req = EmbeddingRequest::new("test").model("text-embedding-3-small");
        assert_eq!(req.model.as_deref(), Some("text-embedding-3-small"));
    }

    #[test]
    fn test_embedding_response()
    {
        let resp =
            EmbeddingResponse::new(vec![vec![0.1, 0.2, 0.3], vec![0.4, 0.5, 0.6]], "embed-model")
                .usage(TokenUsage::new(10, 0));

        assert_eq!(resp.len(), 2);
        assert!(!resp.is_empty());
        assert_eq!(resp.model, "embed-model");
        assert_eq!(resp.usage.prompt_tokens, 10);
        assert_eq!(resp.first().unwrap().len(), 3);
    }

    #[test]
    fn test_embedding_response_empty()
    {
        let resp = EmbeddingResponse::new(vec![], "model");
        assert!(resp.is_empty());
        assert!(resp.first().is_none());
    }

    #[test]
    fn test_cosine_similarity_identical()
    {
        let vec = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&vec, &vec);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal()
    {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite()
    {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_empty()
    {
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }

    #[test]
    fn test_cosine_similarity_mismatched()
    {
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn test_euclidean_distance_same()
    {
        let vec = vec![1.0, 2.0, 3.0];
        let dist = euclidean_distance(&vec, &vec);
        assert!(dist.abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance_different()
    {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance_mismatched()
    {
        assert_eq!(euclidean_distance(&[1.0], &[1.0, 2.0]), f32::MAX);
    }

    #[test]
    fn test_normalize()
    {
        let mut v = vec![3.0, 4.0];
        normalize(&mut v);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
        // Check unit length
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero_vector()
    {
        let mut v = vec![0.0, 0.0, 0.0];
        normalize(&mut v);
        assert_eq!(v, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_normalize_already_unit()
    {
        let mut v = vec![1.0, 0.0, 0.0];
        normalize(&mut v);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!(v[1].abs() < 1e-6);
        assert!(v[2].abs() < 1e-6);
    }
}
