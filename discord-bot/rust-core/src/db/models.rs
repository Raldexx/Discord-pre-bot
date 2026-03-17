use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildConfig {
    pub guild_id: i64,
    pub auto_role_id: Option<i64>,
    pub mod_role_id: Option<i64>,
    pub log_channel_id: Option<i64>,
    pub welcome_channel_id: Option<i64>,
    pub verification_channel_id: Option<i64>,
    pub roles_channel_id: Option<i64>,
    pub verification_enabled: bool,
    pub spam_protection: bool,
    pub raid_protection: bool,
    pub link_filter: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Warning {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub reason: String,
    pub moderator_id: Option<i64>,
    pub created_at: chrono::NaiveDateTime,
}
