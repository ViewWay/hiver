//! RPC client module
//! RPC客户端模块
//!
//! # Overview / 概述
//!
//! This module provides RPC client functionality for interacting with
//! EVM-compatible blockchain nodes via HTTP and WebSocket.
//!
//! 本模块提供RPC客户端功能，用于通过HTTP和WebSocket与EVM兼容的区块链节点交互。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Web3j (Web3j, HttpService, WebSocketService)
//! - Web3j RPC client
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_web3::rpc::RpcClient;
//! use nexus_web3::chain::ChainConfig;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ChainConfig::ethereum_mainnet();
//! let client = RpcClient::new(config.rpc_urls[0].clone())?;
//!
//! // Get the latest block number
//! let block_number = client.get_block_number().await?;
//! println!("Latest block: {}", block_number);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::chain::{Block, BlockNumber};
use crate::tx::TxHash;
use crate::wallet::Address;

/// RPC error
/// RPC错误
#[derive(Debug, Clone)]
pub enum RpcError {
    /// HTTP error
    /// HTTP错误
    HttpError(u16),

    /// Network error
    /// 网络错误
    NetworkError(String),

    /// Parse error
    /// 解析错误
    ParseError(String),

    /// RPC error
    /// RPC错误
    RpcError(String),

    /// HTTP client error
    /// HTTP客户端错误
    HttpClientError(String),
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpError(code) => write!(f, "HTTP error: {}", code),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::RpcError(msg) => write!(f, "RPC error: {}", msg),
            Self::HttpClientError(msg) => write!(f, "HTTP client error: {}", msg),
        }
    }
}

impl std::error::Error for RpcError {}

/// RPC client
/// RPC客户端
///
/// Client for making JSON-RPC calls to blockchain nodes.
/// 用于向区块链节点发起JSON-RPC调用的客户端。
///
/// This is available only when the `rpc` feature is enabled.
/// 仅当启用`rpc`功能时可用。
#[cfg(feature = "rpc")]
#[derive(Clone)]
pub struct RpcClient {
    /// RPC endpoint URL
    /// RPC端点URL
    url: String,

    /// HTTP client
    /// HTTP客户端
    client: Arc<reqwest::Client>,
}

#[cfg(feature = "rpc")]
use std::sync::Arc;

#[cfg(feature = "rpc")]
impl RpcClient {
    /// Create a new RPC client
    /// 创建新的RPC客户端
    pub fn new(url: impl Into<String>) -> Result<Self, RpcError> {
        let url = url.into();
        let client = Arc::new(
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| RpcError::HttpClientError(e.to_string()))?,
        );

        Ok(Self { url, client })
    }

    /// Create with custom timeout
    /// 使用自定义超时创建
    pub fn with_timeout(url: impl Into<String>, timeout_secs: u64) -> Result<Self, RpcError> {
        let url = url.into();
        let client = Arc::new(
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .build()
                .map_err(|e| RpcError::HttpClientError(e.to_string()))?,
        );

        Ok(Self { url, client })
    }

    /// Get the latest block number
    /// 获取最新区块号
    pub async fn get_block_number(&self) -> Result<u64, RpcError> {
        let response: JsonRpcResponse<u64> = self.call("eth_blockNumber", &[]).await?;
        Ok(response.result)
    }

    /// Get block by number
    /// 根据区块号获取区块
    pub async fn get_block(&self, number: BlockNumber) -> Result<Block, RpcError> {
        let param = match number {
            BlockNumber::Latest => serde_json::json!("latest"),
            BlockNumber::Pending => serde_json::json!("pending"),
            BlockNumber::Number(n) => serde_json::json!(format!("0x{:x}", n)),
        };

        let response: JsonRpcResponse<RpcBlock> =
            self.call("eth_getBlockByNumber", &[param]).await?;
        Ok(response.result.into())
    }

    /// Get balance for an address
    /// 获取地址余额
    pub async fn get_balance(
        &self,
        address: &Address,
        block: BlockNumber,
    ) -> Result<String, RpcError> {
        let block_param = match block {
            BlockNumber::Latest => "latest".to_string(),
            BlockNumber::Pending => "pending".to_string(),
            BlockNumber::Number(n) => format!("0x{:x}", n),
        };

        let response: JsonRpcResponse<String> = self
            .call(
                "eth_getBalance",
                &[
                    serde_json::json!(address.to_hex()),
                    serde_json::json!(block_param),
                ],
            )
            .await?;
        Ok(response.result)
    }

    /// Get transaction count (nonce) for an address
    /// 获取地址的交易计数（nonce）
    pub async fn get_transaction_count(
        &self,
        address: &Address,
        block: BlockNumber,
    ) -> Result<u64, RpcError> {
        let block_param = match block {
            BlockNumber::Latest => "latest".to_string(),
            BlockNumber::Pending => "pending".to_string(),
            BlockNumber::Number(n) => format!("0x{:x}", n),
        };

        let response: JsonRpcResponse<String> = self
            .call(
                "eth_getTransactionCount",
                &[
                    serde_json::json!(address.to_hex()),
                    serde_json::json!(block_param),
                ],
            )
            .await?;

        // Parse hex string to u64
        let hex = response
            .result
            .strip_prefix("0x")
            .unwrap_or(&response.result);
        u64::from_str_radix(hex, 16).map_err(|_| RpcError::ParseError("Invalid nonce".to_string()))
    }

    /// Get transaction by hash
    /// 根据哈希获取交易
    pub async fn get_transaction(&self, hash: &TxHash) -> Result<RpcTransaction, RpcError> {
        let response: JsonRpcResponse<RpcTransaction> = self
            .call("eth_getTransactionByHash", &[serde_json::json!(hash.to_hex())])
            .await?;
        Ok(response.result)
    }

    /// Send raw transaction
    /// 发送原始交易
    pub async fn send_raw_transaction(&self, bytes: &[u8]) -> Result<TxHash, RpcError> {
        let hex = format!("0x{}", hex::encode(bytes));
        let response: JsonRpcResponse<String> = self
            .call("eth_sendRawTransaction", &[serde_json::json!(hex)])
            .await?;
        TxHash::from_hex(&response.result).map_err(|e| RpcError::ParseError(e.to_string()))
    }

    /// Call a contract method (read-only)
    /// 调用合约方法（只读）
    pub async fn call_contract(
        &self,
        to: &Address,
        data: &[u8],
        block: BlockNumber,
    ) -> Result<String, RpcError> {
        let block_param = match block {
            BlockNumber::Latest => "latest".to_string(),
            BlockNumber::Pending => "pending".to_string(),
            BlockNumber::Number(n) => format!("0x{:x}", n),
        };

        let hex_data = format!("0x{}", hex::encode(data));
        let response: JsonRpcResponse<String> = self
            .call(
                "eth_call",
                &[
                    serde_json::json!({
                        "to": to.to_hex(),
                        "data": hex_data
                    }),
                    serde_json::json!(block_param),
                ],
            )
            .await?;
        Ok(response.result)
    }

    /// Estimate gas for a transaction
    /// 估算交易的Gas
    pub async fn estimate_gas(
        &self,
        to: Option<&Address>,
        from: Option<&Address>,
        value: Option<&str>,
        data: Option<&[u8]>,
    ) -> Result<u64, RpcError> {
        let mut call_data = serde_json::Map::new();

        if let Some(to) = to {
            call_data.insert("to".to_string(), serde_json::json!(to.to_hex()));
        }
        if let Some(from) = from {
            call_data.insert("from".to_string(), serde_json::json!(from.to_hex()));
        }
        if let Some(value) = value {
            call_data.insert("value".to_string(), serde_json::json!(value));
        }
        if let Some(data) = data {
            call_data
                .insert("data".to_string(), serde_json::json!(format!("0x{}", hex::encode(data))));
        }

        let response: JsonRpcResponse<String> = self
            .call("eth_estimateGas", &[serde_json::json!(call_data), serde_json::json!("latest")])
            .await?;

        let hex = response
            .result
            .strip_prefix("0x")
            .unwrap_or(&response.result);
        u64::from_str_radix(hex, 16)
            .map_err(|_| RpcError::ParseError("Invalid gas estimate".to_string()))
    }

    /// Get gas price
    /// 获取Gas价格
    pub async fn get_gas_price(&self) -> Result<String, RpcError> {
        let response: JsonRpcResponse<String> = self.call("eth_gasPrice", &[]).await?;
        Ok(response.result)
    }

    /// Get chain ID
    /// 获取链ID
    pub async fn get_chain_id(&self) -> Result<u64, RpcError> {
        let response: JsonRpcResponse<String> = self.call("eth_chainId", &[]).await?;

        let hex = response
            .result
            .strip_prefix("0x")
            .unwrap_or(&response.result);
        u64::from_str_radix(hex, 16)
            .map_err(|_| RpcError::ParseError("Invalid chain ID".to_string()))
    }

    /// Make a raw RPC call
    /// 发起原始RPC调用
    pub async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: &[serde_json::Value],
    ) -> Result<JsonRpcResponse<T>, RpcError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method,
            params: params.to_vec(),
        };

        let resp = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| RpcError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RpcError::HttpError(resp.status().as_u16()));
        }

        let response: JsonRpcResponseRaw = resp
            .json()
            .await
            .map_err(|e| RpcError::ParseError(e.to_string()))?;

        if let Some(error) = response.error {
            return Err(RpcError::RpcError(error.message.unwrap_or_default()));
        }

        // Parse the result
        let result = serde_json::from_value(response.result.unwrap_or_default())
            .map_err(|e| RpcError::ParseError(e.to_string()))?;

        Ok(JsonRpcResponse { result })
    }
}

#[cfg(feature = "rpc")]
impl fmt::Debug for RpcClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RpcClient").field("url", &self.url).finish()
    }
}

/// RPC request
/// RPC请求
#[derive(Debug, Serialize)]
struct JsonRpcRequest<'a> {
    /// JSON-RPC version
    jsonrpc: &'a str,
    /// Request ID
    id: u64,
    /// Method name
    method: &'a str,
    /// Method parameters
    params: Vec<serde_json::Value>,
}

/// Raw RPC response
/// 原始RPC响应
#[derive(Debug, Deserialize)]
struct JsonRpcResponseRaw {
    /// Error (if any)
    #[serde(default)]
    error: Option<RpcErrorObject>,
    /// Result (if successful)
    result: Option<serde_json::Value>,
}

/// RPC response
/// RPC响应
#[derive(Debug)]
pub struct JsonRpcResponse<T> {
    /// Result data
    pub result: T,
}

/// RPC error object
/// RPC错误对象
#[derive(Debug, Deserialize)]
struct RpcErrorObject {
    /// Error code
    code: i64,
    /// Error message
    message: Option<String>,
}

/// RPC block (from eth_getBlockByNumber)
/// RPC区块（来自eth_getBlockByNumber）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RpcBlock {
    /// Block hash
    hash: Option<String>,
    /// Parent hash
    parent_hash: String,
    /// Block number
    number: Option<String>,
    /// Timestamp
    timestamp: String,
    /// Gas limit
    gas_limit: String,
    /// Gas used
    gas_used: String,
    /// Transaction count
    transaction_count: Option<String>,
}

impl From<RpcBlock> for Block {
    fn from(block: RpcBlock) -> Self {
        Self {
            hash: block.hash.unwrap_or_default(),
            parent_hash: block.parent_hash,
            number: block
                .number
                .and_then(|s| {
                    s.strip_prefix("0x")
                        .and_then(|h| u64::from_str_radix(h, 16).ok())
                })
                .unwrap_or(0),
            timestamp: block
                .timestamp
                .strip_prefix("0x")
                .and_then(|h| u64::from_str_radix(h, 16).ok())
                .unwrap_or(0),
            gas_limit: block
                .gas_limit
                .strip_prefix("0x")
                .and_then(|h| u64::from_str_radix(h, 16).ok())
                .unwrap_or(0),
            gas_used: block
                .gas_used
                .strip_prefix("0x")
                .and_then(|h| u64::from_str_radix(h, 16).ok())
                .unwrap_or(0),
            transaction_count: block
                .transaction_count
                .and_then(|s| {
                    s.strip_prefix("0x")
                        .and_then(|h| usize::from_str_radix(h, 16).ok())
                })
                .unwrap_or(0),
        }
    }
}

/// RPC transaction (from eth_getTransactionByHash)
/// RPC交易（来自eth_getTransactionByHash）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransaction {
    /// Transaction hash
    pub hash: String,
    /// From address
    pub from: String,
    /// To address (None for contract creation)
    pub to: Option<String>,
    /// Transaction index
    pub transaction_index: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Value
    pub value: String,
    /// Gas price
    pub gas_price: Option<String>,
    /// Gas limit
    pub gas: String,
    /// Input data
    pub input: String,
    /// Nonce
    pub nonce: String,
    /// Chain ID
    pub chain_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_error_display() {
        let err = RpcError::HttpError(404);
        assert_eq!(err.to_string(), "HTTP error: 404");

        let err = RpcError::NetworkError("connection refused".to_string());
        assert!(err.to_string().contains("Network error"));
    }

    #[cfg(feature = "rpc")]
    #[test]
    fn test_rpc_client_new() {
        let client = RpcClient::new("https://eth.llamarpc.com");
        assert!(client.is_ok());
    }

    #[cfg(feature = "rpc")]
    #[test]
    fn test_rpc_client_debug() {
        let client = RpcClient::new("https://eth.llamarpc.com").unwrap();
        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("RpcClient"));
        assert!(debug_str.contains("https://eth.llamarpc.com"));
    }
}
