# Web3 Integration / Web3集成

> **Status**: Phase 6 Complete ✅
> **状态**: 第6阶段完成 ✅

Nexus provides native Web3 support for blockchain applications.

Nexus 为区块链应用程序提供原生 Web3 支持。

---

## Overview / 概述

Web3 features include:

Web3 功能包括：

- **Chain Abstraction** / **链抽象** - EIP-155 chain ID support with pre-configured chains
- **Wallet Management** / **钱包管理** - Local wallet with signing capabilities
- **Transaction Building** / **交易构建** - EIP-1559 and Legacy transaction types
- **RPC Client** / **RPC客户端** - HTTP client for blockchain node communication
- **Smart Contract Interface** / **智能合约接口** - ABI encoding/decoding with ERC20/ERC721 standards

---

## Quick Start / 快速开始

```rust,no_run,ignore
use hiver_web3::{
    ChainConfig, ChainId, Eip155Chain,
    LocalWallet, Address, Contract, FunctionSelector, RpcClient
};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
// Connect to Ethereum / 连接到以太坊
let chain_config = ChainConfig::ethereum_mainnet();
let rpc = RpcClient::new(&chain_config.rpc_urls[0])?;

// Create wallet / 创建钱包
let wallet = LocalWallet::new(Eip155Chain::ETHEREUM);

// Get address / 获取地址
let address = wallet.address();
println!("Wallet address: {}", address.to_checksummed());

// Interact with ERC20 contract / 与ERC20合约交互
let usdc_address = Address::from_hex("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
let contract = Contract::new(usdc_address, &rpc);

// Get balance / 查询余额
let selector = FunctionSelector::from_signature("balanceOf(address)");
let mut call_data = Vec::new();
call_data.extend_from_slice(&selector.0);
call_data.extend_from_slice(&[0u8; 12]); // Pad to 32 bytes
call_data.extend_from_slice(&address.0);

let result = rpc.call_contract(&usdc_address, &call_data, hiver_web3::BlockNumber::Latest).await?;
# Ok(())
# }
```

---

## Features / 功能

### Chain Configuration / 链配置

```rust,no_run,ignore
use hiver_web3::{ChainConfig, ChainId, Eip155Chain};

// Pre-configured chains / 预配置的链
let mainnet = ChainConfig::ethereum_mainnet();
let sepolia = ChainConfig::sepolia_testnet();
let polygon = ChainConfig::polygon();
let base = ChainConfig::base();

// Custom chain / 自定义链
let custom = ChainConfig::new(
    Eip155Chain::custom(12345),
    "My Custom Chain",
    vec!["https://rpc.example.com".to_string()],
);
```

### Wallet Management / 钱包管理

```rust,no_run,ignore
use hiver_web3::{LocalWallet, Wallet, Address};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
// Create new wallet / 创建新钱包
let wallet = LocalWallet::new(hiver_web3::Eip155Chain::ETHEREUM);
let address = wallet.address();
println!("Address: {}", address.to_checksummed());

// Sign message / 签名消息
let message = b"Hello, Web3!";
let signature = wallet.sign(message)?;

// Sign hash / 签名哈希
let hash = [0u8; 32];
let signature = wallet.sign_hash(&hash)?;
# Ok(())
# }
```

### Transaction Building / 交易构建

```rust,no_run,ignore
use hiver_web3::{
    TransactionBuilder, TxType, Address,
    Eip155Chain, LocalWallet
};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
// Build EIP-1559 transaction / 构建EIP-1559交易
let tx = TransactionBuilder::new()
    .ty(TxType::Eip1559)
    .chain_id(1)
    .nonce(0)
    .max_priority_fee_per_gas(1_500_000_000)
    .max_fee_per_gas(30_000_000_000)
    .gas_limit(21_000)
    .to(Some(Address::from_hex("0x...")?))
    .value(1000000000000000) // 0.001 ETH
    .build()?;

// Sign transaction / 签名交易
let wallet = LocalWallet::new(Eip155Chain::ETHEREUM);
let signed_tx = wallet.sign_transaction(&tx)?;
# Ok(())
# }
```

### RPC Client / RPC客户端

```rust,no_run,ignore
use hiver_web3::{RpcClient, Address, BlockNumber};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let rpc = RpcClient::new("https://eth.llamarpc.com")?;

// Get block number / 获取区块号
let block_number = rpc.get_block_number().await?;
println!("Latest block: {}", block_number);

// Get balance / 获取余额
let address = Address::from_hex("0x...")?;
let balance = rpc.get_balance(&address, BlockNumber::Latest).await?;

// Get transaction count / 获取交易计数
let nonce = rpc.get_transaction_count(&address, BlockNumber::Latest).await?;
# Ok(())
# }
```

### Smart Contracts / 智能合约

```rust,no_run,ignore
use hiver_web3::{Contract, FunctionSelector, Address, RpcClient};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let rpc = RpcClient::new("https://eth.llamarpc.com")?;
let contract_address = Address::from_hex("0x...")?;
let contract = Contract::new(contract_address, &rpc);

// ERC20 balanceOf / ERC20余额查询
let user_address = Address::from_hex("0x...")?;
let selector = FunctionSelector::from_signature("balanceOf(address)");
let mut call_data = Vec::new();
call_data.extend_from_slice(&selector.0);
call_data.extend_from_slice(&[0u8; 12]);
call_data.extend_from_slice(&user_address.0);

let result = contract.call_read_only(&selector, &call_data).await?;

// Using ERC20 constants / 使用ERC20常量
use hiver_web3::ERC20;
assert_eq!(ERC20::BALANCE_OF.0, [0x70, 0xa0, 0x82, 0x31]);
assert_eq!(ERC20::TRANSFER.0, [0xa9, 0x05, 0x9c, 0xbb]);
# Ok(())
# }
```

---

## API Reference / API参考

### ChainId / 链ID

```rust
pub enum ChainId {
    Ethereum,      // 1
    Polygon,       // 137
    Bsc,           // 56
    Arbitrum,      // 42161
    Optimism,      // 10
    Base,          // 8453
    Avalanche,     // 43114
    Fantom,        // 250
    Sepolia,       // 11155111
    Custom(u64),
}
```

### Transaction Types / 交易类型

```rust
pub enum TxType {
    Legacy = 0,    // Legacy transaction
    AccessList = 1, // Access list transaction
    Eip1559 = 2,   // EIP-1559 transaction
}
```

### ERC20/ERC721 Selectors / 标准选择器

```rust
// ERC20
pub const BALANCE_OF: FunctionSelector;    // 0x70a08231
pub const TRANSFER: FunctionSelector;      // 0xa9059cbb
pub const APPROVE: FunctionSelector;       // 0x095ea7b3
pub const TOTAL_SUPPLY: FunctionSelector;  // 0x18160ddd

// ERC721
pub const OWNER_OF: FunctionSelector;           // 0x6352211e
pub const TRANSFER_FROM: FunctionSelector;      // 0x23b872dd
pub const SAFE_TRANSFER_FROM: FunctionSelector; // 0x4a39dc06
```

---

## Implementation Status / 实现状态

### Phase 6: Web3 Support ✅ (Complete / 已完成)

- [x] Chain abstraction (EIP-155)
- [x] Wallet with signing
- [x] Transaction builder (EIP-1559, Legacy)
- [x] RPC client (HTTP)
- [x] Contract interface (ABI)
- [x] ERC20/ERC721 standards

### Future Enhancements / 未来增强

- [ ] WebSocket RPC support
- [ ] Event log filtering
- [ ] Contract deployment
- [ ] Hardware wallet support
- [ ] Multi-chain transaction relay

---

*← [Previous / 上一页](./observability.md) | [Next / 下一页](./testing.md) →*
