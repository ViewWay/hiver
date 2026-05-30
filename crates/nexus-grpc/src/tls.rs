//! TLS/mTLS configuration for gRPC server and client.
//! gRPC 服务端和客户端的 TLS/mTLS 配置。

use std::path::PathBuf;

use crate::error::{GrpcError, GrpcResult};

/// TLS configuration for gRPC server and client.
/// gRPC 服务端和客户端的 TLS 配置。
///
/// # Example / 示例
/// ```rust,ignore
/// use nexus_grpc::tls::TlsConfig;
///
/// let tls = TlsConfig::new("certs/server.pem", "certs/server.key")
///     .with_ca_cert("certs/ca.pem");
/// let server_tls = tls.server_tls_config()?;
/// ```
#[allow(clippy::struct_field_names)]
pub struct TlsConfig {
    cert_path: PathBuf,
    key_path: PathBuf,
    ca_cert_path: Option<PathBuf>,
}

impl TlsConfig {
    /// Create a new TLS config with certificate and private key paths.
    /// 使用证书和私钥路径创建新的 TLS 配置。
    pub fn new(cert: impl Into<PathBuf>, key: impl Into<PathBuf>) -> Self {
        Self {
            cert_path: cert.into(),
            key_path: key.into(),
            ca_cert_path: None,
        }
    }

    /// Set the CA certificate path (required for mTLS).
    /// 设置 CA 证书路径（mTLS 必需）。
    pub fn with_ca_cert(mut self, ca: impl Into<PathBuf>) -> Self {
        self.ca_cert_path = Some(ca.into());
        self
    }

    /// Build a tonic `ServerTlsConfig`.
    /// 构建 tonic ServerTlsConfig。
    pub fn server_tls_config(&self) -> GrpcResult<tonic::transport::ServerTlsConfig> {
        let identity = tonic::transport::Identity::from_pem(
            read_file(&self.cert_path)?,
            read_file(&self.key_path)?,
        );
        let mut config = tonic::transport::ServerTlsConfig::new().identity(identity);
        if let Some(ca) = &self.ca_cert_path {
            let ca_cert = read_file(ca)?;
            config = config.client_ca_root(tonic::transport::Certificate::from_pem(ca_cert));
        }
        Ok(config)
    }

    /// Build a tonic `ClientTlsConfig`.
    /// 构建 tonic ClientTlsConfig。
    pub fn client_tls_config(&self) -> GrpcResult<tonic::transport::ClientTlsConfig> {
        let mut config = tonic::transport::ClientTlsConfig::new();
        if let Some(ca) = &self.ca_cert_path {
            let ca_cert = read_file(ca)?;
            config =
                config.ca_certificate(tonic::transport::Certificate::from_pem(ca_cert));
        }
        Ok(config)
    }
}

fn read_file(path: &PathBuf) -> GrpcResult<Vec<u8>> {
    std::fs::read(path).map_err(|e| {
        GrpcError::config(format!(
            "failed to read file {}: {e}",
            path.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_builder() {
        let tls = TlsConfig::new("cert.pem", "key.pem").with_ca_cert("ca.pem");
        assert!(tls.ca_cert_path.is_some());
    }

    #[test]
    fn test_tls_config_no_ca() {
        let tls = TlsConfig::new("cert.pem", "key.pem");
        assert!(tls.ca_cert_path.is_none());
    }
}
