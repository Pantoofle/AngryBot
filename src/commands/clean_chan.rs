use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::futures::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::time::Duration;

#[command]
async fn clean_chan(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let reaction = args.single::<ReactionType>().ok();

    let mut warnings = format!(
        "============ ATTENTION ===========\nJe vais supprimer TOUS les messages de ce channel",
    );
    if let Some(r) = reaction.clone() {
        warnings += " sans réaction ";
        warnings += &r.to_string();
    };
    warnings +=
        "\nPour confirmer que c'est bien ce que tu veux, réajis avec l'émoji :ok: en moins de 30s";

    let warning_message = msg.reply(&ctx, warnings).await?;

    if let Some(check_reaction) = &warning_message
        .await_reaction(ctx)
        .timeout(Duration::from_secs(30))
        .author_id(msg.author.id)
        .await
    {
        let emoji = &check_reaction.as_inner_ref().emoji;

        if emoji.as_data().as_str() == "🆗" {
            delete_messages(ctx, msg.channel_id, reaction).await;
        }
    } else {
        return Ok(());
    }

    Ok(())
}

async fn delete_messages(ctx: &Context, chan: ChannelId, reaction: Option<ReactionType>) {
    let mut messages = chan.messages_iter(&ctx).boxed();

    while let Some(message_result) = messages.next().await {
        match message_result {
            Err(_) => continue,
            Ok(m) => {
                if let Some(reac) = reaction.clone() {
                    if m.reactions
                        .iter()
                        .find(|r| r.reaction_type == reac)
                        .is_some()
                    {
                        continue;
                    }
                }
                let _res = m.delete(&ctx.http).await;
            }
        }
    }
}
