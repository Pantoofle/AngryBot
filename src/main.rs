use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::{async_trait, framework::standard::Args};
use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
    model::id::EmojiId,
};

use std::{cmp::min, fs};

#[group]
#[commands(angry)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let reaction: Option<ReactionType> = match msg.channel_id.name(&ctx).await.unwrap().as_str()
        {
            "coin-pakontan" => Some(ReactionType::Custom {
                animated: false,
                id: EmojiId(781638510390018089),
                name: Some(String::from(":raaa:")),
            }),
            "coin-mignonitude" => Some(ReactionType::Custom {
                animated: false,
                id: EmojiId(780139422246371338),
                name: Some(String::from(":cute:")),
            }),
            "coin-self-love" => Some(ReactionType::Unicode(String::from("❤️"))),
            "romance-est-du-genre-litteraire" => Some(ReactionType::Unicode(String::from("😏"))),
            "blabla-janekke" => Some(ReactionType::Unicode(String::from("🦦"))),
            "jungle-du-grand-singe" => Some(ReactionType::Unicode(String::from("🦧"))),
            "blabla-juliette-eowyn" => Some(ReactionType::Unicode(String::from("🐨"))),
            _ => None,
        };

        if let Some(emoji) = reaction {
            if msg.react(&ctx, emoji).await.is_err() {
                println!("Failed to react");
            };
        }
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = fs::read_to_string("DISCORD_TOKEN").expect("Error when reading the token");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn angry(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (n, emoji) = match args.single::<i64>()? {
        x if x > 0 => (min(x, 50), "<:raaa:781638510390018089>"),
        x if x < 0 => (min(-x, 50), "<:surexcite:781248874253516823>"),
        0 => (1, "<:squint:779480843310989342>"),
        _ => return Err("Error parsing the number".into()),
    };

    let content: String = (0..n).map(|_| emoji).collect::<Vec<_>>().join(" ");

    msg.channel_id.say(ctx, content).await?;

    Ok(())
}

// #[command]
// async fn hearts(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     let emojis = msg.guild_id.emojis

//     let (n, emoji) = match args.single::<i64>()? {
//         x if x > 0 => (min(x, 50), "<:raaa:781638510390018089>"),
//         x if x < 0 => (min(-x, 50), "<:surexcite:781248874253516823>"),
//         0 => (1, "<:squint:779480843310989342>"),
//         _ => return Err("Error parsing the number".into()),
//     };

//     let content: String = (0..n).map(|_| emoji).collect::<Vec<_>>().join(" ");

//     msg.channel_id.say(ctx, content).await?;

//     Ok(())
// }
