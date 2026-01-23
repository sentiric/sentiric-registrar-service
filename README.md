# 📞 Sentiric Registrar Service

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)]()
[![Protocol](https://img.shields.io/badge/protocol-gRPC_&_Redis-green.svg)]()

**Sentiric Registrar Service**, Sentiric platformu için merkezi **SIP Kayıt Sunucusu** görevi görür. SIP uç noktalarının (`SIP Phone`, `Softphone` vb.) kaydını kabul eder, kimlik doğrulamasını yapar ve Contact URI (IP:Port) bilgilerini yüksek hızlı erişim için Redis'te tutar.

Bu servis, platformun "Kim Nerede?" sorusuna cevap veren tek otoritedir.

## 🎯 Temel Sorumluluklar

1.  **SIP Digest Kimlik Doğrulama:** Gelen `REGISTER` isteklerindeki `Authorization` başlığını işler ve `sentiric-user-service` aracılığıyla kimlik bilgilerini doğrular.
2.  **Kayıt (Registration) Yönetimi:** Başarılı kimlik doğrulamadan sonra, SIP AOR (Address of Record) ve Contact URI bilgisini TTL (Expires) değeriyle birlikte Redis'te saklar.
3.  **Adres Arama (Lookup Authority):**
    *   `sentiric-proxy-service`: Gelen çağrının bir iç aboneye gidip gitmediğini anlamak için burayı sorgular.
    *   `sentiric-b2bua-service`: Çağrı transferleri sırasında hedef aboneyi bulmak için burayı sorgular.

## 🛠️ Teknoloji Yığını

*   **Dil:** Rust
*   **Servisler Arası İletişim:** gRPC (Tonic)
*   **Veritabanı:** Redis
*   **Kimlik Kaynağı:** `sentiric-user-service`

## 🔌 API Etkileşimleri

*   **Gelen (Sunucu):**
    *   `sentiric-proxy-service` (gRPC): Kayıt (`Register`) ve Yönlendirme Sorgusu (`LookupContact`).
    *   `sentiric-b2bua-service` (gRPC): Hedef abone sorgusu (`LookupContact`).
*   **Giden (İstemci):**
    *   `sentiric-user-service` (gRPC): SIP kimlik bilgilerini (`HA1 Hash`) almak için.
    *   `Redis`: Kayıt verilerini yönetmek için.

---
## 🏛️ Anayasal Konum

Bu servis, [Sentiric Anayasası'nın](https://github.com/sentiric/sentiric-governance) **Core Logic Layer**'ında yer alan yeni SIP Protokol Yönetimi bileşenidir.

---
