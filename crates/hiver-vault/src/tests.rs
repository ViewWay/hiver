//! Tests for hiver-vault
//! hiver-vault 测试
//!
//! Comprehensive test suite using mockito for HTTP mocking.
//! 使用 mockito 进行 HTTP 模拟的综合测试套件。

use reqwest::Client;
use url::Url;

use crate::{
    auth::{AppRoleAuth, AuthBackend, TokenAuth},
    client::{VaultClient, VaultConfig},
    error::VaultError,
    health, lease, secret,
};

// ============================================================================
// Helpers / 辅助函数
// ============================================================================

/// Build a VaultClient pointing at the mockito server with an initial token.
/// 构建指向 mockito 服务器的 VaultClient，并带有初始 token。
fn mock_client(mock_url: &Url) -> VaultClient {
    let http = Client::builder().build().expect("build reqwest client");
    VaultClient::from_parts(http, mock_url.clone(), Some("test-token".into()), None)
}

/// Build a VaultClient without a token (unauthenticated).
/// 构建没有 token 的 VaultClient（未认证）。
fn mock_client_no_token(mock_url: &Url) -> VaultClient {
    let http = Client::builder().build().expect("build reqwest client");
    VaultClient::from_parts(http, mock_url.clone(), None, None)
}

// ============================================================================
// 1. VaultConfig & VaultClient tests / 配置与客户端测试
// ============================================================================

#[test]
fn config_builder_default_address() {
    // Default address should be https://127.0.0.1:8200
    // 默认地址应为 https://127.0.0.1:8200
    let config = VaultConfig::builder()
        .build()
        .expect("build default config");
    assert_eq!(config.address.as_str(), "https://127.0.0.1:8200/");
}

#[test]
fn config_builder_custom_address() {
    // Custom address should be used
    // 应使用自定义地址
    let config = VaultConfig::builder()
        .address("http://vault.example.com:8200")
        .build()
        .expect("build custom config");
    assert!(config.address.as_str().contains("vault.example.com"));
}

#[test]
fn config_builder_invalid_address() {
    // Invalid address should return error
    // 无效地址应返回错误
    let result = VaultConfig::builder().address("not a url").build();
    assert!(result.is_err());
}

#[test]
fn config_builder_with_token() {
    // Token should be stored in config
    // Token 应存储在配置中
    let config = VaultConfig::builder()
        .token("my-root-token")
        .build()
        .expect("build config with token");
    assert_eq!(config.token.as_deref(), Some("my-root-token"));
}

#[test]
fn config_builder_with_namespace() {
    // Namespace should be stored in config
    // 命名空间应存储在配置中
    let config = VaultConfig::builder()
        .namespace("my-ns")
        .build()
        .expect("build config with namespace");
    assert_eq!(config.namespace.as_deref(), Some("my-ns"));
}

#[test]
fn config_builder_timeout() {
    // Custom timeout should be stored
    // 自定义超时应存储
    let config = VaultConfig::builder()
        .timeout_secs(60)
        .build()
        .expect("build config with timeout");
    assert_eq!(config.timeout.as_secs(), 60);
}

#[test]
fn client_url_building() {
    // url() should correctly join v1/ prefix
    // url() 应正确拼接 v1/ 前缀
    let config = VaultConfig::builder()
        .address("http://localhost:8200")
        .build()
        .expect("build config");
    let client = VaultClient::connect(config).expect("connect");
    let url = client.url("secret/data/myapp").expect("build url");
    assert_eq!(url.as_str(), "http://localhost:8200/v1/secret/data/myapp");
}

#[test]
fn client_set_and_get_token() {
    // set_token and token() should round-trip
    // set_token 和 token() 应能往返
    let config = VaultConfig::builder().build().expect("build config");
    let client = VaultClient::connect(config).expect("connect");
    assert!(client.token().is_none());
    client.set_token("new-token");
    assert_eq!(client.token().as_deref(), Some("new-token"));
}

// ============================================================================
// 2. Error handling tests / 错误处理测试
// ============================================================================

#[test]
fn error_from_status_server_error() {
    // 5xx should produce ServerError
    // 5xx 应产生 ServerError
    let err = VaultError::from_status(500, "internal error");
    match err {
        VaultError::ServerError { status, message } => {
            assert_eq!(status, 500);
            assert_eq!(message, "internal error");
        },
        _ => panic!("Expected ServerError, got {err:?}"),
    }
}

#[test]
fn error_from_status_permission_denied() {
    // 403 should produce PermissionDenied
    // 403 应产生 PermissionDenied
    let err = VaultError::from_status(403, "forbidden");
    match err {
        VaultError::PermissionDenied(msg) => assert_eq!(msg, "forbidden"),
        _ => panic!("Expected PermissionDenied, got {err:?}"),
    }
}

#[test]
fn error_from_status_not_found() {
    // 404 should produce SecretNotFound
    // 404 应产生 SecretNotFound
    let err = VaultError::from_status(404, "secret/data/foo");
    match err {
        VaultError::SecretNotFound { path } => {
            assert_eq!(path, "secret/data/foo");
        },
        _ => panic!("Expected SecretNotFound, got {err:?}"),
    }
}

#[test]
fn error_from_status_client_error() {
    // Other 4xx should produce ClientError
    // 其他 4xx 应产生 ClientError
    let err = VaultError::from_status(400, "bad request");
    match err {
        VaultError::ClientError { status, message } => {
            assert_eq!(status, 400);
            assert_eq!(message, "bad request");
        },
        _ => panic!("Expected ClientError, got {err:?}"),
    }
}

// ============================================================================
// 3. Auth - Token authentication tests / Token 认证测试
// ============================================================================

#[tokio::test]
async fn token_auth_success() {
    // Token lookup-self returns valid auth data
    // Token lookup-self 返回有效认证数据
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/auth/token/lookup-self")
        .match_header("X-Vault-Token", "valid-token")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "accessor": "accessor-123",
                    "policies": ["root", "default"],
                    "token_type": "service",
                    "lease_duration": 86400,
                    "renewable": true
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client_no_token(&base_url);

    let auth = TokenAuth::new("valid-token");
    let result = auth
        .authenticate(&client)
        .await
        .expect("auth should succeed");

    mock.assert_async().await;
    assert_eq!(result.client_token, "valid-token");
    assert_eq!(result.accessor.as_deref(), Some("accessor-123"));
    assert_eq!(result.policies, vec!["root", "default"]);
    assert_eq!(result.token_type.as_deref(), Some("service"));
    assert_eq!(result.lease_duration, Some(86400));
    assert_eq!(result.renewable, Some(true));
}

#[tokio::test]
async fn token_auth_invalid_token() {
    // Invalid token should produce AuthenticationFailed
    // 无效 Token 应产生 AuthenticationFailed
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/auth/token/lookup-self")
        .match_header("X-Vault-Token", "bad-token")
        .with_status(403)
        .with_body("permission denied")
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client_no_token(&base_url);

    let auth = TokenAuth::new("bad-token");
    let result = auth.authenticate(&client).await;

    mock.assert_async().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        VaultError::AuthenticationFailed(msg) => {
            assert!(msg.contains("403"));
        },
        other => panic!("Expected AuthenticationFailed, got {other:?}"),
    }
}

#[tokio::test]
async fn token_auth_missing_data_field() {
    // Response missing data field should produce InvalidResponse
    // 响应缺少 data 字段应产生 InvalidResponse
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/auth/token/lookup-self")
        .match_header("X-Vault-Token", "token-no-data")
        .with_status(200)
        .with_body(serde_json::json!({"no_data": true}).to_string())
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client_no_token(&base_url);

    let auth = TokenAuth::new("token-no-data");
    let result = auth.authenticate(&client).await;

    mock.assert_async().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        VaultError::InvalidResponse(msg) => assert!(msg.contains("Missing data")),
        other => panic!("Expected InvalidResponse, got {other:?}"),
    }
}

// ============================================================================
// 4. Auth - AppRole authentication tests / AppRole 认证测试
// ============================================================================

#[tokio::test]
async fn approle_auth_success() {
    // AppRole login returns valid auth data
    // AppRole 登录返回有效认证数据
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/auth/approle/login")
        .match_body(
            serde_json::json!({
                "role_id": "role-abc",
                "secret_id": "secret-xyz"
            })
            .to_string()
            .as_str(),
        )
        .with_status(200)
        .with_body(
            serde_json::json!({
                "auth": {
                    "client_token": "generated-token-123",
                    "accessor": "acc-456",
                    "policies": ["app-policy"],
                    "token_type": "batch",
                    "lease_duration": 3600,
                    "renewable": false
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client_no_token(&base_url);

    let auth = AppRoleAuth::new("role-abc", "secret-xyz", "approle");
    let result = auth
        .authenticate(&client)
        .await
        .expect("approle auth should succeed");

    mock.assert_async().await;
    assert_eq!(result.client_token, "generated-token-123");
    assert_eq!(result.accessor.as_deref(), Some("acc-456"));
    assert_eq!(result.policies, vec!["app-policy"]);
    assert_eq!(result.lease_duration, Some(3600));
    assert_eq!(result.renewable, Some(false));
}

#[tokio::test]
async fn approle_auth_invalid_credentials() {
    // AppRole login with wrong credentials should fail
    // 使用错误凭据的 AppRole 登录应失败
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/auth/approle/login")
        .with_status(400)
        .with_body("invalid role_id or secret_id")
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client_no_token(&base_url);

    let auth = AppRoleAuth::new("bad-role", "bad-secret", "approle");
    let result = auth.authenticate(&client).await;

    mock.assert_async().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        VaultError::AuthenticationFailed(msg) => {
            assert!(msg.contains("400"));
        },
        other => panic!("Expected AuthenticationFailed, got {other:?}"),
    }
}

#[tokio::test]
async fn approle_auth_custom_mount() {
    // AppRole with custom mount path uses correct URL
    // 使用自定义挂载路径的 AppRole 使用正确的 URL
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/auth/my-approle/login")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "auth": {
                    "client_token": "custom-mount-token",
                    "policies": [],
                    "token_type": "service"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client_no_token(&base_url);

    let auth = AppRoleAuth::new("r", "s", "my-approle");
    let result = auth.authenticate(&client).await.expect("custom mount auth");

    mock.assert_async().await;
    assert_eq!(result.client_token, "custom-mount-token");
}

#[tokio::test]
async fn client_auth_approle_sets_token() {
    // auth_approle on VaultClient should set the client token
    // VaultClient 上的 auth_approle 应设置客户端 token
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/auth/approle/login")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "auth": {
                    "client_token": "via-client-token",
                    "policies": ["default"],
                    "token_type": "service"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client_no_token(&base_url);

    assert!(client.token().is_none());
    let result = client
        .auth_approle("role-id", "secret-id", "approle")
        .await
        .expect("auth_approle");

    mock.assert_async().await;
    assert_eq!(result.client_token, "via-client-token");
    assert_eq!(client.token().as_deref(), Some("via-client-token"));
}

// ============================================================================
// 5. KV v1 tests / KV v1 测试
// ============================================================================

#[tokio::test]
async fn kv_v1_read_secret() {
    // Read a secret from KV v1 backend
    // 从 KV v1 后端读取密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/secret/myapp/config")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "username": "admin",
                    "password": "s3cret"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v1("secret");

    let data = kv.read("myapp/config").await.expect("kv v1 read");
    mock.assert_async().await;
    assert_eq!(data["username"], "admin");
    assert_eq!(data["password"], "s3cret");
}

#[tokio::test]
async fn kv_v1_write_secret() {
    // Write a secret to KV v1 backend
    // 向 KV v1 后端写入密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/secret/myapp/config")
        .match_body(
            serde_json::json!({"data": {"key": "value"}})
                .to_string()
                .as_str(),
        )
        .with_status(200)
        .with_body("{}")
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v1("secret");

    kv.write("myapp/config", serde_json::json!({"key": "value"}))
        .await
        .expect("kv v1 write");
    mock.assert_async().await;
}

#[tokio::test]
async fn kv_v1_delete_secret() {
    // Delete a secret from KV v1 backend
    // 从 KV v1 后端删除密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/secret/myapp/config")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v1("secret");

    kv.delete("myapp/config").await.expect("kv v1 delete");
    mock.assert_async().await;
}

#[tokio::test]
async fn kv_v1_list_secrets() {
    // List secrets in KV v1 backend
    // 列出 KV v1 后端中的密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("LIST", "/v1/secret/myapp")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "keys": ["config", "credentials", "subfolder/"]
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v1("secret");

    let keys = kv.list("myapp").await.expect("kv v1 list");
    mock.assert_async().await;
    assert_eq!(keys, vec!["config", "credentials", "subfolder/"]);
}

#[tokio::test]
async fn kv_v1_read_not_found() {
    // Reading a non-existent secret returns error
    // 读取不存在的密钥返回错误
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/secret/nonexistent")
        .with_status(404)
        .with_body("not found")
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v1("secret");

    let result = kv.read("nonexistent").await;
    mock.assert_async().await;
    assert!(result.is_err());
}

// ============================================================================
// 6. KV v2 tests / KV v2 测试
// ============================================================================

#[tokio::test]
async fn kv_v2_read_secret() {
    // Read latest version of a KV v2 secret
    // 读取 KV v2 密钥的最新版本
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/secret/data/myapp/config")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "data": {"db_url": "postgres://localhost"},
                    "metadata": {
                        "created_time": "2025-01-01T00:00:00Z",
                        "deletion_time": "",
                        "destroyed": false,
                        "version": 3
                    }
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v2("secret");

    let secret = kv.read("myapp/config").await.expect("kv v2 read");
    mock.assert_async().await;
    assert_eq!(secret.data["db_url"], "postgres://localhost");
    assert_eq!(secret.metadata.version, 3);
    assert!(!secret.metadata.destroyed);
}

#[tokio::test]
async fn kv_v2_write_secret() {
    // Write a secret to KV v2 backend
    // 向 KV v2 后端写入密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/secret/data/myapp/config")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "created_time": "2025-06-01T00:00:00Z",
                    "deletion_time": "",
                    "destroyed": false,
                    "version": 1
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v2("secret");

    let metadata = kv
        .write("myapp/config", serde_json::json!({"key": "val"}))
        .await
        .expect("kv v2 write");
    mock.assert_async().await;
    assert_eq!(metadata.version, 1);
}

#[tokio::test]
async fn kv_v2_delete_secret() {
    // Soft-delete latest version of a KV v2 secret
    // 软删除 KV v2 密钥的最新版本
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/secret/data/myapp/config")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v2("secret");

    kv.delete("myapp/config").await.expect("kv v2 delete");
    mock.assert_async().await;
}

#[tokio::test]
async fn kv_v2_delete_versions() {
    // Delete specific versions of a KV v2 secret
    // 删除 KV v2 密钥的指定版本
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/secret/delete/myapp/config")
        .match_body(serde_json::json!({"versions": [1, 2]}).to_string().as_str())
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v2("secret");

    kv.delete_versions("myapp/config", &[1, 2])
        .await
        .expect("kv v2 delete versions");
    mock.assert_async().await;
}

#[tokio::test]
async fn kv_v2_undelete_versions() {
    // Undelete specific versions of a KV v2 secret
    // 恢复 KV v2 密钥的指定版本
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/secret/undelete/myapp/config")
        .match_body(serde_json::json!({"versions": [1]}).to_string().as_str())
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v2("secret");

    kv.undelete_versions("myapp/config", &[1])
        .await
        .expect("kv v2 undelete");
    mock.assert_async().await;
}

#[tokio::test]
async fn kv_v2_destroy_versions() {
    // Permanently destroy versions of a KV v2 secret
    // 永久销毁 KV v2 密钥的指定版本
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/secret/destroy/myapp/config")
        .match_body(
            serde_json::json!({"versions": [1, 2, 3]})
                .to_string()
                .as_str(),
        )
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v2("secret");

    kv.destroy_versions("myapp/config", &[1, 2, 3])
        .await
        .expect("kv v2 destroy");
    mock.assert_async().await;
}

#[tokio::test]
async fn kv_v2_list_secrets() {
    // List secrets in KV v2 backend via metadata endpoint
    // 通过元数据端点列出 KV v2 后端中的密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("LIST", "/v1/secret/metadata/myapp")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "keys": ["config", "db/", "api-key"]
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v2("secret");

    let keys = kv.list("myapp").await.expect("kv v2 list");
    mock.assert_async().await;
    assert_eq!(keys, vec!["config", "db/", "api-key"]);
}

#[tokio::test]
async fn kv_v2_read_metadata() {
    // Read metadata for a KV v2 secret
    // 读取 KV v2 密钥的元数据
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/secret/metadata/myapp/config")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "created_time": "2025-01-01T00:00:00Z",
                    "max_version": 5,
                    "cas_required": false
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v2("secret");

    let meta = kv
        .read_metadata("myapp/config")
        .await
        .expect("kv v2 read metadata");
    mock.assert_async().await;
    assert_eq!(meta.version, 5);
}

#[tokio::test]
async fn kv_v2_delete_metadata() {
    // Delete all versions and metadata for a KV v2 secret
    // 删除 KV v2 密钥的所有版本和元数据
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/secret/metadata/myapp/config")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let kv = client.kv_v2("secret");

    kv.delete_metadata("myapp/config")
        .await
        .expect("kv v2 delete metadata");
    mock.assert_async().await;
}

// ============================================================================
// 7. Transit tests / Transit 测试
// ============================================================================

#[tokio::test]
async fn transit_create_key() {
    // Create a new transit encryption key
    // 创建新的 transit 加密密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/transit/keys/my-key")
        .match_body(
            serde_json::json!({"type": "aes256-gcm96"})
                .to_string()
                .as_str(),
        )
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let transit = client.transit("transit");

    transit
        .create_key("my-key", "aes256-gcm96")
        .await
        .expect("transit create key");
    mock.assert_async().await;
}

#[tokio::test]
async fn transit_read_key() {
    // Read transit key information
    // 读取 transit 密钥信息
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/transit/keys/my-key")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "name": "my-key",
                    "type": "aes256-gcm96",
                    "deletion_allowed": false,
                    "min_decryption_version": 1,
                    "min_encryption_version": 0,
                    "latest_version": 1,
                    "exportable": false,
                    "keys": {"1": {}}
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let transit = client.transit("transit");

    let info = transit.read_key("my-key").await.expect("transit read key");
    mock.assert_async().await;
    assert_eq!(info.name, "my-key");
    assert_eq!(info.key_type, "aes256-gcm96");
    assert_eq!(info.latest_version, Some(1));
}

#[tokio::test]
async fn transit_encrypt() {
    // Encrypt plaintext via Transit engine
    // 通过 Transit 引擎加密明文
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/transit/encrypt/my-key")
        // plaintext "hello" base64 = "aGVsbG8="
        .match_body(serde_json::json!({"plaintext": "aGVsbG8="}).to_string().as_str())
        .with_status(200)
        .with_body(serde_json::json!({
            "data": {
                "ciphertext": "vault:v1:ABCDEFGH"
            }
        }).to_string())
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let transit = client.transit("transit");

    let ct = transit
        .encrypt("my-key", b"hello")
        .await
        .expect("transit encrypt");
    mock.assert_async().await;
    assert_eq!(ct, "vault:v1:ABCDEFGH");
}

#[tokio::test]
async fn transit_decrypt() {
    // Decrypt ciphertext via Transit engine
    // 通过 Transit 引擎解密密文
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/transit/decrypt/my-key")
        .match_body(
            serde_json::json!({"ciphertext": "vault:v1:ABCDEFGH"})
                .to_string()
                .as_str(),
        )
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "plaintext": "aGVsbG8="
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let transit = client.transit("transit");

    let pt = transit
        .decrypt("my-key", "vault:v1:ABCDEFGH")
        .await
        .expect("transit decrypt");
    mock.assert_async().await;
    assert_eq!(pt, b"hello");
}

#[tokio::test]
async fn transit_rotate_key() {
    // Rotate a transit key
    // 轮换 transit 密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/transit/keys/my-key/rotate")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let transit = client.transit("transit");

    transit.rotate_key("my-key").await.expect("transit rotate");
    mock.assert_async().await;
}

#[tokio::test]
async fn transit_rewrap() {
    // Rewrap ciphertext with latest key version
    // 使用最新密钥版本重包装密文
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/transit/rewrap/my-key")
        .match_body(
            serde_json::json!({"ciphertext": "vault:v1:old"})
                .to_string()
                .as_str(),
        )
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "ciphertext": "vault:v2:newCT"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let transit = client.transit("transit");

    let new_ct = transit
        .rewrap("my-key", "vault:v1:old")
        .await
        .expect("transit rewrap");
    mock.assert_async().await;
    assert_eq!(new_ct, "vault:v2:newCT");
}

#[tokio::test]
async fn transit_delete_key() {
    // Delete a transit key
    // 删除 transit 密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/transit/keys/my-key")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let transit = client.transit("transit");

    transit
        .delete_key("my-key")
        .await
        .expect("transit delete key");
    mock.assert_async().await;
}

#[tokio::test]
async fn transit_list_keys() {
    // List transit keys
    // 列出 transit 密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("LIST", "/v1/transit/keys")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {"keys": ["key-a", "key-b"]}
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let transit = client.transit("transit");

    let keys = transit.list_keys().await.expect("transit list keys");
    mock.assert_async().await;
    assert_eq!(keys, vec!["key-a", "key-b"]);
}

// ============================================================================
// 8. PKI tests / PKI 测试
// ============================================================================

#[tokio::test]
async fn pki_generate_certificate() {
    // Generate a certificate via PKI engine
    // 通过 PKI 引擎生成证书
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/pki/issue/my-role")
        .with_status(200)
        .with_body(serde_json::json!({
            "data": {
                "certificate": "-----BEGIN CERTIFICATE-----\nMIID...\n-----END CERTIFICATE-----",
                "private_key": "-----BEGIN PRIVATE KEY-----\nMIIE...\n-----END PRIVATE KEY-----",
                "issuing_ca": "-----BEGIN CERTIFICATE-----\nCA...\n-----END CERTIFICATE-----",
                "serial_number": "12:34:56:78",
                "expiration": 1735689600
            }
        }).to_string())
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let pki = client.pki("pki");

    let cert = pki
        .generate_certificate("my-role", "my.example.com", vec!["san.example.com".into()])
        .await
        .expect("pki generate cert");

    mock.assert_async().await;
    assert!(cert.certificate.contains("BEGIN CERTIFICATE"));
    assert!(cert.private_key.is_some());
    assert!(cert.serial_number.is_some());
    assert_eq!(cert.serial_number.unwrap(), "12:34:56:78");
}

#[tokio::test]
async fn pki_set_role() {
    // Create or update a PKI role
    // 创建或更新 PKI 角色
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/pki/roles/web-server")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let pki = client.pki("pki");

    let role = crate::pki::CertificateRole {
        allowed_domains: Some(vec!["example.com".into()]),
        allow_subdomains: Some(true),
        max_ttl: Some("72h".into()),
        ttl: Some("24h".into()),
        ..Default::default()
    };
    pki.set_role("web-server", &role)
        .await
        .expect("pki set role");
    mock.assert_async().await;
}

#[tokio::test]
async fn pki_read_role() {
    // Read a PKI role
    // 读取 PKI 角色
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/pki/roles/web-server")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "allowed_domains": ["example.com"],
                    "allow_subdomains": true,
                    "max_ttl": "72h"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let pki = client.pki("pki");

    let data = pki.read_role("web-server").await.expect("pki read role");
    mock.assert_async().await;
    assert!(data["allow_subdomains"].as_bool().unwrap());
}

#[tokio::test]
async fn pki_delete_role() {
    // Delete a PKI role
    // 删除 PKI 角色
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/pki/roles/web-server")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let pki = client.pki("pki");

    pki.delete_role("web-server")
        .await
        .expect("pki delete role");
    mock.assert_async().await;
}

#[tokio::test]
async fn pki_revoke_certificate() {
    // Revoke a certificate by serial number
    // 通过序列号撤销证书
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/pki/revoke")
        .match_body(
            serde_json::json!({"serial_number": "12:34:56:78"})
                .to_string()
                .as_str(),
        )
        .with_status(200)
        .with_body("{}")
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let pki = client.pki("pki");

    pki.revoke("12:34:56:78").await.expect("pki revoke");
    mock.assert_async().await;
}

#[tokio::test]
async fn pki_read_ca_certificate() {
    // Read the CA certificate from PKI engine
    // 从 PKI 引擎读取 CA 证书
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/pki/cert/ca")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "certificate": "-----BEGIN CERTIFICATE-----\nCA-CERT\n-----END CERTIFICATE-----"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let pki = client.pki("pki");

    let ca_cert = pki.read_ca_certificate().await.expect("pki read CA cert");
    mock.assert_async().await;
    assert!(ca_cert.contains("CA-CERT"));
}

#[tokio::test]
async fn pki_read_certificate_by_serial() {
    // Read a certificate by serial number
    // 通过序列号读取证书
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/pki/cert/12-34-56-78")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {
                    "certificate": "-----BEGIN CERTIFICATE-----\nCERT\n-----END CERTIFICATE-----"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let pki = client.pki("pki");

    let cert = pki
        .read_certificate("12-34-56-78")
        .await
        .expect("pki read cert");
    mock.assert_async().await;
    assert!(cert.contains("CERT"));
}

#[tokio::test]
async fn pki_rotate_crl() {
    // Rotate the CRL
    // 轮换 CRL
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/pki/crl/rotate")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let pki = client.pki("pki");

    pki.rotate_crl().await.expect("pki rotate crl");
    mock.assert_async().await;
}

#[tokio::test]
async fn pki_tidy() {
    // Tidy up the PKI backend
    // 清理 PKI 后端
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/pki/tidy")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);
    let pki = client.pki("pki");

    pki.tidy().await.expect("pki tidy");
    mock.assert_async().await;
}

// ============================================================================
// 9. Health tests / 健康检查测试
// ============================================================================

#[tokio::test]
async fn health_check_healthy() {
    // Vault returns 200 when healthy
    // Vault 健康时返回 200
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/sys/health")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "initialized": true,
                "sealed": false,
                "standby": false,
                "version": "1.18.0"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    let status = health::check_health(&client).await.expect("health check");
    mock.assert_async().await;
    assert!(status.initialized);
    assert!(!status.sealed);
    assert!(!status.standby);
    assert_eq!(status.version.as_deref(), Some("1.18.0"));
}

#[tokio::test]
async fn health_check_sealed() {
    // Vault returns 503 when sealed (still valid health response)
    // Vault 封印时返回 503（仍然是有效的健康响应）
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/sys/health")
        .with_status(503)
        .with_body(
            serde_json::json!({
                "initialized": true,
                "sealed": true,
                "standby": false,
                "version": "1.18.0"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    let status = health::check_health(&client)
        .await
        .expect("health check sealed");
    mock.assert_async().await;
    assert!(status.sealed);
}

#[tokio::test]
async fn health_check_standby() {
    // Vault returns 429 when in standby mode
    // Vault 待机时返回 429
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/sys/health")
        .with_status(429)
        .with_body(
            serde_json::json!({
                "initialized": true,
                "sealed": false,
                "standby": true,
                "version": "1.18.0"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    let status = health::check_health(&client)
        .await
        .expect("health check standby");
    mock.assert_async().await;
    assert!(status.standby);
}

#[tokio::test]
async fn seal_status_unsealed() {
    // Get seal status when Vault is unsealed
    // Vault 未封印时获取封印状态
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/sys/seal-status")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "type": "shamir",
                "sealed": false,
                "total_shares": 5,
                "threshold": 3,
                "progress": 0,
                "nonce": "",
                "version": "1.18.0",
                "initialized": true
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    let seal = health::get_seal_status(&client).await.expect("seal status");
    mock.assert_async().await;
    assert!(!seal.sealed);
    assert_eq!(seal.seal_type.as_deref(), Some("shamir"));
    assert_eq!(seal.total_shares, Some(5));
    assert_eq!(seal.threshold, Some(3));
}

#[tokio::test]
async fn seal_and_unseal() {
    // Seal and then unseal Vault
    // 封印然后解封 Vault
    let mut server = mockito::Server::new_async().await;

    let seal_mock = server
        .mock("PUT", "/v1/sys/seal")
        .match_header("Authorization", "Bearer test-token")
        .with_status(204)
        .create_async()
        .await;

    let unseal_mock = server
        .mock("PUT", "/v1/sys/unseal")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "type": "shamir",
                "sealed": false,
                "progress": 3,
                "version": "1.18.0"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    // Seal / 封印
    health::seal(&client).await.expect("seal");
    seal_mock.assert_async().await;

    // Unseal / 解封
    let result = health::unseal(&client, "unseal-key-1")
        .await
        .expect("unseal");
    unseal_mock.assert_async().await;
    assert!(!result.sealed);
}

// ============================================================================
// 10. Lease tests / 租约测试
// ============================================================================

#[tokio::test]
async fn lease_renew() {
    // Renew a lease and verify returned info
    // 续订租约并验证返回的信息
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("PUT", "/v1/sys/leases/renew")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "lease_id": "lease-123",
                "lease_duration": 7200,
                "renewable": true
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    let info = lease::renew(&client, "lease-123", Some(3600))
        .await
        .expect("lease renew");
    mock.assert_async().await;
    assert_eq!(info.lease_id, "lease-123");
    assert_eq!(info.lease_duration, 7200);
    assert!(info.renewable);
}

#[tokio::test]
async fn lease_revoke() {
    // Revoke a lease
    // 撤销租约
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("PUT", "/v1/sys/leases/revoke")
        .match_body(
            serde_json::json!({"lease_id": "lease-456"})
                .to_string()
                .as_str(),
        )
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    lease::revoke(&client, "lease-456")
        .await
        .expect("lease revoke");
    mock.assert_async().await;
}

#[tokio::test]
async fn lease_lookup() {
    // Look up lease information
    // 查找租约信息
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("PUT", "/v1/sys/leases/lookup")
        .match_body(
            serde_json::json!({"lease_id": "lease-789"})
                .to_string()
                .as_str(),
        )
        .with_status(200)
        .with_body(
            serde_json::json!({
                "lease_id": "lease-789",
                "lease_duration": 3600,
                "renewable": true
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    let info = lease::lookup(&client, "lease-789")
        .await
        .expect("lease lookup");
    mock.assert_async().await;
    assert_eq!(info.lease_id, "lease-789");
    assert_eq!(info.lease_duration, 3600);
}

// ============================================================================
// 11. Secret generic operations tests / 通用密钥操作测试
// ============================================================================

#[tokio::test]
async fn secret_read() {
    // Read a secret from generic path
    // 从通用路径读取密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/secret/data/foo")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {"username": "admin"},
                "lease_id": "lease-abc",
                "lease_duration": 3600,
                "renewable": true
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    let secret = secret::read(&client, "secret/data/foo")
        .await
        .expect("secret read");
    mock.assert_async().await;
    assert_eq!(secret.data["username"], "admin");
    assert_eq!(secret.lease_id.as_deref(), Some("lease-abc"));
}

#[tokio::test]
async fn secret_write() {
    // Write a secret to generic path
    // 向通用路径写入密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/secret/data/bar")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {"version": 1}
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    let result = secret::write(&client, "secret/data/bar", &serde_json::json!({"key": "val"}))
        .await
        .expect("secret write");
    mock.assert_async().await;
    assert!(result.is_some());
}

#[tokio::test]
async fn secret_delete() {
    // Delete a secret at generic path
    // 删除通用路径上的密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/secret/data/baz")
        .with_status(204)
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    secret::delete(&client, "secret/data/baz")
        .await
        .expect("secret delete");
    mock.assert_async().await;
}

#[tokio::test]
async fn secret_list() {
    // List secrets at a generic path
    // 列出通用路径上的密钥
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("LIST", "/v1/secret/metadata/app")
        .with_status(200)
        .with_body(
            serde_json::json!({
                "data": {"keys": ["config", "creds"]}
            })
            .to_string(),
        )
        .create_async()
        .await;

    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = mock_client(&base_url);

    let keys = secret::list(&client, "secret/metadata/app")
        .await
        .expect("secret list");
    mock.assert_async().await;
    assert_eq!(keys, vec!["config", "creds"]);
}

// ============================================================================
// 12. Namespace header tests / 命名空间头测试
// ============================================================================

#[tokio::test]
async fn client_sends_namespace_header() {
    // Client should send X-Vault-Namespace header when configured
    // 配置后客户端应发送 X-Vault-Namespace 头
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/secret/data/test")
        .match_header("X-Vault-Namespace", "my-namespace")
        .with_status(200)
        .with_body(serde_json::json!({"data": {}}).to_string())
        .create_async()
        .await;

    let http = Client::builder().build().expect("build client");
    let base_url = Url::parse(&server.url()).expect("parse url");
    let client =
        VaultClient::from_parts(http, base_url, Some("tok".into()), Some("my-namespace".into()));

    let _ = client.get("secret/data/test").await;
    mock.assert_async().await;
}

#[tokio::test]
async fn client_sends_bearer_token() {
    // Client should send Authorization: Bearer <token> header
    // 客户端应发送 Authorization: Bearer <token> 头
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/secret/data/test")
        .match_header("Authorization", "Bearer my-token")
        .with_status(200)
        .with_body(serde_json::json!({"data": {}}).to_string())
        .create_async()
        .await;

    let http = Client::builder().build().expect("build client");
    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = VaultClient::from_parts(http, base_url, Some("my-token".into()), None);

    let _ = client.get("secret/data/test").await;
    mock.assert_async().await;
}

#[tokio::test]
async fn client_no_auth_header_without_token() {
    // Client without token should not send Authorization header
    // 没有 token 的客户端不应发送 Authorization 头
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/secret/data/test")
        .with_status(200)
        .with_body(serde_json::json!({"data": {}}).to_string())
        .create_async()
        .await;

    let http = Client::builder().build().expect("build client");
    let base_url = Url::parse(&server.url()).expect("parse url");
    let client = VaultClient::from_parts(http, base_url, None, None);

    let _ = client.get("secret/data/test").await;
    mock.assert_async().await;
}
