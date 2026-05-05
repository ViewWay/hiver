//! Nexus Web3 - Blockchain and Web3 support
//! Nexus Web3 - 区块链和Web3支持
//!
//! # Overview / 概述
//!
//! `nexus-web3` provides blockchain and Web3 functionality including smart
//! contract interaction, wallet management, and transaction handling.
//!
//! `nexus-web3` 提供区块链和Web3功能，包括智能合约交互、钱包管理和交易处理。

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow dead_code: This is a framework library with many public APIs that are
// provided for users but not used internally. This is expected and intentional.
// 允许 dead_code：这是一个框架库，包含许多公共 API 供用户使用但内部未使用。
// 这是预期且有意的设计。
#![allow(dead_code)]

#[cfg(test)]
mod tests;

pub mod chain;
pub mod contract;
pub mod rpc;
pub mod subscribe;
pub mod tx;
pub mod wallet;

pub use chain::{Block, BlockNumber, ChainConfig, ChainId, Eip155Chain};
pub use contract::{CallParams, ContractError, FunctionSelector};
pub use rpc::RpcError;
pub use tx::{Transaction, TransactionBuilder, TxHash, TxType};
pub use wallet::{Address, LocalWallet, Wallet};

#[cfg(feature = "rpc")]
pub use rpc::RpcClient;

#[cfg(feature = "rpc")]
pub use contract::{Contract, ContractCall, ERC20, ERC721};

#[cfg(feature = "ws")]
pub use subscribe::{
    LogFilter, LogNotification, NewBlockHeader, PendingTransaction, SubscriptionId,
    SubscriptionManager, SubscriptionNotification, SubscriptionType, WsClient, WsError,
};
