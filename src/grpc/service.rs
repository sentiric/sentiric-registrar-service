// sentiric-registrar-service/src/grpc/service.rs

use std::sync::Arc;
use tokio::sync::Mutex;
use sentiric_contracts::sentiric::sip::v1::{
    registrar_service_server::RegistrarService, 
    RegisterRequest, RegisterResponse, 
    UnregisterRequest, UnregisterResponse, 
    LookupContactRequest, LookupContactResponse
};
use sentiric_contracts::sentiric::user::v1::GetSipCredentialsRequest;
use tonic::{Request, Response, Status};
use tracing::{info, debug, error, warn, instrument};
use crate::grpc::client::InternalClients;
use crate::data::store::RegistrationStore;
use crate::config::AppConfig;

pub struct MyRegistrarService {
    store: RegistrationStore,
    clients: Arc<Mutex<InternalClients>>,
    config: Arc<AppConfig>,
}

impl MyRegistrarService {
    pub fn new(store: RegistrationStore, clients: Arc<Mutex<InternalClients>>, config: Arc<AppConfig>) -> Self {
        Self { store, clients, config }
    }
}

#[tonic::async_trait]
impl RegistrarService for MyRegistrarService {
    
    #[instrument(skip(self, request), fields(sip_uri = %request.get_ref().sip_uri))]
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        
        // [v1.4.1 ALIGNMENT]: Kullanıcı adını standart yolla ayıkla
        let username = sentiric_sip_core::utils::extract_username_from_uri(&req.sip_uri);

        if username.is_empty() {
            warn!("⚠️ Registration attempt with invalid URI: {}", req.sip_uri);
            return Err(Status::invalid_argument("Invalid SIP URI"));
        }

        // 1. User-Service Sorgusu (HA1 Hash ve Tenant Doğrulaması)
        let mut user_client = {
            let guard = self.clients.lock().await;
            guard.user.clone()
        };

        // [v1.15.0 FIX]: Realm artık zorunlu bir alan.
        let user_res = user_client.get_sip_credentials(Request::new(GetSipCredentialsRequest {
            sip_username: username.clone(),
            realm: self.config.sip_realm.clone(),
        })).await;

        match user_res {
            Ok(res) => {
                let inner = res.into_inner();
                info!("📝 Registering verified user: {} (Tenant: {})", username, inner.tenant_id);
                
                // 2. Redis Kaydı
                if let Err(e) = self.store.register_user(&req.sip_uri, &req.contact_uri, req.expires).await {
                    error!("❌ Redis storage failure for {}: {}", username, e);
                    return Err(Status::internal("Location store failure"));
                }
                
                Ok(Response::new(RegisterResponse { success: true }))
            },
            Err(e) => {
                warn!("❌ Registration rejected: User {} not authorized or not found. Error: {}", username, e);
                // Güvenlik gerekçesiyle detay vermiyoruz
                Err(Status::unauthenticated("Invalid credentials"))
            }
        }
    }

    #[instrument(skip(self, request))]
    async fn unregister(&self, request: Request<UnregisterRequest>) -> Result<Response<UnregisterResponse>, Status> {
        let req = request.into_inner();
        info!("🗑️ Unregistering URI: {}", req.sip_uri);
        
        if let Err(e) = self.store.unregister_user(&req.sip_uri).await {
            error!("❌ Redis delete failure: {}", e);
            return Err(Status::internal("Location store failure"));
        }
        
        Ok(Response::new(UnregisterResponse { success: true }))
    }

    #[instrument(skip(self, request))]
    async fn lookup_contact(&self, request: Request<LookupContactRequest>) -> Result<Response<LookupContactResponse>, Status> {
        let req = request.into_inner();
        
        // Proxy veya B2BUA "Bu kullanıcı nerede?" diye soruyor.
        let contact = self.store.lookup_user(&req.sip_uri).await;

        if let Some(c) = contact {
            debug!("🔍 Lookup Success: {} -> {}", req.sip_uri, c);
            Ok(Response::new(LookupContactResponse { contact_uris: vec![c] }))
        } else {
            debug!("🔍 Lookup Miss: {}", req.sip_uri);
            Ok(Response::new(LookupContactResponse { contact_uris: vec![] }))
        }
    }
}