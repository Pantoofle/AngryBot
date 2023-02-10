use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use std::cmp::min;

// Custom commands
#[command]
async fn angry(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let emojis = if let Some(guild) = msg.guild_id {
        guild.emojis(&ctx).await?
    } else {
        vec![]
    };

    // Adds N :raaa: or :surexcite: when using !angry N
    let (n, emoji) = match args.single::<i64>()? {
        x if x > 0 => (min(x, 50), "raaa"),
        x if x < 0 => (min(-x, 50), "surexcite"),
        0 => (1, "squint"),
        _ => return Err("Error parsing the number".into()),
    };

    let emoji = emojis
        .iter()
        .find(|e| e.name.as_str() == emoji)
        .ok_or("Emoji not found")?;

    let mut message = MessageBuilder::new();

    for _ in 0..n {
        message.emoji(&emoji);
        message.push(" ");
    }
    let content = message.build();

    msg.channel_id.say(ctx, content).await?;

    Ok(())
}
