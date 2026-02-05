use std::sync::Arc;
use tokio::sync::Mutex;
use redis::AsyncCommands;
use tracing::{info, debug, warn};
use sentiric_sip_core::utils as sip_utils;

pub type RedisConn = Arc<Mutex<redis::aio::MultiplexedConnection>>;

#[derive(Clone)]
pub struct RegistrationStore {
    redis: RedisConn,
}

impl RegistrationStore {
    pub fn new(redis: RedisConn) -> Self {
        Self { redis }
    }

    /// Redis Anahtarı Oluşturucu (Normalize Edilmiş)
    /// "sip:1001@domain.com" -> "sip_reg:1001"
    fn generate_key(&self, raw_uri: &str) -> String {
        let username = sip_utils::extract_username_from_uri(raw_uri);
        if username.is_empty() {
             warn!("Username çıkarılamadı, raw URI kullanılıyor: {}", raw_uri);
             return format!("sip_reg:{}", raw_uri);
        }
        format!("sip_reg:{}", username)
    }

    pub async fn register_user(&self, sip_uri: &str, contact_uri: &str, expires: i32) -> anyhow::Result<()> {
        let key = self.generate_key(sip_uri);
        let mut conn = self.redis.lock().await;

        if expires <= 0 {
             let _: () = conn.del(&key).await?;
             info!("🗑️ Kayıt silindi (Expires=0): {}", key);
        } else {
            let _: () = conn.set_ex(&key, contact_uri, expires as u64).await?;
            debug!("💾 Kayıt başarılı: {} -> {}", key, contact_uri);
        }
        Ok(())
    }

    pub async fn unregister_user(&self, sip_uri: &str) -> anyhow::Result<()> {
        let key = self.generate_key(sip_uri);
        let mut conn = self.redis.lock().await;
        
        let _: () = conn.del(&key).await?;
        info!("🗑️ Kayıt manuel silindi: {}", key);
        Ok(())
    }

    pub async fn lookup_user(&self, sip_uri: &str) -> Option<String> {
        let key = self.generate_key(sip_uri);
        let mut conn = self.redis.lock().await;
        
        match conn.get::<_, String>(&key).await {
            Ok(contact) => {
                debug!("✅ Bulundu: {} -> {}", key, contact);
                Some(contact)
            },
            Err(_) => {
                debug!("❌ Bulunamadı (Offline): {}", key);
                None
            }
        }
    }
}