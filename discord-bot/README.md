# 🤖 Discord Guard Bot

**Rust + Python** ile yazılmış, hafif, hızlı ve AI destekli Discord moderasyon botu.

> Küçük sunuculardan büyük topluluklara kadar çalışacak şekilde tasarlanmıştır.

---

## ✨ Özellikler

| Özellik | Açıklama |
|---|---|
| 🛡️ **Spam Koruması** | Token bucket algoritmasıyla hız, mention, caps ve duplicate spam tespiti |
| 🚨 **Raid Koruması** | Kitlesel katılım tespiti, otomatik lockdown |
| 🔗 **Link Filtresi** | Whitelist tabanlı link kontrolü, şüpheli domain engeli |
| 🤖 **AI Moderasyon** | Detoxify ML modeli + Türkçe pattern tabanlı toksisite tespiti |
| 🎭 **Oto Rol** | Katılımda otomatik rol, buton tabanlı rol paneli |
| ✅ **Verification** | Buton doğrulama sistemi |
| 📋 **Loglama** | Üye giriş/çıkış, silinen mesaj, moderasyon aksiyonları |
| ⚙️ **Kolay Kurulum** | `/setup` ile tek komutta yapılandırma |

---

## 🚀 Hızlı Başlangıç

### Gereksinimler

- [Docker](https://docs.docker.com/get-docker/) & Docker Compose
- Discord Bot Token ([buradan al](https://discord.com/developers/applications))

### 1. Repoyu klonla

```bash
git clone https://github.com/your-username/discord-guard-bot.git
cd discord-guard-bot
```

### 2. Environment dosyasını hazırla

```bash
cp .env.example .env
```

`.env` dosyasını aç ve `DISCORD_TOKEN` değerini gir:

```env
DISCORD_TOKEN=your_bot_token_here
```

### 3. Botu başlat

```bash
docker compose up -d
```

İlk başlatmada Docker imajları build edilir (~3-5 dakika). Sonraki başlatmalar çok daha hızlıdır.

Logları izlemek için:
```bash
docker compose logs -f bot
```

### 4. Discord'da yapılandır

Botu sunucuna ekledikten sonra:

```
/setup
```

Komutunu çalıştır. Bot otomatik olarak gerekli kanalları oluşturacak ve temel ayarları yapacaktır.

---

## 📋 Komutlar

### ⚙️ Yönetim
| Komut | Açıklama | Yetki |
|---|---|---|
| `/setup` | Botu otomatik yapılandır | Admin |
| `/config otorol @rol` | Oto rol ayarla | Admin |
| `/config modrolu @rol` | Moderatör rolü ayarla | Admin |
| `/config logkanal #kanal` | Log kanalı ayarla | Admin |
| `/config hosgeldin #kanal` | Hoş geldin kanalı ayarla | Admin |
| `/config spam true/false` | Spam korumasını aç/kapat | Admin |
| `/config raid true/false` | Raid korumasını aç/kapat | Admin |
| `/config linkfiltre true/false` | Link filtresini aç/kapat | Admin |

### 🛡️ Moderasyon
| Komut | Açıklama | Yetki |
|---|---|---|
| `/warn @kullanici neden` | Uyarı ver | Moderatör |
| `/timeout @kullanici dakika` | Sustur | Moderatör |
| `/ban @kullanici neden` | Banla | Ban yetkisi |
| `/unban kullanici_id` | Ban kaldır | Ban yetkisi |
| `/clear adet` | Mesaj temizle (maks 100) | Mesaj yönetimi |
| `/lockdown ac/kapat` | Manuel lockdown | Admin |

### 🎭 Roller
| Komut | Açıklama | Yetki |
|---|---|---|
| `/rolpanel baslik rol1 ...` | Rol seçim paneli oluştur | Rol yönetimi |

### ℹ️ Bilgi
| Komut | Açıklama |
|---|---|
| `/botinfo` | Bot bilgisi |
| `/sunucuinfo` | Sunucu istatistikleri |
| `/uyarilar @kullanici` | Uyarı geçmişi |

---

## 🏗️ Mimari

```
discord-guard-bot/
├── rust-core/          # Ana bot (Serenity + Poise)
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/   # Slash komutlar
│   │   ├── events/     # Discord event handler'ları
│   │   ├── protection/ # Spam, raid, link, AI modülleri
│   │   └── db/         # Veritabanı modelleri
│   └── migrations/     # PostgreSQL migrasyonları
│
├── ai-service/         # Python AI servisi (FastAPI)
│   ├── main.py         # API endpoint'leri
│   └── detector.py     # Toksisite tespiti
│
└── docker-compose.yml  # Tüm servisleri yönetir
```

### Servisler

| Servis | Teknoloji | Görev |
|---|---|---|
| `bot` | Rust / Serenity | Discord gateway, event handling, komutlar |
| `ai-service` | Python / FastAPI | Mesaj analizi, toksisite tespiti |
| `postgres` | PostgreSQL 16 | Kalıcı veri (uyarılar, config, loglar) |
| `redis` | Redis 7 | Hız bazlı cache (spam sayaçları, raid tracker) |

---

## 🤖 AI Moderasyon Nasıl Çalışır?

Bot her mesajı şu sırayla analiz eder:

```
Mesaj geldi
    ↓
1. Ağır tehdit pattern kontrolü  → Anında ban
    ↓
2. Türkçe küfür/hakaret pattern  → Uyarı
    ↓
3. İngilizce offensive pattern   → Uyarı
    ↓
4. Saldırganlık keyword tespiti  → Uyarı
    ↓
5. Detoxify ML model (varsa)     → Skor bazlı karar
```

**3 uyarı → otomatik 30 dakika timeout**

AI servisi çalışmıyorsa bot pattern tabanlı analizle devam eder, hiç durmaz.

---

## ⚡ Performans

- **Bellek:** ~30-50MB (Rust bot) + ~200MB (Python AI modeli)
- **Latency:** Komut yanıt <100ms
- **AI analiz:** Asenkron, botu bloklamaz

---

## 🤝 Katkı

1. Fork et
2. Branch oluştur (`git checkout -b feature/yeni-ozellik`)
3. Commit at (`git commit -m "feat: yeni özellik"`)
4. Push et (`git push origin feature/yeni-ozellik`)
5. Pull Request aç

### Geliştirme ortamı

```bash
# Sadece postgres ve redis'i başlat
docker compose up -d postgres redis

# Rust botu lokal çalıştır
cd rust-core
cargo run

# Python servisini lokal çalıştır
cd ai-service
pip install -r requirements.txt
python main.py
```

---

## 📄 Lisans

MIT — detaylar için [LICENSE](LICENSE) dosyasına bak.

---

## 🙏 Kullanılan Teknolojiler

- [Serenity](https://github.com/serenity-rs/serenity) — Rust Discord kütüphanesi
- [Poise](https://github.com/serenity-rs/poise) — Slash command framework
- [Detoxify](https://github.com/unitaryai/detoxify) — ML toksisite tespiti
- [FastAPI](https://fastapi.tiangolo.com/) — Python web framework
- [Tokio](https://tokio.rs/) — Rust async runtime
