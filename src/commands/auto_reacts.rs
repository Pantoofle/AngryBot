use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::ReactionType;
use serenity::model::id::ChannelId;
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::time::Duration;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// The object storing the Mapping : channel -> set of reactions to add to each message in this channel
struct MapWrap;
impl TypeMapKey for MapWrap {
    type Value = Arc<RwLock<HashMap<ChannelId, HashSet<ReactionType>>>>;
}

pub async fn emoji_react(ctx: &Context, msg: &Message) {
    // When receiving a message, get the emoji map
    let emoji_lock = {
        let data = ctx.data.read().await;
        data.get::<MapWrap>()
            .expect("Did not find EmojiMapper")
            .clone()
    };

    {
        let mapper = emoji_lock.read().await;
        // If we have a set of emojis for this channel, react with each emoji
        if let Some(reactions) = mapper.get(&msg.channel_id) {
            for reac in reactions {
                let _ = msg.react(&ctx, reac.clone()).await;
            }
        }
    }
}

// Adds a new emoji to add as reaction to each message in the channel
async fn add_react(ctx: &Context, channel: ChannelId, emoji: ReactionType) {
    let emoji_lock = {
        let data = ctx.data.write().await;
        data.get::<MapWrap>()
            .expect("Did not find EmojiMapper")
            .clone()
    };

    {
        let mut mapper = emoji_lock.write().await;
        let reactions = mapper.entry(channel).or_insert_with(HashSet::default);
        reactions.insert(emoji);
    }
    dump_react(ctx).await;
}

// Deletes an emoji from the reaction set
async fn del_react(ctx: &Context, channel: ChannelId, emoji: ReactionType) {
    let emoji_lock = {
        let data = ctx.data.write().await;
        data.get::<MapWrap>()
            .expect("Did not find EmojiMapper")
            .clone()
    };

    {
        let mut mapper = emoji_lock.write().await;
        let reactions = mapper.entry(channel).or_insert_with(HashSet::default);
        reactions.remove(&emoji);
    }
    dump_react(ctx).await;
}

// Lists the emojis for this channel
async fn list_react(ctx: &Context, msg: &Message, channel: ChannelId) {
    let emoji_lock = {
        let data = ctx.data.read().await;
        data.get::<MapWrap>()
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

// Writes the mapping ChannelId -> Emoji set to a .yaml file, to remember even after reboot
async fn dump_react(ctx: &Context) {
    let emoji_lock = {
        let data = ctx.data.read().await;
        data.get::<MapWrap>()
            .expect("Did not find EmojiMapper")
            .clone()
    };

    {
        let mapper = emoji_lock.read().await;
        let f = std::fs::File::create("auto_reacts.yaml").unwrap();
        if let Err(err) = serde_yaml::to_writer(f, &mapper.clone()) {
            println!("Error dumping file : {}", err);
        };
    }
}

// Command to control auto emoji reactions
#[command]
async fn auto_react(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command = args.single::<String>()?;
    let channel = args.single::<ChannelId>();

    if let Ok(chan) = channel {
        match command.as_str().trim() {
            // Adds a new emoji to the channel
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
                    add_react(&ctx, chan, reaction.as_inner_ref().emoji.clone()).await;
                    let _ = msg
                        .reply(&ctx, "J'ai bien ajouté l'émoji automatique !")
                        .await;
                } else {
                    let _ = message.delete(&ctx).await;
                }
            }
            // Delets an emoji from the react set
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
                    del_react(&ctx, chan, reaction.as_inner_ref().emoji.clone()).await;
                    let _ = msg
                        .reply(&ctx, "J'ai bien supprimé l'émoji automatique !")
                        .await;
                } else {
                    let _ = message.delete(&ctx).await;
                }
            }
            // Lists the active emojis
            "list" => list_react(&ctx, &msg, chan).await,
            _ => (),
        }
    }

    Ok(())
}
