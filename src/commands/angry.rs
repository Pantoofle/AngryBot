use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::cmp::min;

// Custom commands
#[command]
async fn angry(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Adds N :raaa: or :surexcite: when using !angry N
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
