use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::ReactionType;
use serenity::model::id::ChannelId;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::AngryResult;
use crate::DBPool;
use sqlx::sqlite::SqliteConnection;

//
// Function to auto-react to messages
//
pub async fn emoji_react(ctx: &Context, msg: &Message) -> AngryResult {
    let mut db_connection = {
        let data = ctx.data.read().await;
        let pool = data
            .get::<DBPool>()
            .expect("Did not find the connection pool");
        pool.acquire().await.expect("Could not connect to the DB")
    };

    let reactions = get_reactions(&mut db_connection, msg.channel_id)
        .await
        .or(Err("Error when fetching the reaction list"))?;

    for reac in reactions {
        msg.react(&ctx, reac)
            .await
            .or(Err("Error when reacting to the message"))?;
    }

    Ok(())
}

//
// Commands to control the emojis to auto-react
//

#[group]
#[prefixes("auto_react")]
#[commands(add_react, del_react, list_react)]
struct EmojiReact;

// Adds a new emoji to add as reaction to each message in the channel
#[command]
#[aliases("add")]
async fn add_react(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel = args.single::<ChannelId>()?;
    let channel = *channel.as_u64() as i64;

    let reaction = args.single::<ReactionType>()?;
    let reaction = reaction.to_string();

    let mut db_connection = {
        let data = ctx.data.write().await;
        let pool = data
            .get::<DBPool>()
            .expect("Did not find the connection pool");
        pool.acquire().await.expect("Could not connect to the DB")
    };

    let res = sqlx::query!(
        r#"
            INSERT INTO emoji_react(channel, reaction)
            VALUES (?, ?)
        "#,
        channel,
        reaction
    )
    .execute(&mut db_connection)
    .await?;
    msg.reply(
        ctx,
        format!("J'ai ajouté {} réaction automatique", res.rows_affected()),
    )
    .await?;

    Ok(())
}

// Deletes an emoji from the reaction set
#[command]
#[aliases("del")]
async fn del_react(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel = args.single::<ChannelId>()?;
    let channel = *channel.as_u64() as i64;

    let reaction = args.single::<ReactionType>()?;
    let reaction = reaction.to_string();

    let mut db_connection = {
        let data = ctx.data.write().await;
        let pool = data
            .get::<DBPool>()
            .expect("Did not find the connection pool");
        pool.acquire().await.expect("Could not connect to the DB")
    };

    let res = sqlx::query!(
        r#"
            DELETE FROM emoji_react
            WHERE channel = ? AND reaction = ?
        "#,
        channel,
        reaction
    )
    .execute(&mut db_connection)
    .await?;

    msg.reply(
        ctx,
        format!("J'ai supprimé {} réaction automatique", res.rows_affected()),
    )
    .await?;
    Ok(())
}

// Lists the emojis for this channel
async fn get_reactions(
    db: &mut SqliteConnection,
    channel: ChannelId,
) -> Result<Vec<ReactionType>, sqlx::Error> {
    let chan = *channel.as_u64() as i64;

    let reactions = sqlx::query!(
        r#"
            SELECT reaction 
            FROM emoji_react
            WHERE channel = ?
        "#,
        chan,
    )
    .fetch_all(db)
    .await?;

    Ok(reactions
        .iter()
        .map(|entry| {
            entry
                .reaction
                .as_ref()
                .map(move |r| ReactionType::try_from(r.to_owned()).ok())
        })
        .flatten()
        .flatten()
        .collect::<Vec<ReactionType>>())
}

#[command]
#[aliases("list")]
async fn list_react(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel = args.single::<ChannelId>()?;

    let mut db_connection = {
        let data = ctx.data.write().await;
        let pool = data
            .get::<DBPool>()
            .expect("Did not find the connection pool");
        pool.acquire().await.expect("Could not connect to the DB")
    };

    let reactions = get_reactions(&mut db_connection, channel)
        .await
        .expect("Error when fetching the reaction list");

    for reac in reactions {
        msg.react(&ctx, reac).await?;
    }
    Ok(())
}
