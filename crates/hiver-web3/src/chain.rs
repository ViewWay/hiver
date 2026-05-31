//! Chain module
//! 链模块
//!
//! # Overview / 概述
//!
//! This module provides blockchain abstraction supporting multiple EVM-compatible chains.
//!
//! 本模块提供支持多种EVM兼容链的区块链抽象。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Web3j (`ChainId`, `BlockchainService`)
//! - Web3j Chain Id and Block management
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_web3::chain::{Chain, ChainId, Eip155Chain};
//!
//! let mainnet = Eip155Chain::mainnet();
//! assert_eq!(mainnet.chain_id(), 1u64);
//! assert_eq!(mainnet.name(), "Ethereum Mainnet");
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;
use std::str::FromStr;

/// EIP-155 Chain ID
/// EIP-155 链ID
///
/// Unique identifier for each blockchain following EIP-155.
/// 遵循EIP-155的每个区块链的唯一标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Eip155Chain(pub u64);

impl Eip155Chain {
    /// Create a new EIP-155 chain ID
    /// 创建新的EIP-155链ID
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw chain ID value
    /// 获取原始链ID值
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Check if this is a mainnet
    /// 检查这是否是主网
    pub const fn is_mainnet(self) -> bool {
        self.0 == 1 || self.0 == 56 || self.0 == 137 || self.0 == 42161 || self.0 == 10
    }

    /// Check if this is a testnet
    /// 检查这是否是测试网
    pub const fn is_testnet(self) -> bool {
        !self.is_mainnet()
    }
}

impl fmt::Display for Eip155Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChainId({})", self.0)
    }
}

impl From<u64> for Eip155Chain {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<Eip155Chain> for u64 {
    fn from(chain: Eip155Chain) -> Self {
        chain.0
    }
}

impl FromStr for Eip155Chain {
    type Err = ChainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s
            .parse::<u64>()
            .map_err(|_| ChainError::InvalidChainId(s.to_string()))?;
        Ok(Self(id))
    }
}

/// Chain identifier enum
/// 链标识符枚举
///
/// Predefined chain identifiers for common blockchains.
/// 常见区块链的预定义链标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChainId {
    /// Ethereum Mainnet
    /// 以太坊主网
    Ethereum,

    /// Polygon `PoS` Chain
    /// Polygon `PoS链`
    Polygon,

    /// Binance Smart Chain
    /// 币安智能链
    Bsc,

    /// Arbitrum One
    /// Arbitrum One
    Arbitrum,

    /// Optimism
    /// Optimism
    Optimism,

    /// Base
    /// Base
    Base,

    /// Avalanche C-Chain
    /// Avalanche C链
    Avalanche,

    /// Fantom Opera
    /// Fantom Opera
    Fantom,

    /// Polygon zkEVM
    /// Polygon zkEVM
    PolygonZkEvm,

    /// Sepolia Testnet
    /// Sepolia测试网
    Sepolia,

    /// Goerli Testnet (deprecated)
    /// Goerli测试网（已弃用）
    Goerli,

    /// Custom chain
    /// 自定义链
    Custom(u64),
}

impl ChainId {
    /// Get the EIP-155 chain ID
    /// 获取EIP-155链ID
    pub const fn as_u64(self) -> u64 {
        match self {
            Self::Ethereum => 1,
            Self::Polygon => 137,
            Self::Bsc => 56,
            Self::Arbitrum => 42161,
            Self::Optimism => 10,
            Self::Base => 8453,
            Self::Avalanche => 43114,
            Self::Fantom => 250,
            Self::PolygonZkEvm => 1101,
            Self::Sepolia => 11_155_111,
            Self::Goerli => 5,
            Self::Custom(id) => id,
        }
    }

    /// Get chain name
    /// 获取链名称
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ethereum => "Ethereum Mainnet",
            Self::Polygon => "Polygon PoS",
            Self::Bsc => "BNB Chain",
            Self::Arbitrum => "Arbitrum One",
            Self::Optimism => "Optimism",
            Self::Base => "Base",
            Self::Avalanche => "Avalanche C-Chain",
            Self::Fantom => "Fantom Opera",
            Self::PolygonZkEvm => "Polygon zkEVM",
            Self::Sepolia => "Sepolia Testnet",
            Self::Goerli => "Goerli Testnet",
            Self::Custom(_) => "Custom Chain",
        }
    }

    /// Get short name
    /// 获取短名称
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Ethereum => "eth",
            Self::Polygon => "polygon",
            Self::Bsc => "bsc",
            Self::Arbitrum => "arbitrum",
            Self::Optimism => "optimism",
            Self::Base => "base",
            Self::Avalanche => "avalanche",
            Self::Fantom => "fantom",
            Self::PolygonZkEvm => "polygon-zkevm",
            Self::Sepolia => "sepolia",
            Self::Goerli => "goerli",
            Self::Custom(_) => "custom",
        }
    }

    /// Check if this is a mainnet
    /// 检查这是否是主网
    pub const fn is_mainnet(self) -> bool {
        !matches!(self, Self::Sepolia | Self::Goerli | Self::Custom(_))
    }

    /// Create from EIP-155 chain ID
    /// 从EIP-155链ID创建
    pub const fn from_eip155(id: u64) -> Self {
        match id {
            1 => Self::Ethereum,
            137 => Self::Polygon,
            56 => Self::Bsc,
            42161 => Self::Arbitrum,
            10 => Self::Optimism,
            8453 => Self::Base,
            43114 => Self::Avalanche,
            250 => Self::Fantom,
            1101 => Self::PolygonZkEvm,
            11_155_111 => Self::Sepolia,
            5 => Self::Goerli,
            _ => Self::Custom(id),
        }
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.as_u64())
    }
}

impl From<u64> for ChainId {
    fn from(id: u64) -> Self {
        Self::from_eip155(id)
    }
}

impl From<ChainId> for u64 {
    fn from(chain: ChainId) -> Self {
        chain.as_u64()
    }
}

impl From<Eip155Chain> for ChainId {
    fn from(chain: Eip155Chain) -> Self {
        Self::from_eip155(chain.0)
    }
}

/// Chain configuration
/// 链配置
///
/// Configuration for a blockchain network.
/// 区块链网络的配置。
#[derive(Debug, Clone)]
pub struct ChainConfig {
    /// Chain ID
    /// 链ID
    pub chain_id: Eip155Chain,

    /// Chain name
    /// 链名称
    pub name: String,

    /// RPC endpoints
    /// RPC端点
    pub rpc_urls: Vec<String>,

    /// WebSocket endpoints
    /// `WebSocket端点`
    pub ws_urls: Option<Vec<String>>,

    /// Explorer URL
    /// 浏览器URL
    pub explorer_url: Option<String>,

    /// Native currency symbol
    /// 原生货币符号
    pub native_currency: Currency,

    /// Block time in seconds
    /// 出块时间（秒）
    pub block_time: u64,

    /// Confirmation requirement
    /// 确认要求
    pub confirmations: u64,
}

/// Currency information
/// 货币信息
#[derive(Debug, Clone)]
pub struct Currency {
    /// Symbol (e.g., "ETH")
    /// 符号（例如"ETH"）
    pub symbol: String,

    /// Decimals for display
    /// 显示的小数位数
    pub decimals: u8,

    /// Name
    /// 名称
    pub name: String,
}

impl ChainConfig {
    /// Create a new chain config
    /// 创建新的链配置
    pub fn new(chain_id: impl Into<Eip155Chain>, name: impl Into<String>) -> Self {
        let chain_id = chain_id.into();
        Self {
            chain_id,
            name: name.into(),
            rpc_urls: Vec::new(),
            ws_urls: None,
            explorer_url: None,
            native_currency: Currency {
                symbol: "ETH".to_string(),
                decimals: 18,
                name: "Ether".to_string(),
            },
            block_time: 12,
            confirmations: 1,
        }
    }

    /// Add RPC URL
    /// 添加RPC URL
    pub fn with_rpc_url(mut self, url: impl Into<String>) -> Self {
        self.rpc_urls.push(url.into());
        self
    }

    /// Add multiple RPC URLs
    /// 添加多个RPC URL
    pub fn with_rpc_urls(mut self, urls: Vec<String>) -> Self {
        self.rpc_urls = urls;
        self
    }

    /// Set WebSocket URLs
    /// `设置WebSocket` URL
    pub fn with_ws_urls(mut self, urls: Vec<String>) -> Self {
        self.ws_urls = Some(urls);
        self
    }

    /// Set explorer URL
    /// 设置浏览器URL
    pub fn with_explorer(mut self, url: impl Into<String>) -> Self {
        self.explorer_url = Some(url.into());
        self
    }

    /// Set native currency
    /// 设置原生货币
    pub fn with_native_currency(
        mut self,
        symbol: impl Into<String>,
        decimals: u8,
        name: impl Into<String>,
    ) -> Self {
        self.native_currency = Currency {
            symbol: symbol.into(),
            decimals,
            name: name.into(),
        };
        self
    }

    /// Set block time
    /// 设置出块时间
    pub fn with_block_time(mut self, seconds: u64) -> Self {
        self.block_time = seconds;
        self
    }

    /// Set confirmation requirement
    /// 设置确认要求
    pub fn with_confirmations(mut self, count: u64) -> Self {
        self.confirmations = count;
        self
    }

    /// Get Ethereum mainnet config
    /// 获取以太坊主网配置
    pub fn ethereum_mainnet() -> Self {
        Self::new(1u64, "Ethereum Mainnet")
            .with_rpc_urls(vec![
                "https://eth.llamarpc.com".to_string(),
                "https://rpc.ankr.com/eth".to_string(),
            ])
            .with_explorer("https://etherscan.io")
            .with_native_currency("ETH", 18, "Ether")
            .with_block_time(12)
            .with_confirmations(1)
    }

    /// Get Sepolia testnet config
    /// 获取Sepolia测试网配置
    pub fn sepolia() -> Self {
        Self::new(11_155_111_u64, "Sepolia Testnet")
            .with_rpc_urls(vec![
                "https://rpc.sepolia.org".to_string(),
                "https://ethereum-sepolia-rpc.publicnode.com".to_string(),
            ])
            .with_explorer("https://sepolia.etherscan.io")
            .with_native_currency("ETH", 18, "Ether")
            .with_block_time(12)
            .with_confirmations(1)
    }

    /// Get Polygon config
    /// 获取Polygon配置
    pub fn polygon() -> Self {
        Self::new(137u64, "Polygon PoS")
            .with_rpc_urls(vec![
                "https://polygon-rpc.com".to_string(),
                "https://rpc.ankr.com/polygon".to_string(),
            ])
            .with_explorer("https://polygonscan.com")
            .with_native_currency("MATIC", 18, "Matic")
            .with_block_time(2)
            .with_confirmations(5)
    }

    /// Get Arbitrum config
    /// 获取Arbitrum配置
    pub fn arbitrum() -> Self {
        Self::new(42161u64, "Arbitrum One")
            .with_rpc_urls(vec![
                "https://arb1.arbitrum.io/rpc".to_string(),
                "https://rpc.ankr.com/arbitrum".to_string(),
            ])
            .with_explorer("https://arbiscan.io")
            .with_native_currency("ETH", 18, "Ether")
            .with_block_time(1)
            .with_confirmations(1)
    }

    /// Get Optimism config
    /// 获取Optimism配置
    pub fn optimism() -> Self {
        Self::new(10u64, "Optimism")
            .with_rpc_urls(vec![
                "https://mainnet.optimism.io".to_string(),
                "https://rpc.ankr.com/optimism".to_string(),
            ])
            .with_explorer("https://optimistic.etherscan.io")
            .with_native_currency("ETH", 18, "Ether")
            .with_block_time(2)
            .with_confirmations(1)
    }

    /// Get BSC config
    /// 获取BSC配置
    pub fn bsc() -> Self {
        Self::new(56u64, "BNB Chain")
            .with_rpc_urls(vec![
                "https://bsc-dataseed.binance.org".to_string(),
                "https://rpc.ankr.com/bsc".to_string(),
            ])
            .with_explorer("https://bscscan.com")
            .with_native_currency("BNB", 18, "BNB")
            .with_block_time(3)
            .with_confirmations(1)
    }
}

/// Chain error
/// 链错误
#[derive(Debug, Clone)]
pub enum ChainError {
    /// Invalid chain ID
    /// 无效的链ID
    InvalidChainId(String),

    /// RPC error
    /// RPC错误
    RpcError(String),

    /// Network error
    /// 网络错误
    NetworkError(String),

    /// Parse error
    /// 解析错误
    ParseError(String),

    /// Timeout
    /// 超时
    Timeout,
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChainId(id) => write!(f, "Invalid chain ID: {}", id),
            Self::RpcError(msg) => write!(f, "RPC error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::Timeout => write!(f, "Request timeout"),
        }
    }
}

impl std::error::Error for ChainError {}

/// Block information
/// 区块信息
#[derive(Debug, Clone)]
pub struct Block {
    /// Block hash
    /// 区块哈希
    pub hash: String,

    /// Parent hash
    /// 父哈希
    pub parent_hash: String,

    /// Block number
    /// 区块号
    pub number: u64,

    /// Timestamp
    /// 时间戳
    pub timestamp: u64,

    /// Gas limit
    /// Gas限制
    pub gas_limit: u64,

    /// Gas used
    /// 已用Gas
    pub gas_used: u64,

    /// Transaction count
    /// 交易数量
    pub transaction_count: usize,
}

/// Block number or tag
/// 区块号或标签
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockNumber {
    /// Latest block
    /// 最新区块
    Latest,

    /// Pending block
    /// 待处理区块
    Pending,

    /// Specific block number
    /// 特定区块号
    Number(u64),
}

impl fmt::Display for BlockNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Latest => write!(f, "latest"),
            Self::Pending => write!(f, "pending"),
            Self::Number(n) => write!(f, "{n}"),
        }
    }
}

impl From<u64> for BlockNumber {
    fn from(n: u64) -> Self {
        Self::Number(n)
    }
}

impl From<Option<u64>> for BlockNumber {
    fn from(opt: Option<u64>) -> Self {
        match opt {
            Some(n) => Self::Number(n),
            None => Self::Latest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eip155_chain_new() {
        let chain = Eip155Chain::new(1);
        assert_eq!(chain.as_u64(), 1);
    }

    #[test]
    fn test_eip155_chain_from_u64() {
        let chain = Eip155Chain::from(137u64);
        assert_eq!(chain.as_u64(), 137);
    }

    #[test]
    fn test_eip155_chain_display() {
        let chain = Eip155Chain::new(1);
        assert_eq!(chain.to_string(), "ChainId(1)");
    }

    #[test]
    fn test_chain_id_as_u64() {
        assert_eq!(ChainId::Ethereum.as_u64(), 1);
        assert_eq!(ChainId::Polygon.as_u64(), 137);
        assert_eq!(ChainId::Bsc.as_u64(), 56);
        assert_eq!(ChainId::Arbitrum.as_u64(), 42161);
        assert_eq!(ChainId::Optimism.as_u64(), 10);
    }

    #[test]
    fn test_chain_id_from_u64() {
        assert_eq!(ChainId::from(1u64), ChainId::Ethereum);
        assert_eq!(ChainId::from(137u64), ChainId::Polygon);
        assert_eq!(ChainId::from(56u64), ChainId::Bsc);
        assert_eq!(ChainId::from(999u64), ChainId::Custom(999));
    }

    #[test]
    fn test_chain_id_name() {
        assert_eq!(ChainId::Ethereum.name(), "Ethereum Mainnet");
        assert_eq!(ChainId::Polygon.name(), "Polygon PoS");
        assert_eq!(ChainId::Bsc.name(), "BNB Chain");
    }

    #[test]
    fn test_chain_id_is_mainnet() {
        assert!(ChainId::Ethereum.is_mainnet());
        assert!(ChainId::Polygon.is_mainnet());
        assert!(!ChainId::Sepolia.is_mainnet());
        assert!(!ChainId::Goerli.is_mainnet());
    }

    #[test]
    fn test_chain_config_builder() {
        let config = ChainConfig::new(1u64, "Test Chain")
            .with_rpc_url("https://rpc.example.com")
            .with_explorer("https://explorer.example.com")
            .with_native_currency("TEST", 18, "Test Token")
            .with_block_time(5)
            .with_confirmations(3);

        assert_eq!(config.chain_id.as_u64(), 1);
        assert_eq!(config.name, "Test Chain");
        assert_eq!(config.rpc_urls.len(), 1);
        assert_eq!(config.explorer_url, Some("https://explorer.example.com".to_string()));
        assert_eq!(config.native_currency.symbol, "TEST");
        assert_eq!(config.block_time, 5);
        assert_eq!(config.confirmations, 3);
    }

    #[test]
    fn test_chain_config_ethereum() {
        let config = ChainConfig::ethereum_mainnet();
        assert_eq!(config.chain_id.as_u64(), 1);
        assert!(!config.rpc_urls.is_empty());
        assert_eq!(config.native_currency.symbol, "ETH");
    }

    #[test]
    fn test_chain_config_polygon() {
        let config = ChainConfig::polygon();
        assert_eq!(config.chain_id.as_u64(), 137);
        assert_eq!(config.native_currency.symbol, "MATIC");
    }

    #[test]
    fn test_chain_config_arbitrum() {
        let config = ChainConfig::arbitrum();
        assert_eq!(config.chain_id.as_u64(), 42161);
        assert_eq!(config.block_time, 1);
    }

    #[test]
    fn test_block_number_display() {
        assert_eq!(BlockNumber::Latest.to_string(), "latest");
        assert_eq!(BlockNumber::Pending.to_string(), "pending");
        assert_eq!(BlockNumber::Number(12345).to_string(), "12345");
    }

    #[test]
    fn test_block_number_from_u64() {
        let block: BlockNumber = 100u64.into();
        assert_eq!(block, BlockNumber::Number(100));
    }

    #[test]
    fn test_chain_error_display() {
        let err = ChainError::InvalidChainId("test".to_string());
        assert!(err.to_string().contains("Invalid chain ID"));

        let err = ChainError::RpcError("connection failed".to_string());
        assert!(err.to_string().contains("RPC error"));
    }
}
