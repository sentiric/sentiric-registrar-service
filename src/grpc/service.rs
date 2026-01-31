// sentiric-registrar-service/src/grpc/service.rs

use std::sync::Arc;
use tokio::sync::Mutex;
use redis::AsyncCommands;
use sentiric_contracts::sentiric::sip::v1::{
    registrar_service_server::RegistrarService, 
    RegisterRequest, RegisterResponse, 
    UnregisterRequest, UnregisterResponse, 
    LookupContactRequest, LookupContactResponse
};
use tonic::{Request, Response, Status};
use tracing::{info, error, instrument, debug};
use crate::grpc::client::InternalClients;
// YENİ: Kütüphaneden standart utils kullanımı
use sentiric_sip_core::utils as sip_utils; 

pub struct MyRegistrarService {
    redis: Arc<Mutex<redis::aio::MultiplexedConnection>>,
    #[allow(dead_code)]
    clients: Arc<Mutex<InternalClients>>,
}

impl MyRegistrarService {
    pub fn new(redis: redis::aio::MultiplexedConnection, clients: Arc<Mutex<InternalClients>>) -> Self {
        Self {
            redis: Arc::new(Mutex::new(redis)),
            clients,
        }
    }
    
    // YARDIMCI: Redis Anahtarı Oluşturucu (Standartlaştırılmış)
    fn generate_redis_key(&self, raw_uri: &str) -> String {
        // Core kütüphane ile AOR (Address of Record) çıkarma.
        // Regex tabanlı temizleme: "Bob <sip:bob@biloxi.com>;tag=..." -> "sip:bob@biloxi.com"
        let aor = sip_utils::extract_aor(raw_uri);
        format!("sip_registration:{}", aor)
    }
}

#[tonic::async_trait]
impl RegistrarService for MyRegistrarService {
    
    #[instrument(skip(self), fields(sip_uri = %request.get_ref().sip_uri, contact = %request.get_ref().contact_uri))]
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        
        let key = self.generate_redis_key(&req.sip_uri);
        
        info!("Kayıt işlemi: {} -> {} (Expires: {})", key, req.contact_uri, req.expires);

        let mut conn = self.redis.lock().await;
        // TTL ile Redis'e yazma
        let _: () = conn.set_ex(&key, &req.contact_uri, req.expires as u64)
            .await
            .map_err(|e| {
                error!("Redis yazma hatası: {}", e);
                Status::internal("Redis Error")
            })?;
        
        Ok(Response::new(RegisterResponse { success: true }))
    }

    #[instrument(skip(self), fields(sip_uri = %request.get_ref().sip_uri))]
    async fn unregister(
        &self,
        request: Request<UnregisterRequest>,
    ) -> Result<Response<UnregisterResponse>, Status> {
        let req = request.into_inner();
        
        let key = self.generate_redis_key(&req.sip_uri);
        
        let mut conn = self.redis.lock().await;
        let _: () = conn.del(&key)
            .await
            .map_err(|e| {
                error!("Redis silme hatası: {}", e);
                Status::internal("Redis Error")
            })?;

        info!("Kayıt silindi: {}", key);
        Ok(Response::new(UnregisterResponse { success: true }))
    }

    #[instrument(skip(self), fields(sip_uri = %request.get_ref().sip_uri))]
    async fn lookup_contact(
        &self,
        request: Request<LookupContactRequest>,
    ) -> Result<Response<LookupContactResponse>, Status> {
        let req = request.into_inner();
        
        let key = self.generate_redis_key(&req.sip_uri);
        
        debug!("Lookup Key: '{}'", key);
        
        let mut conn = self.redis.lock().await;
        let contact: Option<String> = conn.get(&key).await.ok();

        if let Some(c) = contact {
            info!("✅ Lookup Başarılı: {} -> {}", key, c);
            Ok(Response::new(LookupContactResponse { 
                contact_uris: vec![c] 
            }))
        } else {
            info!("❌ Lookup Başarısız (Kayıt Yok): {}", key);
            Ok(Response::new(LookupContactResponse { 
                contact_uris: vec![] 
            }))
        }
    }
}