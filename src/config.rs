// sentiric-registrar-service/src/config.rs
use anyhow::{Context, Result};
use std::env;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub grpc_listen_addr: SocketAddr,
    pub http_listen_addr: SocketAddr,
    
    // Dependencies
    pub redis_url: String,
    pub user_service_url: String,
    
    // SIP Config
    pub sip_realm: String, // Digest Auth için Realm
    
    pub env: String,
    pub rust_log: String,
    pub service_version: String,
    
    // TLS Yolları
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: String,
}

impl AppConfig {
    pub fn load_from_env() -> Result<Self> {
        let grpc_port = env::var("REGISTRAR_SERVICE_GRPC_PORT").unwrap_or_else(|_| "13061".to_string());
        let http_port = env::var("REGISTRAR_SERVICE_HTTP_PORT").unwrap_or_else(|_| "13060".to_string());
        
        let grpc_addr: SocketAddr = format!("[::]:{}", grpc_port).parse()?;
        let http_addr: SocketAddr = format!("[::]:{}", http_port).parse()?;
            
        Ok(AppConfig {
            grpc_listen_addr: grpc_addr,
            http_listen_addr: http_addr, 

            redis_url: env::var("REDIS_URL").context("ZORUNLU: REDIS_URL eksik")?,
            user_service_url: env::var("USER_SERVICE_TARGET_GRPC_URL").context("ZORUNLU: USER_SERVICE_TARGET_GRPC_URL eksik")?,
            
            sip_realm: env::var("SIP_SIGNALING_SERVICE_REALM").unwrap_or_else(|_| "sentiric_demo".to_string()),
            
            env: env::var("ENV").unwrap_or_else(|_| "production".to_string()),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            service_version: env::var("SERVICE_VERSION").unwrap_or_else(|_| "1.0.0".to_string()),
            
            cert_path: env::var("REGISTRAR_SERVICE_CERT_PATH").context("ZORUNLU: REGISTRAR_SERVICE_CERT_PATH eksik")?,
            key_path: env::var("REGISTRAR_SERVICE_KEY_PATH").context("ZORUNLU: REGISTRAR_SERVICE_KEY_PATH eksik")?,
            ca_path: env::var("GRPC_TLS_CA_PATH").context("ZORUNLU: GRPC_TLS_CA_PATH eksik")?,
        })
    }
}