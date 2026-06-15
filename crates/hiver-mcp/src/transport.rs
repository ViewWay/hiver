//! Transport abstraction (Strategy Pattern) — stdio and child-process transports.
//! 传输抽象（策略模式）— stdio 和子进程传输。

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};

use crate::error::McpError;

/// Maximum message size in bytes (4 MB). Prevents OOM from unbounded messages.
/// 最大消息大小（4 MB）。防止无限制消息导致 OOM。
const MAX_MESSAGE_SIZE: usize = 4 * 1024 * 1024;

// ============================================================
// Transport Trait / Transport Trait（策略模式）
// ============================================================

/// Transport abstraction for MCP message exchange.
/// MCP 消息交换的传输抽象。
///
/// Implementations choose the underlying I/O strategy (stdio, child process, HTTP, etc.).
/// Messages are newline-delimited JSON strings per MCP stdio spec.
#[async_trait]
pub trait Transport: Send
{
    /// Send a JSON-RPC message (appends newline).
    /// 发送 JSON-RPC 消息（追加换行符）。
    async fn send(&mut self, message: &str) -> Result<(), McpError>;

    /// Receive a JSON-RPC message. Returns `None` when the transport is closed.
    /// 接收 JSON-RPC 消息。传输关闭时返回 `None`。
    async fn receive(&mut self) -> Result<Option<String>, McpError>;

    /// Close the transport gracefully.
    /// 优雅关闭传输。
    async fn close(&mut self) -> Result<(), McpError>;
}

// ============================================================
// Stdio Transport (Server-side) / Stdio 传输（服务器端）
// ============================================================

/// Stdio transport — reads from stdin, writes to stdout.
/// Stdio 传输 — 从 stdin 读取，向 stdout 写入。
///
/// Used by MCP servers launched as child processes
/// (the standard pattern for local MCP servers).
pub struct StdioTransport
{
    lines: Lines<BufReader<tokio::io::Stdin>>,
    stdout: tokio::io::Stdout,
}

impl std::fmt::Debug for StdioTransport
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("StdioTransport").finish_non_exhaustive()
    }
}

impl StdioTransport
{
    /// Creates a new stdio transport.
    /// 创建新的 stdio 传输。
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            lines: BufReader::new(tokio::io::stdin()).lines(),
            stdout: tokio::io::stdout(),
        }
    }
}

impl Default for StdioTransport
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport
{
    async fn send(&mut self, message: &str) -> Result<(), McpError>
    {
        self.stdout.write_all(message.as_bytes()).await?;
        self.stdout.write_all(b"\n").await?;
        self.stdout.flush().await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>, McpError>
    {
        let line = self
            .lines
            .next_line()
            .await
            .map_err(|e| McpError::TransportError(e.to_string()))?;

        match line
        {
            Some(l) if l.len() > MAX_MESSAGE_SIZE => Err(McpError::TransportError(format!(
                "Message too large: {} bytes (max {MAX_MESSAGE_SIZE})",
                l.len()
            ))),
            other => Ok(other),
        }
    }

    async fn close(&mut self) -> Result<(), McpError>
    {
        let _ = self.stdout.flush().await;
        Ok(())
    }
}

// ============================================================
// Child Process Transport (Client-side) / 子进程传输（客户端）
// ============================================================

/// Child-process transport — spawns an MCP server and communicates via its stdin/stdout.
/// 子进程传输 — 启动 MCP 服务器并通过其 stdin/stdout 通信。
pub struct ChildProcessTransport
{
    child: Option<tokio::process::Child>,
    stdin: Option<tokio::process::ChildStdin>,
    lines: Lines<BufReader<tokio::process::ChildStdout>>,
}

impl std::fmt::Debug for ChildProcessTransport
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("ChildProcessTransport")
            .finish_non_exhaustive()
    }
}

impl ChildProcessTransport
{
    /// Spawns the MCP server as a child process.
    /// 将 MCP 服务器作为子进程启动。
    ///
    /// # Arguments
    /// - `program`: The executable to run (e.g., `"npx"`, `"python"`).
    /// - `args`: Command-line arguments.
    pub async fn spawn(program: &str, args: &[&str]) -> Result<Self, McpError>
    {
        let mut child = tokio::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| McpError::TransportError(format!("Failed to spawn {program}: {e}")))?;

        let child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::TransportError("Failed to open child stdin".into()))?;
        let child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::TransportError("Failed to open child stdout".into()))?;

        Ok(Self {
            child: Some(child),
            stdin: Some(child_stdin),
            lines: BufReader::new(child_stdout).lines(),
        })
    }
}

#[async_trait]
impl Transport for ChildProcessTransport
{
    async fn send(&mut self, message: &str) -> Result<(), McpError>
    {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| McpError::TransportError("Child stdin closed".into()))?;
        stdin.write_all(message.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>, McpError>
    {
        let line = self
            .lines
            .next_line()
            .await
            .map_err(|e| McpError::TransportError(e.to_string()))?;

        match line
        {
            Some(l) if l.len() > MAX_MESSAGE_SIZE => Err(McpError::TransportError(format!(
                "Message too large: {} bytes (max {MAX_MESSAGE_SIZE})",
                l.len()
            ))),
            other => Ok(other),
        }
    }

    async fn close(&mut self) -> Result<(), McpError>
    {
        if let Some(mut stdin) = self.stdin.take()
        {
            let _ = stdin.shutdown().await;
        }
        if let Some(mut child) = self.child.take()
        {
            let _ = child.wait().await;
        }
        Ok(())
    }
}

// ============================================================
// In-Memory Transport (for testing) / 内存传输（用于测试）
// ============================================================

/// In-memory transport backed by duplex streams. Create with [`MemoryTransport::pair()`].
/// 基于双工流的内存传输。使用 [`MemoryTransport::pair()`] 创建。
pub struct MemoryTransport
{
    write: tokio::io::DuplexStream,
    lines: Lines<BufReader<tokio::io::DuplexStream>>,
}

impl std::fmt::Debug for MemoryTransport
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("MemoryTransport").finish_non_exhaustive()
    }
}

impl MemoryTransport
{
    /// Creates a connected pair of in-memory transports.
    /// 创建一对已连接的内存传输。
    ///
    /// Data sent by `a` is received by `b`, and vice versa.
    pub fn pair() -> (Self, Self)
    {
        // a_write → b_read, b_write → a_read
        let (a_write, b_read) = tokio::io::duplex(4096);
        let (b_write, a_read) = tokio::io::duplex(4096);

        let a = Self {
            write: a_write,
            lines: BufReader::new(a_read).lines(),
        };
        let b = Self {
            write: b_write,
            lines: BufReader::new(b_read).lines(),
        };
        (a, b)
    }
}

#[async_trait]
impl Transport for MemoryTransport
{
    async fn send(&mut self, message: &str) -> Result<(), McpError>
    {
        self.write.write_all(message.as_bytes()).await?;
        self.write.write_all(b"\n").await?;
        self.write.flush().await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>, McpError>
    {
        let line = self
            .lines
            .next_line()
            .await
            .map_err(|e| McpError::TransportError(e.to_string()))?;

        match line
        {
            Some(l) if l.len() > MAX_MESSAGE_SIZE => Err(McpError::TransportError(format!(
                "Message too large: {} bytes (max {MAX_MESSAGE_SIZE})",
                l.len()
            ))),
            other => Ok(other),
        }
    }

    async fn close(&mut self) -> Result<(), McpError>
    {
        let _ = self.write.shutdown().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_memory_transport_pair()
    {
        let (mut a, mut b) = MemoryTransport::pair();
        a.send(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#)
            .await
            .unwrap();
        let received = b.receive().await.unwrap().unwrap();
        assert!(received.contains("ping"));
    }

    #[tokio::test]
    async fn test_memory_transport_bidirectional()
    {
        let (mut a, mut b) = MemoryTransport::pair();
        a.send("hello from a").await.unwrap();
        b.send("hello from b").await.unwrap();

        assert_eq!(b.receive().await.unwrap().unwrap(), "hello from a");
        assert_eq!(a.receive().await.unwrap().unwrap(), "hello from b");
    }

    #[tokio::test]
    async fn test_memory_transport_close()
    {
        let (mut a, mut b) = MemoryTransport::pair();
        a.send("last message").await.unwrap();
        a.close().await.unwrap();
        // b should still receive the last message
        assert_eq!(b.receive().await.unwrap().unwrap(), "last message");
        // Next receive returns None (closed)
        assert!(b.receive().await.unwrap().is_none());
    }

    #[test]
    fn test_debug_impls()
    {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<StdioTransport>();
        assert_debug::<ChildProcessTransport>();
        assert_debug::<MemoryTransport>();
    }
}
