mod commands;
mod db;
mod events;
mod protection;

use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use std::env;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug)]
pub struct Data {
    pub db: Arc<sqlx::PgPool>,
    pub redis: Arc<tokio::sync::Mutex<redis::aio::MultiplexedConnection>>,
    pub http_client: reqwest::Client,
    pub ai_service_url: String,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("discord_bot=info".parse().unwrap()),
        )
        .init();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not found");
    let ai_service_url = env::var("AI_SERVICE_URL").unwrap_or("http://localhost:8000".to_string());

    // Database connection
    info!("Connecting to database...");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migration failed");

    // Redis connection
    info!("Connecting to Redis...");
    let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    let data = Data {
        db: Arc::new(pool),
        redis: Arc::new(tokio::sync::Mutex::new(redis_conn)),
        http_client: reqwest::Client::new(),
        ai_service_url,
    };

    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all_commands(),
            event_handler: |ctx, event, framework, data| {
                Box::pin(events::handler(ctx, event, framework, data))
            },
            on_error: |error| {
                Box::pin(async move {
                    error!("Bot error: {}", error);
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Starting bot...");
                poise::builtins::register_globally(ctx, &framework.commands()).await?;
                info!("Slash commands registered");
                Ok(data)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Failed to create client");

    info!("Bot is online!");
    if let Err(e) = client.start().await {
        error!("Bot error: {:?}", e);
    }
}
