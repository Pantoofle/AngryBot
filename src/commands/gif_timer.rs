use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use sqlx::types::chrono::{Local, NaiveDateTime};

use humantime::format_duration;

use crate::{AngryResult, DBPool};

struct WatchEntry {
    link: String,
    last_seen: NaiveDateTime,
}

#[command]
async fn monitor_gif(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let link = args.single::<String>()?;

    let mut db_connection = {
        let data = ctx.data.read().await;
        let pool = data
            .get::<DBPool>()
            .expect("Did not find the connection pool");
        pool.acquire().await.expect("Could not connect to the DB")
    };

    let now = Local::now().naive_local();

    let _res = sqlx::query!(
        r#"
            INSERT INTO monitor_gif(link, last_seen)
            VALUES (?, ?)
        "#,
        link,
        now
    )
    .execute(&mut db_connection)
    .await?;

    msg.reply(
        ctx,
        format!(
            "Je vais maintenant monitorer les messages contenant {}",
            link
        ),
    )
    .await?;

    Ok(())
}

pub async fn gif_timer(ctx: &Context, msg: &Message) -> AngryResult {
    let mut db_connection = {
        let data = ctx.data.read().await;
        let pool = data
            .get::<DBPool>()
            .expect("Did not find the connection pool");
        pool.acquire().await.expect("Could not connect to the DB")
    };

    let entries = sqlx::query_as!(
        WatchEntry,
        r#"
            SELECT link, last_seen
            FROM monitor_gif
        "#
    )
    .fetch_all(&mut db_connection)
    .await
    .or(Err("Error querying the database"))?;

    let matching = entries
        .iter()
        .find(|e| msg.content.contains(e.link.as_str()));

    let matching = match matching {
        None => return Ok(()),
        Some(e) => e,
    };

    let now = Local::now().naive_local();
    let delta = now - matching.last_seen;

    msg.reply(
        ctx,
        format!(
            "Ça fait {} qu'on avait pas vu ça !",
            format_duration(delta.to_std().or(Err("Error parsing the timedelta "))?)
        ),
    )
    .await
    .or(Err("Error sending the reply message"))?;

    let _res = sqlx::query!(
        r#"
            UPDATE monitor_gif
            SET last_seen = ?
            WHERE link = ?
        "#,
        now,
        matching.link
    )
    .execute(&mut db_connection)
    .await
    .or(Err("Error writing the new time to the database"))?;

    Ok(())
}
