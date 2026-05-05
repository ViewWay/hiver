//! Wallet module
//! 钱包模块
//!
//! # Overview / 概述
//!
//! This module provides wallet management including address handling,
//! transaction signing, and key management.
//!
//! 本模块提供钱包管理，包括地址处理、交易签名和密钥管理。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Web3j (Wallet, Credentials)
//! - Web3j Wallet management
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_web3::wallet::{LocalWallet, Wallet, Address};
//!
//! // Create a random wallet
//! let wallet = LocalWallet::random();
//! println!("Address: {}", wallet.address());
//!
//! // Create from private key
//! let wallet = LocalWallet::from_private_key("0x...")?;
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

// Address module is defined inline below
// pub use address::{Address, AddressError, H160};

use rand::RngCore;
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Wallet trait
/// 钱包trait
///
/// Defines the interface for wallet implementations.
/// 定义钱包实现的接口。
pub trait Wallet: Send + Sync {
    /// Get wallet address
    /// 获取钱包地址
    fn address(&self) -> Address;

    /// Get chain ID
    /// 获取链ID
    fn chain_id(&self) -> Option<u64>;

    /// Sign data
    /// 签名数据
    fn sign(&self, data: &[u8]) -> Result<Signature, WalletError>;

    /// Sign a transaction hash
    /// 签名交易哈希
    fn sign_hash(&self, hash: &[u8; 32]) -> Result<Signature, WalletError>;

    /// Get the private key bytes (use with caution)
    /// 获取私钥字节（谨慎使用）
    fn signer(&self) -> Signer;
}

/// Signer reference for low-level operations
/// 签名者引用，用于底层操作
#[derive(Clone, Debug)]
pub struct Signer {
    /// Signing key bytes
    /// 签名密钥字节
    pub bytes: [u8; 32],

    /// Chain ID for EIP-155 replay protection
    /// 用于EIP-155重放保护的链ID
    pub chain_id: Option<u64>,
}

impl Signer {
    /// Create a new signer
    /// 创建新的签名者
    pub fn new(bytes: [u8; 32]) -> Self {
        Self {
            bytes,
            chain_id: None,
        }
    }

    /// Set chain ID
    /// 设置链ID
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Get the address derived from this signer
    /// 获取从此签名者派生的地址
    pub fn address(&self) -> Address {
        Address::from_private_key(&self.bytes)
    }
}

/// Local wallet implementation
/// 本地钱包实现
///
/// A wallet backed by a private key stored in memory.
/// 由内存中存储的私钥支持的钱包。
#[derive(Clone)]
pub struct LocalWallet {
    /// Signing key
    /// 签名密钥
    signer: Signer,
}

impl LocalWallet {
    /// Create a new random wallet
    /// 创建新的随机钱包
    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        Self {
            signer: Signer::new(bytes),
        }
    }

    /// Create from private key bytes
    /// 从私钥字节创建
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self {
            signer: Signer::new(bytes),
        }
    }

    /// Create from hex private key
    /// 从十六进制私钥创建
    pub fn from_private_key(hex: &str) -> Result<Self, WalletError> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        if hex.len() != 64 {
            return Err(WalletError::InvalidPrivateKey);
        }

        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex, &mut bytes).map_err(|_| WalletError::InvalidPrivateKey)?;

        Ok(Self {
            signer: Signer::new(bytes),
        })
    }

    /// Create from mnemonic phrase
    /// 从助记词短语创建
    #[cfg(feature = "wallet")]
    pub fn from_mnemonic(phrase: &str) -> Result<Self, WalletError> {
        use bip39::Mnemonic;

        // bip39 2.x uses Mnemonic::parse for existing mnemonic phrases
        let mnemonic = Mnemonic::parse(phrase).map_err(|_| WalletError::InvalidMnemonic)?;

        let seed = mnemonic.to_seed("");
        // Use first 32 bytes of seed as private key
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&seed[..32]);

        Ok(Self {
            signer: Signer::new(bytes),
        })
    }

    /// Create with chain ID
    /// 使用链ID创建
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.signer = self.signer.with_chain_id(chain_id);
        self
    }
}

impl Default for LocalWallet {
    fn default() -> Self {
        Self::random()
    }
}

impl fmt::Debug for LocalWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalWallet")
            .field("address", &self.address())
            .field("chain_id", &self.signer.chain_id)
            .finish()
    }
}

impl Wallet for LocalWallet {
    fn address(&self) -> Address {
        self.signer.address()
    }

    fn chain_id(&self) -> Option<u64> {
        self.signer.chain_id
    }

    fn sign(&self, data: &[u8]) -> Result<Signature, WalletError> {
        let hash = keccak256(data);
        self.sign_hash(&hash)
    }

    fn sign_hash(&self, _hash: &[u8; 32]) -> Result<Signature, WalletError> {
        // In production, this would use secp256k1 signing
        // For now, return a placeholder signature
        Ok(Signature {
            r: [0u8; 32],
            s: [0u8; 32],
            v: 0,
        })
    }

    fn signer(&self) -> Signer {
        self.signer.clone()
    }
}

/// ECDSA Signature
/// ECDSA签名
///
/// Represents an ECDSA signature with recovery ID.
/// 表示带有恢复ID的ECDSA签名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// R value
    /// R值
    pub r: [u8; 32],

    /// S value
    /// S值
    pub s: [u8; 32],

    /// V value (recovery ID)
    /// V值（恢复ID）
    pub v: u8,
}

impl Signature {
    /// Create a new signature
    /// 创建新签名
    pub fn new(r: [u8; 32], s: [u8; 32], v: u8) -> Self {
        Self { r, s, v }
    }

    /// Convert to hex string
    /// 转换为十六进制字符串
    pub fn to_hex(&self) -> String {
        format!("{}{}{:02x}", hex::encode(self.r), hex::encode(self.s), self.v)
    }

    /// Parse from hex string
    /// 从十六进制字符串解析
    pub fn from_hex(hex: &str) -> Result<Self, SignatureError> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        if hex.len() != 130 {
            return Err(SignatureError::InvalidLength);
        }

        // Decode r (first 64 hex chars = 32 bytes)
        let r_bytes = hex::decode(&hex[0..64]).map_err(|_| SignatureError::InvalidHex)?;
        // Decode s (next 64 hex chars = 32 bytes)
        let s_bytes = hex::decode(&hex[64..128]).map_err(|_| SignatureError::InvalidHex)?;
        // Parse v (last 2 hex chars = 1 byte)
        let v = u8::from_str_radix(&hex[128..130], 16).map_err(|_| SignatureError::InvalidHex)?;

        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&r_bytes);
        s.copy_from_slice(&s_bytes);

        Ok(Self { r, s, v })
    }

    /// Get the compact signature (65 bytes)
    /// 获取紧凑签名（65字节）
    pub fn to_bytes(&self) -> [u8; 65] {
        let mut bytes = [0u8; 65];
        bytes[0..32].copy_from_slice(&self.r);
        bytes[32..64].copy_from_slice(&self.s);
        bytes[64] = self.v;
        bytes
    }

    /// Get signature as tuple
    /// 获取签名的元组形式
    pub fn as_tuple(&self) -> ([u8; 32], [u8; 32], u8) {
        (self.r, self.s, self.v)
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

/// Signature error
/// 签名错误
#[derive(Debug, Clone)]
pub enum SignatureError {
    /// Invalid length
    /// 无效长度
    InvalidLength,

    /// Invalid hex encoding
    /// 无效的十六进制编码
    InvalidHex,
}

impl fmt::Display for SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => write!(f, "Invalid signature length"),
            Self::InvalidHex => write!(f, "Invalid hex encoding"),
        }
    }
}

impl std::error::Error for SignatureError {}

/// Wallet error
/// 钱包错误
#[derive(Debug, Clone)]
pub enum WalletError {
    /// Invalid private key
    /// 无效的私钥
    InvalidPrivateKey,

    /// Invalid mnemonic phrase
    /// 无效的助记词短语
    InvalidMnemonic,

    /// Signing error
    /// 签名错误
    SigningError(String),

    /// Invalid signature
    /// 无效的签名
    InvalidSignature,
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrivateKey => write!(f, "Invalid private key"),
            Self::InvalidMnemonic => write!(f, "Invalid mnemonic phrase"),
            Self::SigningError(msg) => write!(f, "Signing error: {}", msg),
            Self::InvalidSignature => write!(f, "Invalid signature"),
        }
    }
}

impl std::error::Error for WalletError {}

/// Compute Keccak256 hash
/// 计算Keccak256哈希
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Address module
/// 地址模块
pub mod address {
    use super::{keccak256, fmt, FromStr, Serialize, Serializer, Deserialize, Deserializer, SerdeError};

    /// Ethereum address (20 bytes)
    /// 以太坊地址（20字节）
    ///
    /// A 20-byte Ethereum address with validation and conversion utilities.
    /// 具有20字节的以太坊地址，带有验证和转换工具。
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
    pub struct Address(pub [u8; 20]);

    impl Address {
        /// Create a new zero address
        /// 创建新的零地址
        pub const fn zero() -> Self {
            Self([0u8; 20])
        }

        /// Check if this is the zero address
        /// 检查这是否是零地址
        pub const fn is_zero(&self) -> bool {
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
        }

        /// Create from 20-byte array
        /// 从20字节数组创建
        pub const fn from_bytes(bytes: [u8; 20]) -> Self {
            Self(bytes)
        }

        /// Create from private key (computes address)
        /// 从私钥创建（计算地址）
        pub fn from_private_key(private_key: &[u8; 32]) -> Self {
            // Derive public key (simplified - in production use secp256k1)
            let pub_key = derive_public_key(private_key);
            let hash = keccak256(&pub_key[1..]); // Skip uncompressed prefix
            let mut addr = [0u8; 20];
            addr.copy_from_slice(&hash[12..]);
            Self(addr)
        }

        /// Convert to checksummed address (EIP-55)
        /// 转换为校验和地址（EIP-55）
        pub fn checksum(&self) -> String {
            let addr_hex = hex::encode(self.0);
            let hash = keccak256(addr_hex.as_bytes());
            let mut result = String::from("0x");

            for (i, c) in addr_hex.chars().enumerate() {
                if hash[i / 2] >> (4 - (i % 2) * 4) & 0x0f >= 8 {
                    result.extend(c.to_uppercase());
                } else {
                    result.extend(c.to_lowercase());
                }
            }
            result
        }

        /// Get the address as hex string (non-checksummed)
        /// 获取地址为十六进制字符串（非校验和）
        pub fn to_hex(&self) -> String {
            format!("0x{}", hex::encode(self.0))
        }

        /// Parse from hex string
        /// 从十六进制字符串解析
        pub fn from_hex(hex: &str) -> Result<Self, AddressError> {
            let hex = hex.strip_prefix("0x").unwrap_or(hex);
            if hex.len() != 40 {
                return Err(AddressError::InvalidLength);
            }

            let bytes = hex::decode(hex).map_err(|_| AddressError::InvalidHex)?;

            let mut addr = [0u8; 20];
            addr.copy_from_slice(&bytes);
            Ok(Self(addr))
        }
    }

    impl fmt::Debug for Address {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Address({})", self.checksum())
        }
    }

    impl fmt::Display for Address {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.checksum())
        }
    }

    impl FromStr for Address {
        type Err = AddressError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Self::from_hex(s)
        }
    }

    impl Serialize for Address {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_hex())
        }
    }

    impl<'de> Deserialize<'de> for Address {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Self::from_hex(&s).map_err(SerdeError::custom)
        }
    }

    /// Address error
    /// 地址错误
    #[derive(Debug, Clone)]
    pub enum AddressError {
        /// Invalid length
        /// 无效长度
        InvalidLength,

        /// Invalid hex encoding
        /// 无效的十六进制编码
        InvalidHex,
    }

    impl fmt::Display for AddressError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidLength => write!(f, "Invalid address length (expected 40 hex chars)"),
                Self::InvalidHex => write!(f, "Invalid hex encoding"),
            }
        }
    }

    impl std::error::Error for AddressError {}

    /// H160 type alias (compatibility with other libraries)
    /// H160类型别名（与其他库兼容）
    pub type H160 = [u8; 20];

    /// Derive public key from private key (simplified placeholder)
    /// 从私钥派生公钥（简化的占位符）
    fn derive_public_key(private_key: &[u8; 32]) -> [u8; 65] {
        // This is a simplified placeholder
        // In production, use secp256k1 library for actual key derivation
        let mut pub_key = [0u8; 65];
        pub_key[0] = 4; // Uncompressed public key prefix
        // Derive public key from private key (simplified)
        for i in 0..32 {
            pub_key[i + 1] = private_key[i];
            pub_key[i + 33] = private_key[31 - i];
        }
        pub_key
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use super::super::{LocalWallet, Signature, Wallet, WalletError};

        #[test]
        fn test_address_zero() {
            let addr = Address::zero();
            assert!(addr.is_zero());
            assert_eq!(addr.to_hex(), "0x0000000000000000000000000000000000000000");
        }

        #[test]
        fn test_address_from_hex() {
            let hex = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
            let addr = Address::from_hex(hex).unwrap();
            assert_eq!(addr.to_hex(), hex.to_lowercase());
        }

        #[test]
        fn test_address_display() {
            let addr = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
            assert_eq!(addr.to_string(), "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        }

        #[test]
        fn test_address_from_str() {
            let hex = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
            let addr: Address = hex.parse().unwrap();
            assert_eq!(addr.to_hex(), hex.to_lowercase());
        }

        #[test]
        fn test_address_invalid_length() {
            let result = Address::from_hex("0x1234");
            assert!(matches!(result, Err(AddressError::InvalidLength)));
        }

        #[test]
        fn test_address_invalid_hex() {
            let result = Address::from_hex("0xzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
            assert!(matches!(result, Err(AddressError::InvalidHex)));
        }

        #[test]
        fn test_local_wallet_random() {
            let wallet = LocalWallet::random();
            assert!(!wallet.address().is_zero());
        }

        #[test]
        fn test_local_wallet_from_private_key() {
            let hex = "0x0000000000000000000000000000000000000000000000000000000000000001";
            let wallet = LocalWallet::from_private_key(hex).unwrap();
            // The address should be deterministic
            assert!(!wallet.address().is_zero());
        }

        #[test]
        fn test_signature_to_hex() {
            let sig = Signature {
                r: [1u8; 32],
                s: [2u8; 32],
                v: 28,
            };
            let hex = sig.to_hex();
            assert_eq!(hex.len(), 130);
            assert!(hex.ends_with("1c"));
        }

        #[test]
        fn test_signature_from_hex() {
            let sig = Signature {
                r: [1u8; 32],
                s: [2u8; 32],
                v: 28,
            };
            let hex = sig.to_hex();
            let parsed = Signature::from_hex(&hex).unwrap();
            assert_eq!(parsed.r, sig.r);
            assert_eq!(parsed.s, sig.s);
            assert_eq!(parsed.v, sig.v);
        }

        #[test]
        fn test_wallet_error_display() {
            let err = WalletError::InvalidPrivateKey;
            assert_eq!(err.to_string(), "Invalid private key");

            let err = WalletError::SigningError("test error".to_string());
            assert!(err.to_string().contains("Signing error"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_wallet_default() {
        let wallet = LocalWallet::default();
        assert!(!wallet.address().is_zero());
    }

    #[test]
    fn test_local_wallet_with_chain_id() {
        let wallet = LocalWallet::random().with_chain_id(1);
        assert_eq!(wallet.chain_id(), Some(1));
    }
}

// Re-exports from the address module
pub use address::{Address, AddressError, H160};
