// sentiric-registrar-service/src/grpc/service.rs
use std::sync::Arc;
use tokio::sync::Mutex;
use sentiric_contracts::sentiric::sip::v1::{
    registrar_service_server::RegistrarService, 
    RegisterRequest, RegisterResponse, 
    UnregisterRequest, UnregisterResponse, 
    LookupContactRequest, LookupContactResponse
};
// KRİTİK DÜZELTME: Nokta (.) yerine Çift İki Nokta (::) kullanıldı.
use sentiric_contracts::sentiric::user::v1::GetSipCredentialsRequest;
use tonic::{Request, Response, Status};
use tracing::{info, error, warn, instrument};
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
        let username = sentiric_sip_core::utils::extract_username_from_uri(&req.sip_uri);

        // 1. User-Service'den HA1 Hash'ini iste (Single Source of Truth)
        let mut user_client = {
            let guard = self.clients.lock().await;
            guard.user.clone()
        };

        // User service çağrısı
        let user_res = user_client.get_sip_credentials(Request::new(GetSipCredentialsRequest {
            sip_username: username.clone(),
            realm: self.config.sip_realm.clone(),
        })).await;

        match user_res {
            Ok(res) => {
                let _creds = res.into_inner();
                // Registrar, Proxy'den gelen "Doğrulandı" onayına güvenerek kaydı yapar.
                info!("📝 Registering verified user: {}", username);
                
                if let Err(e) = self.store.register_user(&req.sip_uri, &req.contact_uri, req.expires).await {
                    error!("Redis storage error: {}", e);
                    return Err(Status::internal("Database failure"));
                }
                
                Ok(Response::new(RegisterResponse { success: true }))
            },
            Err(e) => {
                warn!("❌ Registration rejected: User {} not found or error: {}", username, e);
                Err(Status::unauthenticated("Invalid SIP credentials"))
            }
        }
    }

    async fn unregister(&self, request: Request<UnregisterRequest>) -> Result<Response<UnregisterResponse>, Status> {
        let req = request.into_inner();
        let _ = self.store.unregister_user(&req.sip_uri).await;
        Ok(Response::new(UnregisterResponse { success: true }))
    }

    async fn lookup_contact(&self, request: Request<LookupContactRequest>) -> Result<Response<LookupContactResponse>, Status> {
        let req = request.into_inner();
        let contact = self.store.lookup_user(&req.sip_uri).await;

        if let Some(c) = contact {
            Ok(Response::new(LookupContactResponse { contact_uris: vec![c] }))
        } else {
            Ok(Response::new(LookupContactResponse { contact_uris: vec![] }))
        }
    }
}