mod commands;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

use dotenvy::dotenv;
use sqlx::sqlite::SqlitePool;
use std::env;
use std::sync::Arc;

use crate::commands::angry::*;
use crate::commands::auto_reacts::*;
use crate::commands::gif_timer::*;

#[group]
#[commands(angry, monitor_gif)]
struct General;

struct DBPool;
impl TypeMapKey for DBPool {
    type Value = Arc<SqlitePool>;
}

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        emoji_react(&ctx, &msg).await;
        gif_timer(&ctx, &msg).await;
    }
}

// The main loop
#[tokio::main]
async fn main() {
    // Load env variables
    dotenv().expect(".env file not found");

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // First, build the framework
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "!"
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES;

    // Create a client for the bot, and add the Message handler
    let mut client = Client::builder(token, intents)
        .event_handler(Bot)
        .framework(framework)
        .await
        .expect("Error creating client");

    // Prepare the database connection
    let database_path = env::var("DATABASE_URL").expect("Expected a path to the database");

    let pool = SqlitePool::connect(database_path.as_str())
        .await
        .expect("Could not connect to the DataBase");
    {
        let mut data = client.data.write().await;
        data.insert::<DBPool>(Arc::new(pool));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
