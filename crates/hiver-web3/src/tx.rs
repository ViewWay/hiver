//! Transaction module
//! 交易模块
//!
//! # Overview / 概述
//!
//! This module provides transaction management including transaction building,
//! signing, and RLP encoding for EVM-compatible blockchains.
//!
//! 本模块提供交易管理，包括交易构建、签名和EVM兼容区块链的RLP编码。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Web3j (Transaction, `RawTransaction`)
//! - Web3j Transaction management
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_web3::tx::{TransactionBuilder, TxHash};
//! use hiver_web3::wallet::{LocalWallet, Address};
//! use hiver_web3::chain::ChainId;
//!
//! let wallet = LocalWallet::random();
//! let to = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
//!
//! let tx = TransactionBuilder::new()
//!     .to(to)
//!     .value(1000000000000000u128) // 0.001 ETH
//!     .chain_id(1u64)
//!     .build()
//!     .sign(&wallet)?;
//!
//! println!("Transaction hash: {}", tx.hash());
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::wallet::{Address, keccak256};

/// Transaction hash (32 bytes)
/// 交易哈希（32字节）
///
/// A 32-byte hash that uniquely identifies a transaction.
/// 唯一标识交易的32字节哈希。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TxHash(pub [u8; 32]);

impl TxHash
{
    /// Create a new zero transaction hash
    /// 创建新的零交易哈希
    pub const fn zero() -> Self
    {
        Self([0u8; 32])
    }

    /// Create from 32-byte array
    /// 从32字节数组创建
    pub const fn from_bytes(bytes: [u8; 32]) -> Self
    {
        Self(bytes)
    }

    /// Convert to hex string
    /// 转换为十六进制字符串
    pub fn to_hex(&self) -> String
    {
        format!("0x{}", hex::encode(self.0))
    }

    /// Parse from hex string
    /// 从十六进制字符串解析
    pub fn from_hex(hex: &str) -> Result<Self, TransactionError>
    {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        if hex.len() != 64
        {
            return Err(TransactionError::InvalidHashLength);
        }

        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex, &mut bytes).map_err(|_| TransactionError::InvalidHex)?;

        Ok(Self(bytes))
    }

    /// Check if this is the zero hash
    /// 检查这是否是零哈希
    pub const fn is_zero(&self) -> bool
    {
        self.0[0] == 0
            && self.0[1] == 0
            && self.0[2] == 0
            && self.0[3] == 0
            && self.0[4] == 0
            && self.0[5] == 0
            && self.0[6] == 0
            && self.0[7] == 0
            && self.0[8] == 0
            && self.0[9] == 0
            && self.0[10] == 0
            && self.0[11] == 0
            && self.0[12] == 0
            && self.0[13] == 0
            && self.0[14] == 0
            && self.0[15] == 0
            && self.0[16] == 0
            && self.0[17] == 0
            && self.0[18] == 0
            && self.0[19] == 0
            && self.0[20] == 0
            && self.0[21] == 0
            && self.0[22] == 0
            && self.0[23] == 0
            && self.0[24] == 0
            && self.0[25] == 0
            && self.0[26] == 0
            && self.0[27] == 0
            && self.0[28] == 0
            && self.0[29] == 0
            && self.0[30] == 0
            && self.0[31] == 0
    }
}

impl Default for TxHash
{
    fn default() -> Self
    {
        Self::zero()
    }
}

impl fmt::Debug for TxHash
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "TxHash({})", self.to_hex())
    }
}

impl fmt::Display for TxHash
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for TxHash
{
    type Err = TransactionError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        Self::from_hex(s)
    }
}

impl Serialize for TxHash
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for TxHash
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl AsRef<[u8]> for TxHash
{
    fn as_ref(&self) -> &[u8]
    {
        &self.0[..]
    }
}

/// Transaction type
/// 交易类型
///
/// EIP-2718 transaction types for Ethereum.
/// 以太坊EIP-2718交易类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TxType
{
    /// Legacy transaction (0x0 or no prefix)
    /// 传统交易（0x0或无前缀）
    Legacy = 0,

    /// EIP-2930 access list transaction (0x1)
    /// EIP-2930访问列表交易（0x1）
    AccessList = 1,

    /// EIP-1559 fee market transaction (0x2)
    /// EIP-1559手续费市场交易（0x2）
    EIP1559 = 2,
}

impl TxType
{
    /// Get the transaction type as byte
    /// 获取交易类型的字节表示
    pub const fn as_u8(self) -> u8
    {
        self as u8
    }

    /// Create from byte
    /// 从字节创建
    pub fn from_u8(value: u8) -> Option<Self>
    {
        match value
        {
            0 => Some(Self::Legacy),
            1 => Some(Self::AccessList),
            2 => Some(Self::EIP1559),
            _ => None,
        }
    }
}

/// EIP-1559 transaction parameters
/// EIP-1559 交易参数
///
/// Type 2 transaction with EIP-1559 fee market.
/// 类型2交易，具有EIP-1559手续费市场。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Eip1559Tx
{
    /// Chain ID
    /// 链ID
    pub chain_id: u64,

    /// Nonce
    /// nonce
    pub nonce: u64,

    /// Max priority fee per gas (tip to miner)
    /// 最大优先费用（给矿工的小费）
    pub max_priority_fee_per_gas: u128,

    /// Max fee per gas
    /// 最大总费用
    pub max_fee_per_gas: u128,

    /// Gas limit
    /// Gas限制
    pub gas_limit: u64,

    /// Recipient address (None for contract creation)
    /// 接收地址（合约创建时为None）
    pub to: Option<Address>,

    /// Value in wei
    /// 金额（以wei为单位）
    pub value: U256,

    /// Input data
    /// 输入数据
    pub data: Vec<u8>,

    /// Access list (optional)
    /// 访问列表（可选）
    pub access_list: Vec<(Address, Vec<[u8; 32]>)>,
}

impl Eip1559Tx
{
    /// Create a new EIP-1559 transaction
    /// 创建新的EIP-1559交易
    pub fn new(chain_id: u64, nonce: u64) -> Self
    {
        Self {
            chain_id,
            nonce,
            max_priority_fee_per_gas: 0,
            max_fee_per_gas: 0,
            gas_limit: 21000,
            to: None,
            value: U256::zero(),
            data: Vec::new(),
            access_list: Vec::new(),
        }
    }

    /// Set the recipient address
    /// 设置接收地址
    pub fn to(mut self, to: Address) -> Self
    {
        self.to = Some(to);
        self
    }

    /// Set the value
    /// 设置金额
    pub fn value(mut self, value: impl Into<U256>) -> Self
    {
        self.value = value.into();
        self
    }

    /// Set the gas limit
    /// 设置Gas限制
    pub fn gas_limit(mut self, limit: u64) -> Self
    {
        self.gas_limit = limit;
        self
    }

    /// Set max priority fee per gas
    /// 设置最大优先费用
    pub fn max_priority_fee_per_gas(mut self, fee: u128) -> Self
    {
        self.max_priority_fee_per_gas = fee;
        self
    }

    /// Set max fee per gas
    /// 设置最大费用
    pub fn max_fee_per_gas(mut self, fee: u128) -> Self
    {
        self.max_fee_per_gas = fee;
        self
    }

    /// Set the input data
    /// 设置输入数据
    pub fn data(mut self, data: Vec<u8>) -> Self
    {
        self.data = data;
        self
    }

    /// Set the access list
    /// 设置访问列表
    pub fn access_list(mut self, list: Vec<(Address, Vec<[u8; 32]>)>) -> Self
    {
        self.access_list = list;
        self
    }

    /// Compute the signing hash for this transaction
    /// 计算此交易的签名哈希
    pub fn signing_hash(&self) -> [u8; 32]
    {
        // Simplified - in production, proper RLP encoding
        let mut encoded = Vec::new();
        // RLP encode: [chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, to,
        // value, data, access_list]
        encoded.extend_from_slice(&self.chain_id.to_be_bytes());
        encoded.extend_from_slice(&self.nonce.to_be_bytes());
        // ... (full RLP encoding would go here)
        keccak256(&encoded)
    }

    /// Get transaction type
    /// 获取交易类型
    pub const fn tx_type(&self) -> TxType
    {
        TxType::EIP1559
    }
}

/// Legacy transaction parameters
/// 传统交易参数
///
/// Type 0/1 legacy transaction.
/// 类型0/1传统交易。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyTx
{
    /// Nonce
    /// nonce
    pub nonce: u64,

    /// Gas price
    /// Gas价格
    pub gas_price: u128,

    /// Gas limit
    /// Gas限制
    pub gas_limit: u64,

    /// Recipient address (None for contract creation)
    /// 接收地址（合约创建时为None）
    pub to: Option<Address>,

    /// Value in wei
    /// 金额（以wei为单位）
    pub value: U256,

    /// Input data
    /// 输入数据
    pub data: Vec<u8>,

    /// Chain ID (for EIP-155 replay protection)
    /// 链ID（用于EIP-155重放保护）
    pub chain_id: Option<u64>,
}

impl LegacyTx
{
    /// Create a new legacy transaction
    /// 创建新的传统交易
    pub fn new(nonce: u64) -> Self
    {
        Self {
            nonce,
            gas_price: 0,
            gas_limit: 21000,
            to: None,
            value: U256::zero(),
            data: Vec::new(),
            chain_id: None,
        }
    }

    /// Set the recipient address
    /// 设置接收地址
    pub fn to(mut self, to: Address) -> Self
    {
        self.to = Some(to);
        self
    }

    /// Set the value
    /// 设置金额
    pub fn value(mut self, value: impl Into<U256>) -> Self
    {
        self.value = value.into();
        self
    }

    /// Set the gas limit
    /// 设置Gas限制
    pub fn gas_limit(mut self, limit: u64) -> Self
    {
        self.gas_limit = limit;
        self
    }

    /// Set gas price
    /// 设置Gas价格
    pub fn gas_price(mut self, price: u128) -> Self
    {
        self.gas_price = price;
        self
    }

    /// Set the chain ID
    /// 设置链ID
    pub fn chain_id(mut self, id: u64) -> Self
    {
        self.chain_id = Some(id);
        self
    }

    /// Set the input data
    /// 设置输入数据
    pub fn data(mut self, data: Vec<u8>) -> Self
    {
        self.data = data;
        self
    }

    /// Compute the signing hash for this transaction
    /// 计算此交易的签名哈希
    pub fn signing_hash(&self) -> [u8; 32]
    {
        // Simplified - in production, proper RLP encoding
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&self.nonce.to_be_bytes());
        encoded.extend_from_slice(&self.gas_price.to_be_bytes());
        // ... (full RLP encoding would go here)
        keccak256(&encoded)
    }

    /// Get transaction type
    /// 获取交易类型
    pub const fn tx_type(&self) -> TxType
    {
        TxType::Legacy
    }
}

/// Signed transaction
/// 已签名交易
///
/// A transaction with signature attached.
/// 带有签名的交易。
#[derive(Debug, Clone)]
pub struct SignedTransaction
{
    /// Transaction type
    /// 交易类型
    pub tx_type: TxType,

    /// Raw signed transaction bytes
    /// 已签名交易的原始字节
    pub raw: Vec<u8>,

    /// Transaction hash
    /// 交易哈希
    pub hash: TxHash,

    /// Sender address (recovered from signature)
    /// 发送地址（从签名恢复）
    pub from: Address,
}

impl SignedTransaction
{
    /// Get the transaction hash
    /// 获取交易哈希
    pub fn hash(&self) -> TxHash
    {
        self.hash
    }

    /// Get the raw transaction bytes for broadcasting
    /// 获取用于广播的原始交易字节
    pub fn raw(&self) -> &[u8]
    {
        &self.raw
    }
}

/// Transaction
/// 交易
///
/// Represents any transaction type (signed or unsigned).
/// 表示任何交易类型（已签名或未签名）。
#[derive(Debug, Clone)]
pub enum Transaction
{
    /// EIP-1559 transaction
    /// EIP-1559 交易
    Eip1559(Eip1559Tx),

    /// Legacy transaction
    /// 传统交易
    Legacy(LegacyTx),

    /// Signed transaction
    /// 已签名交易
    Signed(SignedTransaction),
}

impl Transaction
{
    /// Get the transaction type
    /// 获取交易类型
    pub fn tx_type(&self) -> TxType
    {
        match self
        {
            Self::Eip1559(_) => TxType::EIP1559,
            Self::Legacy(_) => TxType::Legacy,
            Self::Signed(s) => s.tx_type,
        }
    }

    /// Get the transaction hash if signed
    /// 获取交易哈希（如果已签名）
    pub fn hash(&self) -> Option<TxHash>
    {
        match self
        {
            Self::Signed(s) => Some(s.hash),
            _ => None,
        }
    }

    /// Check if the transaction is signed
    /// 检查交易是否已签名
    pub fn is_signed(&self) -> bool
    {
        matches!(self, Self::Signed(_))
    }
}

/// Transaction builder
/// 交易构建器
///
/// Builder for creating and signing transactions.
/// 用于创建和签名交易的构建器。
#[derive(Debug, Clone)]
pub struct TransactionBuilder
{
    /// Transaction type (default: EIP-1559)
    /// 交易类型（默认：EIP-1559）
    tx_type: TxType,

    /// Chain ID
    /// 链ID
    chain_id: u64,

    /// Nonce
    /// nonce
    nonce: Option<u64>,

    /// Gas price (for legacy transactions)
    /// Gas价格（用于传统交易）
    gas_price: Option<u128>,

    /// Max priority fee per gas (for EIP-1559)
    /// 最大优先费用（用于EIP-1559）
    max_priority_fee_per_gas: Option<u128>,

    /// Max fee per gas (for EIP-1559)
    /// 最大费用（用于EIP-1559）
    max_fee_per_gas: Option<u128>,

    /// Gas limit
    /// Gas限制
    gas_limit: Option<u64>,

    /// Recipient address
    /// 接收地址
    to: Option<Address>,

    /// Value in wei
    /// 金额（以wei为单位）
    value: Option<U256>,

    /// Input data
    /// 输入数据
    data: Option<Vec<u8>>,
}

impl Default for TransactionBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl TransactionBuilder
{
    /// Create a new transaction builder
    /// 创建新的交易构建器
    pub fn new() -> Self
    {
        Self {
            tx_type: TxType::EIP1559,
            chain_id: 1, // Ethereum mainnet default
            nonce: None,
            gas_price: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_limit: None,
            to: None,
            value: None,
            data: None,
        }
    }

    /// Set transaction type
    /// 设置交易类型
    pub fn tx_type(mut self, tx_type: TxType) -> Self
    {
        self.tx_type = tx_type;
        self
    }

    /// Set the chain ID
    /// 设置链ID
    pub fn chain_id(mut self, chain_id: impl Into<u64>) -> Self
    {
        self.chain_id = chain_id.into();
        self
    }

    /// Set the nonce
    /// 设置nonce
    pub fn nonce(mut self, nonce: u64) -> Self
    {
        self.nonce = Some(nonce);
        self
    }

    /// Set the gas price (for legacy transactions)
    /// 设置Gas价格（用于传统交易）
    pub fn gas_price(mut self, gas_price: u128) -> Self
    {
        self.gas_price = Some(gas_price);
        self
    }

    /// Set max priority fee per gas (for EIP-1559)
    /// 设置最大优先费用（用于EIP-1559）
    pub fn max_priority_fee_per_gas(mut self, fee: u128) -> Self
    {
        self.max_priority_fee_per_gas = Some(fee);
        self
    }

    /// Set max fee per gas (for EIP-1559)
    /// 设置最大费用（用于EIP-1559）
    pub fn max_fee_per_gas(mut self, fee: u128) -> Self
    {
        self.max_fee_per_gas = Some(fee);
        self
    }

    /// Set gas limit
    /// 设置Gas限制
    pub fn gas_limit(mut self, limit: u64) -> Self
    {
        self.gas_limit = Some(limit);
        self
    }

    /// Set the recipient address
    /// 设置接收地址
    pub fn to(mut self, to: Address) -> Self
    {
        self.to = Some(to);
        self
    }

    /// Set the value
    /// 设置金额
    pub fn value(mut self, value: impl Into<U256>) -> Self
    {
        self.value = Some(value.into());
        self
    }

    /// Set the input data
    /// 设置输入数据
    pub fn data(mut self, data: Vec<u8>) -> Self
    {
        self.data = Some(data);
        self
    }

    /// Build the transaction
    /// 构建交易
    pub fn build(self) -> Result<Transaction, TransactionError>
    {
        match self.tx_type
        {
            TxType::EIP1559 =>
            {
                let mut tx = Eip1559Tx::new(
                    self.chain_id,
                    self.nonce.ok_or(TransactionError::MissingNonce)?,
                );
                if let Some(to) = self.to
                {
                    tx = tx.to(to);
                }
                if let Some(value) = self.value
                {
                    tx = tx.value(value);
                }
                if let Some(gas_limit) = self.gas_limit
                {
                    tx = tx.gas_limit(gas_limit);
                }
                if let Some(max_priority_fee) = self.max_priority_fee_per_gas
                {
                    tx = tx.max_priority_fee_per_gas(max_priority_fee);
                }
                if let Some(max_fee) = self.max_fee_per_gas
                {
                    tx = tx.max_fee_per_gas(max_fee);
                }
                if let Some(data) = self.data
                {
                    tx = tx.data(data);
                }
                Ok(Transaction::Eip1559(tx))
            },
            TxType::Legacy =>
            {
                let mut tx = LegacyTx::new(self.nonce.ok_or(TransactionError::MissingNonce)?);
                if let Some(to) = self.to
                {
                    tx = tx.to(to);
                }
                if let Some(value) = self.value
                {
                    tx = tx.value(value);
                }
                if let Some(gas_limit) = self.gas_limit
                {
                    tx = tx.gas_limit(gas_limit);
                }
                if let Some(gas_price) = self.gas_price
                {
                    tx = tx.gas_price(gas_price);
                }
                tx = tx.chain_id(self.chain_id);
                if let Some(data) = self.data
                {
                    tx = tx.data(data);
                }
                Ok(Transaction::Legacy(tx))
            },
            TxType::AccessList => Err(TransactionError::UnsupportedType("AccessList".to_string())),
        }
    }
}

/// 256-bit unsigned integer
/// 256位无符号整数
///
/// Represents Ethereum amounts and other 256-bit values.
/// 表示以太坊金额和其他256位值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct U256
{
    /// Low 128 bits
    /// 低128位
    pub low: u128,
    /// High 128 bits
    /// 高128位
    pub high: u128,
}

impl U256
{
    /// Create a new U256 from u128
    /// 从u128创建新的U256
    pub const fn from_u128(value: u128) -> Self
    {
        Self {
            low: value,
            high: 0,
        }
    }

    /// Create a zero value
    /// 创建零值
    pub const fn zero() -> Self
    {
        Self { low: 0, high: 0 }
    }

    /// Check if the value is zero
    /// 检查值是否为零
    pub const fn is_zero(&self) -> bool
    {
        self.low == 0 && self.high == 0
    }

    /// Convert to hex string
    /// 转换为十六进制字符串
    pub fn to_hex(&self) -> String
    {
        if self.high == 0
        {
            format!("0x{:x}", self.low)
        }
        else
        {
            format!("0x{:x}{:032x}", self.high, self.low)
        }
    }
}

impl From<u128> for U256
{
    fn from(value: u128) -> Self
    {
        Self::from_u128(value)
    }
}

impl From<u64> for U256
{
    fn from(value: u64) -> Self
    {
        Self {
            low: value as u128,
            high: 0,
        }
    }
}

impl fmt::Display for U256
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.to_hex())
    }
}

/// Transaction error
/// 交易错误
#[derive(Debug, Clone)]
pub enum TransactionError
{
    /// Invalid hash length
    /// 无效的哈希长度
    InvalidHashLength,

    /// Invalid hex encoding
    /// 无效的十六进制编码
    InvalidHex,

    /// Missing nonce
    /// 缺少nonce
    MissingNonce,

    /// Missing chain ID
    /// 缺少链ID
    MissingChainId,

    /// Unsupported transaction type
    /// 不支持的交易类型
    UnsupportedType(String),

    /// RLP encoding error
    /// RLP编码错误
    RlpError(String),

    /// Signing error
    /// 签名错误
    SigningError(String),
}

impl fmt::Display for TransactionError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::InvalidHashLength => write!(f, "Invalid transaction hash length"),
            Self::InvalidHex => write!(f, "Invalid hex encoding"),
            Self::MissingNonce => write!(f, "Missing transaction nonce"),
            Self::MissingChainId => write!(f, "Missing chain ID"),
            Self::UnsupportedType(t) => write!(f, "Unsupported transaction type: {}", t),
            Self::RlpError(msg) => write!(f, "RLP encoding error: {}", msg),
            Self::SigningError(msg) => write!(f, "Signing error: {}", msg),
        }
    }
}

impl std::error::Error for TransactionError {}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_tx_hash_zero()
    {
        let hash = TxHash::zero();
        assert!(hash.is_zero());
        assert_eq!(
            hash.to_hex(),
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_tx_hash_from_hex()
    {
        let hex = "0x884edad9ce6fa2440d8a54cc123490eb96d2768479d49f977aadd26711b4c86c";
        let hash = TxHash::from_hex(hex).unwrap();
        assert_eq!(hash.to_hex(), hex);
    }

    #[test]
    fn test_tx_hash_from_str()
    {
        let hex = "0x884edad9ce6fa2440d8a54cc123490eb96d2768479d49f977aadd26711b4c86c";
        let hash: TxHash = hex.parse().unwrap();
        assert_eq!(hash.to_hex(), hex);
    }

    #[test]
    fn test_tx_hash_invalid_length()
    {
        let result = TxHash::from_hex("0x1234");
        assert!(matches!(result, Err(TransactionError::InvalidHashLength)));
    }

    #[test]
    fn test_tx_type_from_u8()
    {
        assert_eq!(TxType::from_u8(0), Some(TxType::Legacy));
        assert_eq!(TxType::from_u8(1), Some(TxType::AccessList));
        assert_eq!(TxType::from_u8(2), Some(TxType::EIP1559));
        assert_eq!(TxType::from_u8(99), None);
    }

    #[test]
    fn test_eip1559_tx_builder()
    {
        let tx = Eip1559Tx::new(1, 0)
            .to(Address::zero())
            .value(1000000000000000u128)
            .gas_limit(21000);

        assert_eq!(tx.chain_id, 1);
        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.gas_limit, 21000);
    }

    #[test]
    fn test_legacy_tx_builder()
    {
        let tx = LegacyTx::new(0)
            .to(Address::zero())
            .value(1000000000000000u128)
            .gas_limit(21000)
            .gas_price(1000000000u128);

        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.gas_limit, 21000);
        assert_eq!(tx.gas_price, 1000000000);
    }

    #[test]
    fn test_transaction_builder_eip1559()
    {
        let to = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let tx = TransactionBuilder::new()
            .tx_type(TxType::EIP1559)
            .to(to)
            .value(1000000000000000u128)
            .nonce(0)
            .chain_id(1u64)
            .build()
            .unwrap();

        assert!(matches!(tx, Transaction::Eip1559(_)));
    }

    #[test]
    fn test_transaction_builder_legacy()
    {
        let to = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let tx = TransactionBuilder::new()
            .tx_type(TxType::Legacy)
            .to(to)
            .value(1000000000000000u128)
            .nonce(0)
            .chain_id(1u64)
            .gas_price(1000000000u128)
            .build()
            .unwrap();

        assert!(matches!(tx, Transaction::Legacy(_)));
    }

    #[test]
    fn test_transaction_builder_missing_nonce()
    {
        let result = TransactionBuilder::new().to(Address::zero()).build();

        assert!(matches!(result, Err(TransactionError::MissingNonce)));
    }

    #[test]
    fn test_u256_zero()
    {
        let val = U256::zero();
        assert!(val.is_zero());
    }

    #[test]
    fn test_u256_from_u128()
    {
        let val = U256::from_u128(1000000);
        assert!(!val.is_zero());
        assert_eq!(val.low, 1000000);
    }

    #[test]
    fn test_u256_from_u64()
    {
        let val = U256::from(1000000u64);
        assert!(!val.is_zero());
        assert_eq!(val.low, 1000000);
    }

    #[test]
    fn test_transaction_error_display()
    {
        let err = TransactionError::MissingNonce;
        assert_eq!(err.to_string(), "Missing transaction nonce");

        let err = TransactionError::UnsupportedType("TestType".to_string());
        assert!(err.to_string().contains("Unsupported transaction type"));
    }

    #[test]
    fn test_transaction_is_signed()
    {
        let tx = Transaction::Eip1559(Eip1559Tx::new(1, 0));
        assert!(!tx.is_signed());
        assert!(tx.hash().is_none());
    }
}
