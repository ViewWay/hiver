//! PKI secrets engine — Certificate management
//! PKI 密钥引擎 — 证书管理
//!
//! The PKI secrets engine generates dynamic TLS certificates.
/// PKI 密钥引擎生成动态 TLS 证书。
use serde::{Deserialize, Serialize};

use crate::client::VaultClient;
use crate::error::{VaultError, VaultResult};

/// PKI certificate management service / PKI 证书管理服务
///
/// Equivalent to Spring Vault's `PkiOperations`.
/// 等价于 Spring Vault 的 `PkiOperations`。
///
/// # Example
///
/// ```rust,no_run,ignore
/// use nexus_vault::VaultClient;
///
/// async fn example(client: &VaultClient) -> Result<(), Box<dyn std::error::Error>> {
///     let pki = client.pki("pki");
///
///     // Generate a certificate / 生成证书
///     let cert = pki.generate_certificate("my-role", "my.example.com", vec!["san.example.com".to_string()]).await?;
///     println!("Certificate: {}", cert.certificate);
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Pki<'a> {
    client: &'a VaultClient,
    mount: String,
}

/// Certificate generation request / 证书生成请求
#[derive(Debug, Clone, Serialize)]
pub struct CertificateRequest {
    /// Common name / 通用名称
    #[serde(rename = "common_name")]
    pub common_name: String,
    /// Subject Alternative Names (SANs) / 主题备用名称
    #[serde(rename = "alt_names", skip_serializing_if = "Vec::is_empty")]
    pub alt_names: Vec<String>,
    /// IP SANs / IP 主题备用名称
    #[serde(rename = "ip_sans", skip_serializing_if = "Vec::is_empty")]
    pub ip_sans: Vec<String>,
    /// TTL for the certificate / 证书 TTL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
    /// Key type / 密钥类型
    #[serde(rename = "key_type", skip_serializing_if = "Option::is_none")]
    pub key_type: Option<String>,
    /// Key bits / 密钥位数
    #[serde(rename = "key_bits", skip_serializing_if = "Option::is_none")]
    pub key_bits: Option<u32>,
    /// Private key format / 私钥格式
    #[serde(rename = "format", skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Certificate issue/generate response / 证书签发/生成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// The certificate in PEM format / PEM 格式的证书
    pub certificate: String,
    /// The private key in PEM format / PEM 格式的私钥
    #[serde(rename = "private_key")]
    pub private_key: Option<String>,
    /// The issuing CA certificate / 签发 CA 证书
    #[serde(rename = "issuing_ca")]
    pub issuing_ca: Option<String>,
    /// The serial number / 序列号
    pub serial_number: Option<String>,
    /// Expiration time / 过期时间
    pub expiration: Option<i64>,
}

/// Certificate role configuration / 证书角色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRole {
    /// Allowed domains / 允许的域名
    #[serde(rename = "allowed_domains", skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
    /// Allow subdomains / 允许子域名
    #[serde(rename = "allow_subdomains", skip_serializing_if = "Option::is_none")]
    pub allow_subdomains: Option<bool>,
    /// Allow any name / 允许任意名称
    #[serde(rename = "allow_any_name", skip_serializing_if = "Option::is_none")]
    pub allow_any_name: Option<bool>,
    /// Maximum TTL / 最大 TTL
    #[serde(rename = "max_ttl", skip_serializing_if = "Option::is_none")]
    pub max_ttl: Option<String>,
    /// Default TTL / 默认 TTL
    #[serde(rename = "ttl", skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
    /// Key type / 密钥类型
    #[serde(rename = "key_type", skip_serializing_if = "Option::is_none")]
    pub key_type: Option<String>,
    /// Key bits / 密钥位数
    #[serde(rename = "key_bits", skip_serializing_if = "Option::is_none")]
    pub key_bits: Option<u32>,
    /// Server flag / 服务器标志
    #[serde(rename = "server_flag", skip_serializing_if = "Option::is_none")]
    pub server_flag: Option<bool>,
    /// Client flag / 客户端标志
    #[serde(rename = "client_flag", skip_serializing_if = "Option::is_none")]
    pub client_flag: Option<bool>,
}

/// CA certificate set request / CA 证书设置请求
#[derive(Debug, Clone, Serialize)]
struct CaSetRequest {
    #[serde(rename = "pem_bundle")]
    pem_bundle: String,
}

/// CRL information / CRL 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrlInfo {
    /// Whether CRL building is enabled / 是否启用 CRL 构建
    pub building: Option<bool>,
}

/// Data response wrapper / 数据响应包装
#[derive(Debug, Clone, Deserialize)]
struct DataResponse<T> {
    data: T,
}

impl<'a> Pki<'a> {
    /// Create a new PKI handle / 创建新的 PKI 句柄
    pub fn new(client: &'a VaultClient, mount: &str) -> Self {
        Self {
            client,
            mount: mount.to_string(),
        }
    }

    /// Generate a certificate from a role / 从角色生成证书
    ///
    /// Generates a new certificate using the specified PKI role.
    /// 使用指定的 PKI 角色生成新证书。
    pub async fn generate_certificate(
        &self,
        role: &str,
        common_name: &str,
        alt_names: Vec<String>,
    ) -> VaultResult<Certificate> {
        let path = format!("{}/issue/{}", self.mount, role);
        let body = CertificateRequest {
            common_name: common_name.to_string(),
            alt_names,
            ip_sans: vec![],
            ttl: None,
            key_type: None,
            key_bits: None,
            format: Some("pem".to_string()),
        };

        let resp = self.client.post(&path, &body).await?;
        let cert_resp: DataResponse<Certificate> = resp.json().await?;
        Ok(cert_resp.data)
    }

    /// Generate a certificate with full request options / 使用完整请求选项生成证书
    pub async fn generate_certificate_full(
        &self,
        role: &str,
        request: &CertificateRequest,
    ) -> VaultResult<Certificate> {
        let path = format!("{}/issue/{}", self.mount, role);
        let resp = self.client.post(&path, request).await?;
        let cert_resp: DataResponse<Certificate> = resp.json().await?;
        Ok(cert_resp.data)
    }

    /// Create or update a certificate role / 创建或更新证书角色
    pub async fn set_role(
        &self,
        role_name: &str,
        role: &CertificateRole,
    ) -> VaultResult<()> {
        let path = format!("{}/roles/{}", self.mount, role_name);
        self.client.post(&path, role).await?;
        Ok(())
    }

    /// Read a certificate role / 读取证书角色
    pub async fn read_role(
        &self,
        role_name: &str,
    ) -> VaultResult<serde_json::Value> {
        let path = format!("{}/roles/{}", self.mount, role_name);
        let resp = self.client.get(&path).await?;
        let body: serde_json::Value = resp.json().await?;
        body.get("data")
            .cloned()
            .ok_or_else(|| VaultError::InvalidResponse("Missing data field".into()))
    }

    /// List certificate roles / 列出证书角色
    pub async fn list_roles(&self) -> VaultResult<Vec<String>> {
        let path = format!("{}/roles", self.mount);
        crate::secret::list(self.client, &path).await
    }

    /// Delete a certificate role / 删除证书角色
    pub async fn delete_role(&self, role_name: &str) -> VaultResult<()> {
        let path = format!("{}/roles/{}", self.mount, role_name);
        self.client.delete(&path).await?;
        Ok(())
    }

    /// Revoke a certificate by serial number / 通过序列号撤销证书
    pub async fn revoke(
        &self,
        serial_number: &str,
    ) -> VaultResult<()> {
        let path = format!("{}/revoke", self.mount);
        let body = serde_json::json!({
            "serial_number": serial_number
        });
        self.client.post(&path, &body).await?;
        Ok(())
    }

    /// Read the CA certificate / 读取 CA 证书
    pub async fn read_ca_certificate(&self) -> VaultResult<String> {
        let path = format!("{}/cert/ca", self.mount);
        let resp = self.client.get(&path).await?;
        let body: serde_json::Value = resp.json().await?;
        body.get("data")
            .and_then(|d| d.get("certificate"))
            .and_then(|c| c.as_str())
            .map(String::from)
            .ok_or_else(|| VaultError::PkiError("CA certificate not found".into()))
    }

    /// Read a certificate by serial number / 通过序列号读取证书
    pub async fn read_certificate(
        &self,
        serial_number: &str,
    ) -> VaultResult<String> {
        let path = format!("{}/cert/{}", self.mount, serial_number);
        let resp = self.client.get(&path).await?;
        let body: serde_json::Value = resp.json().await?;
        body.get("data")
            .and_then(|d| d.get("certificate"))
            .and_then(|c| c.as_str())
            .map(String::from)
            .ok_or_else(|| VaultError::PkiError("Certificate not found".into()))
    }

    /// List all certificates / 列出所有证书
    pub async fn list_certificates(&self) -> VaultResult<Vec<String>> {
        let path = format!("{}/certs", self.mount);
        crate::secret::list(self.client, &path).await
    }

    /// Set the CA certificate and private key / 设置 CA 证书和私钥
    pub async fn set_ca(
        &self,
        pem_bundle: &str,
    ) -> VaultResult<()> {
        let path = format!("{}/config/ca", self.mount);
        let body = CaSetRequest {
            pem_bundle: pem_bundle.to_string(),
        };
        self.client.post(&path, &body).await?;
        Ok(())
    }

    /// Generate a new internal CA key / 生成新的内部 CA 密钥
    pub async fn generate_root(
        &self,
        common_name: &str,
        key_type: &str,
        key_bits: u32,
        ttl: &str,
    ) -> VaultResult<Certificate> {
        let path = format!("{}/root/generate/internal", self.mount);
        let body = serde_json::json!({
            "common_name": common_name,
            "key_type": key_type,
            "key_bits": key_bits,
            "ttl": ttl,
            "format": "pem"
        });
        let resp = self.client.post(&path, &body).await?;
        let cert_resp: DataResponse<Certificate> = resp.json().await?;
        Ok(cert_resp.data)
    }

    /// Read CRL information / 读取 CRL 信息
    pub async fn read_crl(&self) -> VaultResult<String> {
        let path = format!("{}/crl", self.mount);
        let resp = self.client.get(&path).await?;
        Ok(resp.text().await?)
    }

    /// Rotate the CRL / 轮换 CRL
    pub async fn rotate_crl(&self) -> VaultResult<()> {
        let path = format!("{}/crl/rotate", self.mount);
        self.client.post(&path, &serde_json::json!({})).await?;
        Ok(())
    }

    /// Set PKI engine URLs / 设置 PKI 引擎 URL
    pub async fn set_urls(
        &self,
        issuing_certificates: Vec<String>,
        crl_distribution_points: Vec<String>,
    ) -> VaultResult<()> {
        let path = format!("{}/config/urls", self.mount);
        let body = serde_json::json!({
            "issuing_certificates": issuing_certificates,
            "crl_distribution_points": crl_distribution_points
        });
        self.client.post(&path, &body).await?;
        Ok(())
    }

    /// Tidy up the PKI backend / 清理 PKI 后端
    pub async fn tidy(&self) -> VaultResult<()> {
        let path = format!("{}/tidy", self.mount);
        self.client.post(&path, &serde_json::json!({})).await?;
        Ok(())
    }
}
