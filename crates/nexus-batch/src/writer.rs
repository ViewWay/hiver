//! Item writer for batch processing
//! 批处理项目写入器

use crate::error::BatchResult;
use async_trait::async_trait;

/// Item writer trait
/// 项目写入器trait
///
/// Writes items in chunks (batches) for efficient batch processing.
/// 以块（批次）写入项目以实现高效的批处理。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface ItemWriter<T> {
///     void write(Chunk<? extends T> chunk) throws Exception;
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_batch::prelude::*;
/// use async_trait::async_trait;
///
/// struct DatabaseWriter {
///     pool: sqlx::PgPool,
/// }
///
/// #[async_trait]
/// impl ItemWriter for DatabaseWriter {
///     type Item = User;
///
///     async fn write(&mut self, items: Vec<User>) -> BatchResult<()> {
///         let mut tx = self.pool.begin().await?;
///         for user in items {
///             sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
///                 .bind(&user.name)
///                 .bind(&user.email)
///                 .execute(&mut *tx)
///                 .await?;
///         }
///         tx.commit().await?;
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait ItemWriter: Send + Sync {
    /// Item type
    /// 项目类型
    type Item: Send + Sync;

    /// Write a chunk of items
    /// 写入一批项目
    ///
    /// Items are written as a chunk for efficiency.
    /// 项目作为一批写入以提高效率。
    async fn write(&mut self, items: Vec<Self::Item>) -> BatchResult<()>;

    /// Open the writer for writing
    /// 打开写入器以进行写入
    ///
    /// Called before any write operations.
    /// 在任何写入操作之前调用。
    async fn open(&mut self) -> BatchResult<()> {
        Ok(())
    }

    /// Close the writer
    /// 关闭写入器
    ///
    /// Called after all write operations are complete.
    /// 在所有写入操作完成后调用。
    async fn close(&mut self) -> BatchResult<()> {
        Ok(())
    }
}

/// Item stream writer
/// 项目流写入器
///
/// A writer that collects items into a vector.
/// 将项目收集到向量中的写入器。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_batch::prelude::*;
///
/// let writer = ItemStreamWriter::new();
/// ```
#[derive(Debug, Clone)]
pub struct ItemStreamWriter<T> {
    items: Vec<T>,
}

impl<T> Default for ItemStreamWriter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ItemStreamWriter<T> {
    /// Create new stream writer
    /// 创建新流写入器
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Get all written items
    /// 获取所有已写入项目
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Take items, leaving the writer empty
    /// 获取项目，清空写入器
    pub fn take_items(&mut self) -> Vec<T> {
        std::mem::take(&mut self.items)
    }

    /// Get count of written items
    /// 获取已写入项目计数
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Clear all items
    /// 清除所有项目
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

#[async_trait]
impl<T> ItemWriter for ItemStreamWriter<T>
where
    T: Send + Sync + Clone,
{
    type Item = T;

    async fn write(&mut self, items: Vec<T>) -> BatchResult<()> {
        self.items.extend(items);
        Ok(())
    }
}

/// No-op writer
/// 空操作写入器
///
/// Discards all items written to it.
/// 丢弃所有写入的项目。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // NullItemWriter - discards all items
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NoOpWriter<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for NoOpWriter<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> NoOpWriter<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T: Send + Sync> ItemWriter for NoOpWriter<T> {
    type Item = T;

    async fn write(&mut self, _items: Vec<T>) -> BatchResult<()> {
        Ok(())
    }
}

/// Console writer
/// 控制台写入器
///
/// Writes items to stdout for debugging.
/// 将项目写入标准输出用于调试。
#[derive(Debug)]
pub struct ConsoleWriter<T>
where
    T: std::fmt::Display + Send + Sync,
{
    prefix: Option<String>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ConsoleWriter<T>
where
    T: std::fmt::Display + Send + Sync,
{
    /// Create new console writer
    /// 创建新控制台写入器
    pub fn new() -> Self {
        Self {
            prefix: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set prefix for each line
    /// 设置每行前缀
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }
}

impl<T> Default for ConsoleWriter<T>
where
    T: std::fmt::Display + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for ConsoleWriter<T>
where
    T: std::fmt::Display + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> ItemWriter for ConsoleWriter<T>
where
    T: std::fmt::Display + Send + Sync,
{
    type Item = T;

    async fn write(&mut self, items: Vec<T>) -> BatchResult<()> {
        for item in items {
            if let Some(prefix) = &self.prefix {
                println!("{}: {}", prefix, item);
            } else {
                println!("{}", item);
            }
        }
        Ok(())
    }
}

/// File writer
/// 文件写入器
///
/// Writes items to a file, one per line.
/// 将项目写入文件，每行一个。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public FlatFileItemWriter<Person> writer() {
///     FlatFileItemWriter<Person> writer = new FlatFileItemWriter<>();
///     writer.setResource(new FileSystemResource("output.csv"));
///     writer.setLineAggregator(new DelimitedLineAggregator());
///     return writer;
/// }
/// ```
pub struct FileWriter<T>
where
    T: Send + Sync,
{
    file_path: String,
    formatter: Box<dyn Fn(&T) -> String + Send + Sync>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FileWriter<T>
where
    T: Send + Sync,
{
    /// Create new file writer
    /// 创建新文件写入器
    pub fn new(file_path: impl Into<String>) -> Self
    where
        T: std::fmt::Display,
    {
        Self {
            file_path: file_path.into(),
            formatter: Box::new(|item| item.to_string()),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set custom formatter
    /// 设置自定义格式化器
    pub fn with_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&T) -> String + Send + Sync + 'static,
    {
        self.formatter = Box::new(formatter);
        self
    }

    /// Append to existing file
    /// 追加到现有文件
    pub async fn append(&mut self, items: Vec<T>) -> BatchResult<()>
    where
        T: Clone,
    {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .await?;

        use tokio::io::AsyncWriteExt;
        for item in items {
            let line = (self.formatter)(&item);
            file.write_all(line.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }

        file.flush().await?;
        Ok(())
    }

    /// Write items to file (overwrite mode)
    /// 将项目写入文件（覆盖模式）
    pub async fn write_all(&mut self, items: Vec<T>) -> BatchResult<()>
    where
        T: Clone,
    {
        let mut file = tokio::fs::File::create(&self.file_path).await?;

        use tokio::io::AsyncWriteExt;
        for item in items {
            let line = (self.formatter)(&item);
            file.write_all(line.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }

        file.flush().await?;
        Ok(())
    }
}

#[async_trait]
impl<T> ItemWriter for FileWriter<T>
where
    T: Send + Sync + Clone,
{
    type Item = T;

    async fn write(&mut self, items: Vec<T>) -> BatchResult<()> {
        self.append(items).await
    }
}

/// CSV file writer
/// CSV文件写入器
///
/// Writes items to a CSV file.
/// 将项目写入CSV文件。
pub struct CsvWriter<T>
where
    T: Send + Sync,
{
    file_path: String,
    headers: Option<Vec<String>>,
    #[allow(clippy::type_complexity)]
    formatter: Box<dyn Fn(&T) -> Vec<String> + Send + Sync>,
    write_headers: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> CsvWriter<T>
where
    T: Send + Sync,
{
    /// Create new CSV writer
    /// 创建新CSV写入器
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            headers: None,
            formatter: Box::new(|_| Vec::new()),
            write_headers: true,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set headers
    /// 设置标题
    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Set formatter
    /// 设置格式化器
    pub fn with_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&T) -> Vec<String> + Send + Sync + 'static,
    {
        self.formatter = Box::new(formatter);
        self
    }

    /// Set whether to write headers
    /// 设置是否写入标题
    pub fn with_write_headers(mut self, write: bool) -> Self {
        self.write_headers = write;
        self
    }

    /// Escape CSV value
    /// 转义CSV值
    fn escape_csv(value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace("\"", "\"\""))
        } else {
            value.to_string()
        }
    }

    async fn write_line(&mut self, values: Vec<String>) -> BatchResult<()> {
        let line = values
            .iter()
            .map(|v| Self::escape_csv(v))
            .collect::<Vec<_>>()
            .join(",");

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .await?;

        use tokio::io::AsyncWriteExt;
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;

        Ok(())
    }
}

#[async_trait]
impl<T> ItemWriter for CsvWriter<T>
where
    T: Send + Sync + Clone,
{
    type Item = T;

    async fn open(&mut self) -> BatchResult<()> {
        // Clear file if exists
        if self.write_headers
            && let Some(headers) = &self.headers {
                self.write_line(headers.clone()).await?;
            }
        Ok(())
    }

    async fn write(&mut self, items: Vec<T>) -> BatchResult<()> {
        for item in items {
            let values = (self.formatter)(&item);
            self.write_line(values).await?;
        }
        Ok(())
    }
}

/// Composite writer
/// 组合写入器
///
/// Writes to multiple writers.
/// 写入到多个写入器。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_batch::prelude::*;
///
/// let writer = CompositeWriter::new()
///     .add(db_writer)
///     .add(file_writer)
///     .add(cache_writer);
/// ```
pub struct CompositeWriter<T>
where
    T: Send + Sync,
{
    writers: Vec<Box<dyn ItemWriter<Item = T> + Send + Sync>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> CompositeWriter<T>
where
    T: Send + Sync + 'static,
{
    /// Create new composite writer
    /// 创建新组合写入器
    pub fn new() -> Self {
        Self {
            writers: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add writer to composite
    /// 向组合添加写入器
    #[allow(clippy::should_implement_trait)]
    pub fn add<W>(mut self, writer: W) -> Self
    where
        W: ItemWriter<Item = T> + Send + Sync + 'static,
    {
        self.writers.push(Box::new(writer));
        self
    }
}

impl<T> Default for CompositeWriter<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> ItemWriter for CompositeWriter<T>
where
    T: Send + Sync + Clone,
{
    type Item = T;

    async fn write(&mut self, items: Vec<T>) -> BatchResult<()> {
        for writer in &mut self.writers {
            writer.write(items.clone()).await?;
        }
        Ok(())
    }

    async fn open(&mut self) -> BatchResult<()> {
        for writer in &mut self.writers {
            writer.open().await?;
        }
        Ok(())
    }

    async fn close(&mut self) -> BatchResult<()> {
        for writer in &mut self.writers {
            writer.close().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_item_stream_writer() {
        let mut writer = ItemStreamWriter::new();

        writer.write(vec![1, 2, 3]).await.unwrap();
        assert_eq!(writer.count(), 3);
        assert_eq!(writer.items(), &[1, 2, 3]);

        writer.write(vec![4, 5]).await.unwrap();
        assert_eq!(writer.count(), 5);

        let items = writer.take_items();
        assert_eq!(items, vec![1, 2, 3, 4, 5]);
        assert_eq!(writer.count(), 0);
    }

    #[tokio::test]
    async fn test_noop_writer() {
        let mut writer = NoOpWriter::<i32>::new();

        writer.write(vec![1, 2, 3]).await.unwrap();
        writer.write(vec![]).await.unwrap();
    }

    #[tokio::test]
    async fn test_console_writer() {
        let mut writer = ConsoleWriter::new().with_prefix("[TEST]");

        // Should not error
        writer.write(vec!["hello", "world"]).await.unwrap();
    }

    #[tokio::test]
    async fn test_composite_writer() {
        let mut writer = CompositeWriter::new()
            .add(ItemStreamWriter::<i32>::new())
            .add(NoOpWriter::<i32>::new());

        writer.write(vec![1, 2, 3]).await.unwrap();
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(CsvWriter::<String>::escape_csv("simple"), "simple");
        assert_eq!(CsvWriter::<String>::escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(CsvWriter::<String>::escape_csv("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(
            CsvWriter::<String>::escape_csv("with\nnewline"),
            "\"with\nnewline\""
        );
    }
}
