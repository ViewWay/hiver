//! Item reader for batch processing
//! 批处理项目读取器

use async_trait::async_trait;

use crate::error::BatchResult;

/// Item reader trait
/// 项目读取器trait
///
/// Reads items one at a time for processing in a batch job.
/// 一次读取一个项目供批处理作业处理。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface ItemReader<T> {
///     T read() throws Exception;
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_batch::prelude::*;
/// use async_trait::async_trait;
///
/// struct CsvReader {
///     lines: std::io::Lines<std::io::BufReader<std::fs::File>>,
/// }
///
/// #[async_trait]
/// impl ItemReader for CsvReader {
///     type Item = String;
///
///     async fn read(&mut self) -> BatchResult<Option<String>> {
///         Ok(self.lines.next().transpose()?.map(|r| r.unwrap()))
///     }
/// }
/// ```
#[async_trait]
pub trait ItemReader: Send + Sync {
    /// Item type
    /// 项目类型
    type Item: Send + Sync;

    /// Read next item
    /// 读取下一个项目
    ///
    /// Returns `None` when no more items are available.
    /// 当没有更多项目可用时返回 `None`。
    async fn read(&mut self) -> BatchResult<Option<Self::Item>>;

    /// Open the reader for reading
    /// 打开读取器以进行读取
    ///
    /// Called before any read operations.
    /// 在任何读取操作之前调用。
    async fn open(&mut self) -> BatchResult<()> {
        Ok(())
    }

    /// Close the reader
    /// 关闭读取器
    ///
    /// Called after all read operations are complete.
    /// 在所有读取操作完成后调用。
    async fn close(&mut self) -> BatchResult<()> {
        Ok(())
    }

    /// Get current position/mark
    /// 获取当前位置/标记
    ///
    /// Used for restartability.
    /// 用于可重启性。
    fn mark(&mut self) -> Option<String> {
        None
    }

    /// Reset to mark
    /// 重置到标记
    ///
    /// Used for restartability.
    /// 用于可重启性。
    async fn reset(&mut self) -> BatchResult<()> {
        Ok(())
    }
}

/// Item stream reader
/// 项目流读取器
///
/// A reader that reads from an iterator/stream of items.
/// 从迭代器/项目流读取的读取器。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_batch::prelude::*;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let reader = ItemStreamReader::new(items);
/// ```
pub struct ItemStreamReader<T> {
    items: Vec<T>,
    index: usize,
    marked_index: Option<usize>,
}

impl<T> ItemStreamReader<T> {
    /// Create new stream reader
    /// 创建新流读取器
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            index: 0,
            marked_index: None,
        }
    }

    /// Create empty reader
    /// 创建空读取器
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            index: 0,
            marked_index: None,
        }
    }

    /// Get remaining count
    /// 获取剩余计数
    pub fn remaining(&self) -> usize {
        self.items.len().saturating_sub(self.index)
    }

    /// Check if has more items
    /// 检查是否还有更多项目
    pub fn has_more(&self) -> bool {
        self.index < self.items.len()
    }
}

impl<T> Clone for ItemStreamReader<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            index: self.index,
            marked_index: self.marked_index,
        }
    }
}

#[async_trait]
impl<T> ItemReader for ItemStreamReader<T>
where
    T: Send + Sync + Clone,
{
    type Item = T;

    async fn read(&mut self) -> BatchResult<Option<T>> {
        if self.index < self.items.len() {
            let item = self.items[self.index].clone();
            self.index += 1;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn mark(&mut self) -> Option<String> {
        // Return current position (index), but save index-1 for reset
        // This way reset() will re-read the last successfully read item
        let current = self.index;
        self.marked_index = Some(self.index.saturating_sub(1));
        Some(current.to_string())
    }

    async fn reset(&mut self) -> BatchResult<()> {
        if let Some(idx) = self.marked_index {
            self.index = idx;
        }
        Ok(())
    }
}

/// Iterator item reader
/// 迭代器项目读取器
///
/// Converts any iterator into an ItemReader.
/// 将任何迭代器转换为 ItemReader。
pub struct IteratorItemReader<T, I>
where
    I: Iterator<Item = T> + Send + Sync,
{
    iter: Option<I>,
    buffered: Option<T>,
}

impl<T, I> IteratorItemReader<T, I>
where
    I: Iterator<Item = T> + Send + Sync,
{
    /// Create from iterator
    /// 从迭代器创建
    pub fn new(iter: I) -> Self {
        Self {
            iter: Some(iter),
            buffered: None,
        }
    }
}

#[async_trait]
impl<T, I> ItemReader for IteratorItemReader<T, I>
where
    T: Send + Sync,
    I: Iterator<Item = T> + Send + Sync,
{
    type Item = T;

    async fn read(&mut self) -> BatchResult<Option<T>> {
        if let Some(buffered) = self.buffered.take() {
            return Ok(Some(buffered));
        }

        if let Some(iter) = &mut self.iter {
            Ok(iter.next())
        } else {
            Ok(None)
        }
    }
}

/// CSV file reader
/// CSV文件读取器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public FlatFileItemReader<Person> reader() {
///     FlatFileItemReader<Person> reader = new FlatFileItemReader<>();
///     reader.setResource(new FileSystemResource("data.csv"));
///     reader.setLineMapper(lineMapper);
///     return reader;
/// }
/// ```
pub struct CsvFileReader {
    file_path: String,
    lines: Vec<String>,
    index: usize,
    has_header: bool,
}

impl CsvFileReader {
    /// Create new CSV reader
    /// 创建新CSV读取器
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            lines: Vec::new(),
            index: 0,
            has_header: true,
        }
    }

    /// Set whether file has header
    /// 设置文件是否有标题行
    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    /// Load file into memory
    /// 将文件加载到内存
    pub async fn load(&mut self) -> BatchResult<()> {
        let content = tokio::fs::read_to_string(&self.file_path).await?;
        self.lines = content.lines().map(|s| s.to_string()).collect();

        // Skip header if present
        if self.has_header && !self.lines.is_empty() {
            self.index = 1;
        }

        Ok(())
    }

    /// Parse CSV line
    /// 解析CSV行
    pub fn parse_line(line: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    in_quotes = !in_quotes;
                    // Check for escaped quote
                    if chars.peek() == Some(&'"') {
                        current.push('"');
                        chars.next();
                    }
                },
                ',' if !in_quotes => {
                    result.push(current.trim().to_string());
                    current = String::new();
                },
                _ => {
                    current.push(c);
                },
            }
        }

        result.push(current.trim().to_string());
        result
    }
}

#[async_trait]
impl ItemReader for CsvFileReader {
    type Item = String;

    async fn read(&mut self) -> BatchResult<Option<String>> {
        if self.index < self.lines.len() {
            let line = self.lines[self.index].clone();
            self.index += 1;
            Ok(Some(line))
        } else {
            Ok(None)
        }
    }

    async fn open(&mut self) -> BatchResult<()> {
        self.load().await?;
        Ok(())
    }
}

/// JSON file reader
/// JSON文件读取器
pub struct JsonFileReader {
    file_path: String,
    lines: Vec<String>,
    index: usize,
}

impl JsonFileReader {
    /// Create new JSON reader
    /// 创建新JSON读取器
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            lines: Vec::new(),
            index: 0,
        }
    }

    /// Load file into memory
    /// 将文件加载到内存
    pub async fn load(&mut self) -> BatchResult<()> {
        let content = tokio::fs::read_to_string(&self.file_path).await?;
        self.lines = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|s| s.to_string())
            .collect();
        Ok(())
    }
}

#[async_trait]
impl ItemReader for JsonFileReader {
    type Item = String;

    async fn read(&mut self) -> BatchResult<Option<String>> {
        if self.index < self.lines.len() {
            let line = self.lines[self.index].clone();
            self.index += 1;
            Ok(Some(line))
        } else {
            Ok(None)
        }
    }

    async fn open(&mut self) -> BatchResult<()> {
        self.load().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_item_stream_reader() {
        let items = vec![1, 2, 3, 4, 5];
        let mut reader = ItemStreamReader::new(items);

        assert!(reader.has_more());
        assert_eq!(reader.remaining(), 5);

        assert_eq!(reader.read().await.unwrap(), Some(1));
        assert_eq!(reader.read().await.unwrap(), Some(2));
        assert_eq!(reader.remaining(), 3);

        assert_eq!(reader.read().await.unwrap(), Some(3));
        assert_eq!(reader.read().await.unwrap(), Some(4));
        assert_eq!(reader.read().await.unwrap(), Some(5));
        assert_eq!(reader.read().await.unwrap(), None);
        assert!(!reader.has_more());
    }

    #[tokio::test]
    async fn test_item_stream_reader_empty() {
        let mut reader = ItemStreamReader::<i32>::empty();

        assert_eq!(reader.read().await.unwrap(), None);
        assert!(!reader.has_more());
    }

    #[tokio::test]
    async fn test_item_stream_reader_mark_reset() {
        let items = vec![1, 2, 3, 4, 5];
        let mut reader = ItemStreamReader::new(items);

        assert_eq!(reader.read().await.unwrap(), Some(1));
        assert_eq!(reader.read().await.unwrap(), Some(2));

        let mark = reader.mark();
        assert_eq!(mark, Some("2".to_string()));

        assert_eq!(reader.read().await.unwrap(), Some(3));
        assert_eq!(reader.read().await.unwrap(), Some(4));

        reader.reset().await.unwrap();
        assert_eq!(reader.read().await.unwrap(), Some(2));
        assert_eq!(reader.read().await.unwrap(), Some(3));
    }

    #[tokio::test]
    async fn test_iterator_item_reader() {
        let items = vec!["a", "b", "c"];
        let mut reader = IteratorItemReader::new(items.into_iter());

        assert_eq!(reader.read().await.unwrap(), Some("a"));
        assert_eq!(reader.read().await.unwrap(), Some("b"));
        assert_eq!(reader.read().await.unwrap(), Some("c"));
        assert_eq!(reader.read().await.unwrap(), None);
    }

    #[test]
    fn test_csv_parse_line() {
        let line = "John,Doe,30";
        let parsed = CsvFileReader::parse_line(line);
        assert_eq!(parsed, vec!["John", "Doe", "30"]);

        let line = "\"John, Jr.\",Doe,30";
        let parsed = CsvFileReader::parse_line(line);
        assert_eq!(parsed, vec!["John, Jr.", "Doe", "30"]);

        let line = "John,\"Doe, Jr.\",30";
        let parsed = CsvFileReader::parse_line(line);
        assert_eq!(parsed, vec!["John", "Doe, Jr.", "30"]);
    }

    #[tokio::test]
    async fn test_csv_reader_open_close() {
        let mut reader = CsvFileReader::new("test.csv").with_header(false);

        // Should not error on close without open
        reader.close().await.unwrap();
    }
}
