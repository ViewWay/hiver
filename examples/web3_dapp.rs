//! # Web3 DApp Backend Example / Web3 DApp 后端示例
//!
//! Demonstrates a decentralized application backend with smart contract
//! interaction, wallet management, token operations (ERC20/ERC721),
//! and blockchain event subscriptions.
//!
//! 演示去中心化应用后端，包括智能合约交互、钱包管理、
//! Token操作（ERC20/ERC721）和区块链事件订阅。
//!
//! ## Equivalent to / 等价于
//!
//! Spring Boot with:
//! - Web3j (Ethereum Java client)
//! - Smart contract wrappers
//! - Wallet management
//! - Event listeners
//!
//! ## Features / 功能
//!
//! - Wallet creation and management / 钱包创建和管理
//! - ERC20 token operations / ERC20 Token操作
//! - ERC721 NFT operations / ERC721 NFT操作
//! - Smart contract interaction / 智能合约交互
//! - Transaction building and signing / 交易构建和签名
//! - Blockchain event subscriptions / 区块链事件订阅
//!
//! ## Run / 运行
//!
//! ```bash
//! cargo run --bin web3_dapp
//! ```

use hiver_web3::{
    chain::{ChainConfig, ChainId},
    wallet::{Address, LocalWallet, Wallet, Signature, WalletError},
};
use hiver_http::{Request, Response, StatusCode};
use hiver_observability::{Tracer, info, warn, error as log_error};
use hiver_cache::{CacheConfig, Cached, MemoryCache};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// ============================================================================
// Configuration / 配置
// ============================================================================

/// DApp configuration / DApp配置
#[derive(Debug, Clone)]
struct DappConfig {
    /// RPC endpoint URL / RPC端点URL
    rpc_url: String,
    /// Chain ID / 链ID
    chain_id: ChainId,
    /// Contract addresses / 合约地址
    contracts: ContractAddresses,
    /// Gas settings / Gas设置
    gas_config: GasConfig,
}

/// Contract addresses used by the DApp / DApp使用的合约地址
#[derive(Debug, Clone)]
struct ContractAddresses {
    /// ERC20 token contract / ERC20代币合约
    token: String,
    /// ERC721 NFT contract / ERC721 NFT合约
    nft: String,
    /// Marketplace contract / 市场合约
    marketplace: String,
}

/// Gas configuration / Gas配置
#[derive(Debug, Clone)]
struct GasConfig {
    /// Gas price in wei / Gas价格（wei）
    gas_price: u64,
    /// Gas limit for transfers / 转账Gas限制
    transfer_gas_limit: u64,
    /// Gas limit for contract calls / 合约调用Gas限制
    contract_gas_limit: u64,
}

impl Default for DappConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://eth.llamarpc.com".to_string(),
            chain_id: ChainId::Ethereum,
            contracts: ContractAddresses {
                // TODO: Replace with actual deployed contract addresses
                // TODO: 替换为实际部署的合约地址
                token: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                nft: "0xBC4CA0EdA76356A2a5F9Dc95f7aE6b75d7b2aB84".to_string(),
                marketplace: "0x0000000000000000000000000000000000000001".to_string(),
            },
            gas_config: GasConfig {
                gas_price: 20_000_000_000, // 20 Gwei
                transfer_gas_limit: 21_000,
                contract_gas_limit: 200_000,
            },
        }
    }
}

// ============================================================================
// Data Models / 数据模型
// ============================================================================

/// Token balance response / 代币余额响应
#[derive(Debug, Serialize)]
struct BalanceResponse {
    address: String,
    native_balance: String,
    token_balance: String,
    token_symbol: String,
}

/// NFT item / NFT项目
#[derive(Debug, Serialize, Clone)]
struct NftItem {
    token_id: String,
    owner: String,
    token_uri: String,
    collection: String,
}

/// Marketplace listing / 市场挂单
#[derive(Debug, Serialize, Clone)]
struct Listing {
    id: i64,
    nft_token_id: String,
    seller: String,
    price: String,
    currency: String,
    status: String,
}

/// Create listing request / 创建挂单请求
#[derive(Debug, Deserialize)]
struct CreateListingRequest {
    nft_token_id: String,
    price: String,
    currency: String,
}

/// Buy request / 购买请求
#[derive(Debug, Deserialize)]
struct BuyRequest {
    listing_id: i64,
    buyer_address: String,
}

/// Transaction status response / 交易状态响应
#[derive(Debug, Serialize)]
struct TxStatusResponse {
    tx_hash: String,
    status: String,
    block_number: Option<u64>,
    gas_used: Option<u64>,
}

/// API response wrapper / API响应包装
#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

// ============================================================================
// Wallet Manager / 钱包管理器
// ============================================================================

/// Manages user wallets for the DApp / 管理DApp的用户钱包
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Service
/// public class WalletService {
///     private final Map<String, LocalWallet> wallets = new ConcurrentHashMap<>();
///     public Wallet createWallet(String userId) { ... }
///     public LocalWallet getWallet(String userId) { ... }
/// }
/// ```
struct WalletManager {
    /// User wallets (userId -> wallet) / 用户钱包
    wallets: Arc<RwLock<HashMap<String, LocalWallet>>>,
    /// Chain configuration / 链配置
    /// TODO: Use for transaction signing in production
    _chain_config: ChainConfig,
}

impl WalletManager {
    fn new(chain_id: ChainId) -> Self {
        Self {
            wallets: Arc::new(RwLock::new(HashMap::new())),
            _chain_config: ChainConfig::ethereum_mainnet(),
        }
    }

    /// Create a new wallet for a user / 为用户创建新钱包
    async fn create_wallet(&self, user_id: &str) -> Result<Address, WalletError> {
        let wallet = LocalWallet::random();
        let address = wallet.address();

        let mut wallets = self.wallets.write().await;
        wallets.insert(user_id.to_string(), wallet);

        info!("Wallet created for user: {} -> {}", user_id, address);
        Ok(address)
    }

    /// Import wallet from private key / 从私钥导入钱包
    async fn import_wallet(
        &self,
        user_id: &str,
        private_key: &str,
    ) -> Result<Address, WalletError> {
        let wallet = LocalWallet::from_private_key(private_key)?;
        let address = wallet.address();

        let mut wallets = self.wallets.write().await;
        wallets.insert(user_id.to_string(), wallet);

        info!("Wallet imported for user: {} -> {}", user_id, address);
        Ok(address)
    }

    /// Get wallet address by user ID / 通过用户ID获取钱包地址
    async fn get_address(&self, user_id: &str) -> Option<Address> {
        let wallets = self.wallets.read().await;
        wallets.get(user_id).map(|w| w.address())
    }

    /// List all wallet addresses / 列出所有钱包地址
    async fn list_wallets(&self) -> Vec<(String, Address)> {
        let wallets = self.wallets.read().await;
        wallets
            .iter()
            .map(|(id, w)| (id.clone(), w.address()))
            .collect()
    }

    /// Sign data with user's wallet / 用用户钱包签名数据
    async fn sign_data(&self, user_id: &str, data: &[u8]) -> Result<Signature, String> {
        let wallets = self.wallets.read().await;
        let wallet = wallets
            .get(user_id)
            .ok_or_else(|| format!("Wallet not found for user: {}", user_id))?;
        wallet.sign(data).map_err(|e| format!("Signing failed: {:?}", e))
    }
}

// ============================================================================
// NFT Marketplace Service / NFT市场服务
// ============================================================================

/// NFT marketplace business logic / NFT市场业务逻辑
struct MarketplaceService {
    /// Active listings / 活跃挂单
    listings: Arc<RwLock<Vec<Listing>>>,
    /// Listing ID counter / 挂单ID计数器
    id_counter: Arc<RwLock<i64>>,
    /// NFT cache / NFT缓存
    nft_cache: Arc<MemoryCache<String, NftItem>>,
}

impl MarketplaceService {
    fn new() -> Self {
        let cache_config = CacheConfig::new("nfts")
            .max_capacity(1000)
            .ttl_secs(3600); // 1 hour TTL
        Self {
            listings: Arc::new(RwLock::new(Vec::new())),
            id_counter: Arc::new(RwLock::new(0)),
            nft_cache: Arc::new(MemoryCache::new(cache_config)),
        }
    }

    async fn next_id(&self) -> i64 {
        let mut counter = self.id_counter.write().await;
        *counter += 1;
        *counter
    }

    /// Create a new listing / 创建新挂单
    async fn create_listing(
        &self,
        seller: &str,
        nft_token_id: &str,
        price: &str,
        currency: &str,
    ) -> Listing {
        let id = self.next_id().await;
        let listing = Listing {
            id,
            nft_token_id: nft_token_id.to_string(),
            seller: seller.to_string(),
            price: price.to_string(),
            currency: currency.to_string(),
            status: "active".to_string(),
        };

        let mut listings = self.listings.write().await;
        listings.push(listing.clone());

        info!(
            "Listing created: id={}, nft={}, seller={}, price={}",
            id, nft_token_id, seller, price
        );
        listing
    }

    /// Buy an NFT from a listing / 从挂单购买NFT
    async fn buy_nft(&self, listing_id: i64, buyer: &str) -> Result<Listing, String> {
        let mut listings = self.listings.write().await;
        let listing = listings
            .iter_mut()
            .find(|l| l.id == listing_id)
            .ok_or_else(|| format!("Listing {} not found", listing_id))?;

        if listing.status != "active" {
            return Err(format!("Listing {} is not active (status: {})", listing_id, listing.status));
        }

        listing.status = "sold".to_string();
        let result = listing.clone();

        info!(
            "NFT sold: listing_id={}, buyer={}, price={}",
            listing_id, buyer, result.price
        );
        Ok(result)
    }

    /// List active listings / 列出活跃挂单
    async fn list_active(&self) -> Vec<Listing> {
        let listings = self.listings.read().await;
        listings
            .iter()
            .filter(|l| l.status == "active")
            .cloned()
            .collect()
    }

    /// Get NFT metadata (with cache) / 获取NFT元数据（带缓存）
    async fn get_nft_metadata(&self, token_id: &str) -> Option<NftItem> {
        let cache_key = format!("nft:{}", token_id);
        Cached::get_or_fetch(self.nft_cache.as_ref(), &cache_key, || async {
            // TODO: In production, fetch from ERC721 contract via tokenURI
            // TODO: 在生产环境中，通过 tokenURI 从 ERC721 合约获取
            Some(NftItem {
                token_id: token_id.to_string(),
                owner: "0x0000000000000000000000000000000000000000".to_string(),
                token_uri: format!("ipfs://Qm...{}", token_id),
                collection: "HiverCollection".to_string(),
            })
        })
        .await
    }
}

// ============================================================================
// DApp Controller / DApp控制器
// ============================================================================

/// Main DApp controller / 主DApp控制器
struct DappController {
    wallet_manager: Arc<WalletManager>,
    marketplace: Arc<MarketplaceService>,
    config: Arc<DappConfig>,
}

impl DappController {
    fn new(
        wallet_manager: Arc<WalletManager>,
        marketplace: Arc<MarketplaceService>,
        config: Arc<DappConfig>,
    ) -> Self {
        Self {
            wallet_manager,
            marketplace,
            config,
        }
    }

    /// Create a new wallet / 创建新钱包
    async fn create_wallet(&self, user_id: &str) -> Response {
        match self.wallet_manager.create_wallet(user_id).await {
            Ok(address) => json_ok(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "user_id": user_id,
                    "address": address.to_string(),
                })),
                message: Some("Wallet created".to_string()),
            }),
            Err(e) => json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "WALLET_ERROR",
                &format!("Failed to create wallet: {:?}", e),
            ),
        }
    }

    /// List all wallets / 列出所有钱包
    async fn list_wallets(&self) -> Response {
        let wallets = self.wallet_manager.list_wallets().await;
        let wallet_list: Vec<serde_json::Value> = wallets
            .into_iter()
            .map(|(id, addr)| {
                serde_json::json!({
                    "user_id": id,
                    "address": addr.to_string(),
                })
            })
            .collect();

        json_ok(ApiResponse {
            success: true,
            data: Some(wallet_list),
            message: None,
        })
    }

    /// Create NFT listing / 创建NFT挂单
    async fn create_listing(&self, seller_id: &str, req: CreateListingRequest) -> Response {
        // Verify seller has a wallet / 验证卖家有钱包
        let seller_addr = match self.wallet_manager.get_address(seller_id).await {
            Some(addr) => addr.to_string(),
            None => {
                return json_error(
                    StatusCode::NOT_FOUND,
                    "WALLET_NOT_FOUND",
                    &format!("No wallet for user: {}", seller_id),
                )
            }
        };

        let listing = self
            .marketplace
            .create_listing(&seller_addr, &req.nft_token_id, &req.price, &req.currency)
            .await;

        json_ok(ApiResponse {
            success: true,
            data: Some(listing),
            message: Some("Listing created".to_string()),
        })
    }

    /// Buy NFT / 购买NFT
    async fn buy_nft(&self, buyer_id: &str, req: BuyRequest) -> Response {
        let buyer_addr = match self.wallet_manager.get_address(buyer_id).await {
            Some(addr) => addr.to_string(),
            None => {
                return json_error(
                    StatusCode::NOT_FOUND,
                    "WALLET_NOT_FOUND",
                    &format!("No wallet for user: {}", buyer_id),
                )
            }
        };

        match self.marketplace.buy_nft(req.listing_id, &buyer_addr).await {
            Ok(listing) => json_ok(ApiResponse {
                success: true,
                data: Some(listing),
                message: Some("NFT purchased successfully".to_string()),
            }),
            Err(e) => json_error(StatusCode::BAD_REQUEST, "BUY_FAILED", &e),
        }
    }

    /// List active marketplace listings / 列出活跃市场挂单
    async fn list_listings(&self) -> Response {
        let listings = self.marketplace.list_active().await;
        json_ok(ApiResponse {
            success: true,
            data: Some(listings),
            message: None,
        })
    }

    /// Get NFT metadata / 获取NFT元数据
    async fn get_nft(&self, token_id: &str) -> Response {
        match self.marketplace.get_nft_metadata(token_id).await {
            Some(nft) => json_ok(ApiResponse {
                success: true,
                data: Some(nft),
                message: None,
            }),
            None => json_error(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                &format!("NFT {} not found", token_id),
            ),
        }
    }

    /// Build a token transfer transaction / 构建代币转账交易
    fn build_transfer_tx(&self, from: &str, to: &str, amount: &str) -> Response {
        // Build unsigned transaction data / 构建未签名交易数据
        // TODO: In production, use TransactionBuilder with proper nonce from RPC
        // TODO: 在生产环境中，使用从RPC获取的nonce通过TransactionBuilder构建
        //
        // let tx = LegacyTransactionBuilder::new(nonce)
        //     .to(to_address)
        //     .value(amount)
        //     .gas_price(gas_price)
        //     .gas_limit(gas_limit)
        //     .chain_id(chain_id);

        json_ok(ApiResponse {
            success: true,
            data: Some(serde_json::json!({
                "transaction": {
                    "from": from,
                    "to": to,
                    "value": amount,
                    "gas_price": format!("{} wei ({} Gwei)", self.config.gas_config.gas_price, self.config.gas_config.gas_price / 1_000_000_000),
                    "gas_limit": self.config.gas_config.transfer_gas_limit,
                    "chain_id": self.config.chain_id.as_u64(),
                    "tx_type": "Legacy",
                },
                "status": "unsigned",
                "message": "Transaction built. Sign with wallet before sending.",
            })),
            message: None,
        })
    }

    /// Get chain info / 获取链信息
    fn get_chain_info(&self) -> Response {
        json_ok(ApiResponse {
            success: true,
            data: Some(serde_json::json!({
                "chain_id": self.config.chain_id.as_u64(),
                "chain_name": format!("{:?}", self.config.chain_id),
                "rpc_url": self.config.rpc_url,
                "contracts": {
                    "token": self.config.contracts.token,
                    "nft": self.config.contracts.nft,
                    "marketplace": self.config.contracts.marketplace,
                },
                "gas": {
                    "gas_price": format!("{} Gwei", self.config.gas_config.gas_price / 1_000_000_000),
                    "transfer_limit": self.config.gas_config.transfer_gas_limit,
                    "contract_limit": self.config.gas_config.contract_gas_limit,
                },
            })),
            message: None,
        })
    }
}

// ============================================================================
// Response helpers / 响应辅助函数
// ============================================================================

fn json_ok<T: Serialize>(data: T) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).unwrap_or_default())
        .unwrap_or_default()
}

fn json_error(status: StatusCode, error: &str, message: &str) -> Response {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "success": false,
                "error": error,
                "message": message,
            })
            .to_string(),
        )
        .unwrap_or_default()
}

// ============================================================================
// Main / 主函数
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  Hiver Web3 DApp Backend Example / Hiver Web3 DApp 后端示例");
    println!("  Equivalent to Spring Boot + Web3j");
    println!("================================================================\n");

    let _tracer = Tracer::new("web3-dapp");

    // Configuration / 配置
    let config = Arc::new(DappConfig::default());

    println!("--- Chain Configuration / 链配置 ---\n");
    println!("  Chain ID: {} ({:?})",
        config.chain_id.as_u64(),
        config.chain_id
    );
    println!("  RPC URL: {}", config.rpc_url);
    println!("  Gas Price: {} Gwei", config.gas_config.gas_price / 1_000_000_000);
    println!();

    // Initialize services / 初始化服务
    let wallet_manager = Arc::new(WalletManager::new(config.chain_id));
    let marketplace = Arc::new(MarketplaceService::new());
    let controller = DappController::new(
        wallet_manager.clone(),
        marketplace.clone(),
        config.clone(),
    );

    // ================================================================
    // Scenario 1: Wallet Management / 钱包管理
    // ================================================================
    println!("--- Scenario 1: Wallet Management / 场景 1：钱包管理 ---\n");

    // Create wallets / 创建钱包
    println!("Creating wallets for users... / 为用户创建钱包...");
    let resp = controller.create_wallet("alice").await;
    println!("  POST /api/wallets (alice)");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    let resp = controller.create_wallet("bob").await;
    println!("  POST /api/wallets (bob)");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // List wallets / 列出钱包
    let resp = controller.list_wallets().await;
    println!("  GET /api/wallets");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // ================================================================
    // Scenario 2: NFT Marketplace / NFT市场
    // ================================================================
    println!("--- Scenario 2: NFT Marketplace / 场景 2：NFT市场 ---\n");

    // Create listing / 创建挂单
    let list_req = CreateListingRequest {
        nft_token_id: "42".to_string(),
        price: "0.5".to_string(),
        currency: "ETH".to_string(),
    };
    let resp = controller.create_listing("alice", list_req).await;
    println!("  POST /api/marketplace/listings (alice lists NFT #42)");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // Create another listing / 创建另一个挂单
    let list_req2 = CreateListingRequest {
        nft_token_id: "99".to_string(),
        price: "1.2".to_string(),
        currency: "ETH".to_string(),
    };
    let resp = controller.create_listing("alice", list_req2).await;
    println!("  POST /api/marketplace/listings (alice lists NFT #99)");
    println!("  Status: {}", resp.status());
    println!();

    // List active listings / 列出活跃挂单
    let resp = controller.list_listings().await;
    println!("  GET /api/marketplace/listings");
    println!("  Status: {}", resp.status());
    let listings_body: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(resp.body().as_ref())).unwrap();
    println!(
        "  Active listings: {}\n",
        listings_body["data"].as_array().map(|a| a.len()).unwrap_or(0)
    );

    // Buy NFT / 购买NFT
    let buy_req = BuyRequest {
        listing_id: 1,
        buyer_address: "bob".to_string(),
    };
    let resp = controller.buy_nft("bob", buy_req).await;
    println!("  POST /api/marketplace/buy (bob buys listing #1)");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // Verify listing is sold / 验证挂单已售出
    let resp = controller.list_listings().await;
    let remaining: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(resp.body().as_ref())).unwrap();
    println!(
        "  Remaining active listings: {}\n",
        remaining["data"].as_array().map(|a| a.len()).unwrap_or(0)
    );

    // ================================================================
    // Scenario 3: NFT Metadata / NFT元数据
    // ================================================================
    println!("--- Scenario 3: NFT Metadata / 场景 3：NFT元数据 ---\n");

    let resp = controller.get_nft("42").await;
    println!("  GET /api/nfts/42");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // Second fetch should hit cache / 第二次获取应命中缓存
    let resp = controller.get_nft("42").await;
    println!("  GET /api/nfts/42 (cache hit / 缓存命中)");
    println!("  Status: {}", resp.status());
    println!();

    // ================================================================
    // Scenario 4: Transaction Building / 交易构建
    // ================================================================
    println!("--- Scenario 4: Transaction Building / 场景 4：交易构建 ---\n");

    // TODO: Replace with actual wallet addresses
    // TODO: 替换为实际钱包地址
    let resp = controller.build_transfer_tx(
        "0x1111111111111111111111111111111111111111",
        "0x2222222222222222222222222222222222222222",
        "1000000000000000000", // 1 ETH in wei
    ).await;
    println!("  POST /api/transactions/build");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // ================================================================
    // Scenario 5: Chain Info / 链信息
    // ================================================================
    println!("--- Scenario 5: Chain Info / 场景 5：链信息 ---\n");

    let resp = controller.get_chain_info();
    println!("  GET /api/chain/info");
    println!("  Status: {}", resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(resp.body().as_ref()));

    // ================================================================
    // Scenario 6: Smart Contract Interaction / 智能合约交互
    // ================================================================
    println!("--- Scenario 6: Smart Contract / 场景 6：智能合约 ---\n");

    // Demonstrate contract call patterns / 演示合约调用模式
    println!("  Contract call patterns:");
    println!("    1. ERC20.balanceOf(address) -> uint256");
    println!("    2. ERC20.transfer(address, uint256) -> bool");
    println!("    3. ERC721.ownerOf(uint256) -> address");
    println!("    4. ERC721.tokenURI(uint256) -> string");
    println!();

    // Demonstrate ERC20 usage pattern / 演示 ERC20 使用模式
    println!("  // ERC20 usage pattern / ERC20 使用模式:");
    println!("  // let client = RpcClient::new(&config.rpc_url)?;");
    println!("  // let token_addr = Address::from_hex(&config.contracts.token)?;");
    println!("  // let erc20 = ERC20::new(token_addr, &client);");
    println!("  // let balance = erc20.balance_of(user_addr).await?;");
    println!("  // let name = erc20.name().await?;");
    println!("  // let symbol = erc20.symbol().await?;");
    println!();

    // ================================================================
    // Summary / 总结
    // ================================================================
    println!("================================================================");
    println!("  Web3 DApp example complete / Web3 DApp 示例完成");
    println!("================================================================\n");
    println!("Key patterns demonstrated / 展示的关键模式:");
    println!("  1. Wallet creation & management / 钱包创建与管理");
    println!("  2. NFT marketplace (list, buy) / NFT市场（挂单、购买）");
    println!("  3. NFT metadata with caching / 带缓存的NFT元数据");
    println!("  4. Transaction building / 交易构建");
    println!("  5. ERC20 / ERC721 contract interaction patterns");
    println!("     ERC20 / ERC721 合约交互模式");
    println!("  6. Chain configuration / 链配置");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wallet_creation() {
        let manager = WalletManager::new(ChainId::Ethereum);

        let addr1 = manager.create_wallet("user1").await.unwrap();
        let addr2 = manager.create_wallet("user2").await.unwrap();

        // Each wallet gets a unique address / 每个钱包获得唯一地址
        assert_ne!(addr1.to_string(), addr2.to_string());

        // Can retrieve address / 可以检索地址
        let retrieved = manager.get_address("user1").await.unwrap();
        assert_eq!(retrieved.to_string(), addr1.to_string());
    }

    #[tokio::test]
    async fn test_marketplace_create_and_buy() {
        let marketplace = MarketplaceService::new();

        // Create listing / 创建挂单
        let listing = marketplace
            .create_listing("0xSeller", "42", "1.0", "ETH")
            .await;
        assert_eq!(listing.status, "active");

        // Buy / 购买
        let result = marketplace.buy_nft(listing.id, "0xBuyer").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "sold");

        // Cannot buy again / 不能再次购买
        let result = marketplace.buy_nft(1, "0xOther").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_marketplace_list_active() {
        let marketplace = MarketplaceService::new();

        marketplace.create_listing("0xA", "1", "1.0", "ETH").await;
        marketplace.create_listing("0xA", "2", "2.0", "ETH").await;
        marketplace.buy_nft(1, "0xB").await.unwrap();

        let active = marketplace.list_active().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].nft_token_id, "2");
    }

    #[tokio::test]
    async fn test_nft_cache() {
        let marketplace = MarketplaceService::new();

        // First fetch (miss) / 第一次获取（未命中）
        let nft1 = marketplace.get_nft_metadata("42").await;
        assert!(nft1.is_some());

        // Second fetch (hit) / 第二次获取（命中）
        let nft2 = marketplace.get_nft_metadata("42").await;
        assert!(nft2.is_some());
        assert_eq!(nft1.unwrap().token_id, nft2.unwrap().token_id);
    }
}
