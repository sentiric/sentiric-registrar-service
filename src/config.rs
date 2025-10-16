// sentiric-registrar-service/src/config.rs
use anyhow::{Context, Result};
use std::env;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct AppConfig {
    pub grpc_listen_addr: SocketAddr,
    pub http_listen_addr: SocketAddr,
    
    pub env: String,
    pub rust_log: String,
    pub service_version: String,
    
    // TLS Yolları (Bu servise ait)
    pub cert_path: String,
    pub key_path: String,
    // CA Yolu (Ortak)
    pub ca_path: String,
    
    // Bağımlılıklar
    pub redis_url: String,
    pub user_service_url: String,
    pub sip_realm: String,
}

impl AppConfig {
    pub fn load_from_env() -> Result<Self> {
        // Registrar servisine varsayılan portlar atandı (Harmonik Mimari'de atanacak)
        let grpc_port = env::var("REGISTRAR_SERVICE_GRPC_PORT").unwrap_or_else(|_| "12061".to_string());
        let http_port = env::var("REGISTRAR_SERVICE_HTTP_PORT").unwrap_or_else(|_| "12060".to_string());
        
        let grpc_addr: SocketAddr = format!("[::]:{}", grpc_port).parse()?;
        let http_addr: SocketAddr = format!("[::]:{}", http_port).parse()?;
            
        Ok(AppConfig {
            grpc_listen_addr: grpc_addr,
            http_listen_addr: http_addr, 

            redis_url: env::var("REDIS_URL").context("ZORUNLU: REDIS_URL eksik")?,
            user_service_url: env::var("USER_SERVICE_TARGET_GRPC_URL").context("ZORUNLU: USER_SERVICE_TARGET_GRPC_URL eksik")?,
            sip_realm: env::var("SIP_SIGNALING_SERVICE_REALM").context("ZORUNLU: SIP_SIGNALING_SERVICE_REALM eksik")?,
            
            env: env::var("ENV").unwrap_or_else(|_| "production".to_string()),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            service_version: env::var("SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
            
            cert_path: env::var("REGISTRAR_SERVICE_CERT_PATH").context("ZORUNLU: REGISTRAR_SERVICE_CERT_PATH eksik")?,
            key_path: env::var("REGISTRAR_SERVICE_KEY_PATH").context("ZORUNLU: REGISTRAR_SERVICE_KEY_PATH eksik")?,
            ca_path: env::var("GRPC_TLS_CA_PATH").context("ZORUNLU: GRPC_TLS_CA_PATH eksik")?,
        })
    }
}