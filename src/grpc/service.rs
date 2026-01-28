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

/// SIP URI'yı normalize eder: user@domain formatına çevirir.
/// Desteklenen formatlar:
///   - "sip:user@domain" → "user@domain"
///   - "<sip:user@domain>" → "user@domain"  
///   - "user@domain" → "user@domain" (değişmez)
///   - "<sip:user@domain;transport=udp>" → "user@domain"
fn normalize_sip_uri(uri: &str) -> String {
    let mut s = uri.trim();
    
    // Açı parantezlerini temizle: <...>
    s = s.trim_start_matches('<').trim_end_matches('>');
    
    // sip: veya sips: prefix'ini temizle
    if s.starts_with("sip:") {
        s = &s[4..];
    } else if s.starts_with("sips:") {
        s = &s[5..];
    }
    
    // URI parametrelerini temizle: ;transport=udp, ;user=phone vb.
    if let Some(semicolon_idx) = s.find(';') {
        s = &s[..semicolon_idx];
    }
    
    s.to_string()
}

pub struct MyRegistrarService {
    redis: Arc<Mutex<redis::aio::MultiplexedConnection>>,
    // DÜZELTME: Gelecekteki Auth işlemleri için tutuyoruz, şimdilik uyarıyı bastır.
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
}

#[tonic::async_trait]
impl RegistrarService for MyRegistrarService {
    
    #[instrument(skip(self), fields(sip_uri = %request.get_ref().sip_uri))]
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        let normalized_uri = normalize_sip_uri(&req.sip_uri);
        let key = format!("sip_registration:{}", normalized_uri);
        
        debug!("URI Normalizasyonu: '{}' -> '{}'", req.sip_uri, normalized_uri);
        info!("Kayıt işlemi: {} -> {} (Expires: {})", normalized_uri, req.contact_uri, req.expires);

        // Redis'e yaz
        let mut conn = self.redis.lock().await;
        // DÜZELTME: req.expires (int32) -> u64 dönüşümü yapıldı
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
        let normalized_uri = normalize_sip_uri(&req.sip_uri);
        let key = format!("sip_registration:{}", normalized_uri);
        
        let mut conn = self.redis.lock().await;
        let _: () = conn.del(&key)
            .await
            .map_err(|e| {
                error!("Redis silme hatası: {}", e);
                Status::internal("Redis Error")
            })?;

        info!("Kayıt silindi: {}", normalized_uri);
        Ok(Response::new(UnregisterResponse { success: true }))
    }

    #[instrument(skip(self), fields(sip_uri = %request.get_ref().sip_uri))]
    async fn lookup_contact(
        &self,
        request: Request<LookupContactRequest>,
    ) -> Result<Response<LookupContactResponse>, Status> {
        let req = request.into_inner();
        let normalized_uri = normalize_sip_uri(&req.sip_uri);
        let key = format!("sip_registration:{}", normalized_uri);
        
        debug!("Lookup URI Normalizasyonu: '{}' -> '{}'", req.sip_uri, normalized_uri);
        
        let mut conn = self.redis.lock().await;
        // Redis'ten oku (String döner)
        let contact: Option<String> = conn.get(&key).await.ok();

        if let Some(c) = contact {
            info!("Lookup başarılı: {} -> {}", normalized_uri, c);
            Ok(Response::new(LookupContactResponse { 
                contact_uris: vec![c] 
            }))
        } else {
            // Eğer Redis'te yoksa boş liste dön
            info!("Lookup başarısız (Kayıt yok): {}", normalized_uri);
            Ok(Response::new(LookupContactResponse { 
                contact_uris: vec![] 
            }))
        }
    }
}