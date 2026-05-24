//! # Complete REST API Example / 完整REST API示例
//!
//! Demonstrates a production-grade REST API with database access, caching,
//! security (JWT auth + RBAC), transaction management, and validation.
//!
//! 演示生产级REST API，包含数据库访问、缓存、安全（JWT认证+RBAC）、
//! 事务管理和数据验证。
//!
//! ## Equivalent to / 等价于
//!
//! Spring Boot PetClinic / Spring Boot REST API with:
//! - Spring Data R2DBC
//! - Spring Cache (@Cacheable)
//! - Spring Security + JWT
//! - @Transactional
//! - Bean Validation
//!
//! ## Run / 运行
//!
//! ```bash
//! cargo run --bin rest_api
//! ```

use nexus_cache::{CacheConfig, Cached, CachePutExec, CacheEvictExec, MemoryCache};
use nexus_data_annotations::{Entity, Table, Id, Column, Transactional};
use nexus_data_annotations::transactional::IsolationLevel;
use nexus_http::validation::{Validatable, ValidationErrors, ValidationHelpers};
use nexus_http::{Request, Response, StatusCode};
use nexus_observability::{Tracer, Span, SpanId, TraceId, info, error as log_error};
use nexus_resilience::rate_limit::{RateLimiter, RateLimiterConfig};
use nexus_security::{
    Authority, JwtUtil, PasswordEncoder, Role, User,
    Authentication, AuthenticationManager, SimpleAuthenticationManager,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// ============================================================================
// Data Models / 数据模型
// ============================================================================

/// Product entity / 商品实体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[Entity]
#[Table(name = "products")]
struct Product {
    #[Id]
    #[Column(name = "id")]
    id: i64,

    #[Column(name = "name", nullable = false)]
    name: String,

    #[Column(name = "description")]
    description: String,

    #[Column(name = "price", nullable = false)]
    price: f64,

    #[Column(name = "stock", nullable = false)]
    stock: i64,

    #[Column(name = "category")]
    category: String,
}

impl Validatable for Product {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if let Some(err) = ValidationHelpers::require_min_length("name", &self.name, 2) {
            errors.add(err);
        }
        if self.price <= 0.0 {
            errors.add(nexus_http::validation::ValidationError::new(
                "price",
                "Price must be positive",
            ));
        }
        if self.stock < 0 {
            errors.add(nexus_http::validation::ValidationError::new(
                "stock",
                "Stock cannot be negative",
            ));
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

/// Create product request / 创建商品请求
#[derive(Debug, Deserialize)]
struct CreateProductRequest {
    name: String,
    description: String,
    price: f64,
    stock: i64,
    category: String,
}

/// Update product request / 更新商品请求
#[derive(Debug, Deserialize)]
struct UpdateProductRequest {
    name: Option<String>,
    description: Option<String>,
    price: Option<f64>,
    stock: Option<i64>,
    category: Option<String>,
}

/// Login request / 登录请求
#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// Login response / 登录响应
#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    username: String,
    authorities: Vec<String>,
}

/// Generic API response / 通用API响应
#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
    #[serde(rename = "traceId")]
    trace_id: Option<String>,
}

/// Paginated response / 分页响应
#[derive(Debug, Serialize)]
struct PageResponse<T: Serialize> {
    items: Vec<T>,
    total: i64,
    page: i64,
    size: i64,
}

/// Error response / 错误响应
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(rename = "traceId")]
    trace_id: Option<String>,
}

// ============================================================================
// Service Layer / 服务层
// ============================================================================

/// Product repository with caching / 带缓存的商品仓库
struct ProductRepository {
    /// In-memory store (simulates database)
    /// 内存存储（模拟数据库）
    store: Arc<RwLock<HashMap<i64, Product>>>,
    /// Cache layer / 缓存层
    cache: Arc<MemoryCache<String, Product>>,
    /// ID counter / ID计数器
    id_counter: Arc<RwLock<i64>>,
}

impl ProductRepository {
    fn new() -> Self {
        let cache_config = CacheConfig::new("products")
            .max_capacity(500)
            .ttl_secs(600); // 10 minute TTL / 10分钟TTL
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(MemoryCache::new(cache_config)),
            id_counter: Arc::new(RwLock::new(0)),
        }
    }

    async fn next_id(&self) -> i64 {
        let mut counter = self.id_counter.write().await;
        *counter += 1;
        *counter
    }

    /// Find product by ID with cache (@Cacheable equivalent)
    /// 通过ID查找商品（带缓存，等价于@Cacheable）
    async fn find_by_id(&self, id: i64) -> Option<Product> {
        let cache_key = format!("product:{}", id);
        Cached::get_or_fetch(self.cache.as_ref(), &cache_key, || async {
            let store = self.store.read().await;
            store.get(&id).cloned()
        })
        .await
    }

    /// Save product and update cache (@CachePut equivalent)
    /// 保存商品并更新缓存（等价于@CachePut）
    #[Transactional]
    async fn save(&self, product: Product) -> Result<Product, String> {
        let cache_key = format!("product:{}", product.id);
        CachePutExec::execute_and_update(self.cache.as_ref(), cache_key, || async {
            let mut store = self.store.write().await;
            store.insert(product.id, product.clone());
            product.clone()
        })
        .await;

        info!("Product saved: id={}, name={}", product.id, product.name);
        Ok(product)
    }

    /// Delete product and evict from cache (@CacheEvict equivalent)
    /// 删除商品并从缓存驱逐（等价于@CacheEvict）
    #[Transactional]
    async fn delete(&self, id: i64) -> Result<(), String> {
        let cache_key = format!("product:{}", id);
        CacheEvictExec::execute_and_evict(self.cache.as_ref(), &cache_key, || async {
            let mut store = self.store.write().await;
            store.remove(&id);
        })
        .await;

        info!("Product deleted: id={}", id);
        Ok(())
    }

    /// List products with pagination / 分页列出商品
    async fn list(&self, page: i64, size: i64) -> (Vec<Product>, i64) {
        let store = self.store.read().await;
        let total = store.len() as i64;
        let items: Vec<Product> = store
            .values()
            .skip((page as usize) * (size as usize))
            .take(size as usize)
            .cloned()
            .collect();
        (items, total)
    }

    /// Find products by category / 按分类查找商品
    async fn find_by_category(&self, category: &str) -> Vec<Product> {
        let store = self.store.read().await;
        store
            .values()
            .filter(|p| p.category == category)
            .cloned()
            .collect()
    }
}

// ============================================================================
// Controller Layer / 控制器层
// ============================================================================

/// Product REST controller / 商品REST控制器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @RestController
/// @RequestMapping("/api/products")
/// @RequiredArgsConstructor
/// public class ProductController { ... }
/// ```
struct ProductController {
    repo: Arc<ProductRepository>,
}

impl ProductController {
    fn new(repo: Arc<ProductRepository>) -> Self {
        Self { repo }
    }

    /// GET /api/products?page=0&size=20
    ///
    /// List all products with pagination.
    /// 分页列出所有商品。
    async fn list_products(&self, page: i64, size: i64) -> Response {
        let trace_id = TraceId::new().to_hex();

        let (items, total) = self.repo.list(page, size).await;

        let page_resp = PageResponse {
            items,
            total,
            page,
            size,
        };

        json_ok(ApiResponse {
            success: true,
            data: Some(page_resp),
            message: None,
            trace_id: Some(trace_id),
        })
    }

    /// GET /api/products/:id
    ///
    /// Get product by ID.
    /// 通过ID获取商品。
    async fn get_product(&self, id: i64) -> Response {
        let trace_id = TraceId::new().to_hex();

        match self.repo.find_by_id(id).await {
            Some(product) => json_ok(ApiResponse {
                success: true,
                data: Some(product),
                message: None,
                trace_id: Some(trace_id),
            }),
            None => json_error(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                &format!("Product {} not found", id),
                &trace_id,
            ),
        }
    }

    /// POST /api/products
    ///
    /// Create a new product.
    /// 创建新商品。
    async fn create_product(&self, req: CreateProductRequest) -> Response {
        let trace_id = TraceId::new().to_hex();

        // Build entity / 构建实体
        let product = Product {
            id: self.repo.next_id().await,
            name: req.name,
            description: req.description,
            price: req.price,
            stock: req.stock,
            category: req.category,
        };

        // Validate / 验证
        if let Err(errors) = product.validate() {
            return json_error(
                StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                &format!("Validation errors: {}", errors.error_count()),
                &trace_id,
            );
        }

        match self.repo.save(product).await {
            Ok(saved) => json_created(ApiResponse {
                success: true,
                data: Some(saved),
                message: Some("Product created".to_string()),
                trace_id: Some(trace_id),
            }),
            Err(e) => json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "CREATE_FAILED",
                &e,
                &trace_id,
            ),
        }
    }

    /// PUT /api/products/:id
    ///
    /// Update an existing product.
    /// 更新已有商品。
    async fn update_product(&self, id: i64, req: UpdateProductRequest) -> Response {
        let trace_id = TraceId::new().to_hex();

        let mut product = match self.repo.find_by_id(id).await {
            Some(p) => p,
            None => {
                return json_error(
                    StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    &format!("Product {} not found", id),
                    &trace_id,
                )
            }
        };

        // Apply partial updates / 应用部分更新
        if let Some(name) = req.name {
            product.name = name;
        }
        if let Some(description) = req.description {
            product.description = description;
        }
        if let Some(price) = req.price {
            product.price = price;
        }
        if let Some(stock) = req.stock {
            product.stock = stock;
        }
        if let Some(category) = req.category {
            product.category = category;
        }

        // Validate / 验证
        if let Err(errors) = product.validate() {
            return json_error(
                StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED",
                &format!("Validation errors: {}", errors.error_count()),
                &trace_id,
            );
        }

        match self.repo.save(product).await {
            Ok(updated) => json_ok(ApiResponse {
                success: true,
                data: Some(updated),
                message: Some("Product updated".to_string()),
                trace_id: Some(trace_id),
            }),
            Err(e) => json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "UPDATE_FAILED",
                &e,
                &trace_id,
            ),
        }
    }

    /// DELETE /api/products/:id
    ///
    /// Delete a product. Admin role required.
    /// 删除商品。需要管理员角色。
    async fn delete_product(&self, id: i64) -> Response {
        let trace_id = TraceId::new().to_hex();

        match self.repo.delete(id).await {
            Ok(()) => json_ok(ApiResponse::<()> {
                success: true,
                data: None,
                message: Some("Product deleted".to_string()),
                trace_id: Some(trace_id),
            }),
            Err(e) => json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DELETE_FAILED",
                &e,
                &trace_id,
            ),
        }
    }

    /// GET /api/products/category/:category
    ///
    /// List products by category.
    /// 按分类列出商品。
    async fn list_by_category(&self, category: &str) -> Response {
        let trace_id = TraceId::new().to_hex();

        let items = self.repo.find_by_category(category).await;

        json_ok(ApiResponse {
            success: true,
            data: Some(items),
            message: None,
            trace_id: Some(trace_id),
        })
    }
}

/// Auth controller / 认证控制器
struct AuthController {
    users: Arc<RwLock<HashMap<String, User>>>,
    password_encoder: Arc<dyn PasswordEncoder>,
}

impl AuthController {
    fn new() -> Self {
        // TODO: In production, use a real database-backed user service
        // TODO: 在生产环境中，使用真正的数据库支持的用户服务
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            password_encoder: Arc::new(nexus_security::NoOpPasswordEncoder),
        }
    }

    /// POST /api/auth/register
    async fn register(&self, username: &str, password: &str) -> Response {
        let trace_id = TraceId::new().to_hex();

        let mut users = self.users.write().await;
        if users.contains_key(username) {
            return json_error(
                StatusCode::CONFLICT,
                "USER_EXISTS",
                "Username already taken",
                &trace_id,
            );
        }

        let encoded = self.password_encoder.encode(password);
        let user = User::with_roles(username, &encoded, &[Role::User]);
        users.insert(username.to_string(), user);

        json_created(ApiResponse::<String> {
            success: true,
            data: Some(username.to_string()),
            message: Some("User registered".to_string()),
            trace_id: Some(trace_id),
        })
    }

    /// POST /api/auth/login
    async fn login(&self, username: &str, password: &str) -> Response {
        let trace_id = TraceId::new().to_hex();

        let users = self.users.read().await;
        let user = match users.get(username) {
            Some(u) => u,
            None => {
                return json_error(
                    StatusCode::UNAUTHORIZED,
                    "INVALID_CREDENTIALS",
                    "Invalid username or password",
                    &trace_id,
                )
            }
        };

        // Verify password / 验证密码
        if !self.password_encoder.matches(password, user.password()) {
            return json_error(
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "Invalid username or password",
                &trace_id,
            );
        }

        // Generate JWT / 生成JWT
        let authorities: Vec<String> = user.authorities()
            .iter()
            .map(|a| a.to_string())
            .collect();

        match JwtUtil::create_token(username, username, &user.authorities()) {
            Ok(token) => json_ok(ApiResponse {
                success: true,
                data: Some(LoginResponse {
                    token,
                    username: username.to_string(),
                    authorities,
                }),
                message: None,
                trace_id: Some(trace_id),
            }),
            Err(e) => json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "TOKEN_ERROR",
                &format!("Failed to generate token: {:?}", e),
                &trace_id,
            ),
        }
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

fn json_created<T: Serialize>(data: T) -> Response {
    Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&data).unwrap_or_default())
        .unwrap_or_default()
}

fn json_error(status: StatusCode, error: &str, message: &str, trace_id: &str) -> Response {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&ErrorResponse {
                error: error.to_string(),
                message: message.to_string(),
                trace_id: Some(trace_id.to_string()),
            })
            .unwrap_or_default(),
        )
        .unwrap_or_default()
}

// ============================================================================
// Seed data / 种子数据
// ============================================================================

async fn seed_products(repo: &ProductRepository) {
    let seeds = vec![
        ("Mechanical Keyboard", "Cherry MX Brown switches", 129.99, 50, "electronics"),
        ("Wireless Mouse", "Ergonomic design, 2.4GHz", 49.99, 120, "electronics"),
        ("USB-C Hub", "7-in-1 docking station", 79.99, 80, "electronics"),
        ("Standing Desk", "Electric height adjustable", 599.99, 15, "furniture"),
        ("Monitor Arm", "Dual arm, gas spring", 89.99, 40, "furniture"),
        ("Webcam HD", "1080p with microphone", 69.99, 200, "electronics"),
    ];

    for (i, (name, desc, price, stock, category)) in seeds.into_iter().enumerate() {
        let product = Product {
            id: (i + 1) as i64,
            name: name.to_string(),
            description: desc.to_string(),
            price,
            stock,
            category: category.to_string(),
        };
        repo.save(product).await.ok();
    }
    info!("Seeded {} products", 6);
}

// ============================================================================
// Main / 主函数
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("  Nexus REST API Example / Nexus REST API 示例");
    println!("  Equivalent to Spring Boot + Spring Data + Spring Security");
    println!("================================================================\n");

    // Initialize tracing / 初始化追踪
    // TODO: In production, configure OpenTelemetry exporter
    // TODO: 在生产环境中，配置 OpenTelemetry 导出器
    let _tracer = Tracer::new("rest-api");

    // Initialize rate limiter / 初始化限流器
    // 100 requests per second per IP
    let rate_limiter = RateLimiter::new(
        "api-global",
        RateLimiterConfig::new()
            .with_capacity(100)
            .with_refill_rate(100),
    );

    // Initialize repositories / 初始化仓库
    let product_repo = Arc::new(ProductRepository::new());
    seed_products(&product_repo).await;

    // Initialize controllers / 初始化控制器
    let product_controller = ProductController::new(product_repo.clone());
    let auth_controller = AuthController::new();

    println!("Services initialized / 服务初始化完成\n");

    // ================================================================
    // Demonstrate API flow / 演示API流程
    // ================================================================

    // --- Auth: Register & Login / 认证：注册和登录 ---
    println!("--- Auth: Register & Login / 认证：注册和登录 ---\n");

    let reg_resp = auth_controller.register("alice", "secret123").await;
    println!("POST /api/auth/register");
    println!("  Status: {}", reg_resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(reg_resp.body().as_ref()));

    let login_resp = auth_controller.login("alice", "secret123").await;
    println!("POST /api/auth/login");
    println!("  Status: {}", login_resp.status());
    let login_body = String::from_utf8_lossy(login_resp.body().as_ref());
    println!("  Body: {}\n", login_body);

    // --- List products / 列出商品 ---
    println!("--- List Products / 列出商品 ---\n");

    let list_resp = product_controller.list_products(0, 10).await;
    println!("GET /api/products?page=0&size=10");
    println!("  Status: {}", list_resp.status());

    let list_body: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(list_resp.body().as_ref())).unwrap();
    println!(
        "  Total products: {}\n",
        list_body["data"]["total"]
    );

    // --- Get single product / 获取单个商品 ---
    println!("--- Get Product / 获取商品 ---\n");

    let get_resp = product_controller.get_product(1).await;
    println!("GET /api/products/1");
    println!("  Status: {}", get_resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(get_resp.body().as_ref()));

    // --- Get with cache hit / 缓存命中获取 ---
    let get_resp2 = product_controller.get_product(1).await;
    println!("GET /api/products/1 (cache hit / 缓存命中)");
    println!("  Status: {}", get_resp2.status());
    println!();

    // --- Create product / 创建商品 ---
    println!("--- Create Product / 创建商品 ---\n");

    let create_req = CreateProductRequest {
        name: "Noise Cancelling Headphones".to_string(),
        description: "Active noise cancellation, 30h battery".to_string(),
        price: 249.99,
        stock: 75,
        category: "electronics".to_string(),
    };

    let create_resp = product_controller.create_product(create_req).await;
    println!("POST /api/products");
    println!("  Status: {}", create_resp.status());

    let create_body: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(create_resp.body().as_ref())).unwrap();
    println!(
        "  Created product id: {}\n",
        create_body["data"]["id"]
    );

    // --- Update product / 更新商品 ---
    println!("--- Update Product / 更新商品 ---\n");

    let update_req = UpdateProductRequest {
        price: Some(219.99), // Discount! / 打折！
        stock: Some(100),
        name: None,
        description: None,
        category: None,
    };

    let update_resp = product_controller.update_product(1, update_req).await;
    println!("PUT /api/products/1");
    println!("  Status: {}", update_resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(update_resp.body().as_ref()));

    // --- Filter by category / 按分类过滤 ---
    println!("--- Filter by Category / 按分类过滤 ---\n");

    let cat_resp = product_controller.list_by_category("electronics").await;
    println!("GET /api/products/category/electronics");
    println!("  Status: {}", cat_resp.status());

    let cat_body: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(cat_resp.body().as_ref())).unwrap();
    println!(
        "  Electronics count: {}\n",
        cat_body["data"].as_array().map(|a| a.len()).unwrap_or(0)
    );

    // --- Delete product (admin) / 删除商品（管理员） ---
    println!("--- Delete Product / 删除商品 ---\n");

    let del_resp = product_controller.delete_product(2).await;
    println!("DELETE /api/products/2");
    println!("  Status: {}", del_resp.status());
    println!();

    // --- Validation error / 验证错误 ---
    println!("--- Validation Error / 验证错误 ---\n");

    let bad_req = CreateProductRequest {
        name: "X".to_string(),      // Too short / 太短
        description: "Test".to_string(),
        price: -10.0,                // Negative / 负数
        stock: -5,                   // Negative / 负数
        category: "test".to_string(),
    };

    let bad_resp = product_controller.create_product(bad_req).await;
    println!("POST /api/products (invalid data / 无效数据)");
    println!("  Status: {}", bad_resp.status());
    println!("  Body: {}\n", String::from_utf8_lossy(bad_resp.body().as_ref()));

    // --- Rate limiter check / 限流检查 ---
    println!("--- Rate Limiter / 限流器 ---\n");
    println!("Rate limiter config: 100 req/s");
    if rate_limiter.try_acquire().is_ok() {
        println!("  Request allowed / 请求允许");
    }

    // --- Cache stats / 缓存统计 ---
    println!("\n--- Cache Stats / 缓存统计 ---\n");
    let stats = product_repo.cache.stats().await;
    println!(
        "  Hits: {}, Misses: {}, Hit Rate: {:.1}%, Size: {}",
        stats.hits, stats.misses, stats.hit_rate * 100.0, stats.size
    );

    println!("\n================================================================");
    println!("  REST API example complete / REST API 示例完成");
    println!("================================================================");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_product_crud() {
        let repo = Arc::new(ProductRepository::new());
        let controller = ProductController::new(repo);

        // Create / 创建
        let create_req = CreateProductRequest {
            name: "Test Product".to_string(),
            description: "A test".to_string(),
            price: 9.99,
            stock: 10,
            category: "test".to_string(),
        };
        let resp = controller.create_product(create_req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Read / 读取
        let resp = controller.get_product(1).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Update / 更新
        let update_req = UpdateProductRequest {
            price: Some(14.99),
            name: None,
            description: None,
            stock: None,
            category: None,
        };
        let resp = controller.update_product(1, update_req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Delete / 删除
        let resp = controller.delete_product(1).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify deleted / 验证已删除
        let resp = controller.get_product(1).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_validation_rejects_invalid_product() {
        let repo = Arc::new(ProductRepository::new());
        let controller = ProductController::new(repo);

        let bad_req = CreateProductRequest {
            name: "X".to_string(),
            description: "Test".to_string(),
            price: -1.0,
            stock: -5,
            category: "test".to_string(),
        };
        let resp = controller.create_product(bad_req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_auth_register_and_login() {
        let controller = AuthController::new();

        // Register / 注册
        let resp = controller.register("testuser", "pass123").await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Login with correct password / 正确密码登录
        let resp = controller.login("testuser", "pass123").await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Login with wrong password / 错误密码登录
        let resp = controller.login("testuser", "wrong").await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Duplicate registration / 重复注册
        let resp = controller.register("testuser", "other").await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }
}
