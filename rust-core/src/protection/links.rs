const ALLOWED_DOMAINS: &[&str] = &[
    "discord.com", "discord.gg", "youtube.com", "youtu.be",
    "twitter.com", "x.com", "github.com", "twitch.tv",
    "imgur.com", "tenor.com", "giphy.com",
];

const SUSPICIOUS_PATTERNS: &[&str] = &[
    "free nitro", "freenitr", "discord.gift", "steamcommunity.ru",
    "bit.ly", "tinyurl",
];

pub fn has_unauthorized_link(content: &str) -> bool {
    let lower = content.to_lowercase();
    if !lower.contains("http://") && !lower.contains("https://") && !lower.contains("www.") {
        return false;
    }
    for pattern in SUSPICIOUS_PATTERNS {
        if lower.contains(pattern) {
            return true;
        }
    }
    for domain in ALLOWED_DOMAINS {
        if lower.contains(domain) {
            return false;
        }
    }
    true
}
