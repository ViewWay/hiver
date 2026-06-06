//! Vector store abstractions for semantic document storage and retrieval.
//! 用于语义文档存储和检索的向量存储抽象。
//!
//! This module defines the interface for vector databases that store
//! documents with their embeddings and support similarity-based search,
//! which is fundamental for Retrieval Augmented Generation (RAG).
//!
//! 本模块定义了向量数据库的接口，用于存储带有嵌入的文档
//! 并支持基于相似性的搜索，这是检索增强生成 (RAG) 的基础。

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    chat_model::ModelError,
    embedding::{EmbeddingModel, cosine_similarity},
};

/// A document stored in a vector store.
/// 存储在向量存储中的文档。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document
{
    /// Unique identifier for the document.
    /// 文档的唯一标识符。
    pub id: String,
    /// The text content of the document.
    /// 文档的文本内容。
    pub content: String,
    /// Optional metadata associated with the document.
    /// 与文档关联的可选元数据。
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Optional pre-computed embedding vector.
    /// 可选的预计算嵌入向量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

impl Document
{
    /// Creates a new document with the given ID and content.
    /// 使用给定 ID 和内容创建新文档。
    #[must_use]
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self
    {
        Self {
            id: id.into(),
            content: content.into(),
            metadata: HashMap::new(),
            embedding: None,
        }
    }

    /// Sets metadata for the document.
    /// 设置文档的元数据。
    #[must_use]
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self
    {
        self.metadata = metadata;
        self
    }

    /// Adds a single metadata key-value pair.
    /// 添加单个元数据键值对。
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Sets the embedding vector for the document.
    /// 设置文档的嵌入向量。
    #[must_use]
    pub fn embedding(mut self, embedding: Vec<f32>) -> Self
    {
        self.embedding = Some(embedding);
        self
    }
}

/// A search result from a vector store query.
/// 向量存储查询的搜索结果。
#[derive(Debug, Clone)]
pub struct SearchResult
{
    /// The matched document.
    /// 匹配的文档。
    pub document: Document,
    /// Similarity score (0.0 to 1.0, higher is more similar).
    /// 相似度分数 (0.0 到 1.0，越高越相似)。
    pub score: f32,
}

impl SearchResult
{
    /// Creates a new search result.
    /// 创建新的搜索结果。
    #[must_use]
    pub fn new(document: Document, score: f32) -> Self
    {
        Self { document, score }
    }
}

/// Trait for vector store implementations.
/// 向量存储实现的 trait。
///
/// This trait defines the interface for vector databases that support
/// storing, searching, updating, and deleting documents based on semantic similarity.
///
/// 此 trait 定义了支持基于语义相似性存储、搜索、更新和删除文档的
/// 向量数据库接口。
#[async_trait::async_trait]
pub trait VectorStore: Send + Sync
{
    /// Adds documents to the vector store.
    /// 将文档添加到向量存储中。
    async fn add(&self, documents: Vec<Document>) -> Result<(), ModelError>;

    /// Searches for documents similar to the given query text.
    /// 搜索与给定查询文本相似的文档。
    async fn search(&self, query: &str, k: usize) -> Result<Vec<SearchResult>, ModelError>;

    /// Deletes documents by their IDs.
    /// 按 ID 删除文档。
    async fn delete(&self, ids: &[&str]) -> Result<(), ModelError>;

    /// Returns the total number of documents in the store.
    /// 返回存储中的文档总数。
    async fn count(&self) -> Result<usize, ModelError>;

    /// Updates existing documents by ID. Documents with unknown IDs are ignored.
    /// 按 ID 更新现有文档。未知 ID 的文档将被忽略。
    async fn update(&self, documents: Vec<Document>) -> Result<(), ModelError>;
}

/// A simple in-memory vector store for testing and development.
/// 用于测试和开发的简单内存向量存储。
///
/// Supports two modes:
/// - **With embedding model**: automatically generates embeddings and performs cosine similarity
///   search.
/// - **Without embedding model**: falls back to keyword matching for search.
///
/// 支持两种模式：
/// - **带嵌入模型**：自动生成嵌入并执行余弦相似度搜索。
/// - **无嵌入模型**：回退到关键字匹配进行搜索。
pub struct InMemoryVectorStore
{
    /// Stored documents.
    /// 存储的文档。
    documents: tokio::sync::RwLock<Vec<Document>>,
    /// Optional embedding model for generating vectors.
    /// 可选的嵌入模型，用于生成向量。
    embedding_model: Option<Arc<dyn EmbeddingModel>>,
}

impl std::fmt::Debug for InMemoryVectorStore
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("InMemoryVectorStore")
            .field("documents", &"<RwLock>")
            .field(
                "embedding_model",
                &self
                    .embedding_model
                    .as_ref()
                    .map(|_| "Some(EmbeddingModel)"),
            )
            .finish()
    }
}

impl Default for InMemoryVectorStore
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl InMemoryVectorStore
{
    /// Creates a new empty in-memory vector store without embedding model.
    /// 创建新的空内存向量存储（不带嵌入模型）。
    ///
    /// Search will fall back to keyword matching.
    /// 搜索将回退到关键字匹配。
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            documents: tokio::sync::RwLock::new(Vec::new()),
            embedding_model: None,
        }
    }

    /// Creates a new in-memory vector store with an embedding model.
    /// 创建带嵌入模型的内存向量存储。
    ///
    /// When an embedding model is provided:
    /// - `add()` will auto-generate embeddings for documents that don't already have one.
    /// - `search()` will embed the query and perform real cosine similarity search.
    ///
    /// 当提供嵌入模型时：
    /// - `add()` 会为没有嵌入的文档自动生成嵌入。
    /// - `search()` 会对查询进行嵌入并执行真正的余弦相似度搜索。
    pub fn with_embedding_model(model: Arc<dyn EmbeddingModel>) -> Self
    {
        Self {
            documents: tokio::sync::RwLock::new(Vec::new()),
            embedding_model: Some(model),
        }
    }
}

#[async_trait::async_trait]
impl VectorStore for InMemoryVectorStore
{
    async fn add(&self, documents: Vec<Document>) -> Result<(), ModelError>
    {
        let mut guard = self.documents.write().await;

        // If we have an embedding model, generate embeddings for documents that lack them
        if let Some(ref model) = self.embedding_model
        {
            let mut enriched: Vec<Document> = Vec::new();
            for mut doc in documents
            {
                if doc.embedding.is_none() && !doc.content.is_empty()
                {
                    if let Ok(embedding) = model.embed_text(&doc.content).await
                    {
                        doc.embedding = Some(embedding);
                    }
                }
                enriched.push(doc);
            }
            guard.extend(enriched);
        }
        else
        {
            guard.extend(documents);
        }

        Ok(())
    }

    async fn search(&self, query: &str, k: usize) -> Result<Vec<SearchResult>, ModelError>
    {
        let guard = self.documents.read().await;

        if guard.is_empty() || k == 0
        {
            return Ok(Vec::new());
        }

        // If we have an embedding model, use cosine similarity search
        if let Some(ref model) = self.embedding_model
        {
            let query_embedding = model.embed_text(query).await?;

            let mut scored: Vec<SearchResult> = guard
                .iter()
                .filter(|d| d.embedding.is_some())
                .map(|d| {
                    let embedding = d.embedding.as_ref().expect("checked above");
                    let score = cosine_similarity(&query_embedding, embedding);
                    SearchResult::new(d.clone(), score)
                })
                .collect();

            // Sort by score descending
            scored.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            scored.truncate(k);
            return Ok(scored);
        }

        // Fallback: keyword matching when no embedding model is available
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<SearchResult> = guard
            .iter()
            .map(|d| {
                let content_lower = d.content.to_lowercase();
                let score = if query_lower.is_empty()
                {
                    0.0
                }
                else if content_lower == query_lower
                {
                    // Exact match
                    1.0
                }
                else if content_lower.contains(&query_lower)
                {
                    // Substring match
                    0.8
                }
                else
                {
                    // Word-level partial match: score proportional to matching words
                    let matched = query_words
                        .iter()
                        .filter(|w| content_lower.contains(*w))
                        .count();
                    if query_words.is_empty()
                    {
                        #[allow(clippy::cast_precision_loss)]
                        0.0
                    }
                    else
                    {
                        (matched as f32) / (query_words.len() as f32) * 0.5
                    }
                };
                SearchResult::new(d.clone(), score)
            })
            .filter(|r| r.score > 0.0)
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        scored.truncate(k);
        Ok(scored)
    }

    async fn delete(&self, ids: &[&str]) -> Result<(), ModelError>
    {
        let id_set: std::collections::HashSet<&str> = ids.iter().copied().collect();
        let mut guard = self.documents.write().await;
        guard.retain(|d| !id_set.contains(d.id.as_str()));
        Ok(())
    }

    async fn count(&self) -> Result<usize, ModelError>
    {
        let guard = self.documents.read().await;
        Ok(guard.len())
    }

    async fn update(&self, documents: Vec<Document>) -> Result<(), ModelError>
    {
        let mut guard = self.documents.write().await;

        for doc in documents
        {
            if let Some(existing) = guard.iter_mut().find(|d| d.id == doc.id)
            {
                *existing = doc;
            }
            // Documents with unknown IDs are silently ignored
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::{chat_model::TokenUsage, embedding::EmbeddingRequest};

    /// A mock embedding model that maps text to deterministic vectors.
    /// 将文本映射为确定性向量的模拟嵌入模型。
    struct MockEmbeddingModel;

    #[async_trait::async_trait]
    impl EmbeddingModel for MockEmbeddingModel
    {
        async fn embed(
            &self,
            request: EmbeddingRequest,
        ) -> Result<crate::embedding::EmbeddingResponse, ModelError>
        {
            let embeddings: Vec<Vec<f32>> = request
                .inputs
                .iter()
                .map(|text| {
                    // Deterministic pseudo-embedding: hash characters to a 3-dimensional vector
                    let mut v = vec![0.0f32, 0.0f32, 0.0f32];
                    for (i, ch) in text.chars().enumerate()
                    {
                        let idx = i % 3;
                        v[idx] += ch as u32 as f32;
                    }
                    // Normalize to unit length
                    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                    if norm > 0.0
                    {
                        for x in &mut v
                        {
                            *x /= norm;
                        }
                    }
                    v
                })
                .collect();

            Ok(crate::embedding::EmbeddingResponse::new(embeddings, "mock-embedding")
                .usage(TokenUsage::new(0, 0)))
        }
    }

    // --- Document tests ---

    #[test]
    fn test_document_new()
    {
        let doc = Document::new("doc-1", "Hello world");
        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.content, "Hello world");
        assert!(doc.metadata.is_empty());
        assert!(doc.embedding.is_none());
    }

    #[test]
    fn test_document_with_metadata()
    {
        let mut meta = HashMap::new();
        meta.insert("source".to_string(), "test".to_string());

        let doc = Document::new("doc-2", "Content").metadata(meta);
        assert_eq!(doc.metadata.get("source").unwrap(), "test");
    }

    #[test]
    fn test_document_with_metadata_pair()
    {
        let doc = Document::new("doc-3", "Content")
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        assert_eq!(doc.metadata.len(), 2);
        assert_eq!(doc.metadata.get("key1").unwrap(), "value1");
    }

    #[test]
    fn test_document_with_embedding()
    {
        let doc = Document::new("doc-4", "Text").embedding(vec![0.1, 0.2, 0.3]);
        assert_eq!(doc.embedding.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_search_result()
    {
        let doc = Document::new("1", "test");
        let result = SearchResult::new(doc, 0.95);
        assert_eq!(result.score, 0.95);
        assert_eq!(result.document.id, "1");
    }

    // --- Cosine similarity with known vectors ---

    #[tokio::test]
    async fn test_cosine_similarity_with_known_vectors()
    {
        // Verify cosine similarity directly via embedding module
        // Orthogonal vectors: similarity should be 0
        let sim_ortho = cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]);
        assert!(sim_ortho.abs() < 1e-6, "orthogonal should be ~0, got {sim_ortho}");

        // Same direction: similarity should be 1.0
        let sim_same = cosine_similarity(&[2.0, 3.0], &[4.0, 6.0]);
        assert!((sim_same - 1.0).abs() < 1e-6, "same direction should be ~1.0, got {sim_same}");

        // Opposite: similarity should be -1.0
        let sim_opposite = cosine_similarity(&[1.0, 0.0], &[-1.0, 0.0]);
        assert!(
            (sim_opposite - (-1.0)).abs() < 1e-6,
            "opposite should be ~-1.0, got {sim_opposite}"
        );
    }

    // --- Search sorted by similarity ---

    #[tokio::test]
    async fn test_search_sorted_by_similarity_with_embedding_model()
    {
        let model = Arc::new(MockEmbeddingModel);
        let store = InMemoryVectorStore::with_embedding_model(model);

        // MockEmbeddingModel normalizes character-sum vectors.
        // "aaa" -> [97*3, 0, 0] normalized -> [1.0, 0.0, 0.0]
        // "abc" -> [97, 98, 99] normalized
        // "xyz" -> [120, 121, 122] normalized
        let doc1 = Document::new("1", "aaa");
        let doc2 = Document::new("2", "abc");
        let doc3 = Document::new("3", "xyz");

        // Add with embedding model (auto-embed)
        store
            .add(vec![doc1, doc2, doc3])
            .await
            .expect("add should succeed");

        // Search for "aaa" - should be most similar to doc1
        let results = store.search("aaa", 3).await.expect("search should succeed");
        assert_eq!(results.len(), 3, "should return all 3 docs");
        // First result should be doc1 (exact same text -> identical embedding -> similarity 1.0)
        assert_eq!(results[0].document.id, "1");
        assert!(
            (results[0].score - 1.0).abs() < 1e-4,
            "self-similarity should be ~1.0, got {}",
            results[0].score
        );
        // Results should be sorted descending by score
        for i in 1..results.len()
        {
            assert!(
                results[i - 1].score >= results[i].score,
                "results not sorted: [{}] score {} < [{}] score {}",
                i - 1,
                results[i - 1].score,
                i,
                results[i].score
            );
        }
    }

    // --- Keyword fallback search ---

    #[tokio::test]
    async fn test_keyword_fallback_exact_match()
    {
        let store = InMemoryVectorStore::new();

        store
            .add(vec![
                Document::new("1", "hello world"),
                Document::new("2", "goodbye world"),
                Document::new("3", "unrelated"),
            ])
            .await
            .expect("add should succeed");

        let results = store
            .search("hello world", 10)
            .await
            .expect("search should succeed");
        assert_eq!(results.len(), 2, "should match 'hello world' and 'goodbye world'");
        // First should be the exact match (score 1.0)
        assert_eq!(results[0].document.id, "1");
        assert!((results[0].score - 1.0).abs() < 1e-6, "exact match should score 1.0");
    }

    #[tokio::test]
    async fn test_keyword_fallback_substring_match()
    {
        let store = InMemoryVectorStore::new();

        store
            .add(vec![Document::new("1", "the quick brown fox jumps")])
            .await
            .expect("add should succeed");

        let results = store
            .search("quick brown", 10)
            .await
            .expect("search should succeed");
        assert_eq!(results.len(), 1);
        assert!((results[0].score - 0.8).abs() < 1e-6, "substring match should score 0.8");
    }

    #[tokio::test]
    async fn test_keyword_fallback_word_partial_match()
    {
        let store = InMemoryVectorStore::new();

        store
            .add(vec![Document::new("1", "rust programming language")])
            .await
            .expect("add should succeed");

        let results = store
            .search("rust language tutorial", 10)
            .await
            .expect("search should succeed");
        assert_eq!(results.len(), 1);
        // 2 out of 3 words match -> 2/3 * 0.5 = 0.333...
        let expected = 2.0 / 3.0 * 0.5;
        assert!(
            (results[0].score - expected).abs() < 1e-6,
            "expected ~{}, got {}",
            expected,
            results[0].score
        );
    }

    #[tokio::test]
    async fn test_keyword_fallback_no_match()
    {
        let store = InMemoryVectorStore::new();

        store
            .add(vec![Document::new("1", "rust programming language")])
            .await
            .expect("add should succeed");

        let results = store
            .search("python java", 10)
            .await
            .expect("search should succeed");
        assert!(results.is_empty(), "no matching words should return empty");
    }

    // --- Add with auto-embedding ---

    #[tokio::test]
    async fn test_add_with_auto_embedding()
    {
        let model = Arc::new(MockEmbeddingModel);
        let store = InMemoryVectorStore::with_embedding_model(model);

        // Add document without embedding
        let doc = Document::new("1", "test content");
        store.add(vec![doc]).await.expect("add should succeed");

        // Verify embedding was auto-generated
        let guard = store.documents.read().await;
        assert!(guard[0].embedding.is_some(), "embedding should be auto-generated");
        assert_eq!(guard[0].embedding.as_ref().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_add_preserves_existing_embedding()
    {
        let model = Arc::new(MockEmbeddingModel);
        let store = InMemoryVectorStore::with_embedding_model(model);

        // Add document with a pre-computed embedding
        let doc = Document::new("1", "test content").embedding(vec![0.5, 0.5, 0.5]);
        store.add(vec![doc]).await.expect("add should succeed");

        // Verify the original embedding is preserved (not overwritten)
        let guard = store.documents.read().await;
        let embedding = guard[0].embedding.as_ref().expect("should have embedding");
        assert!((embedding[0] - 0.5).abs() < 1e-6, "pre-computed embedding should be preserved");
    }

    // --- Delete and re-search ---

    #[tokio::test]
    async fn test_delete_and_research()
    {
        let store = InMemoryVectorStore::new();

        store
            .add(vec![
                Document::new("1", "alpha beta"),
                Document::new("2", "gamma delta"),
                Document::new("3", "alpha gamma"),
            ])
            .await
            .expect("add should succeed");

        assert_eq!(store.count().await.unwrap(), 3);

        // Delete doc 1
        store.delete(&["1"]).await.expect("delete should succeed");
        assert_eq!(store.count().await.unwrap(), 2);

        // Search for "alpha" - should only find doc 3 now
        let results = store
            .search("alpha", 10)
            .await
            .expect("search should succeed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.id, "3");
    }

    // --- Empty store behavior ---

    #[tokio::test]
    async fn test_empty_store_count()
    {
        let store = InMemoryVectorStore::new();
        assert_eq!(store.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_empty_store_search()
    {
        let store = InMemoryVectorStore::new();
        let results = store
            .search("test", 10)
            .await
            .expect("search should succeed");
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_empty_store_delete()
    {
        let store = InMemoryVectorStore::new();
        store
            .delete(&["nonexistent"])
            .await
            .expect("delete should succeed");
        assert_eq!(store.count().await.unwrap(), 0);
    }

    // --- k=0 edge case ---

    #[tokio::test]
    async fn test_search_k_zero()
    {
        let store = InMemoryVectorStore::new();
        store
            .add(vec![Document::new("1", "test")])
            .await
            .expect("add should succeed");

        let results = store
            .search("test", 0)
            .await
            .expect("search should succeed");
        assert!(results.is_empty());
    }

    // --- Search limits k ---

    #[tokio::test]
    async fn test_search_limits_k()
    {
        let model = Arc::new(MockEmbeddingModel);
        let store = InMemoryVectorStore::with_embedding_model(model);

        for i in 0..5
        {
            store
                .add(vec![Document::new(
                    format!("doc-{i}"),
                    format!("Content number {i}"),
                )])
                .await
                .expect("add should succeed");
        }

        let results = store
            .search("Content", 3)
            .await
            .expect("search should succeed");
        assert!(results.len() <= 3, "should return at most k results, got {}", results.len());
    }

    // --- Update ---

    #[tokio::test]
    async fn test_update_existing_document()
    {
        let store = InMemoryVectorStore::new();

        store
            .add(vec![Document::new("1", "original content")])
            .await
            .expect("add should succeed");

        store
            .update(vec![Document::new("1", "updated content")])
            .await
            .expect("update should succeed");

        let results = store
            .search("updated", 10)
            .await
            .expect("search should succeed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.content, "updated content");
    }

    #[tokio::test]
    async fn test_update_nonexistent_document()
    {
        let store = InMemoryVectorStore::new();

        // Updating a non-existent ID should silently ignore
        store
            .update(vec![Document::new("ghost", "does not exist")])
            .await
            .expect("update should succeed");

        assert_eq!(store.count().await.unwrap(), 0);
    }

    // --- Serde roundtrip ---

    #[test]
    fn test_document_serde_roundtrip()
    {
        let doc = Document::new("1", "test content")
            .with_metadata("key", "value")
            .embedding(vec![0.1, 0.2]);

        let json = serde_json::to_string(&doc).expect("serialize");
        let deserialized: Document = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, "1");
        assert_eq!(deserialized.content, "test content");
        assert_eq!(deserialized.metadata.get("key").unwrap(), "value");
    }

    // --- Full integration: add, search, delete, re-search with embedding model ---

    #[tokio::test]
    async fn test_full_lifecycle_with_embedding_model()
    {
        let model = Arc::new(MockEmbeddingModel);
        let store = InMemoryVectorStore::with_embedding_model(model);

        // Add documents
        store
            .add(vec![
                Document::new("1", "rust programming"),
                Document::new("2", "python programming"),
                Document::new("3", "cooking recipes"),
            ])
            .await
            .expect("add should succeed");

        assert_eq!(store.count().await.unwrap(), 3);

        // Search
        let results = store
            .search("rust programming", 10)
            .await
            .expect("search should succeed");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].document.id, "1");
        assert!((results[0].score - 1.0).abs() < 1e-4, "self-similarity should be ~1.0");

        // Delete
        store.delete(&["3"]).await.expect("delete should succeed");
        assert_eq!(store.count().await.unwrap(), 2);

        // Re-search
        let results = store
            .search("cooking", 10)
            .await
            .expect("search should succeed");
        assert_eq!(results.len(), 2); // doc 3 is gone, but doc 1 and 2 remain

        // Update doc 1 (note: update replaces the whole doc including embedding)
        store
            .update(vec![Document::new("1", "rust systems programming")])
            .await
            .expect("update should succeed");

        // Doc 1 no longer has embedding after update (update doesn't auto-embed)
        // so search with embedding model only returns docs with embeddings
        let results = store
            .search("rust systems programming", 10)
            .await
            .expect("search should succeed");
        // Only doc 2 still has an embedding
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.id, "2");
    }
}
