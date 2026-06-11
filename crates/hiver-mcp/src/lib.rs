//! # Hiver MCP — Model Context Protocol
//!
//! Production-grade MCP server & client for the Hiver framework.
//! Implements MCP specification version `2025-03-26` over JSON-RPC 2.0.
//!
//! Hiver 框架的模型上下文协议(MCP)实现，支持 `2025-03-26` 版本规范。
//!
//! # Design Patterns / 设计模式
//!
//! | Pattern       | Application                                    |
//! |---------------|------------------------------------------------|
//! | Strategy      | `Transport` trait — stdio / HTTP swappable     |
//! | State         | `ServerLifecycle` — state-machine lifecycle    |
//! | Builder       | `McpServerBuilder` / client config             |
//! | Registry      | Separate registries for tools/resources/prompts|
//! | Adapter       | `HiverToolAdapter` ↔ hiver-ai `ToolCallback`  |
//! | Command       | Method dispatch on incoming JSON-RPC requests  |
//! | Observer      | Notification callbacks for list changes        |
//!
//! # Quick Start — MCP Server / 快速开始 — MCP 服务器
//!
//! ```rust,ignore
//! use hiver_mcp::{McpServer, StdioTransport, Tool, CallToolResult};
//!
//! let server = McpServer::builder("my-server", "1.0.0")
//!     .tool("echo", "Echo input", echo_handler)
//!     .build();
//!
//! let transport = StdioTransport::new();
//! server.run(transport).await?;
//! ```
//!
//! # Quick Start — MCP Client / 快速开始 — MCP 客户端
//!
//! ```rust,ignore
//! use hiver_mcp::{McpClient, ChildProcessTransport};
//!
//! let transport = ChildProcessTransport::new("npx", &["-y", "@anthropic/mcp-server"]);
//! let client = McpClient::connect(transport).await?;
//! let tools = client.list_tools().await?;
//! ```

#![warn(missing_docs)]
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

pub mod adapter;
pub mod client;
pub mod error;
pub mod lifecycle;
pub mod message;
pub mod registry;
pub mod server;
pub mod transport;
pub mod types;

// Re-export primary types / 重新导出主要类型
pub use adapter::{HiverToolAdapter, McpToolBridge};
pub use client::McpClient;
pub use error::McpError;
pub use lifecycle::ServerLifecycle;
pub use message::{
    parse_message, JsonRpcError, JsonRpcId, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse,
};
pub use registry::{
    McpPromptProvider, McpResourceProvider, McpToolHandler, PromptRegistry, ResourceRegistry,
    ToolRegistry,
};
pub use server::McpServer;
pub use transport::{ChildProcessTransport, StdioTransport, Transport};
pub use types::{
    CallToolResult, ClientCapabilities, Content, EmbeddedResource, GetPromptResult,
    Implementation, InitializeResult, Prompt, PromptArgument, PromptMessage, PromptRole,
    ReadResourceResult, Resource, ResourceContent, RootsCapability, SamplingCapability,
    ServerCapabilities, Tool,
};
