//! Contract module
//! 合约模块
//!
//! # Overview / 概述
//!
//! This module provides smart contract interaction including ABI encoding/decoding,
//! function calls, and event parsing for EVM-compatible blockchains.
//!
//! 本模块提供智能合约交互，包括ABI编码/解码、函数调用和EVM兼容区块链的事件解析。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Web3j (`SmartContract`, Contract)
//! - Web3j smart contract wrapper
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_web3::contract::{Contract, ContractCall};
//! use hiver_web3::wallet::Address;
//! use hiver_web3::rpc::RpcClient;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = RpcClient::new("https://eth.llamarpc.com")?;
//! let address = Address::from_hex("0x...")?;
//!
//! let contract = Contract::new(address, &client);
//! let balance: String = contract.call("balanceOf", &[address.to_hex()]).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

use crate::wallet::Address;

#[cfg(feature = "rpc")]
use crate::chain::BlockNumber;

#[cfg(feature = "rpc")]
use crate::tx::TxHash;

/// Contract error
/// 合约错误
#[derive(Debug, Clone)]
pub enum ContractError
{
    /// ABI encoding error
    /// ABI编码错误
    AbiError(String),

    /// ABI decoding error
    /// ABI解码错误
    DecodingError(String),

    /// Contract call error
    /// 合约调用错误
    CallError(String),

    /// RPC error
    /// RPC错误
    RpcError(String),
}

impl fmt::Display for ContractError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::AbiError(msg) => write!(f, "ABI encoding error: {}", msg),
            Self::DecodingError(msg) => write!(f, "ABI decoding error: {}", msg),
            Self::CallError(msg) => write!(f, "Contract call error: {}", msg),
            Self::RpcError(msg) => write!(f, "RPC error: {}", msg),
        }
    }
}

impl std::error::Error for ContractError {}

/// Contract ABI function selector
/// 合约ABI函数选择器
///
/// First 4 bytes of the Keccak-256 hash of the function signature.
/// 函数签名的Keccak-256哈希的前4个字节。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSelector(pub [u8; 4]);

impl FunctionSelector
{
    /// Create from bytes
    /// 从字节创建
    pub const fn from_bytes(bytes: [u8; 4]) -> Self
    {
        Self(bytes)
    }

    /// Compute from function signature
    /// 从函数签名计算
    pub fn from_signature(signature: &str) -> Self
    {
        let hash = crate::wallet::keccak256(signature.as_bytes());
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash[..4]);
        Self(selector)
    }

    /// Convert to hex
    /// 转换为十六进制
    pub fn to_hex(&self) -> String
    {
        format!("0x{}", hex::encode(self.0))
    }
}

/// Contract call parameters
/// 合约调用参数
#[derive(Debug, Clone)]
pub struct CallParams
{
    /// Function selector
    /// 函数选择器
    pub selector: FunctionSelector,

    /// Encoded parameters
    /// 编码后的参数
    pub data: Vec<u8>,
}

impl CallParams
{
    /// Create new call parameters
    /// 创建新的调用参数
    pub fn new(selector: FunctionSelector) -> Self
    {
        Self {
            selector,
            data: Vec::new(),
        }
    }

    /// Add address parameter
    /// 添加地址参数
    pub fn push_address(mut self, address: &Address) -> Self
    {
        self.data.extend_from_slice(&address.0);
        self
    }

    /// Add uint256 parameter (32 bytes)
    /// 添加uint256参数（32字节）
    pub fn push_uint256(mut self, value: &[u8; 32]) -> Self
    {
        self.data.extend_from_slice(value);
        self
    }

    /// Add bytes parameter
    /// 添加bytes参数
    pub fn push_bytes(mut self, bytes: &[u8]) -> Self
    {
        // Encode length and offset (simplified)
        let len = bytes.len();
        // Pad to 32 bytes
        let padding = (32 - (len % 32)) % 32;
        self.data.extend_from_slice(&[0u8; 32]);
        self.data.extend_from_slice(bytes);
        self.data.extend_from_slice(&vec![0u8; padding]);
        self
    }

    /// Build the full call data
    /// 构建完整的调用数据
    pub fn build(self) -> Vec<u8>
    {
        let mut result = Vec::with_capacity(4 + self.data.len());
        result.extend_from_slice(&self.selector.0);
        result.extend_from_slice(&self.data);
        result
    }
}

/// Contract
/// 合约
///
/// Represents a smart contract that can be called.
/// 表示可以调用的智能合约。
///
/// This is available only when the `rpc` feature is enabled.
/// 仅当启用`rpc`功能时可用。
#[cfg(feature = "rpc")]
pub struct Contract<'a>
{
    /// Contract address
    /// 合约地址
    address: Address,

    /// RPC client
    /// RPC客户端
    client: &'a crate::rpc::RpcClient,
}

#[cfg(feature = "rpc")]
impl<'a> Contract<'a>
{
    /// Create a new contract instance
    /// 创建新的合约实例
    pub fn new(address: Address, client: &'a crate::rpc::RpcClient) -> Self
    {
        Self { address, client }
    }

    /// Get the contract address
    /// 获取合约地址
    pub fn address(&self) -> Address
    {
        self.address
    }

    /// Call a read-only contract function
    /// 调用只读合约函数
    pub async fn call_read_only(
        &self,
        selector: &FunctionSelector,
        params: &[u8],
    ) -> Result<Vec<u8>, ContractError>
    {
        let mut call_data = Vec::with_capacity(4 + params.len());
        call_data.extend_from_slice(&selector.0);
        call_data.extend_from_slice(params);

        self.client
            .call_contract(&self.address, &call_data, BlockNumber::Latest)
            .await
            .map_err(|e| ContractError::RpcError(e.to_string()))
            .and_then(|result| {
                // Remove "0x" prefix and decode
                let hex = result.strip_prefix("0x").unwrap_or(&result);
                hex::decode(hex).map_err(|e| ContractError::DecodingError(e.to_string()))
            })
    }

    /// Call a contract function by name
    /// 按名称调用合约函数
    pub async fn call_by_name(
        &self,
        function: &str,
        params: &[String],
    ) -> Result<String, ContractError>
    {
        let selector = FunctionSelector::from_signature(function);
        let call_data = selector.to_hex();

        let mut full_call = call_data;
        for param in params
        {
            // For simple hex addresses, append them directly
            if param.starts_with("0x")
            {
                full_call.push_str(&param[2..]);
            }
            else
            {
                // Encode as uint256 (left-padded to 64 hex chars)
                let value = if param.starts_with("0x")
                {
                    &param[2..]
                }
                else
                {
                    param
                };
                full_call
                    .push_str(&format!("{:064x}", u64::from_str_radix(value, 16).unwrap_or(0)));
            }
        }

        self.client
            .call_contract(
                &self.address,
                &hex::decode(&full_call).map_err(|e| ContractError::AbiError(e.to_string()))?,
                BlockNumber::Latest,
            )
            .await
            .map_err(|e| ContractError::RpcError(e.to_string()))
    }

    /// Send a transaction to a contract function
    /// 向合约函数发送交易
    pub async fn send_transaction(
        &self,
        selector: &FunctionSelector,
        params: &[u8],
        value: Option<u128>,
    ) -> Result<TxHash, ContractError>
    {
        // This would require signing a transaction
        // For now, return a placeholder error
        Err(ContractError::CallError("Transaction signing not implemented".to_string()))
    }
}

#[cfg(feature = "rpc")]
impl<'a> fmt::Debug for Contract<'a>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Contract")
            .field("address", &self.address)
            .finish()
    }
}

/// Contract call builder
/// 合约调用构建器
///
/// Builder for making contract calls.
/// 用于进行合约调用的构建器。
#[cfg(feature = "rpc")]
pub struct ContractCall<'a, 'b>
{
    /// Contract reference
    /// 合约引用
    contract: &'a Contract<'b>,

    /// Function selector
    /// 函数选择器
    selector: Option<FunctionSelector>,

    /// Call parameters
    /// 调用参数
    params: Vec<u8>,

    /// ETH value to send
    /// 要发送的ETH金额
    value: Option<u128>,
}

#[cfg(feature = "rpc")]
impl<'a, 'b> ContractCall<'a, 'b>
{
    /// Create a new contract call
    /// 创建新的合约调用
    pub fn new(contract: &'a Contract<'b>) -> Self
    {
        Self {
            contract,
            selector: None,
            params: Vec::new(),
            value: None,
        }
    }

    /// Set the function selector
    /// 设置函数选择器
    pub fn selector(mut self, selector: FunctionSelector) -> Self
    {
        self.selector = Some(selector);
        self
    }

    /// Add a parameter
    /// 添加参数
    pub fn param(mut self, param: &[u8]) -> Self
    {
        self.params.extend_from_slice(param);
        self
    }

    /// Add an address parameter
    /// 添加地址参数
    pub fn address_param(mut self, address: &Address) -> Self
    {
        // Pad address to 32 bytes
        self.params.extend_from_slice(&[0u8; 12]);
        self.params.extend_from_slice(&address.0);
        self
    }

    /// Add a uint256 parameter
    /// 添加uint256参数
    pub fn uint256_param(mut self, value: &[u8; 32]) -> Self
    {
        self.params.extend_from_slice(value);
        self
    }

    /// Set the ETH value to send
    /// 设置要发送的ETH金额
    pub fn value(mut self, value: u128) -> Self
    {
        self.value = Some(value);
        self
    }

    /// Execute the call (read-only)
    /// 执行调用（只读）
    pub async fn call(self) -> Result<Vec<u8>, ContractError>
    {
        let selector = self
            .selector
            .ok_or_else(|| ContractError::CallError("Function selector not set".to_string()))?;

        self.contract._call(&selector, &self.params).await
    }

    /// Execute the call and decode as String
    /// 执行调用并解码为字符串
    pub async fn call_string(self) -> Result<String, ContractError>
    {
        let bytes = self.call().await?;
        String::from_utf8(bytes).map_err(|e| ContractError::DecodingError(e.to_string()))
    }
}

#[cfg(feature = "rpc")]
impl<'a, 'b> Contract<'a>
{
    /// Internal call method
    /// 内部调用方法
    async fn _call(
        &self,
        selector: &FunctionSelector,
        params: &[u8],
    ) -> Result<Vec<u8>, ContractError>
    {
        let mut call_data = Vec::with_capacity(4 + params.len());
        call_data.extend_from_slice(&selector.0);
        call_data.extend_from_slice(params);

        self.client
            .call_contract(&self.address, &call_data, BlockNumber::Latest)
            .await
            .map_err(|e| ContractError::RpcError(e.to_string()))
            .and_then(|result| {
                let hex = result.strip_prefix("0x").unwrap_or(&result);
                hex::decode(hex).map_err(|e| ContractError::DecodingError(e.to_string()))
            })
    }
}

/// ERC20 standard interface
/// ERC20标准接口
///
/// Pre-defined interface for ERC20 tokens.
/// ERC20代币的预定义接口。
#[cfg(feature = "rpc")]
#[derive(Debug)]
pub struct ERC20;

#[cfg(feature = "rpc")]
impl ERC20
{
    /// Function selector for approve
    /// approve的函数选择器
    pub const APPROVE: FunctionSelector = FunctionSelector::from_bytes([0x09, 0x5e, 0xa7, 0xb3]);
    /// Function selector for balanceOf
    /// balanceOf的函数选择器
    pub const BALANCE_OF: FunctionSelector = FunctionSelector::from_bytes([0x70, 0xa0, 0x82, 0x31]);
    /// Function selector for totalSupply
    /// totalSupply的函数选择器
    pub const TOTAL_SUPPLY: FunctionSelector =
        FunctionSelector::from_bytes([0x18, 0x16, 0x0d, 0xdd]);
    /// Function selector for transfer
    /// transfer的函数选择器
    pub const TRANSFER: FunctionSelector = FunctionSelector::from_bytes([0xa9, 0x05, 0x9c, 0xbb]);
}

/// ERC721 standard interface
/// ERC721标准接口
///
/// Pre-defined interface for ERC721 NFTs.
/// ERC721 NFT的预定义接口。
#[cfg(feature = "rpc")]
#[derive(Debug)]
pub struct ERC721;

#[cfg(feature = "rpc")]
impl ERC721
{
    /// Function selector for ownerOf
    /// ownerOf的函数选择器
    pub const OWNER_OF: FunctionSelector = FunctionSelector::from_bytes([0x63, 0x52, 0x21, 0x1e]);
    /// Function selector for safeTransferFrom
    /// safeTransferFrom的函数选择器
    pub const SAFE_TRANSFER_FROM: FunctionSelector =
        FunctionSelector::from_bytes([0x4a, 0x39, 0xdc, 0x06]);
    /// Function selector for transferFrom
    /// transferFrom的函数选择器
    pub const TRANSFER_FROM: FunctionSelector =
        FunctionSelector::from_bytes([0x23, 0xb8, 0x72, 0xdd]);
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_function_selector_from_signature()
    {
        let selector = FunctionSelector::from_signature("balanceOf(address)");
        assert_eq!(selector.0.len(), 4);
    }

    #[test]
    fn test_function_selector_to_hex()
    {
        let selector = FunctionSelector::from_bytes([0x70, 0xa0, 0x82, 0x31]);
        assert_eq!(selector.to_hex(), "0x70a08231");
    }

    #[test]
    fn test_call_params_new()
    {
        let selector = FunctionSelector::from_bytes([0x70, 0xa0, 0x82, 0x31]);
        let params = CallParams::new(selector);
        assert_eq!(params.data.len(), 0);
    }

    #[test]
    fn test_call_params_push_address()
    {
        let selector = FunctionSelector::from_bytes([0x70, 0xa0, 0x82, 0x31]);
        let addr = Address::zero();
        let params = CallParams::new(selector).push_address(&addr);
        assert_eq!(params.data.len(), 20);
    }

    #[test]
    fn test_contract_error_display()
    {
        let err = ContractError::AbiError("test error".to_string());
        assert!(err.to_string().contains("ABI encoding error"));
    }

    #[test]
    #[cfg(feature = "rpc")]
    fn test_erc20_constants()
    {
        assert_eq!(ERC20::BALANCE_OF.0, [0x70, 0xa0, 0x82, 0x31]);
        assert_eq!(ERC20::TRANSFER.0, [0xa9, 0x05, 0x9c, 0xbb]);
    }

    #[test]
    #[cfg(feature = "rpc")]
    fn test_erc721_constants()
    {
        assert_eq!(ERC721::OWNER_OF.0, [0x63, 0x52, 0x21, 0x1e]);
        assert_eq!(ERC721::TRANSFER_FROM.0, [0x23, 0xb8, 0x72, 0xdd]);
    }
}
