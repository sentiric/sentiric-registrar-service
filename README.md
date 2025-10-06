# 📞 Sentiric Registrar Service

[![Status](https://img.shields.io/badge/status-vision-lightgrey.svg)]()
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)]()
[![Protocol](https://img.shields.io/badge/protocol-gRPC_&_Redis-green.svg)]()

**Sentiric Registrar Service**, Sentiric platformu için merkezi **SIP Kayıt Sunucusu** görevi görür. SIP uç noktalarının (`SIP Phone`, `Softphone` vb.) kaydını kabul eder, kimlik doğrulamasını yapar ve Contact URI (IP:Port) bilgilerini yüksek hızlı erişim için Redis'te tutar.

Bu servis, gelen SIP REGISTER isteklerinin işlenmesi, kimlik doğrulama (SIP Digest) ve anlık adres defteri yönetimi konularında uzmandır.

## 🎯 Temel Sorumluluklar

1.  **SIP Digest Kimlik Doğrulama:** Gelen `REGISTER` isteklerindeki `Authorization` başlığını işler ve `sentiric-user-service` aracılığıyla kimlik bilgilerini doğrular.
2.  **Kayıt (Registration) Yönetimi:** Başarılı kimlik doğrulamadan sonra, SIP AOR (Address of Record) ve Contact URI bilgisini TTL (Expires) değeriyle birlikte Redis'te saklar.
3.  **Adres Arama (Lookup):** `sentiric-b2bua-service` gibi iç servisler tarafından, aranan bir SIP AOR'una karşılık gelen güncel Contact URI'sini bulmak için kullanılır.

## 🛠️ Teknoloji Yığını

*   **Dil:** Rust (Yüksek eşzamanlılık ve protokol işleme için)
*   **Servisler Arası İletişim:** gRPC (Tonic)
*   **Veritabanı:** Redis (Hızlı kayıt ve TTL yönetimi için)
*   **Kimlik Doğrulama Kaynağı:** `sentiric-user-service` (gRPC istemcisi)

## 🔌 API Etkileşimleri

*   **Gelen (Sunucu):**
    *   `sentiric-proxy-service` (gRPC): Kayıt isteklerini işlemek için SIP mesajlarını alır.
    *   İç Servisler (gRPC): `LookupContact` (giden çağrı yönlendirmeleri için)
*   **Giden (İstemci):**
    *   `sentiric-user-service` (gRPC): SIP kimlik bilgilerini (`HA1 Hash`) almak için.
    *   `Redis`: Kayıt verilerini okuma/yazma/silme.

---
## 🏛️ Anayasal Konum

Bu servis, [Sentiric Anayasası'nın](https://github.com/sentiric/sentiric-governance) **Core Logic Layer**'ında yer alan yeni SIP Protokol Yönetimi bileşenidir.