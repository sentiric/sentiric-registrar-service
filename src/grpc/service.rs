// sentiric-registrar-service/src/grpc/service.rs

use std::sync::Arc;
use tokio::sync::Mutex;
use sentiric_contracts::sentiric::sip::v1::{
    registrar_service_server::RegistrarService, 
    RegisterRequest, RegisterResponse, 
    UnregisterRequest, UnregisterResponse, 
    LookupContactRequest, LookupContactResponse
};
use tonic::{Request, Response, Status};
use tracing::{info, error, instrument};
use crate::grpc::client::InternalClients;
use crate::data::store::RegistrationStore; // ✅ Store Eklendi

pub struct MyRegistrarService {
    store: RegistrationStore, // RedisConn yerine Store
    #[allow(dead_code)] 
    clients: Arc<Mutex<InternalClients>>,
}

impl MyRegistrarService {
    pub fn new(store: RegistrationStore, clients: Arc<Mutex<InternalClients>>) -> Self {
        Self {
            store,
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
        
        info!("📝 Kayıt İsteği: {} (TTL: {}s)", req.sip_uri, req.expires);

        // Store üzerinden işlem
        match self.store.register_user(&req.sip_uri, &req.contact_uri, req.expires).await {
            Ok(_) => Ok(Response::new(RegisterResponse { success: true })),
            Err(e) => {
                error!("Store error: {}", e);
                Err(Status::internal("Database Error"))
            }
        }
    }

    #[instrument(skip(self), fields(sip_uri = %request.get_ref().sip_uri))]
    async fn unregister(
        &self,
        request: Request<UnregisterRequest>,
    ) -> Result<Response<UnregisterResponse>, Status> {
        let req = request.into_inner();
        
        match self.store.unregister_user(&req.sip_uri).await {
            Ok(_) => Ok(Response::new(UnregisterResponse { success: true })),
            Err(e) => {
                error!("Store error: {}", e);
                Err(Status::internal("Database Error"))
            }
        }
    }

    #[instrument(skip(self), fields(sip_uri = %request.get_ref().sip_uri))]
    async fn lookup_contact(
        &self,
        request: Request<LookupContactRequest>,
    ) -> Result<Response<LookupContactResponse>, Status> {
        let req = request.into_inner();
        
        let contact = self.store.lookup_user(&req.sip_uri).await;

        if let Some(c) = contact {
            Ok(Response::new(LookupContactResponse { 
                contact_uris: vec![c] 
            }))
        } else {
            Ok(Response::new(LookupContactResponse { 
                contact_uris: vec![] 
            }))
        }
    }
}