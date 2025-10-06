// sentiric-registrar-service/src/grpc/service.rs
use sentiric_contracts::sentiric::sip::v1::{
    registrar_service_server::RegistrarService, 
    RegisterRequest, RegisterResponse, 
    UnregisterRequest, UnregisterResponse, 
    LookupContactRequest, LookupContactResponse
};
use tonic::{Request, Response, Status};
use tracing::{info, instrument};

// Bu servis şimdilik minimalist tutulacaktır. Gerçek Redis/User Client'ları
// app.rs'deki AppState içinde yönetilecektir.
pub struct MyRegistrarService {
    // app_state: Arc<AppState>
}

#[tonic::async_trait]
impl RegistrarService for MyRegistrarService {
    
    #[instrument(skip_all, fields(sip_uri = %request.get_ref().sip_uri))]
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        info!("REGISTER RPC isteği alındı. (mTLS tarafından doğrulanmış olmalı)");
        let req = request.into_inner();
        info!(
            uri = %req.sip_uri, 
            contact = %req.contact_uri, 
            expires = %req.expires,
            "Kayıt işlemi başlatıldı."
        );
        
        // Bu, başarılı bir operasyonu temsil eden yer tutucudur.
        Ok(Response::new(RegisterResponse { success: true }))
    }

    #[instrument(skip_all, fields(sip_uri = %request.get_ref().sip_uri))]
    async fn unregister(
        &self,
        request: Request<UnregisterRequest>,
    ) -> Result<Response<UnregisterResponse>, Status> {
        info!("UNREGISTER RPC isteği alındı.");
        let req = request.into_inner();
        info!(uri = %req.sip_uri, "Kayıt silme işlemi başlatıldı.");
        
        // Bu, başarılı bir operasyonu temsil eden yer tutucudur.
        Ok(Response::new(UnregisterResponse { success: true }))
    }

    #[instrument(skip_all, fields(sip_uri = %request.get_ref().sip_uri))]
    async fn lookup_contact(
        &self,
        request: Request<LookupContactRequest>,
    ) -> Result<Response<LookupContactResponse>, Status> {
        info!("LOOKUP CONTACT RPC isteği alındı.");
        
        // Hardcoded demo yanıtı.
        let default_contact = "sip:10.88.30.2:13024".to_string();
        
        Ok(Response::new(LookupContactResponse { 
            contact_uris: vec![default_contact] 
        }))
    }
}