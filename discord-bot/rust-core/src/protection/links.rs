// Whitelist - bu domainlere izin ver
const ALLOWED_DOMAINS: &[&str] = &[
    "discord.com",
    "discord.gg",
    "youtube.com",
    "youtu.be",
    "twitter.com",
    "x.com",
    "github.com",
    "twitch.tv",
    "imgur.com",
    "tenor.com",
    "giphy.com",
];

// Şüpheli pattern'lar
const SUSPICIOUS_PATTERNS: &[&str] = &[
    "free nitro",
    "freenitr",
    "discord.gift",
    "steamcommunity.ru",
    "bit.ly",
    "tinyurl",
];

pub fn has_unauthorized_link(content: &str) -> bool {
    let lower = content.to_lowercase();

    // URL var mı kontrol et
    if !lower.contains("http://") && !lower.contains("https://") && !lower.contains("www.") {
        return false;
    }

    // Şüpheli pattern kontrolü (whitelist'ten önce)
    for pattern in SUSPICIOUS_PATTERNS {
        if lower.contains(pattern) {
            return true;
        }
    }

    // Whitelist kontrolü - izin verilen domain varsa geç
    for domain in ALLOWED_DOMAINS {
        if lower.contains(domain) {
            return false;
        }
    }

    // URL var ama whitelist'te değil
    true
}
