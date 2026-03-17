pub mod config;
pub mod info;
pub mod moderation;
pub mod roles;
pub mod setup;

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
        roles::rolepanel(),
        info::botinfo(),
        info::serverinfo(),
        info::warnings(),
    ]
}
