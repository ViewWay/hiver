//! Hiver Web3 - Blockchain and Web3 support
//! Hiver Web3 - 区块链和Web3支持
//!
//! # Overview / 概述
//!
//! `hiver-web3` provides blockchain and Web3 functionality including smart
//! contract interaction, wallet management, and transaction handling.
//!
//! `hiver-web3` 提供区块链和Web3功能，包括智能合约交互、钱包管理和交易处理。

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow dead_code: This is a framework library with many public APIs that are
// provided for users but not used internally. This is expected and intentional.
// 允许 dead_code：这是一个框架库，包含许多公共 API 供用户使用但内部未使用。
// 这是预期且有意的设计。

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

pub mod chain;
pub mod contract;
pub mod defi;
pub mod hd_wallet;
pub mod multichain;
pub mod rpc;
pub mod subscribe;
pub mod tx;
pub mod wallet;

pub use chain::{Block, BlockNumber, ChainConfig, ChainId, Eip155Chain};
pub use contract::{CallParams, ContractError, FunctionSelector};
#[cfg(feature = "rpc")]
pub use contract::{Contract, ContractCall, ERC20, ERC721};
pub use defi::{DeFiError, Erc20, Erc721, Erc1155, UniswapV2Router};
pub use hd_wallet::{DerivedAccount, HdWallet, HdWalletError, MultiSigWallet, WordCount};
pub use multichain::{BridgeError, ChainRegistry, GasFeeEstimate, GasOracle};
#[cfg(feature = "rpc")]
pub use rpc::RpcClient;
pub use rpc::RpcError;
#[cfg(feature = "ws")]
pub use subscribe::{
    LogFilter, LogNotification, NewBlockHeader, PendingTransaction, SubscriptionId,
    SubscriptionManager, SubscriptionNotification, SubscriptionType, WsClient, WsError,
};
pub use tx::{Transaction, TransactionBuilder, TxHash, TxType};
pub use wallet::{Address, LocalWallet, Wallet};
