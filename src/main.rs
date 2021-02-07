use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::{
    client::{Client, Context, EventHandler},
    model::id::ChannelId,
};
// use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{async_trait, framework::standard::Args};
use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
    model::id::EmojiId,
};

use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fs,
    sync::Arc,
    time::Duration,
};

struct EmojiMapper;

impl TypeMapKey for EmojiMapper {
    type Value = Arc<RwLock<HashMap<ChannelId, HashSet<ReactionType>>>>;
}

#[group]
#[commands(angry, auto_react)]
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
            "le-manoir-d-alban" => Some(ReactionType::Custom {
                animated: false,
                id: EmojiId(779432272885186590),
                name: Some(String::from(":cthulhu:")),
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

    // Populating the initial dict of emojis
    // Todo Read/Write to an external file to keep config during reboots
    {
        let mut data = client.data.write().await;
        data.insert::<EmojiMapper>(Arc::new(RwLock::new(HashMap::default())));
    }

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

async fn add_react(ctx: &Context, channel: ChannelId, emoji: ReactionType) {
    let emoji_lock = {
        let data = ctx.data.write().await;
        data.get::<EmojiMapper>()
            .expect("Did not find EmojiMapper")
            .clone()
    };

    {
        let mut mapper = emoji_lock.write().await;
        let reactions = mapper.entry(channel).or_insert_with(HashSet::default);
        reactions.insert(emoji);
    }
}
async fn del_react(ctx: &Context, channel: ChannelId, emoji: ReactionType) {
    let emoji_lock = {
        let data = ctx.data.write().await;
        data.get::<EmojiMapper>()
            .expect("Did not find EmojiMapper")
            .clone()
    };

    {
        let mut mapper = emoji_lock.write().await;
        let reactions = mapper.entry(channel).or_insert_with(HashSet::default);
        reactions.remove(&emoji);
    }
}
async fn list_react(ctx: &Context, msg: &Message, channel: ChannelId) {
    let emoji_lock = {
        let data = ctx.data.read().await;
        data.get::<EmojiMapper>()
            .expect("Did not find EmojiMapper")
            .clone()
    };

    {
        let mapper = emoji_lock.read().await;
        if let Some(reactions) = mapper.get(&channel) {
            for reac in reactions {
                let _ = msg.react(&ctx, reac.clone()).await;
            }
        }
    }
}

#[command]
async fn auto_react(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command = args.single::<String>()?;
    let channel = msg.mention_channels.get(0);

    if let Some(chan) = channel {
        match command.as_str() {
            "add" => {
                let message = msg
                    .reply(
                        &ctx,
                        "React à ce message avec l'emoji que tu veux ajouter au channel... Je te laisse 30s",
                    )
                    .await.unwrap();

                if let Some(reaction) = &message
                    .await_reaction(&ctx)
                    .timeout(Duration::from_secs(30))
                    .author_id(msg.author.id)
                    .await
                {
                    add_react(&ctx, chan.id, reaction.as_inner_ref().emoji.clone()).await;
                    let _ = msg
                        .reply(&ctx, "J'ai bien ajouté l'émoji automatique !")
                        .await;
                } else {
                    let _ = message.delete(&ctx).await;
                }
            }
            "del" => {
                let message = msg
                    .reply(
                        &ctx,
                        "React à ce message avec l'emoji que tu veux supprimer du channel... Je te laisse 30s",
                    )
                    .await.unwrap();

                if let Some(reaction) = &message
                    .await_reaction(&ctx)
                    .timeout(Duration::from_secs(30))
                    .author_id(msg.author.id)
                    .await
                {
                    del_react(&ctx, chan.id, reaction.as_inner_ref().emoji.clone()).await;
                    let _ = msg
                        .reply(&ctx, "J'ai bien supprimé l'émoji automatique !")
                        .await;
                } else {
                    let _ = message.delete(&ctx).await;
                }
            }
            "list" => list_react(&ctx, &msg, chan.id).await,
            _ => (),
        }
    }

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
