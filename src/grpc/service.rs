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
use tracing::{info, error, instrument, debug, warn};
use crate::grpc::client::InternalClients;

// YENİ: SIP Core'dan standart utils
use sentiric_sip_core::utils as sip_utils; 

pub struct MyRegistrarService {
    redis: Arc<Mutex<redis::aio::MultiplexedConnection>>,
    #[allow(dead_code)] // Gelecekte Auth için kullanılacak
    clients: Arc<Mutex<InternalClients>>,
}

impl MyRegistrarService {
    pub fn new(redis: redis::aio::MultiplexedConnection, clients: Arc<Mutex<InternalClients>>) -> Self {
        Self {
            redis: Arc::new(Mutex::new(redis)),
            clients,
        }
    }
    
    // YARDIMCI: Redis Anahtarı Oluşturucu (Normalize Edilmiş)
    // "sip:1001@domain.com" -> "sip_reg:1001"
    fn generate_redis_key(&self, raw_uri: &str) -> String {
        // Core kütüphane ile username çıkarma.
        let username = sip_utils::extract_username_from_uri(raw_uri);
        if username.is_empty() {
             // Eğer username çıkmazsa raw URI kullan (fallback)
             warn!("Username çıkarılamadı, raw URI kullanılıyor: {}", raw_uri);
             return format!("sip_reg:{}", raw_uri);
        }
        format!("sip_reg:{}", username)
    }
}

#[tonic::async_trait]
impl RegistrarService for MyRegistrarService {
    
    /// Kayıt İşlemi (REGISTER)
    /// Bir SIP kullanıcısının anlık konumunu (IP:Port) kaydeder.
    #[instrument(skip(self), fields(sip_uri = %request.get_ref().sip_uri, contact = %request.get_ref().contact_uri))]
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        
        // Anahtarı normalize et
        let key = self.generate_redis_key(&req.sip_uri);
        
        info!("📝 Kayıt İsteği: {} -> {} (TTL: {}s)", key, req.contact_uri, req.expires);

        let mut conn = self.redis.lock().await;
        
        // Expires 0 ise silme isteğidir
        if req.expires <= 0 {
             let _: () = conn.del(&key).await.map_err(|e| {
                error!("Redis silme hatası: {}", e);
                Status::internal("Redis Error")
            })?;
            info!("🗑️ Kayıt silindi (Expires=0): {}", key);
        } else {
            // TTL ile Redis'e yazma
            let _: () = conn.set_ex(&key, &req.contact_uri, req.expires as u64)
                .await
                .map_err(|e| {
                    error!("Redis yazma hatası: {}", e);
                    Status::internal("Redis Error")
                })?;
            debug!("💾 Kayıt başarılı: {}", key);
        }
        
        Ok(Response::new(RegisterResponse { success: true }))
    }

    /// Kayıt Silme (UNREGISTER)
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

        info!("🗑️ Kayıt manuel silindi: {}", key);
        Ok(Response::new(UnregisterResponse { success: true }))
    }

    /// Konum Sorgulama (LOOKUP)
    /// Proxy ve B2BUA, bir kullanıcıya ulaşmak için burayı sorgular.
    #[instrument(skip(self), fields(sip_uri = %request.get_ref().sip_uri))]
    async fn lookup_contact(
        &self,
        request: Request<LookupContactRequest>,
    ) -> Result<Response<LookupContactResponse>, Status> {
        let req = request.into_inner();
        
        let key = self.generate_redis_key(&req.sip_uri);
        
        debug!("🔍 Lookup Sorgusu: '{}'", key);
        
        let mut conn = self.redis.lock().await;
        let contact: Option<String> = conn.get(&key).await.ok();

        if let Some(c) = contact {
            info!("✅ Bulundu: {} -> {}", key, c);
            Ok(Response::new(LookupContactResponse { 
                contact_uris: vec![c] 
            }))
        } else {
            info!("❌ Bulunamadı (Offline): {}", key);
            Ok(Response::new(LookupContactResponse { 
                contact_uris: vec![] 
            }))
        }
    }
}