# 🤖 Discord Guard Bot

A fast, lightweight, AI-powered Discord moderation bot built with **Rust + Python**.

> Designed to scale from small communities to large servers — and do what MEE6 can't.

---

## ✨ Why Guard Bot?

Most bots catch **words**. Guard Bot catches **context**.

| MEE6 | Guard Bot |
|---|---|
| Keyword matching only | Context-aware AI analysis |
| Static rules | Learns from false positives |
| Misses cipher bypasses (`f*ck`) | Detects cipher attempts |
| No fight detection | Detects escalating tension |
| Expensive premium tier | Core features free |

> **"MEE6 catches words. Guard Bot catches fights before they happen."**

---

## 🚀 Features

| Feature | Description |
|---|---|
| 🛡️ **Spam Protection** | Token bucket algorithm — rate, mention, caps & duplicate spam |
| 🚨 **Raid Protection** | Mass join detection, automatic lockdown |
| 🔗 **Link Filter** | Whitelist-based link control, suspicious domain blocking |
| 🤖 **AI Moderation** | Detoxify ML model + pattern-based toxicity detection |
| 🎭 **Auto Role** | Assign role on join, button-based role panel |
| ✅ **Verification** | One-click button verification system |
| 📋 **Logging** | Member join/leave, deleted messages, moderation actions |
| ⚙️ **Easy Setup** | Configure everything with `/setup` in seconds |

---

## ⚡ Quick Start

### Requirements

- [Docker](https://docs.docker.com/get-docker/) & Docker Compose
- Discord Bot Token ([get one here](https://discord.com/developers/applications))

### 1. Clone the repo

```bash
git clone https://github.com/your-username/discord-guard-bot.git
cd discord-guard-bot
```

### 2. Set up environment

```bash
cp .env.example .env
```

Open `.env` and fill in your token:

```env
DISCORD_TOKEN=your_bot_token_here
```

### 3. Start the bot

```bash
docker compose up -d
```

First run takes ~3-5 minutes (Rust compilation + Python model download).

Watch the logs:
```bash
docker compose logs -f bot
```

### 4. Configure in Discord

```
/setup
```

That's it. The bot will create all necessary channels and apply default settings.

---

## 📋 Commands

### ⚙️ Admin
| Command | Description |
|---|---|
| `/setup` | Auto-configure the bot |
| `/config autorole @role` | Set auto role for new members |
| `/config modrole @role` | Set moderator role |
| `/config logchannel #channel` | Set log channel |
| `/config welcome #channel` | Set welcome channel |
| `/config spam true/false` | Toggle spam protection |
| `/config raid true/false` | Toggle raid protection |
| `/config linkfilter true/false` | Toggle link filter |

### 🛡️ Moderation
| Command | Description | Permission |
|---|---|---|
| `/warn @user reason` | Issue a warning | Moderate Members |
| `/timeout @user minutes` | Mute a user | Moderate Members |
| `/ban @user reason` | Ban a user | Ban Members |
| `/unban user_id` | Remove a ban | Ban Members |
| `/clear amount` | Delete messages (max 100) | Manage Messages |
| `/lockdown on/off` | Manual lockdown | Administrator |

### 🎭 Roles
| Command | Description |
|---|---|
| `/rolepanel title role1 ...` | Create a role selection panel |

### ℹ️ Info
| Command | Description |
|---|---|
| `/botinfo` | Bot information |
| `/serverinfo` | Server statistics |
| `/warnings @user` | View warning history |

---

## 🤖 How AI Moderation Works

Every message goes through this pipeline:

```
Message received
    ↓
1. Severe threat check      → Immediate action
    ↓
2. Self-harm detection      → Immediate action
    ↓
3. Threat pattern check     → Warning
    ↓
4. Offensive / slur check   → Warning
    ↓
5. Aggression keyword scan  → Warning
    ↓
6. Detoxify ML model        → Score-based decision
```

**3 warnings → automatic 30 minute timeout**

If the AI service is unavailable, the bot continues with pattern-based analysis — it never goes down.

---

## 🏗️ Architecture

```
discord-guard-bot/
├── rust-core/          # Core bot (Serenity + Poise)
│   ├── src/
│   │   ├── commands/   # Slash commands
│   │   ├── events/     # Discord event handlers
│   │   ├── protection/ # Spam, raid, link, AI modules
│   │   └── db/         # Database models
│   └── migrations/     # PostgreSQL migrations
│
├── ai-service/         # Python AI service (FastAPI)
│   ├── main.py         # API endpoints
│   └── detector.py     # Toxicity detection engine
│
└── docker-compose.yml  # Manages all services
```

| Service | Tech | Role |
|---|---|---|
| `bot` | Rust / Serenity | Gateway, events, commands |
| `ai-service` | Python / FastAPI | Message analysis, toxicity scoring |
| `postgres` | PostgreSQL 16 | Persistent data (warnings, config, logs) |
| `redis` | Redis 7 | Speed-critical cache (spam counters, raid tracker) |

---

## ⚡ Performance

- **Memory:** ~30-50MB (Rust bot) + ~200MB (Python AI model)
- **Latency:** Command response <100ms
- **AI analysis:** Async — never blocks the bot

---

## 🤝 Contributing

1. Fork the repo
2. Create a branch (`git checkout -b feature/my-feature`)
3. Commit (`git commit -m "feat: my feature"`)
4. Push (`git push origin feature/my-feature`)
5. Open a Pull Request

---

## 📄 License

MIT — see [LICENSE](LICENSE) for details.

---

## 🙏 Built With

- [Serenity](https://github.com/serenity-rs/serenity) — Rust Discord library
- [Poise](https://github.com/serenity-rs/poise) — Slash command framework
- [Detoxify](https://github.com/unitaryai/detoxify) — ML toxicity detection
- [FastAPI](https://fastapi.tiangolo.com/) — Python web framework
- [Tokio](https://tokio.rs/) — Rust async runtime
