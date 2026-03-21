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

// Suspicious patterns
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

    // Check if URL is present
    if !lower.contains("http://") && !lower.contains("https://") && !lower.contains("www.") {
        return false;
    }

    // Suspicious pattern check (before whitelist)
    for pattern in SUSPICIOUS_PATTERNS {
        if lower.contains(pattern) {
            return true;
        }
    }

    // Whitelist check — allow if domain is whitelisted
    for domain in ALLOWED_DOMAINS {
        if lower.contains(domain) {
            return false;
        }
    }

    // URL present but not in whitelist
    true
}
