pub mod config;
pub mod moderation;
pub mod roles;
pub mod setup;
pub mod info;

use crate::{Data, Error};

pub fn all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        setup::setup(),
        config::config(),
        moderation::warn(),
        moderation::timeout(),
        moderation::ban(),
        moderation::unban(),
        moderation::clear(),
        moderation::lockdown(),
        roles::rolpanel(),
        info::botinfo(),
        info::sunucuinfo(),
        info::uyarilar(),
    ]
}
