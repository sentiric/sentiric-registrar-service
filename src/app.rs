// sentiric-registrar-service/src/app.rs
use crate::config::AppConfig;
use crate::grpc::service::MyRegistrarService;
use crate::grpc::client::InternalClients;
use crate::data::store::{RegistrationStore, RedisConn}; // KRİTİK DÜZELTME: RedisConn doğrudan import edildi.
use crate::tls::load_server_tls_config;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tonic::transport::Server as GrpcServer;
use sentiric_contracts::sentiric::sip::v1::registrar_service_server::RegistrarServiceServer;
use tracing::{info, error};

pub struct App {
    config: Arc<AppConfig>,
}

impl App {
    pub async fn bootstrap() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let config = Arc::new(AppConfig::load_from_env()?);
        
        // Loglama başlatma (tek seferlik kontrol ile)
        if tracing::dispatcher::get_default(|_| true) {
            // Logger zaten aktif.
        } else {
            tracing_subscriber::fmt::init();
        }
        
        Ok(Self { config })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        // 1. Redis Connection with Retry
        let redis_mutex = self.init_redis().await?;
        let store = RegistrationStore::new(redis_mutex);

        // 2. Internal gRPC Clients
        let clients = Arc::new(Mutex::new(InternalClients::connect(&self.config).await?));

        // 3. gRPC Server
        let tls_config = load_server_tls_config(&self.config).await?;
        let grpc_service = MyRegistrarService::new(store, clients, self.config.clone());

        info!("🚀 Registrar Service active on {}", self.config.grpc_listen_addr);

        GrpcServer::builder()
            .tls_config(tls_config)?
            .add_service(RegistrarServiceServer::new(grpc_service))
            .serve_with_shutdown(self.config.grpc_listen_addr, async {
                shutdown_rx.recv().await;
                info!("gRPC server shutting down...");
            })
            .await?;

        let _ = shutdown_tx.send(());
        Ok(())
    }

    // KRİTİK DÜZELTME: Dönüş tipi RegistrationStore::RedisConn yerine doğrudan RedisConn yapıldı.
    async fn init_redis(&self) -> anyhow::Result<RedisConn> {
        loop {
            match redis::Client::open(self.config.redis_url.as_str()) {
                Ok(client) => {
                    match client.get_multiplexed_async_connection().await {
                        Ok(conn) => {
                            info!("✅ Connected to Redis successfully.");
                            return Ok(Arc::new(Mutex::new(conn)));
                        },
                        Err(e) => error!("Redis async connection failed: {}. Retrying in 5s...", e),
                    }
                },
                Err(e) => error!("Redis client creation failed: {}. Retrying in 5s...", e),
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}