//! Retrieval-Augmented Generation (RAG) pipeline for AI applications.
//! 用于 AI 应用的检索增强生成 (RAG) 管道。
//!
//! This module provides a complete RAG pipeline that combines document ingestion,
//! chunking, embedding, vector storage, and context-aware answer generation.
//!
//! 本模块提供完整的 RAG 管道，结合了文档摄取、分块、嵌入、向量存储
//! 和基于上下文的答案生成。
//!
//! # Overview / 概述
//!
//! The RAG pipeline follows these steps:
//! 1. **Ingest**: Split documents into chunks using a `DocumentChunker`.
//! 2. **Embed**: Generate vector embeddings for each chunk.
//! 3. **Store**: Persist chunks and embeddings in a `VectorStore`.
//! 4. **Query**: Embed the user question, retrieve relevant chunks, and generate an answer.
//!
//! RAG 管道遵循以下步骤：
//! 1. **摄取**：使用 `DocumentChunker` 将文档拆分为块。
//! 2. **嵌入**：为每个块生成向量嵌入。
//! 3. **存储**：将块和嵌入持久化到 `VectorStore`。
//! 4. **查询**：嵌入用户问题，检索相关块，并生成答案。

use std::sync::Arc;

use crate::{
    chat_model::{ChatModel, ChatRequest, ChatStream, ModelError},
    embedding::EmbeddingModel,
    vector_store::{Document, SearchResult, VectorStore},
};

/// Strategy for splitting text into chunks.
/// 将文本拆分为块的策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkStrategy
{
    /// Split by sentences (separated by '.', '!', '?' followed by whitespace).
    /// 按句子拆分（以 '.'、'!'、'?' 后跟空白符分隔）。
    Sentence,
    /// Split by paragraphs (separated by double newlines).
    /// 按段落拆分（以双换行符分隔）。
    Paragraph,
    /// Split by fixed character count with optional overlap.
    /// 按固定字符数拆分，可选重叠。
    FixedSize,
}

/// Configuration for the RAG pipeline.
/// RAG 管道的配置。
#[derive(Debug, Clone)]
pub struct RagConfig
{
    /// Maximum number of chunks to retrieve per query.
    /// 每次查询检索的最大块数。
    pub top_k: usize,
    /// Minimum similarity score for retrieved chunks (0.0 to 1.0).
    /// 检索块的最低相似度分数（0.0 到 1.0）。
    pub min_score: f32,
    /// Chunking strategy for document ingestion.
    /// 文档摄取的分块策略。
    pub chunk_strategy: ChunkStrategy,
    /// Chunk size for fixed-size chunking (in characters).
    /// 固定大小分块的块大小（字符数）。
    pub chunk_size: usize,
    /// Overlap between consecutive chunks (in characters).
    /// 连续块之间的重叠（字符数）。
    pub chunk_overlap: usize,
    /// System prompt template for answer generation.
    /// 答案生成的系统提示模板。
    pub system_prompt: String,
}

impl Default for RagConfig
{
    fn default() -> Self
    {
        Self {
            top_k: 5,
            min_score: 0.0,
            chunk_strategy: ChunkStrategy::FixedSize,
            chunk_size: 500,
            chunk_overlap: 50,
            system_prompt: "You are a helpful assistant. Answer the user's question based on the \
                            provided context. If the context does not contain enough information, \
                            say so."
                .to_string(),
        }
    }
}

impl RagConfig
{
    /// Creates a new RAG configuration with default values.
    /// 使用默认值创建新的 RAG 配置。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Sets the number of top chunks to retrieve.
    /// 设置检索的顶部块数。
    #[must_use]
    pub fn top_k(mut self, k: usize) -> Self
    {
        self.top_k = k;
        self
    }

    /// Sets the minimum similarity score threshold.
    /// 设置最低相似度分数阈值。
    #[must_use]
    pub fn min_score(mut self, score: f32) -> Self
    {
        self.min_score = score;
        self
    }

    /// Sets the chunking strategy.
    /// 设置分块策略。
    #[must_use]
    pub fn chunk_strategy(mut self, strategy: ChunkStrategy) -> Self
    {
        self.chunk_strategy = strategy;
        self
    }

    /// Sets the chunk size for fixed-size chunking.
    /// 设置固定大小分块的块大小。
    #[must_use]
    pub fn chunk_size(mut self, size: usize) -> Self
    {
        self.chunk_size = size;
        self
    }

    /// Sets the overlap between consecutive chunks.
    /// 设置连续块之间的重叠。
    #[must_use]
    pub fn chunk_overlap(mut self, overlap: usize) -> Self
    {
        self.chunk_overlap = overlap;
        self
    }

    /// Sets the system prompt for answer generation.
    /// 设置答案生成的系统提示。
    #[must_use]
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self
    {
        self.system_prompt = prompt.into();
        self
    }
}

/// Splits documents into smaller chunks for embedding and retrieval.
/// 将文档拆分为较小的块，用于嵌入和检索。
///
/// Supports multiple chunking strategies to balance between preserving
/// context and keeping chunks small enough for effective embedding.
///
/// 支持多种分块策略，以在保留上下文和保持块足够小以有效嵌入之间取得平衡。
#[derive(Debug, Clone)]
pub struct DocumentChunker
{
    /// The chunking strategy to use.
    /// 使用的分块策略。
    strategy: ChunkStrategy,
    /// Size of each chunk in characters (for fixed-size strategy).
    /// 每个块的字符大小（用于固定大小策略）。
    chunk_size: usize,
    /// Overlap between consecutive chunks in characters.
    /// 连续块之间的字符重叠。
    chunk_overlap: usize,
}

impl DocumentChunker
{
    /// Creates a new document chunker with the given strategy.
    /// 使用给定策略创建新的文档分块器。
    #[must_use]
    pub fn new(strategy: ChunkStrategy) -> Self
    {
        Self {
            strategy,
            chunk_size: 500,
            chunk_overlap: 50,
        }
    }

    /// Creates a chunker with fixed-size strategy and specified parameters.
    /// 创建具有固定大小策略和指定参数的分块器。
    #[must_use]
    pub fn fixed_size(chunk_size: usize, chunk_overlap: usize) -> Self
    {
        Self {
            strategy: ChunkStrategy::FixedSize,
            chunk_size,
            chunk_overlap,
        }
    }

    /// Sets the chunk size.
    /// 设置块大小。
    #[must_use]
    pub fn chunk_size(mut self, size: usize) -> Self
    {
        self.chunk_size = size;
        self
    }

    /// Sets the chunk overlap.
    /// 设置块重叠。
    #[must_use]
    pub fn chunk_overlap(mut self, overlap: usize) -> Self
    {
        self.chunk_overlap = overlap;
        self
    }

    /// Splits a text into chunks according to the configured strategy.
    /// 根据配置的策略将文本拆分为块。
    pub fn chunk(&self, text: &str) -> Vec<String>
    {
        match self.strategy
        {
            ChunkStrategy::Sentence => self.chunk_by_sentence(text),
            ChunkStrategy::Paragraph => self.chunk_by_paragraph(text),
            ChunkStrategy::FixedSize => self.chunk_by_fixed_size(text),
        }
    }

    /// Splits text by sentences.
    /// 按句子拆分文本。
    fn chunk_by_sentence(&self, text: &str) -> Vec<String>
    {
        let mut chunks = Vec::new();
        let mut current = String::new();

        for sentence in text.split_inclusive(['.', '!', '?'])
        {
            let trimmed = sentence.trim();
            if trimmed.is_empty()
            {
                continue;
            }

            if !current.is_empty()
            {
                current.push(' ');
            }
            current.push_str(trimmed);

            if current.len() >= self.chunk_size
            {
                chunks.push(current.clone());
                current.clear();
            }
        }

        if !current.is_empty()
        {
            chunks.push(current);
        }

        chunks
    }

    /// Splits text by paragraphs (double newlines).
    /// 按段落（双换行符）拆分文本。
    fn chunk_by_paragraph(&self, text: &str) -> Vec<String>
    {
        let mut chunks = Vec::new();
        let mut current = String::new();

        for paragraph in text.split("\n\n")
        {
            let trimmed = paragraph.trim();
            if trimmed.is_empty()
            {
                continue;
            }

            if !current.is_empty()
            {
                current.push_str("\n\n");
            }
            current.push_str(trimmed);

            if current.len() >= self.chunk_size
            {
                chunks.push(current.clone());
                current.clear();
            }
        }

        if !current.is_empty()
        {
            chunks.push(current);
        }

        chunks
    }

    /// Splits text by fixed character count with overlap.
    /// 按固定字符数拆分文本，带重叠。
    fn chunk_by_fixed_size(&self, text: &str) -> Vec<String>
    {
        if text.is_empty()
        {
            return Vec::new();
        }
        if text.len() <= self.chunk_size
        {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut start = 0;

        while start < text.len()
        {
            let end = (start + self.chunk_size).min(text.len());
            chunks.push(text[start..end].to_string());

            if end >= text.len()
            {
                break;
            }

            start += self.chunk_size - self.chunk_overlap;
        }

        chunks
    }
}

impl Default for DocumentChunker
{
    fn default() -> Self
    {
        Self::new(ChunkStrategy::FixedSize)
    }
}

/// Builds context prompts from retrieved document chunks.
/// 从检索到的文档块构建上下文提示。
///
/// Formats retrieved chunks into a structured prompt with optional
/// citation support (source references and chunk numbers).
///
/// 将检索到的块格式化为结构化提示，支持可选的引用功能（来源引用和块编号）。
#[derive(Debug, Clone)]
pub struct ContextBuilder
{
    /// Whether to include citation numbers in the context.
    /// 是否在上下文中包含引用编号。
    include_citations: bool,
    /// Maximum total characters for the context section.
    /// 上下文部分的最大总字符数。
    max_context_chars: usize,
    /// Separator between context chunks.
    /// 上下文块之间的分隔符。
    separator: String,
}

impl Default for ContextBuilder
{
    fn default() -> Self
    {
        Self {
            include_citations: true,
            max_context_chars: 4000,
            separator: "\n\n---\n\n".to_string(),
        }
    }
}

impl ContextBuilder
{
    /// Creates a new context builder with default settings.
    /// 使用默认设置创建新的上下文构建器。
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Enables or disables citation support.
    /// 启用或禁用引用支持。
    #[must_use]
    pub fn include_citations(mut self, include: bool) -> Self
    {
        self.include_citations = include;
        self
    }

    /// Sets the maximum context length in characters.
    /// 设置上下文的最大字符长度。
    #[must_use]
    pub fn max_context_chars(mut self, max: usize) -> Self
    {
        self.max_context_chars = max;
        self
    }

    /// Sets the separator between context chunks.
    /// 设置上下文块之间的分隔符。
    #[must_use]
    pub fn separator(mut self, sep: impl Into<String>) -> Self
    {
        self.separator = sep.into();
        self
    }

    /// Builds a context string from search results.
    /// 从搜索结果构建上下文字符串。
    pub fn build(&self, results: &[SearchResult]) -> String
    {
        let mut context = String::new();
        let mut total_chars = 0;

        for (i, result) in results.iter().enumerate()
        {
            let chunk_text = if self.include_citations
            {
                let source = result
                    .document
                    .metadata
                    .get("source")
                    .cloned()
                    .unwrap_or_else(|| result.document.id.clone());
                format!("[{}] (source: {})\n{}", i + 1, source, result.document.content)
            }
            else
            {
                result.document.content.clone()
            };

            if total_chars + chunk_text.len() > self.max_context_chars
            {
                // Truncate this chunk to fit
                let remaining = self.max_context_chars.saturating_sub(total_chars);
                if remaining > 0
                {
                    context.push_str(&chunk_text[..remaining.min(chunk_text.len())]);
                }
                break;
            }

            if !context.is_empty()
            {
                context.push_str(&self.separator);
            }
            context.push_str(&chunk_text);
            total_chars += chunk_text.len();
        }

        context
    }

    /// Builds a full prompt with system instructions, context, and user question.
    /// 构建包含系统指令、上下文和用户问题的完整提示。
    pub fn build_prompt(
        &self,
        system_prompt: &str,
        results: &[SearchResult],
        question: &str,
    ) -> String
    {
        let context = self.build(results);
        format!("{system_prompt}\n\nContext:\n{context}\n\nQuestion: {question}\n\nAnswer:")
    }
}

/// Result of a RAG query containing the generated answer and source documents.
/// RAG 查询的结果，包含生成的答案和来源文档。
#[derive(Debug, Clone)]
pub struct RagResponse
{
    /// The generated answer text.
    /// 生成的答案文本。
    pub answer: String,
    /// The source chunks used to generate the answer.
    /// 用于生成答案的来源块。
    pub sources: Vec<SearchResult>,
    /// Token usage statistics.
    /// Token 使用统计。
    pub total_tokens: u32,
}

impl RagResponse
{
    /// Creates a new RAG response.
    /// 创建新的 RAG 响应。
    #[must_use]
    pub fn new(answer: impl Into<String>, sources: Vec<SearchResult>) -> Self
    {
        Self {
            answer: answer.into(),
            sources,
            total_tokens: 0,
        }
    }

    /// Sets the total token usage.
    /// 设置总 token 使用量。
    #[must_use]
    pub fn total_tokens(mut self, tokens: u32) -> Self
    {
        self.total_tokens = tokens;
        self
    }
}

/// A full Retrieval-Augmented Generation pipeline.
/// 完整的检索增强生成管道。
///
/// Combines document chunking, embedding, vector storage, and LLM generation
/// into a single pipeline that can ingest documents and answer questions
/// based on the ingested knowledge.
///
/// 将文档分块、嵌入、向量存储和 LLM 生成组合为单一管道，
/// 可以摄取文档并基于摄取的知识回答问题。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_ai::rag::{RagPipeline, RagConfig};
/// use std::sync::Arc;
///
/// let pipeline = RagPipeline::new(
///     chat_model,
///     embedding_model,
///     vector_store,
/// );
///
/// // Ingest documents / 摄取文档
/// pipeline.ingest(vec![
///     Document::new("1", "Rust is a systems programming language..."),
/// ]).await?;
///
/// // Query / 查询
/// let response = pipeline.query("What is Rust?").await?;
/// println!("{}", response.answer);
/// ```
pub struct RagPipeline
{
    /// The chat model for generating answers.
    /// 用于生成答案的聊天模型。
    chat_model: Arc<dyn ChatModel>,
    /// The embedding model for generating vectors.
    /// 用于生成向量的嵌入模型。
    /// Retained for future direct embedding access in the pipeline.
    /// 为管道中未来的直接嵌入访问而保留。
    #[allow(dead_code)]
    embedding_model: Arc<dyn EmbeddingModel>,
    /// The vector store for storing and retrieving document chunks.
    /// 用于存储和检索文档块的向量存储。
    vector_store: Arc<dyn VectorStore>,
    /// Pipeline configuration.
    /// 管道配置。
    config: RagConfig,
    /// Document chunker for splitting text.
    /// 用于拆分文本的文档分块器。
    chunker: DocumentChunker,
    /// Context builder for formatting retrieved chunks.
    /// 用于格式化检索块的上下文构建器。
    context_builder: ContextBuilder,
}

impl std::fmt::Debug for RagPipeline
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("RagPipeline")
            .field("config", &self.config)
            .field("chunker", &self.chunker)
            .field("context_builder", &self.context_builder)
            .finish_non_exhaustive()
    }
}

impl RagPipeline
{
    /// Creates a new RAG pipeline with the given models and store.
    /// 使用给定的模型和存储创建新的 RAG 管道。
    pub fn new(
        chat_model: impl ChatModel + 'static,
        embedding_model: impl EmbeddingModel + 'static,
        vector_store: impl VectorStore + 'static,
    ) -> Self
    {
        Self {
            chat_model: Arc::new(chat_model),
            embedding_model: Arc::new(embedding_model),
            vector_store: Arc::new(vector_store),
            config: RagConfig::default(),
            chunker: DocumentChunker::default(),
            context_builder: ContextBuilder::default(),
        }
    }

    /// Creates a new RAG pipeline from Arc references.
    /// 从 Arc 引用创建新的 RAG 管道。
    pub fn from_arc(
        chat_model: Arc<dyn ChatModel>,
        embedding_model: Arc<dyn EmbeddingModel>,
        vector_store: Arc<dyn VectorStore>,
    ) -> Self
    {
        Self {
            chat_model,
            embedding_model,
            vector_store,
            config: RagConfig::default(),
            chunker: DocumentChunker::default(),
            context_builder: ContextBuilder::default(),
        }
    }

    /// Sets the pipeline configuration.
    /// 设置管道配置。
    #[must_use]
    pub fn with_config(mut self, config: RagConfig) -> Self
    {
        self.chunker = DocumentChunker::new(config.chunk_strategy)
            .chunk_size(config.chunk_size)
            .chunk_overlap(config.chunk_overlap);
        self.config = config;
        self
    }

    /// Sets the context builder.
    /// 设置上下文构建器。
    #[must_use]
    pub fn with_context_builder(mut self, builder: ContextBuilder) -> Self
    {
        self.context_builder = builder;
        self
    }

    /// Ingests documents into the pipeline by chunking, embedding, and storing them.
    /// 通过分块、嵌入和存储将文档摄取到管道中。
    ///
    /// Each document is split into chunks using the configured `DocumentChunker`,
    /// then each chunk is stored as a separate document in the vector store.
    /// If the vector store has an embedding model, embeddings are auto-generated.
    ///
    /// 每个文档使用配置的 `DocumentChunker` 拆分为块，然后每个块作为
    /// 单独的文档存储在向量存储中。如果向量存储有嵌入模型，嵌入会自动生成。
    pub async fn ingest(&self, documents: Vec<Document>) -> Result<usize, ModelError>
    {
        let mut chunk_count = 0;
        let mut all_chunks: Vec<Document> = Vec::new();

        for doc in documents
        {
            let source = doc.id.clone();
            let chunks = self.chunker.chunk(&doc.content);

            for (i, chunk_text) in chunks.into_iter().enumerate()
            {
                let chunk_id = format!("{}-chunk-{}", doc.id, i);
                let mut metadata = doc.metadata.clone();
                metadata.insert("source".to_string(), source.clone());
                metadata.insert("chunk_index".to_string(), i.to_string());

                let chunk_doc = Document::new(chunk_id, chunk_text).metadata(metadata);

                all_chunks.push(chunk_doc);
                chunk_count += 1;
            }
        }

        self.vector_store.add(all_chunks).await?;
        Ok(chunk_count)
    }

    /// Queries the pipeline with a question and returns an answer based on retrieved context.
    /// 使用问题查询管道，并基于检索到的上下文返回答案。
    pub async fn query(&self, question: &str) -> Result<RagResponse, ModelError>
    {
        // Retrieve relevant chunks from the vector store
        // 从向量存储中检索相关块
        let mut results = self
            .vector_store
            .search(question, self.config.top_k)
            .await?;

        // Filter by minimum score
        // 按最低分数过滤
        results.retain(|r| r.score >= self.config.min_score);

        if results.is_empty()
        {
            return Ok(RagResponse::new(
                "I could not find any relevant information to answer your question.",
                Vec::new(),
            ));
        }

        // Build context from retrieved chunks
        // 从检索到的块构建上下文
        let context = self.context_builder.build(&results);

        // Build the prompt for the chat model
        // 为聊天模型构建提示
        let prompt = format!(
            "{}\n\nContext:\n{}\n\nQuestion: {}\n\nAnswer:",
            self.config.system_prompt, context, question
        );

        let request = ChatRequest::new()
            .message(crate::chat_model::ChatMessage::system(&self.config.system_prompt))
            .message(crate::chat_model::ChatMessage::user(&prompt));

        let response = self.chat_model.complete(request).await?;

        Ok(RagResponse::new(response.content, results).total_tokens(response.usage.total_tokens))
    }

    /// Queries the pipeline and returns a streaming response.
    /// 查询管道并返回流式响应。
    pub async fn query_stream(
        &self,
        question: &str,
    ) -> Result<(ChatStream, Vec<SearchResult>), ModelError>
    {
        let mut results = self
            .vector_store
            .search(question, self.config.top_k)
            .await?;

        results.retain(|r| r.score >= self.config.min_score);

        let context = self.context_builder.build(&results);

        let prompt = format!(
            "{}\n\nContext:\n{}\n\nQuestion: {}\n\nAnswer:",
            self.config.system_prompt, context, question
        );

        let request = ChatRequest::new()
            .message(crate::chat_model::ChatMessage::system(&self.config.system_prompt))
            .message(crate::chat_model::ChatMessage::user(&prompt));

        let stream = self.chat_model.stream(request).await?;
        Ok((stream, results))
    }

    /// Returns the total number of documents in the vector store.
    /// 返回向量存储中的文档总数。
    pub async fn document_count(&self) -> Result<usize, ModelError>
    {
        self.vector_store.count().await
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
mod tests
{
    use super::*;

    #[test]
    fn test_chunk_strategy_sentence()
    {
        let chunker = DocumentChunker::new(ChunkStrategy::Sentence).chunk_size(100);
        let text = "Hello world. This is a test! How are you? I am fine.";
        let chunks = chunker.chunk(text);
        assert!(!chunks.is_empty());
        // All text should be covered
        let total: String = chunks.join(" ");
        assert!(total.contains("Hello world"));
    }

    #[test]
    fn test_chunk_strategy_paragraph()
    {
        let chunker = DocumentChunker::new(ChunkStrategy::Paragraph).chunk_size(200);
        let text = "First paragraph here.\n\nSecond paragraph here.\n\nThird paragraph.";
        let chunks = chunker.chunk(text);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_chunk_strategy_fixed_size()
    {
        let chunker = DocumentChunker::fixed_size(10, 2);
        let text = "Hello world this is a test";
        let chunks = chunker.chunk(text);
        assert!(chunks.len() >= 2);
        assert_eq!(chunks[0], "Hello worl");
    }

    #[test]
    fn test_chunk_fixed_size_short_text()
    {
        let chunker = DocumentChunker::fixed_size(100, 10);
        let text = "Short text";
        let chunks = chunker.chunk(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Short text");
    }

    #[test]
    fn test_chunk_empty_text()
    {
        let chunker = DocumentChunker::fixed_size(100, 10);
        let chunks = chunker.chunk("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_config_default()
    {
        let config = RagConfig::default();
        assert_eq!(config.top_k, 5);
        assert_eq!(config.chunk_size, 500);
        assert_eq!(config.chunk_overlap, 50);
    }

    #[test]
    fn test_config_builder()
    {
        let config = RagConfig::new()
            .top_k(10)
            .min_score(0.5)
            .chunk_size(1000)
            .chunk_overlap(100);

        assert_eq!(config.top_k, 10);
        assert!((config.min_score - 0.5).abs() < f32::EPSILON);
        assert_eq!(config.chunk_size, 1000);
        assert_eq!(config.chunk_overlap, 100);
    }

    #[test]
    fn test_context_builder_basic()
    {
        let builder = ContextBuilder::new().include_citations(false);
        let results = vec![
            SearchResult::new(Document::new("1", "First chunk"), 0.9),
            SearchResult::new(Document::new("2", "Second chunk"), 0.8),
        ];

        let context = builder.build(&results);
        assert!(context.contains("First chunk"));
        assert!(context.contains("Second chunk"));
    }

    #[test]
    fn test_context_builder_with_citations()
    {
        let builder = ContextBuilder::new().include_citations(true);
        let results = vec![
            SearchResult::new(
                Document::new("1", "Content A").with_metadata("source", "doc1.txt"),
                0.9,
            ),
            SearchResult::new(
                Document::new("2", "Content B").with_metadata("source", "doc2.txt"),
                0.8,
            ),
        ];

        let context = builder.build(&results);
        assert!(context.contains("[1]"));
        assert!(context.contains("[2]"));
        assert!(context.contains("doc1.txt"));
        assert!(context.contains("doc2.txt"));
    }

    #[test]
    fn test_context_builder_max_chars()
    {
        let builder = ContextBuilder::new()
            .max_context_chars(20)
            .include_citations(false);

        let results = vec![
            SearchResult::new(Document::new("1", "A very long piece of text"), 0.9),
            SearchResult::new(Document::new("2", "Another piece"), 0.8),
        ];

        let context = builder.build(&results);
        assert!(context.len() <= 25); // Allow for separator
    }

    #[test]
    fn test_context_builder_empty_results()
    {
        let builder = ContextBuilder::new();
        let context = builder.build(&[]);
        assert!(context.is_empty());
    }

    #[test]
    fn test_rag_response()
    {
        let response = RagResponse::new("Test answer", Vec::new()).total_tokens(42);
        assert_eq!(response.answer, "Test answer");
        assert!(response.sources.is_empty());
        assert_eq!(response.total_tokens, 42);
    }

    #[test]
    fn test_document_chunker_default()
    {
        let chunker = DocumentChunker::default();
        assert_eq!(chunker.strategy, ChunkStrategy::FixedSize);
        assert_eq!(chunker.chunk_size, 500);
        assert_eq!(chunker.chunk_overlap, 50);
    }
}
