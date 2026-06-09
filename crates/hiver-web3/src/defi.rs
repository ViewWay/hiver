//! DeFi primitives module
//! DeFi原语模块
//!
//! # Overview / 概述
//!
//! This module provides DeFi primitive interfaces including token standards
//! (ERC-20, ERC-721, ERC-1155) and DEX router interactions (Uniswap V2).
//!
//! 本模块提供DeFi原语接口，包括代币标准（ERC-20、ERC-721、ERC-1155）
//! 和DEX路由器交互（Uniswap V2）。

#![allow(clippy::indexing_slicing)]
#![allow(clippy::cast_precision_loss)]
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Uniswap V2 SDK, OpenZeppelin token interfaces
//! - DeFi protocol integration layer
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_web3::defi::{Erc20, UniswapV2Router};
//! use hiver_web3::wallet::Address;
//!
//! let token_addr = Address::from_hex("0x...")?;
//! let erc20 = Erc20::new(token_addr);
//! println!("Selector: {}", erc20.balance_of_selector().to_hex());
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

use crate::{
    contract::{ContractError, FunctionSelector},
    wallet::Address,
};

// ---------------------------------------------------------------------------
// ABI encoding helpers / ABI编码辅助工具
// ---------------------------------------------------------------------------

/// Encode a 32-byte big-endian uint256 from a u64 value.
/// 从u64值编码32字节大端uint256。
fn encode_uint256(value: u64) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[24..32].copy_from_slice(&value.to_be_bytes());
    buf
}

/// Encode an address as a 32-byte ABI word (left-padded with zeros).
/// 将地址编码为32字节ABI字（左侧补零）。
fn encode_address(addr: &Address) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[12..32].copy_from_slice(&addr.0);
    buf
}

/// Decode a uint256 from a 32-byte ABI word into u64 (truncates upper bytes).
/// 从32字节ABI字解码uint256为u64（截断高位字节）。
fn decode_uint64(bytes: &[u8]) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes[24..32]);
    u64::from_be_bytes(arr)
}

/// Build call data from a selector and a slice of 32-byte ABI-encoded params.
/// 从选择器和32字节ABI编码参数切片构建调用数据。
fn build_call_data(selector: &FunctionSelector, params: &[[u8; 32]]) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 + params.len() * 32);
    data.extend_from_slice(&selector.0);
    for p in params {
        data.extend_from_slice(p);
    }
    data
}

// ---------------------------------------------------------------------------
// ERC-20
// ---------------------------------------------------------------------------

/// ERC-20 token interface.
/// ERC-20代币接口。
///
/// Provides function selectors and ABI-encoded call builders for the standard
/// ERC-20 token operations.
///
/// 提供标准ERC-20代币操作的函数选择器和ABI编码调用构建器。
#[derive(Debug, Clone)]
pub struct Erc20 {
    /// Token contract address.
    /// 代币合约地址。
    address: Address,
}

impl Erc20 {
    /// Create a new ERC-20 wrapper for the given contract address.
    /// 为给定的合约地址创建新的ERC-20包装器。
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    /// Get the token contract address.
    /// 获取代币合约地址。
    pub fn address(&self) -> Address {
        self.address
    }

    // -- Function selectors / 函数选择器 --

    /// Selector for `balanceOf(address)`.
    /// `balanceOf(address)` 的选择器。
    pub fn balance_of_selector() -> FunctionSelector {
        FunctionSelector::from_signature("balanceOf(address)")
    }

    /// Selector for `transfer(address,uint256)`.
    /// `transfer(address,uint256)` 的选择器。
    pub fn transfer_selector() -> FunctionSelector {
        FunctionSelector::from_signature("transfer(address,uint256)")
    }

    /// Selector for `approve(address,uint256)`.
    /// `approve(address,uint256)` 的选择器。
    pub fn approve_selector() -> FunctionSelector {
        FunctionSelector::from_signature("approve(address,uint256)")
    }

    /// Selector for `allowance(address,address)`.
    /// `allowance(address,address)` 的选择器。
    pub fn allowance_selector() -> FunctionSelector {
        FunctionSelector::from_signature("allowance(address,address)")
    }

    /// Selector for `totalSupply()`.
    /// `totalSupply()` 的选择器。
    pub fn total_supply_selector() -> FunctionSelector {
        FunctionSelector::from_signature("totalSupply()")
    }

    /// Selector for `decimals()`.
    /// `decimals()` 的选择器。
    pub fn decimals_selector() -> FunctionSelector {
        FunctionSelector::from_signature("decimals()")
    }

    /// Selector for `name()`.
    /// `name()` 的选择器。
    pub fn name_selector() -> FunctionSelector {
        FunctionSelector::from_signature("name()")
    }

    /// Selector for `symbol()`.
    /// `symbol()` 的选择器。
    pub fn symbol_selector() -> FunctionSelector {
        FunctionSelector::from_signature("symbol()")
    }

    // -- ABI-encoded call data builders / ABI编码调用数据构建器 --

    /// Build `balanceOf(address)` call data.
    /// 构建 `balanceOf(address)` 调用数据。
    pub fn balance_of_call(owner: &Address) -> Vec<u8> {
        build_call_data(&Self::balance_of_selector(), &[encode_address(owner)])
    }

    /// Build `transfer(address,uint256)` call data.
    /// 构建 `transfer(address,uint256)` 调用数据。
    pub fn transfer_call(to: &Address, amount: u64) -> Vec<u8> {
        build_call_data(&Self::transfer_selector(), &[encode_address(to), encode_uint256(amount)])
    }

    /// Build `approve(address,uint256)` call data.
    /// 构建 `approve(address,uint256)` 调用数据。
    pub fn approve_call(spender: &Address, amount: u64) -> Vec<u8> {
        build_call_data(
            &Self::approve_selector(),
            &[encode_address(spender), encode_uint256(amount)],
        )
    }

    /// Build `allowance(address,address)` call data.
    /// 构建 `allowance(address,address)` 调用数据。
    pub fn allowance_call(owner: &Address, spender: &Address) -> Vec<u8> {
        build_call_data(
            &Self::allowance_selector(),
            &[encode_address(owner), encode_address(spender)],
        )
    }

    /// Build `totalSupply()` call data.
    /// 构建 `totalSupply()` 调用数据。
    pub fn total_supply_call() -> Vec<u8> {
        build_call_data(&Self::total_supply_selector(), &[])
    }

    /// Build `decimals()` call data.
    /// 构建 `decimals()` 调用数据。
    pub fn decimals_call() -> Vec<u8> {
        build_call_data(&Self::decimals_selector(), &[])
    }

    /// Build `name()` call data.
    /// 构建 `name()` 调用数据。
    pub fn name_call() -> Vec<u8> {
        build_call_data(&Self::name_selector(), &[])
    }

    /// Build `symbol()` call data.
    /// 构建 `symbol()` 调用数据。
    pub fn symbol_call() -> Vec<u8> {
        build_call_data(&Self::symbol_selector(), &[])
    }

    // -- Response decoders / 响应解码器 --

    /// Decode a `balanceOf` response (uint256) into tokens with the given decimals.
    /// 将 `balanceOf` 响应（uint256）解码为具有给定精度的代币数量。
    pub fn decode_balance(raw: &[u8], decimals: u8) -> f64 {
        if raw.len() < 32 {
            return 0.0;
        }
        let val = decode_uint64(raw);
        val as f64 / 10u64.pow(decimals as u32) as f64
    }

    /// Decode a `decimals` response (uint8) from the raw return data.
    /// 从原始返回数据解码 `decimals` 响应（uint8）。
    pub fn decode_decimals(raw: &[u8]) -> u8 {
        if raw.len() < 32 {
            return 18;
        }
        raw[31]
    }

    /// Decode a `totalSupply` response (uint256) into a u64.
    /// 将 `totalSupply` 响应（uint256）解码为u64。
    pub fn decode_total_supply(raw: &[u8]) -> u64 {
        if raw.len() < 32 {
            return 0;
        }
        decode_uint64(raw)
    }

    /// Decode a boolean response (`transfer`, `approve`) from raw return data.
    /// 从原始返回数据解码布尔响应（`transfer`、`approve`）。
    pub fn decode_bool(raw: &[u8]) -> bool {
        if raw.len() < 32 {
            return false;
        }
        raw[31] != 0
    }

    /// Decode a `name` or `symbol` response (string) from raw return data.
    /// 从原始返回数据解码 `name` 或 `symbol` 响应（字符串）。
    ///
    /// Handles the standard ABI encoding for dynamic `string` / `bytes`.
    /// 处理动态 `string` / `bytes` 的标准ABI编码。
    pub fn decode_string(raw: &[u8]) -> Result<String, DeFiError> {
        if raw.len() < 64 {
            return Err(DeFiError::DecodingError("Insufficient data for string".into()));
        }
        // offset of the dynamic data
        let offset = decode_uint64(&raw[0..32]) as usize;
        if raw.len() < offset + 32 {
            return Err(DeFiError::DecodingError("Invalid string offset".into()));
        }
        let len = decode_uint64(&raw[offset..offset + 32]) as usize;
        if raw.len() < offset + 32 + len {
            return Err(DeFiError::DecodingError("String data truncated".into()));
        }
        String::from_utf8(raw[offset + 32..offset + 32 + len].to_vec())
            .map_err(|e| DeFiError::DecodingError(format!("Invalid UTF-8: {}", e)))
    }
}

// ---------------------------------------------------------------------------
// ERC-721
// ---------------------------------------------------------------------------

/// ERC-721 non-fungible token interface.
/// ERC-721非同质化代币接口。
///
/// Provides function selectors and ABI-encoded call builders for the standard
/// ERC-721 NFT operations.
///
/// 提供标准ERC-721 NFT操作的函数选择器和ABI编码调用构建器。
#[derive(Debug, Clone)]
pub struct Erc721 {
    /// NFT contract address.
    /// NFT合约地址。
    address: Address,
}

impl Erc721 {
    /// Create a new ERC-721 wrapper for the given contract address.
    /// 为给定的合约地址创建新的ERC-721包装器。
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    /// Get the NFT contract address.
    /// 获取NFT合约地址。
    pub fn address(&self) -> Address {
        self.address
    }

    // -- Function selectors / 函数选择器 --

    /// Selector for `balanceOf(address)`.
    /// `balanceOf(address)` 的选择器。
    pub fn balance_of_selector() -> FunctionSelector {
        FunctionSelector::from_signature("balanceOf(address)")
    }

    /// Selector for `ownerOf(uint256)`.
    /// `ownerOf(uint256)` 的选择器。
    pub fn owner_of_selector() -> FunctionSelector {
        FunctionSelector::from_signature("ownerOf(uint256)")
    }

    /// Selector for `transferFrom(address,address,uint256)`.
    /// `transferFrom(address,address,uint256)` 的选择器。
    pub fn transfer_from_selector() -> FunctionSelector {
        FunctionSelector::from_signature("transferFrom(address,address,uint256)")
    }

    /// Selector for `approve(address,uint256)`.
    /// `approve(address,uint256)` 的选择器。
    pub fn approve_selector() -> FunctionSelector {
        FunctionSelector::from_signature("approve(address,uint256)")
    }

    /// Selector for `tokenURI(uint256)`.
    /// `tokenURI(uint256)` 的选择器。
    pub fn token_uri_selector() -> FunctionSelector {
        FunctionSelector::from_signature("tokenURI(uint256)")
    }

    /// Selector for `mint(address,uint256)`.
    /// `mint(address,uint256)` 的选择器。
    pub fn mint_selector() -> FunctionSelector {
        FunctionSelector::from_signature("mint(address,uint256)")
    }

    // -- ABI-encoded call data builders / ABI编码调用数据构建器 --

    /// Build `balanceOf(address)` call data.
    /// 构建 `balanceOf(address)` 调用数据。
    pub fn balance_of_call(owner: &Address) -> Vec<u8> {
        build_call_data(&Self::balance_of_selector(), &[encode_address(owner)])
    }

    /// Build `ownerOf(uint256)` call data.
    /// 构建 `ownerOf(uint256)` 调用数据。
    pub fn owner_of_call(token_id: u64) -> Vec<u8> {
        build_call_data(&Self::owner_of_selector(), &[encode_uint256(token_id)])
    }

    /// Build `transferFrom(address,address,uint256)` call data.
    /// 构建 `transferFrom(address,address,uint256)` 调用数据。
    pub fn transfer_from_call(from: &Address, to: &Address, token_id: u64) -> Vec<u8> {
        build_call_data(
            &Self::transfer_from_selector(),
            &[
                encode_address(from),
                encode_address(to),
                encode_uint256(token_id),
            ],
        )
    }

    /// Build `approve(address,uint256)` call data.
    /// 构建 `approve(address,uint256)` 调用数据。
    pub fn approve_call(to: &Address, token_id: u64) -> Vec<u8> {
        build_call_data(&Self::approve_selector(), &[encode_address(to), encode_uint256(token_id)])
    }

    /// Build `tokenURI(uint256)` call data.
    /// 构建 `tokenURI(uint256)` 调用数据。
    pub fn token_uri_call(token_id: u64) -> Vec<u8> {
        build_call_data(&Self::token_uri_selector(), &[encode_uint256(token_id)])
    }

    /// Build `mint(address,uint256)` call data.
    /// 构建 `mint(address,uint256)` 调用数据。
    pub fn mint_call(to: &Address, token_id: u64) -> Vec<u8> {
        build_call_data(&Self::mint_selector(), &[encode_address(to), encode_uint256(token_id)])
    }

    // -- Response decoders / 响应解码器 --

    /// Decode `ownerOf` response into an Address.
    /// 将 `ownerOf` 响应解码为地址。
    pub fn decode_owner(raw: &[u8]) -> Result<Address, DeFiError> {
        if raw.len() < 32 {
            return Err(DeFiError::DecodingError("Insufficient data for address".into()));
        }
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&raw[12..32]);
        Ok(Address(addr))
    }

    /// Decode `balanceOf` response (uint256) into u64.
    /// 将 `balanceOf` 响应（uint256）解码为u64。
    pub fn decode_balance(raw: &[u8]) -> u64 {
        if raw.len() < 32 {
            return 0;
        }
        decode_uint64(raw)
    }

    /// Decode `tokenURI` response (string).
    /// 解码 `tokenURI` 响应（字符串）。
    pub fn decode_token_uri(raw: &[u8]) -> Result<String, DeFiError> {
        Erc20::decode_string(raw)
    }
}

// ---------------------------------------------------------------------------
// ERC-1155
// ---------------------------------------------------------------------------

/// ERC-1155 multi-token interface.
/// ERC-1155多代币接口。
///
/// Provides function selectors and ABI-encoded call builders for the standard
/// ERC-1155 multi-token operations.
///
/// 提供标准ERC-1155多代币操作的函数选择器和ABI编码调用构建器。
#[derive(Debug, Clone)]
pub struct Erc1155 {
    /// Multi-token contract address.
    /// 多代币合约地址。
    address: Address,
}

impl Erc1155 {
    /// Create a new ERC-1155 wrapper for the given contract address.
    /// 为给定的合约地址创建新的ERC-1155包装器。
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    /// Get the multi-token contract address.
    /// 获取多代币合约地址。
    pub fn address(&self) -> Address {
        self.address
    }

    // -- Function selectors / 函数选择器 --

    /// Selector for `balanceOf(address,uint256)`.
    /// `balanceOf(address,uint256)` 的选择器。
    pub fn balance_of_selector() -> FunctionSelector {
        FunctionSelector::from_signature("balanceOf(address,uint256)")
    }

    /// Selector for `balanceOfBatch(address[],uint256[])`.
    /// `balanceOfBatch(address[],uint256[])` 的选择器。
    pub fn balance_of_batch_selector() -> FunctionSelector {
        FunctionSelector::from_signature("balanceOfBatch(address[],uint256[])")
    }

    /// Selector for `safeTransferFrom(address,address,uint256,uint256,bytes)`.
    /// `safeTransferFrom(address,address,uint256,uint256,bytes)` 的选择器。
    pub fn safe_transfer_from_selector() -> FunctionSelector {
        FunctionSelector::from_signature("safeTransferFrom(address,address,uint256,uint256,bytes)")
    }

    // -- ABI-encoded call data builders / ABI编码调用数据构建器 --

    /// Build `balanceOf(address,uint256)` call data.
    /// 构建 `balanceOf(address,uint256)` 调用数据。
    pub fn balance_of_call(account: &Address, token_id: u64) -> Vec<u8> {
        build_call_data(
            &Self::balance_of_selector(),
            &[encode_address(account), encode_uint256(token_id)],
        )
    }

    /// Build `balanceOfBatch(address[],uint256[])` call data.
    /// 构建 `balanceOfBatch(address[],uint256[])` 调用数据。
    pub fn balance_of_batch_call(accounts: &[Address], token_ids: &[u64]) -> Vec<u8> {
        assert_eq!(
            accounts.len(),
            token_ids.len(),
            "accounts and token_ids must have the same length"
        );
        let n = accounts.len() as u64;
        let mut params = Vec::with_capacity(2 + accounts.len() * 2 + token_ids.len() * 2);
        // Offset to first dynamic array (accounts[]) = 0x60 (3 words head)
        let offset_accounts: [u8; 32] = encode_uint256(0x60);
        // Offset to second dynamic array (ids[]) = 0x60 + 0x20 + n*0x20
        let offset_ids = 0x60u64 + 0x20 + n * 0x20;
        let offset_ids_enc: [u8; 32] = encode_uint256(offset_ids);

        params.push(offset_accounts);
        params.push(offset_ids_enc);
        params.push(encode_uint256(n));
        for acc in accounts {
            params.push(encode_address(acc));
        }
        params.push(encode_uint256(n));
        for id in token_ids {
            params.push(encode_uint256(*id));
        }

        let mut data = Vec::with_capacity(4 + params.len() * 32);
        data.extend_from_slice(&Self::balance_of_batch_selector().0);
        for p in &params {
            data.extend_from_slice(p);
        }
        data
    }

    /// Build `safeTransferFrom(address,address,uint256,uint256,bytes)` call data.
    /// 构建 `safeTransferFrom(address,address,uint256,uint256,bytes)` 调用数据。
    ///
    /// Uses an empty bytes parameter (`0x`).
    /// 使用空的bytes参数（`0x`）。
    pub fn safe_transfer_from_call(
        from: &Address,
        to: &Address,
        token_id: u64,
        amount: u64,
    ) -> Vec<u8> {
        // Head: [from, to, id, amount, dataOffset]  = 5 words
        // dataOffset = 5 * 32 = 160 = 0xa0
        // dataLength = 0
        let mut data = Vec::with_capacity(4 + 7 * 32);
        data.extend_from_slice(&Self::safe_transfer_from_selector().0);
        data.extend_from_slice(&encode_address(from));
        data.extend_from_slice(&encode_address(to));
        data.extend_from_slice(&encode_uint256(token_id));
        data.extend_from_slice(&encode_uint256(amount));
        data.extend_from_slice(&encode_uint256(0xa0)); // offset to bytes
        data.extend_from_slice(&[0u8; 32]); // length = 0
        data
    }

    // -- Response decoders / 响应解码器 --

    /// Decode `balanceOf` response (uint256) into u64.
    /// 将 `balanceOf` 响应（uint256）解码为u64。
    pub fn decode_balance(raw: &[u8]) -> u64 {
        if raw.len() < 32 {
            return 0;
        }
        decode_uint64(raw)
    }

    /// Decode `balanceOfBatch` response (uint256[]) into a Vec of u64.
    /// 将 `balanceOfBatch` 响应（uint256[]）解码为u64向量。
    pub fn decode_balance_batch(raw: &[u8]) -> Result<Vec<u64>, DeFiError> {
        if raw.len() < 64 {
            return Err(DeFiError::DecodingError("Insufficient data for batch".into()));
        }
        let offset = decode_uint64(&raw[0..32]) as usize;
        if raw.len() < offset + 32 {
            return Err(DeFiError::DecodingError("Invalid batch offset".into()));
        }
        let count = decode_uint64(&raw[offset..offset + 32]) as usize;
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let start = offset + 32 + i * 32;
            if raw.len() < start + 32 {
                return Err(DeFiError::DecodingError("Batch data truncated".into()));
            }
            result.push(decode_uint64(&raw[start..start + 32]));
        }
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Uniswap V2 Router
// ---------------------------------------------------------------------------

/// Uniswap V2 Router interface.
/// Uniswap V2路由器接口。
///
/// Provides function selectors and ABI-encoded call builders for common
/// Uniswap V2 DEX operations: token swaps, liquidity provision/removal,
/// and amount-out quoting.
///
/// 提供常见Uniswap V2 DEX操作的函数选择器和ABI编码调用构建器：
/// 代币兑换、流动性添加/移除和输出数量查询。
#[derive(Debug, Clone)]
pub struct UniswapV2Router {
    /// Router contract address.
    /// 路由器合约地址。
    address: Address,
}

impl UniswapV2Router {
    /// Create a new Uniswap V2 Router wrapper for the given contract address.
    /// 为给定的合约地址创建新的Uniswap V2路由器包装器。
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    /// Get the router contract address.
    /// 获取路由器合约地址。
    pub fn address(&self) -> Address {
        self.address
    }

    // -- Function selectors / 函数选择器 --

    /// Selector for `getAmountsOut(uint256,address[])`.
    /// `getAmountsOut(uint256,address[])` 的选择器。
    pub fn get_amounts_out_selector() -> FunctionSelector {
        FunctionSelector::from_signature("getAmountsOut(uint256,address[])")
    }

    /// Selector for `swapExactTokensForTokens(uint256,uint256,address[],address,uint256)`.
    /// `swapExactTokensForTokens(...)` 的选择器。
    pub fn swap_exact_tokens_for_tokens_selector() -> FunctionSelector {
        FunctionSelector::from_signature(
            "swapExactTokensForTokens(uint256,uint256,address[],address,uint256)",
        )
    }

    /// Selector for `swapExactETHForTokens(uint256,address[],address,uint256)`.
    /// `swapExactETHForTokens(...)` 的选择器。
    pub fn swap_exact_eth_for_tokens_selector() -> FunctionSelector {
        FunctionSelector::from_signature("swapExactETHForTokens(uint256,address[],address,uint256)")
    }

    /// Selector for `swapExactTokensForETH(uint256,uint256,address[],address,uint256)`.
    /// `swapExactTokensForETH(...)` 的选择器。
    pub fn swap_exact_tokens_for_eth_selector() -> FunctionSelector {
        FunctionSelector::from_signature(
            "swapExactTokensForETH(uint256,uint256,address[],address,uint256)",
        )
    }

    /// Selector for
    /// `addLiquidity(address,address,uint256,uint256,uint256,uint256,address,uint256)`.
    /// `addLiquidity(...)` 的选择器。
    pub fn add_liquidity_selector() -> FunctionSelector {
        FunctionSelector::from_signature(
            "addLiquidity(address,address,uint256,uint256,uint256,uint256,address,uint256)",
        )
    }

    /// Selector for `removeLiquidity(address,address,uint256,uint256,uint256,address,uint256)`.
    /// `removeLiquidity(...)` 的选择器。
    pub fn remove_liquidity_selector() -> FunctionSelector {
        FunctionSelector::from_signature(
            "removeLiquidity(address,address,uint256,uint256,uint256,address,uint256)",
        )
    }

    // -- ABI-encoded call data builders / ABI编码调用数据构建器 --

    /// Build `getAmountsOut(uint256,address[])` call data.
    /// 构建 `getAmountsOut(uint256,address[])` 调用数据。
    pub fn get_amounts_out_call(amount_in: u64, path: &[Address]) -> Vec<u8> {
        // Head: [amountIn, pathOffset]  = 2 words
        // pathOffset = 0x40
        // Dynamic array: [length, addr0, addr1, ...]
        let path_offset = encode_uint256(0x40);
        let len_enc = encode_uint256(path.len() as u64);
        let mut params = vec![encode_uint256(amount_in), path_offset, len_enc];
        for addr in path {
            params.push(encode_address(addr));
        }

        let mut data = Vec::with_capacity(4 + params.len() * 32);
        data.extend_from_slice(&Self::get_amounts_out_selector().0);
        for p in &params {
            data.extend_from_slice(p);
        }
        data
    }

    /// Build `swapExactTokensForTokens(...)` call data.
    /// 构建 `swapExactTokensForTokens(...)` 调用数据。
    pub fn swap_exact_tokens_for_tokens_call(
        amount_in: u64,
        amount_out_min: u64,
        path: &[Address],
        to: &Address,
        deadline: u64,
    ) -> Vec<u8> {
        Self::build_swap_call(
            &Self::swap_exact_tokens_for_tokens_selector(),
            amount_in,
            amount_out_min,
            path,
            to,
            deadline,
        )
    }

    /// Build `swapExactETHForTokens(...)` call data.
    /// 构建 `swapExactETHForTokens(...)` 调用数据。
    pub fn swap_exact_eth_for_tokens_call(
        amount_out_min: u64,
        path: &[Address],
        to: &Address,
        deadline: u64,
    ) -> Vec<u8> {
        // Head: [amountOutMin, pathOffset, to, deadline]
        // pathOffset = 0x80 (4 words head)
        let path_offset = encode_uint256(0x80);
        let len_enc = encode_uint256(path.len() as u64);
        let mut params = vec![
            encode_uint256(amount_out_min),
            path_offset,
            encode_address(to),
            encode_uint256(deadline),
            len_enc,
        ];
        for addr in path {
            params.push(encode_address(addr));
        }

        let mut data = Vec::with_capacity(4 + params.len() * 32);
        data.extend_from_slice(&Self::swap_exact_eth_for_tokens_selector().0);
        for p in &params {
            data.extend_from_slice(p);
        }
        data
    }

    /// Build `swapExactTokensForETH(...)` call data.
    /// 构建 `swapExactTokensForETH(...)` 调用数据。
    pub fn swap_exact_tokens_for_eth_call(
        amount_in: u64,
        amount_out_min: u64,
        path: &[Address],
        to: &Address,
        deadline: u64,
    ) -> Vec<u8> {
        Self::build_swap_call(
            &Self::swap_exact_tokens_for_eth_selector(),
            amount_in,
            amount_out_min,
            path,
            to,
            deadline,
        )
    }

    /// Build `addLiquidity(...)` call data.
    /// 构建 `addLiquidity(...)` 调用数据。
    pub fn add_liquidity_call(
        token_a: &Address,
        token_b: &Address,
        amount_a_desired: u64,
        amount_b_desired: u64,
        amount_a_min: u64,
        amount_b_min: u64,
        to: &Address,
        deadline: u64,
    ) -> Vec<u8> {
        let params: [[u8; 32]; 8] = [
            encode_address(token_a),
            encode_address(token_b),
            encode_uint256(amount_a_desired),
            encode_uint256(amount_b_desired),
            encode_uint256(amount_a_min),
            encode_uint256(amount_b_min),
            encode_address(to),
            encode_uint256(deadline),
        ];
        build_call_data(&Self::add_liquidity_selector(), &params)
    }

    /// Build `removeLiquidity(...)` call data.
    /// 构建 `removeLiquidity(...)` 调用数据。
    pub fn remove_liquidity_call(
        token_a: &Address,
        token_b: &Address,
        liquidity: u64,
        amount_a_min: u64,
        amount_b_min: u64,
        to: &Address,
        deadline: u64,
    ) -> Vec<u8> {
        let params: [[u8; 32]; 7] = [
            encode_address(token_a),
            encode_address(token_b),
            encode_uint256(liquidity),
            encode_uint256(amount_a_min),
            encode_uint256(amount_b_min),
            encode_address(to),
            encode_uint256(deadline),
        ];
        build_call_data(&Self::remove_liquidity_selector(), &params)
    }

    // -- Response decoders / 响应解码器 --

    /// Decode `getAmountsOut` response (uint256[]) into a Vec of u64.
    /// 将 `getAmountsOut` 响应（uint256[]）解码为u64向量。
    pub fn decode_amounts(raw: &[u8]) -> Result<Vec<u64>, DeFiError> {
        Erc1155::decode_balance_batch(raw)
    }

    // -- Private helpers / 私有辅助方法 --

    /// Internal helper for swap calls with the standard
    /// `(uint256,uint256,address[],address,uint256)` signature.
    /// 内部辅助方法，用于标准 `(uint256,uint256,address[],address,uint256)` 签名的兑换调用。
    fn build_swap_call(
        selector: &FunctionSelector,
        amount_in: u64,
        amount_out_min: u64,
        path: &[Address],
        to: &Address,
        deadline: u64,
    ) -> Vec<u8> {
        // Head: [amountIn, amountOutMin, pathOffset, to, deadline] = 5 words
        // pathOffset = 5 * 32 = 0xa0
        let path_offset = encode_uint256(0xa0);
        let len_enc = encode_uint256(path.len() as u64);
        let mut params = vec![
            encode_uint256(amount_in),
            encode_uint256(amount_out_min),
            path_offset,
            encode_address(to),
            encode_uint256(deadline),
            len_enc,
        ];
        for addr in path {
            params.push(encode_address(addr));
        }

        let mut data = Vec::with_capacity(4 + params.len() * 32);
        data.extend_from_slice(&selector.0);
        for p in &params {
            data.extend_from_slice(p);
        }
        data
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// DeFi error type.
/// DeFi错误类型。
#[derive(Debug, Clone)]
pub enum DeFiError {
    /// ABI encoding error.
    /// ABI编码错误。
    AbiError(String),

    /// ABI decoding error.
    /// ABI解码错误。
    DecodingError(String),

    /// Contract call error.
    /// 合约调用错误。
    CallError(String),

    /// RPC error.
    /// RPC错误。
    RpcError(String),
}

impl fmt::Display for DeFiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbiError(msg) => write!(f, "ABI encoding error: {}", msg),
            Self::DecodingError(msg) => write!(f, "ABI decoding error: {}", msg),
            Self::CallError(msg) => write!(f, "Contract call error: {}", msg),
            Self::RpcError(msg) => write!(f, "RPC error: {}", msg),
        }
    }
}

impl std::error::Error for DeFiError {}

impl From<ContractError> for DeFiError {
    fn from(err: ContractError) -> Self {
        match err {
            ContractError::AbiError(msg) => DeFiError::AbiError(msg),
            ContractError::DecodingError(msg) => DeFiError::DecodingError(msg),
            ContractError::CallError(msg) => DeFiError::CallError(msg),
            ContractError::RpcError(msg) => DeFiError::RpcError(msg),
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

    #[test]
    fn test_erc20_selectors() {
        // Known Uniswap V2 selector for balanceOf(address)
        let sel = Erc20::balance_of_selector();
        assert_eq!(sel.0, [0x70, 0xa0, 0x82, 0x31]);

        let sel = Erc20::transfer_selector();
        assert_eq!(sel.0, [0xa9, 0x05, 0x9c, 0xbb]);

        let sel = Erc20::approve_selector();
        assert_eq!(sel.0, [0x09, 0x5e, 0xa7, 0xb3]);

        let sel = Erc20::total_supply_selector();
        assert_eq!(sel.0, [0x18, 0x16, 0x0d, 0xdd]);
    }

    #[test]
    fn test_erc20_call_data_balance_of() {
        let owner = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let data = Erc20::balance_of_call(&owner);
        assert_eq!(&data[0..4], &[0x70, 0xa0, 0x82, 0x31]);
        // 12 zero bytes + 20-byte address
        assert_eq!(&data[4..16], &[0u8; 12]);
        assert_eq!(&data[16..36], &owner.0);
    }

    #[test]
    fn test_erc20_call_data_transfer() {
        let to = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let data = Erc20::transfer_call(&to, 1000);
        assert_eq!(&data[0..4], &[0xa9, 0x05, 0x9c, 0xbb]);
        // Second param at offset 36
        let amount = decode_uint64(&data[36..68]);
        assert_eq!(amount, 1000);
    }

    #[test]
    fn test_erc20_decode_balance() {
        let mut raw = [0u8; 32];
        raw[31] = 42;
        let bal = Erc20::decode_balance(&raw, 18);
        assert!((bal - 42e-18).abs() < f64::EPSILON);
    }

    #[test]
    fn test_erc20_decode_decimals() {
        let mut raw = [0u8; 32];
        raw[31] = 6;
        assert_eq!(Erc20::decode_decimals(&raw), 6);
    }

    #[test]
    fn test_erc20_decode_bool() {
        let mut raw = [0u8; 32];
        raw[31] = 1;
        assert!(Erc20::decode_bool(&raw));

        raw[31] = 0;
        assert!(!Erc20::decode_bool(&raw));
    }

    #[test]
    fn test_erc20_decode_string() {
        // Manually ABI-encode "ETH" (offset=32, length=3, data="ETH" + pad)
        let mut raw = Vec::new();
        raw.extend_from_slice(&encode_uint256(32)); // offset
        raw.extend_from_slice(&encode_uint256(3)); // length
        raw.extend_from_slice(b"ETH");
        raw.extend_from_slice(&[0u8; 29]); // pad to 32
        let s = Erc20::decode_string(&raw).unwrap();
        assert_eq!(s, "ETH");
    }

    #[test]
    fn test_erc721_selectors() {
        let sel = Erc721::owner_of_selector();
        assert_eq!(sel.0, [0x63, 0x52, 0x21, 0x1e]);

        let sel = Erc721::transfer_from_selector();
        assert_eq!(sel.0, [0x23, 0xb8, 0x72, 0xdd]);
    }

    #[test]
    fn test_erc721_decode_owner() {
        let addr = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let mut raw = [0u8; 32];
        raw[12..32].copy_from_slice(&addr.0);
        let decoded = Erc721::decode_owner(&raw).unwrap();
        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_erc1155_balance_of_call() {
        let account = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let data = Erc1155::balance_of_call(&account, 42);
        // selector + address + token_id
        assert_eq!(data.len(), 4 + 64);
    }

    #[test]
    fn test_erc1155_safe_transfer_from_call() {
        let from = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let to = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let data = Erc1155::safe_transfer_from_call(&from, &to, 1, 100);
        // selector + 6 words (5 head params + 1 empty bytes length)
        assert_eq!(data.len(), 4 + 6 * 32);
    }

    #[test]
    fn test_erc1155_decode_balance_batch() {
        // Encode: offset=32, count=2, [10, 20]
        let mut raw = Vec::new();
        raw.extend_from_slice(&encode_uint256(32));
        raw.extend_from_slice(&encode_uint256(2));
        raw.extend_from_slice(&encode_uint256(10));
        raw.extend_from_slice(&encode_uint256(20));
        let result = Erc1155::decode_balance_batch(&raw).unwrap();
        assert_eq!(result, vec![10, 20]);
    }

    #[test]
    fn test_uniswap_v2_router_selectors() {
        let sel = UniswapV2Router::get_amounts_out_selector();
        assert_eq!(sel.0.len(), 4);

        let sel = UniswapV2Router::swap_exact_tokens_for_tokens_selector();
        assert_eq!(sel.0.len(), 4);

        let sel = UniswapV2Router::add_liquidity_selector();
        assert_eq!(sel.0.len(), 4);

        let sel = UniswapV2Router::remove_liquidity_selector();
        assert_eq!(sel.0.len(), 4);
    }

    #[test]
    fn test_uniswap_v2_get_amounts_out_call() {
        let token_a = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let token_b = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let data = UniswapV2Router::get_amounts_out_call(1000, &[token_a, token_b]);
        // selector + amountIn + offset + length + 2 addresses
        assert_eq!(data.len(), 4 + 5 * 32);
    }

    #[test]
    fn test_uniswap_v2_add_liquidity_call() {
        let token_a = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let token_b = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let to = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();
        let data = UniswapV2Router::add_liquidity_call(
            &token_a, &token_b, 1000, 2000, 900, 1800, &to, 9999999999,
        );
        // selector + 8 words
        assert_eq!(data.len(), 4 + 8 * 32);
    }

    #[test]
    fn test_encode_uint256() {
        let enc = encode_uint256(255);
        let mut expected = [0u8; 32];
        expected[31] = 255;
        assert_eq!(enc, expected);

        let enc = encode_uint256(256);
        let mut expected = [0u8; 32];
        expected[30] = 1;
        expected[31] = 0;
        assert_eq!(enc, expected);
    }

    #[test]
    fn test_encode_address() {
        let addr = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        let enc = encode_address(&addr);
        // First 12 bytes should be zero
        assert_eq!(&enc[0..12], &[0u8; 12]);
        // Last 20 bytes should be the address
        assert_eq!(&enc[12..32], &addr.0);
    }

    #[test]
    fn test_defi_error_display() {
        let err = DeFiError::AbiError("bad encoding".into());
        assert!(err.to_string().contains("ABI encoding error"));

        let err = DeFiError::DecodingError("bad data".into());
        assert!(err.to_string().contains("ABI decoding error"));
    }

    #[test]
    fn test_defi_error_from_contract_error() {
        let contract_err = ContractError::AbiError("test".into());
        let defi_err: DeFiError = contract_err.into();
        assert!(matches!(defi_err, DeFiError::AbiError(_)));
    }
}
