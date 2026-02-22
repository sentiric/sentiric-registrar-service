// sentiric-registrar-service/src/grpc/client.rs

use crate::config::AppConfig;
use anyhow::Result;
use sentiric_contracts::sentiric::user::v1::user_service_client::UserServiceClient;
use tonic::transport::{Channel, ClientTlsConfig, Certificate, Identity};
use std::time::Duration;
use tracing::{info, warn};

pub struct InternalClients {
    pub user: UserServiceClient<Channel>,
}

impl InternalClients {
    pub async fn connect(config: &AppConfig) -> Result<Self> {
        info!("User Service'e bağlanılıyor...");
        let user_channel = create_secure_channel(&config.user_service_url, "user-service", config).await?;
        
        Ok(Self {
            user: UserServiceClient::new(user_channel),
        })
    }
}

async fn create_secure_channel(url: &str, server_name: &str, config: &AppConfig) -> Result<Channel> {
    let target_url = if url.starts_with("https://") {
        url.to_string()
    } else if url.starts_with("http://") {
        warn!(url, "Güvensiz şema (http) algılandı, HTTPS'e zorlanıyor.");
        url.replace("http://", "https://")
    } else {
        format!("https://{}", url)
    };

    let cert = tokio::fs::read(&config.cert_path).await?;
    let key = tokio::fs::read(&config.key_path).await?;
    let identity = Identity::from_pem(cert, key);
    let ca_cert = tokio::fs::read(&config.ca_path).await?;
    let ca_certificate = Certificate::from_pem(ca_cert);

    let tls_config = ClientTlsConfig::new()
        .domain_name(server_name)
        .ca_certificate(ca_certificate)
        .identity(identity);

    info!(url=%target_url, server_name=%server_name, "Güvenli gRPC kanalına bağlanılıyor...");

    // [KRİTİK DÜZELTME]: HTTP/2 Keep-Alive eklendi.
    let channel = Channel::from_shared(target_url)?
        .connect_timeout(Duration::from_secs(5))
        .keep_alive_while_idle(true)
        .http2_keep_alive_interval(Duration::from_secs(15))
        .keep_alive_timeout(Duration::from_secs(5))
        .tls_config(tls_config)?
        .connect()
        .await?;

    info!("gRPC bağlantısı başarılı.");
    Ok(channel)
}