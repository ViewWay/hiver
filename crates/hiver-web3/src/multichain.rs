//! Multi-chain support module
//! 多链支持模块
//!
//! # Overview / 概述
//!
//! This module provides a registry of pre-configured blockchain networks,
//! a cross-chain bridge interface, and an EIP-1559 gas price oracle.
//!
//! 本模块提供预配置的区块链网络注册表、跨链桥接口和EIP-1559 Gas价格预言机。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Cloud service discovery for multi-chain management
//! - Cross-chain bridge abstraction
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_web3::multichain::ChainRegistry;
//!
//! let registry = ChainRegistry::with_defaults();
//! let eth = registry.get_by_chain_id(1).unwrap();
//! println!("{}: {}", eth.name, eth.chain_id);
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{collections::HashMap, fmt};

use crate::{chain::ChainConfig, wallet::Address};

// ---------------------------------------------------------------------------
// ChainRegistry
// ---------------------------------------------------------------------------

/// Registry of blockchain network configurations.
/// 区块链网络配置注册表。
///
/// Stores chain configurations keyed by their EIP-155 chain ID and provides
/// lookup by chain ID or name.
///
/// 以EIP-155链ID为键存储链配置，并支持按链ID或名称查找。
#[derive(Debug, Clone)]
pub struct ChainRegistry {
    /// Chain configurations keyed by chain ID.
    /// 以链ID为键的链配置。
    chains: HashMap<u64, ChainEntry>,
}

/// A single chain entry in the registry.
/// 注册表中的单条链记录。
#[derive(Debug, Clone)]
pub struct ChainEntry {
    /// Chain configuration.
    /// 链配置。
    pub config: ChainConfig,
    /// Chain enum identifier (for convenience).
    /// 链枚举标识符（便于使用）。
    pub chain: Chain,
    /// Block explorer transaction URL template (e.g. `https://etherscan.io/tx/`).
    /// 区块浏览器交易URL模板（例如 `https://etherscan.io/tx/`）。
    pub explorer_tx_url: String,
}

/// Predefined chain identifiers.
/// 预定义的链标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    /// Ethereum Mainnet.
    /// 以太坊主网。
    Ethereum,
    /// Polygon PoS.
    /// Polygon PoS。
    Polygon,
    /// BNB Smart Chain.
    /// BNB智能链。
    Bsc,
    /// Arbitrum One.
    /// Arbitrum One。
    Arbitrum,
    /// Optimism.
    /// Optimism。
    Optimism,
    /// Avalanche C-Chain.
    /// Avalanche C链。
    Avalanche,
    /// Base.
    /// Base。
    Base,
    /// zkSync Era.
    /// zkSync Era。
    ZkSync,
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ethereum => write!(f, "Ethereum"),
            Self::Polygon => write!(f, "Polygon"),
            Self::Bsc => write!(f, "BSC"),
            Self::Arbitrum => write!(f, "Arbitrum"),
            Self::Optimism => write!(f, "Optimism"),
            Self::Avalanche => write!(f, "Avalanche"),
            Self::Base => write!(f, "Base"),
            Self::ZkSync => write!(f, "zkSync"),
        }
    }
}

/// Native currency descriptor.
/// 原生货币描述符。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCurrency {
    /// Symbol (e.g. "ETH").
    /// 符号（例如"ETH"）。
    pub symbol: &'static str,
    /// Decimal places.
    /// 小数位数。
    pub decimals: u8,
    /// Full name.
    /// 全名。
    pub name: &'static str,
}

impl Chain {
    /// Get the EIP-155 chain ID for this chain.
    /// 获取此链的EIP-155链ID。
    pub const fn chain_id(self) -> u64 {
        match self {
            Self::Ethereum => 1,
            Self::Polygon => 137,
            Self::Bsc => 56,
            Self::Arbitrum => 42161,
            Self::Optimism => 10,
            Self::Avalanche => 43114,
            Self::Base => 8453,
            Self::ZkSync => 324,
        }
    }

    /// Get the native currency for this chain.
    /// 获取此链的原生货币。
    pub const fn native_currency(self) -> NativeCurrency {
        match self {
            Self::Ethereum | Self::Arbitrum | Self::Optimism | Self::Base | Self::ZkSync => {
                NativeCurrency {
                    symbol: "ETH",
                    decimals: 18,
                    name: "Ether",
                }
            },
            Self::Polygon => NativeCurrency {
                symbol: "MATIC",
                decimals: 18,
                name: "Matic",
            },
            Self::Bsc => NativeCurrency {
                symbol: "BNB",
                decimals: 18,
                name: "BNB",
            },
            Self::Avalanche => NativeCurrency {
                symbol: "AVAX",
                decimals: 18,
                name: "Avalanche",
            },
        }
    }

    /// Get the default RPC URL for this chain.
    /// 获取此链的默认RPC URL。
    pub const fn default_rpc_url(self) -> &'static str {
        match self {
            Self::Ethereum => "https://eth.llamarpc.com",
            Self::Polygon => "https://polygon-rpc.com",
            Self::Bsc => "https://bsc-dataseed.binance.org",
            Self::Arbitrum => "https://arb1.arbitrum.io/rpc",
            Self::Optimism => "https://mainnet.optimism.io",
            Self::Avalanche => "https://api.avax.network/ext/bc/C/rpc",
            Self::Base => "https://mainnet.base.org",
            Self::ZkSync => "https://mainnet.era.zksync.io",
        }
    }

    /// Get the block explorer base URL for this chain.
    /// 获取此链的区块浏览器基础URL。
    pub const fn block_explorer_url(self) -> &'static str {
        match self {
            Self::Ethereum => "https://etherscan.io",
            Self::Polygon => "https://polygonscan.com",
            Self::Bsc => "https://bscscan.com",
            Self::Arbitrum => "https://arbiscan.io",
            Self::Optimism => "https://optimistic.etherscan.io",
            Self::Avalanche => "https://snowtrace.io",
            Self::Base => "https://basescan.org",
            Self::ZkSync => "https://explorer.zksync.io",
        }
    }
}

impl ChainRegistry {
    /// Create an empty registry.
    /// 创建空的注册表。
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with all supported chains.
    /// 创建预填充所有支持链的注册表。
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register all pre-configured chains.
    /// 注册所有预配置链。
    pub fn register_defaults(&mut self) {
        self.register_chain(Chain::Ethereum);
        self.register_chain(Chain::Polygon);
        self.register_chain(Chain::Bsc);
        self.register_chain(Chain::Arbitrum);
        self.register_chain(Chain::Optimism);
        self.register_chain(Chain::Avalanche);
        self.register_chain(Chain::Base);
        self.register_chain(Chain::ZkSync);
    }

    /// Register a predefined chain.
    /// 注册预定义链。
    pub fn register_chain(&mut self, chain: Chain) {
        let id = chain.chain_id();
        let currency = chain.native_currency();
        let explorer = chain.block_explorer_url();

        let config = ChainConfig::new(id, chain.to_string())
            .with_rpc_url(chain.default_rpc_url())
            .with_explorer(explorer)
            .with_native_currency(currency.symbol, currency.decimals, currency.name)
            .with_block_time(match chain {
                Chain::Ethereum => 12,
                Chain::Polygon | Chain::Optimism | Chain::Avalanche | Chain::Base => 2,
                Chain::Bsc => 3,
                Chain::Arbitrum | Chain::ZkSync => 1,
            });

        let entry = ChainEntry {
            config,
            chain,
            explorer_tx_url: format!("{}/tx/", explorer),
        };
        self.chains.insert(id, entry);
    }

    /// Register a custom chain configuration.
    /// 注册自定义链配置。
    pub fn register_custom(&mut self, config: ChainConfig, explorer_tx_url: impl Into<String>) {
        let id = config.chain_id.as_u64();
        let entry = ChainEntry {
            config,
            chain: Chain::Ethereum, // placeholder for custom chains
            explorer_tx_url: explorer_tx_url.into(),
        };
        self.chains.insert(id, entry);
    }

    /// Look up a chain entry by its EIP-155 chain ID.
    /// 根据EIP-155链ID查找链记录。
    pub fn get_by_chain_id(&self, chain_id: u64) -> Option<&ChainEntry> {
        self.chains.get(&chain_id)
    }

    /// Look up a chain entry by its predefined identifier.
    /// 根据预定义标识符查找链记录。
    pub fn get(&self, chain: Chain) -> Option<&ChainEntry> {
        self.chains.get(&chain.chain_id())
    }

    /// Get all registered chain IDs.
    /// 获取所有已注册的链ID。
    pub fn chain_ids(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self.chains.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    /// Get the number of registered chains.
    /// 获取已注册链的数量。
    pub fn len(&self) -> usize {
        self.chains.len()
    }

    /// Check if the registry is empty.
    /// 检查注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.chains.is_empty()
    }
}

impl Default for ChainRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ---------------------------------------------------------------------------
// Bridge
// ---------------------------------------------------------------------------

/// Cross-chain bridge status.
/// 跨链桥状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeStatus {
    /// Transaction has been submitted to the source chain.
    /// 交易已提交到源链。
    Pending,
    /// Transaction confirmed on the source chain.
    /// 交易已在源链确认。
    SourceConfirmed,
    /// Assets locked/burned on the source chain.
    /// 资产已在源链锁定/销毁。
    Locked,
    /// Transaction relayed to the destination chain.
    /// 交易已中继到目标链。
    Relayed,
    /// Transaction completed on the destination chain.
    /// 交易已在目标链完成。
    Completed,
    /// Transaction failed.
    /// 交易失败。
    Failed(String),
}

impl fmt::Display for BridgeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::SourceConfirmed => write!(f, "SourceConfirmed"),
            Self::Locked => write!(f, "Locked"),
            Self::Relayed => write!(f, "Relayed"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed(msg) => write!(f, "Failed: {}", msg),
        }
    }
}

/// Bridge fee estimate.
/// 桥接费用估算。
#[derive(Debug, Clone)]
pub struct BridgeFeeEstimate {
    /// Fee in native token (wei).
    /// 原生代币费用（wei）。
    pub fee_wei: u64,
    /// Estimated time for completion in seconds.
    /// 预计完成时间（秒）。
    pub estimated_time_secs: u64,
    /// Minimum amount that will be received on the destination chain (wei).
    /// 目标链上最少收到的金额（wei）。
    pub min_received: u64,
}

/// Bridge transaction request.
/// 桥接交易请求。
#[derive(Debug, Clone)]
pub struct BridgeRequest {
    /// Source chain ID.
    /// 源链ID。
    pub from_chain_id: u64,
    /// Destination chain ID.
    /// 目标链ID。
    pub to_chain_id: u64,
    /// Token address on source chain (or zero-address for native token).
    /// 源链上的代币地址（零地址表示原生代币）。
    pub token: Address,
    /// Amount to bridge (wei).
    /// 桥接金额（wei）。
    pub amount: u64,
    /// Recipient address on destination chain.
    /// 目标链上的接收地址。
    pub recipient: Address,
}

impl BridgeRequest {
    /// Create a new bridge request.
    /// 创建新的桥接请求。
    pub fn new(
        from_chain_id: u64,
        to_chain_id: u64,
        token: Address,
        amount: u64,
        recipient: Address,
    ) -> Self {
        Self {
            from_chain_id,
            to_chain_id,
            token,
            amount,
            recipient,
        }
    }
}

/// Bridge transaction result.
/// 桥接交易结果。
#[derive(Debug, Clone)]
pub struct BridgeTransaction {
    /// Transaction hash on the source chain.
    /// 源链上的交易哈希。
    pub tx_hash: String,
    /// Bridge request details.
    /// 桥接请求详情。
    pub request: BridgeRequest,
}

/// Cross-chain bridge interface.
/// 跨链桥接口。
///
/// Defines the interface for cross-chain token bridge operations.
/// Individual bridge implementations (e.g. across chain, Stargate) should
/// implement this trait.
///
/// 定义跨链代币桥接操作的接口。
/// 各个桥接实现（例如 across chain、Stargate）应实现此trait。
pub trait Bridge: Send + Sync {
    /// Estimate the fee for a bridge transfer.
    /// 估算桥接转账的费用。
    fn estimate_fee(&self, request: &BridgeRequest) -> Result<BridgeFeeEstimate, BridgeError>;

    /// Initiate a bridge transfer.
    /// 发起桥接转账。
    fn bridge_tokens(&self, request: &BridgeRequest) -> Result<BridgeTransaction, BridgeError>;

    /// Track the status of a bridge transaction.
    /// 跟踪桥接交易的状态。
    fn track_transaction(&self, tx_hash: &str) -> Result<BridgeStatus, BridgeError>;

    /// Get the name of this bridge provider.
    /// 获取此桥接提供商的名称。
    fn name(&self) -> &str;

    /// Check if this bridge supports the given pair of chains.
    /// 检查此桥是否支持给定的链对。
    fn supports_route(&self, from_chain_id: u64, to_chain_id: u64) -> bool;
}

/// Bridge error.
/// 桥接错误。
#[derive(Debug, Clone)]
pub enum BridgeError {
    /// Unsupported route (chain pair not supported).
    /// 不支持的路由（不支持该链对）。
    UnsupportedRoute(u64, u64),

    /// Insufficient liquidity.
    /// 流动性不足。
    InsufficientLiquidity,

    /// Amount is below the minimum.
    /// 金额低于最低限额。
    AmountTooLow,

    /// Amount exceeds the maximum.
    /// 金额超过最高限额。
    AmountTooHigh,

    /// Transaction failed.
    /// 交易失败。
    TransactionFailed(String),

    /// RPC error.
    /// RPC错误。
    RpcError(String),

    /// Timeout waiting for confirmation.
    /// 等待确认超时。
    Timeout,
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRoute(from, to) => {
                write!(f, "Unsupported bridge route: chain {} -> {}", from, to)
            },
            Self::InsufficientLiquidity => write!(f, "Insufficient bridge liquidity"),
            Self::AmountTooLow => write!(f, "Bridge amount below minimum"),
            Self::AmountTooHigh => write!(f, "Bridge amount exceeds maximum"),
            Self::TransactionFailed(msg) => write!(f, "Bridge transaction failed: {}", msg),
            Self::RpcError(msg) => write!(f, "RPC error: {}", msg),
            Self::Timeout => write!(f, "Bridge confirmation timeout"),
        }
    }
}

impl std::error::Error for BridgeError {}

// ---------------------------------------------------------------------------
// GasOracle
// ---------------------------------------------------------------------------

/// EIP-1559 gas fee components.
/// EIP-1559 Gas费用组成。
#[derive(Debug, Clone)]
pub struct GasFeeEstimate {
    /// Suggested base fee (wei).
    /// 建议的基础费用（wei）。
    pub base_fee: u64,
    /// Suggested priority fee (wei).
    /// 建议的优先费用（wei）。
    pub priority_fee: u64,
    /// Maximum total fee per gas (wei).
    /// 最大总费用每单位Gas（wei）。
    pub max_fee_per_gas: u64,
    /// Estimated gas limit for a simple transfer.
    /// 简单转账的预估Gas限制。
    pub gas_limit: u64,
    /// Estimated total cost (max_fee_per_gas * gas_limit).
    /// 预估总成本（max_fee_per_gas * gas_limit）。
    pub estimated_cost: u64,
}

impl GasFeeEstimate {
    /// Create a new gas fee estimate.
    /// 创建新的Gas费用估算。
    pub fn new(base_fee: u64, priority_fee: u64, gas_limit: u64) -> Self {
        let max_fee_per_gas = base_fee * 2 + priority_fee;
        let estimated_cost = max_fee_per_gas * gas_limit;
        Self {
            base_fee,
            priority_fee,
            max_fee_per_gas,
            gas_limit,
            estimated_cost,
        }
    }

    /// Get the max fee per gas as a Gwei float.
    /// 获取最大费用每单位Gas的Gwei浮点数。
    #[allow(clippy::cast_precision_loss)]
    pub fn max_fee_gwei(&self) -> f64 {
        self.max_fee_per_gas as f64 / 1e9
    }

    /// Get the base fee as a Gwei float.
    /// 获取基础费用的Gwei浮点数。
    #[allow(clippy::cast_precision_loss)]
    pub fn base_fee_gwei(&self) -> f64 {
        self.base_fee as f64 / 1e9
    }
}

/// Gas price estimation with EIP-1559 support.
/// 支持EIP-1559的Gas价格估算。
///
/// Computes gas fee estimates from base fee and priority fee components,
/// following the EIP-1559 fee market model.
///
/// 根据基础费用和优先费用组件计算Gas费用估算，遵循EIP-1559费用市场模型。
#[derive(Debug, Clone)]
pub struct GasOracle {
    /// Chain ID this oracle is configured for.
    /// 此预言机配置的链ID。
    chain_id: u64,
    /// Default priority fee (wei) used when no suggestion is available.
    /// 无可用建议时使用的默认优先费用（wei）。
    default_priority_fee: u64,
    /// Default gas limit for a simple transfer.
    /// 简单转账的默认Gas限制。
    default_gas_limit: u64,
    /// Maximum acceptable max fee per gas (wei).
    /// 最大可接受的最大费用每单位Gas（wei）。
    max_fee_cap: u64,
}

impl GasOracle {
    /// Create a new gas oracle for the given chain.
    /// 为给定链创建新的Gas预言机。
    pub fn new(chain_id: u64) -> Self {
        Self {
            chain_id,
            default_priority_fee: 1_500_000_000, // 1.5 Gwei
            default_gas_limit: 21_000,
            max_fee_cap: 500_000_000_000, // 500 Gwei
        }
    }

    /// Set the default priority fee.
    /// 设置默认优先费用。
    pub fn with_default_priority_fee(mut self, fee: u64) -> Self {
        self.default_priority_fee = fee;
        self
    }

    /// Set the default gas limit.
    /// 设置默认Gas限制。
    pub fn with_default_gas_limit(mut self, limit: u64) -> Self {
        self.default_gas_limit = limit;
        self
    }

    /// Set the maximum fee cap.
    /// 设置最大费用上限。
    pub fn with_max_fee_cap(mut self, cap: u64) -> Self {
        self.max_fee_cap = cap;
        self
    }

    /// Get the chain ID.
    /// 获取链ID。
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Estimate gas fees given the current base fee.
    /// 根据当前基础费用估算Gas费用。
    pub fn estimate(&self, base_fee: u64) -> GasFeeEstimate {
        let priority_fee = self.default_priority_fee;
        let max_fee = (base_fee * 2 + priority_fee).min(self.max_fee_cap);
        let gas_limit = self.default_gas_limit;
        GasFeeEstimate {
            base_fee,
            priority_fee,
            max_fee_per_gas: max_fee,
            gas_limit,
            estimated_cost: max_fee * gas_limit,
        }
    }

    /// Estimate gas fees with a custom priority fee.
    /// 使用自定义优先费用估算Gas费用。
    pub fn estimate_with_priority(&self, base_fee: u64, priority_fee: u64) -> GasFeeEstimate {
        let max_fee = (base_fee * 2 + priority_fee).min(self.max_fee_cap);
        let gas_limit = self.default_gas_limit;
        GasFeeEstimate {
            base_fee,
            priority_fee,
            max_fee_per_gas: max_fee,
            gas_limit,
            estimated_cost: max_fee * gas_limit,
        }
    }

    /// Estimate gas fees for a contract call with a custom gas limit.
    /// 使用自定义Gas限制估算合约调用的Gas费用。
    pub fn estimate_for_gas_limit(&self, base_fee: u64, gas_limit: u64) -> GasFeeEstimate {
        let priority_fee = self.default_priority_fee;
        let max_fee = (base_fee * 2 + priority_fee).min(self.max_fee_cap);
        GasFeeEstimate {
            base_fee,
            priority_fee,
            max_fee_per_gas: max_fee,
            gas_limit,
            estimated_cost: max_fee * gas_limit,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    // -- ChainRegistry tests --

    #[test]
    fn test_registry_with_defaults() {
        let registry = ChainRegistry::with_defaults();
        assert_eq!(registry.len(), 8);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_registry_get_by_chain_id() {
        let registry = ChainRegistry::with_defaults();

        let eth = registry.get_by_chain_id(1).unwrap();
        assert_eq!(eth.config.chain_id.as_u64(), 1);
        assert_eq!(eth.chain, Chain::Ethereum);

        let poly = registry.get_by_chain_id(137).unwrap();
        assert_eq!(poly.config.chain_id.as_u64(), 137);
        assert_eq!(poly.chain, Chain::Polygon);

        let bsc = registry.get_by_chain_id(56).unwrap();
        assert_eq!(bsc.config.chain_id.as_u64(), 56);
        assert_eq!(bsc.chain, Chain::Bsc);
    }

    #[test]
    fn test_registry_get_by_chain_enum() {
        let registry = ChainRegistry::with_defaults();

        let arb = registry.get(Chain::Arbitrum).unwrap();
        assert_eq!(arb.config.chain_id.as_u64(), 42161);

        let base = registry.get(Chain::Base).unwrap();
        assert_eq!(base.config.chain_id.as_u64(), 8453);
    }

    #[test]
    fn test_registry_chain_ids() {
        let registry = ChainRegistry::with_defaults();
        let ids = registry.chain_ids();
        assert!(ids.contains(&1));
        assert!(ids.contains(&137));
        assert!(ids.contains(&56));
        assert!(ids.contains(&42161));
        assert!(ids.contains(&10));
        assert!(ids.contains(&43114));
        assert!(ids.contains(&8453));
        assert!(ids.contains(&324));
    }

    #[test]
    fn test_registry_missing_chain() {
        let registry = ChainRegistry::with_defaults();
        assert!(registry.get_by_chain_id(99999).is_none());
    }

    #[test]
    fn test_registry_custom_chain() {
        let mut registry = ChainRegistry::new();
        let config = ChainConfig::new(12345u64, "My Custom Chain")
            .with_rpc_url("https://rpc.example.com")
            .with_explorer("https://explorer.example.com");
        registry.register_custom(config, "https://explorer.example.com/tx/");

        let entry = registry.get_by_chain_id(12345).unwrap();
        assert_eq!(entry.config.chain_id.as_u64(), 12345);
        assert_eq!(entry.config.name, "My Custom Chain");
    }

    #[test]
    fn test_registry_default() {
        let registry = ChainRegistry::default();
        assert_eq!(registry.len(), 8);
    }

    // -- Chain enum tests --

    #[test]
    fn test_chain_chain_ids() {
        assert_eq!(Chain::Ethereum.chain_id(), 1);
        assert_eq!(Chain::Polygon.chain_id(), 137);
        assert_eq!(Chain::Bsc.chain_id(), 56);
        assert_eq!(Chain::Arbitrum.chain_id(), 42161);
        assert_eq!(Chain::Optimism.chain_id(), 10);
        assert_eq!(Chain::Avalanche.chain_id(), 43114);
        assert_eq!(Chain::Base.chain_id(), 8453);
        assert_eq!(Chain::ZkSync.chain_id(), 324);
    }

    #[test]
    fn test_chain_native_currency() {
        let eth_currency = Chain::Ethereum.native_currency();
        assert_eq!(eth_currency.symbol, "ETH");
        assert_eq!(eth_currency.decimals, 18);

        let bsc_currency = Chain::Bsc.native_currency();
        assert_eq!(bsc_currency.symbol, "BNB");

        let avax_currency = Chain::Avalanche.native_currency();
        assert_eq!(avax_currency.symbol, "AVAX");
    }

    #[test]
    fn test_chain_display() {
        assert_eq!(Chain::Ethereum.to_string(), "Ethereum");
        assert_eq!(Chain::Bsc.to_string(), "BSC");
        assert_eq!(Chain::ZkSync.to_string(), "zkSync");
    }

    #[test]
    fn test_chain_block_explorer() {
        assert_eq!(Chain::Ethereum.block_explorer_url(), "https://etherscan.io");
        assert_eq!(Chain::Polygon.block_explorer_url(), "https://polygonscan.com");
        assert_eq!(Chain::Base.block_explorer_url(), "https://basescan.org");
    }

    #[test]
    fn test_chain_default_rpc() {
        assert!(Chain::Ethereum.default_rpc_url().starts_with("https://"));
        assert!(Chain::Bsc.default_rpc_url().starts_with("https://"));
    }

    // -- Bridge tests --

    #[test]
    fn test_bridge_request_new() {
        let from = Address::zero();
        let recipient = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let req = BridgeRequest::new(1, 137, from, 1000, recipient);
        assert_eq!(req.from_chain_id, 1);
        assert_eq!(req.to_chain_id, 137);
        assert_eq!(req.amount, 1000);
    }

    #[test]
    fn test_bridge_status_display() {
        assert_eq!(BridgeStatus::Pending.to_string(), "Pending");
        assert_eq!(BridgeStatus::Completed.to_string(), "Completed");
        assert!(
            BridgeStatus::Failed("timeout".into())
                .to_string()
                .contains("timeout")
        );
    }

    #[test]
    fn test_bridge_error_display() {
        let err = BridgeError::UnsupportedRoute(1, 137);
        assert!(err.to_string().contains("Unsupported bridge route"));

        let err = BridgeError::InsufficientLiquidity;
        assert!(err.to_string().contains("Insufficient bridge liquidity"));
    }

    // -- GasOracle tests --

    #[test]
    fn test_gas_oracle_estimate() {
        let oracle = GasOracle::new(1);
        // base_fee = 20 Gwei
        let base_fee = 20_000_000_000u64;
        let estimate = oracle.estimate(base_fee);

        assert_eq!(estimate.base_fee, base_fee);
        assert_eq!(estimate.priority_fee, 1_500_000_000);
        // max_fee = min(base_fee * 2 + priority, cap)
        let expected_max = base_fee * 2 + 1_500_000_000;
        assert_eq!(estimate.max_fee_per_gas, expected_max);
        assert_eq!(estimate.gas_limit, 21_000);
        assert_eq!(estimate.estimated_cost, expected_max * 21_000);
    }

    #[test]
    fn test_gas_oracle_estimate_gwei() {
        let oracle = GasOracle::new(1);
        let base_fee = 10_000_000_000u64; // 10 Gwei
        let estimate = oracle.estimate(base_fee);

        let expected_max_gwei = (10.0 * 2.0) + 1.5; // 21.5 Gwei
        assert!((estimate.max_fee_gwei() - expected_max_gwei).abs() < 0.01);
        assert!((estimate.base_fee_gwei() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_gas_oracle_with_priority() {
        let oracle = GasOracle::new(1);
        let base_fee = 10_000_000_000u64;
        let estimate = oracle.estimate_with_priority(base_fee, 3_000_000_000);

        assert_eq!(estimate.priority_fee, 3_000_000_000);
        let expected_max = base_fee * 2 + 3_000_000_000;
        assert_eq!(estimate.max_fee_per_gas, expected_max);
    }

    #[test]
    fn test_gas_oracle_custom_gas_limit() {
        let oracle = GasOracle::new(1);
        let base_fee = 10_000_000_000u64;
        let estimate = oracle.estimate_for_gas_limit(base_fee, 200_000);

        assert_eq!(estimate.gas_limit, 200_000);
    }

    #[test]
    fn test_gas_oracle_fee_cap() {
        let oracle = GasOracle::new(1).with_max_fee_cap(100_000_000_000u64); // 100 Gwei cap
        let base_fee = 200_000_000_000u64; // 200 Gwei base fee (very high)
        let estimate = oracle.estimate(base_fee);

        // Should be capped at 100 Gwei
        assert_eq!(estimate.max_fee_per_gas, 100_000_000_000);
    }

    #[test]
    fn test_gas_oracle_builder() {
        let oracle = GasOracle::new(137)
            .with_default_priority_fee(2_000_000_000)
            .with_default_gas_limit(50_000)
            .with_max_fee_cap(1_000_000_000_000);

        assert_eq!(oracle.chain_id(), 137);
    }

    #[test]
    fn test_gas_fee_estimate_new() {
        let estimate = GasFeeEstimate::new(10_000_000_000, 1_500_000_000, 21_000);
        let expected_max = 10_000_000_000 * 2 + 1_500_000_000;
        assert_eq!(estimate.max_fee_per_gas, expected_max);
        assert_eq!(estimate.estimated_cost, expected_max * 21_000);
    }
}
