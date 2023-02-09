use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use chrono::offset::Local;
use chrono::DateTime;
use humantime::format_duration;

// The last poop joke date
type LastJoke = HashMap<String, Option<DateTime<Local>>>;
pub struct LastJokeWrap;
impl TypeMapKey for LastJokeWrap {
    type Value = Arc<RwLock<LastJoke>>;
}

#[command]
async fn monitor_gif(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let joke_lock = {
        let data = ctx.data.read().await;
        data.get::<LastJokeWrap>()
            .expect("Did not find LastJoke")
            .clone()
    };
    {
        let mut jokes = joke_lock.write().await;
        let link = args.single::<String>().unwrap();
        jokes.insert(link.to_string(), None);
        msg.reply(&ctx, format!("Now monitoring {}", link)).await?;
    }
    Ok(())
}

pub async fn gif_timer(ctx: &Context, msg: &Message) {
    let joke_lock = {
        let data = ctx.data.read().await;
        data.get::<LastJokeWrap>()
            .expect("Did not find LastJoke")
            .clone()
    };
    {
        let mut jokes = joke_lock.write().await;
        let res = jokes
            .iter()
            .find(|(key, _)| msg.content.contains(key.as_str()))
            .to_owned();
        let update = match res {
            Some((key, old_time)) => {
                if let Some(time) = old_time {
                    let delta = Local::now() - *time;
                    let _ = msg
                        .reply(
                            &ctx,
                            format!(
                                "La dernière utilisation de ce gif date de {}",
                                format_duration(delta.to_std().expect("Could not parse date"))
                                    .to_string()
                            ),
                        )
                        .await;
                }
                Some(key.to_owned())
            }
            None => None,
        };
        if let Some(key) = update {
            jokes.insert(key, Some(Local::now()));
        }
    }
}
