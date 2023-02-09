mod commands;

use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;
// use serenity::model::prelude::*;
use serenity::async_trait;
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::prelude::*;

use dotenvy::dotenv;
use sqlx::postgres::PgPool;
use std::env;

use crate::commands::angry::*;
use crate::commands::auto_reacts::*;
use crate::commands::gif_timer::*;

#[group]
#[commands(angry, auto_react, monitor_gif)]
struct General;

struct Bot {
    database: PgPool,
}

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

    let database_path = env::var("DB_PATH").expect("Expected a path to the database");

    let database = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_path)
        .await
        .expect("Could not connect to database");

    let bot = Bot { database };

    // Create a client for the bot, and add the Message handler
    let mut client = Client::builder(token, intents)
        .event_handler(bot)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
