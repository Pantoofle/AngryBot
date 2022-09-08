use serenity::model::channel::{GuildChannel, Message};
use serenity::model::channel::ReactionType;
use serenity::{
    client::{Client, Context, EventHandler},
    model::id::{ChannelId, UserId},
};
// use serenity::model::prelude::*;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::prelude::*;
use serenity::{async_trait, framework::standard::Args};

use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fs,
    sync::Arc,
    time::Duration,
};
use chrono::Date;

use chrono::naive::NaiveDate;
use chrono::DateTime;
use chrono::offset::Local;
use serenity::model::id::GuildId;


// Custom types to store in Discord data map
// Discord data field maps a Type to an object. So we define new types, to store objets in front

// The object storing the Mapping : channel -> set of reactions to add to each message in this channel
type EmojiMapper = HashMap<ChannelId, HashSet<ReactionType>>;
struct MapWrap;
impl TypeMapKey for MapWrap {
    type Value = Arc<RwLock<HashMap<ChannelId, HashSet<ReactionType>>>>;
}

// The map of birthdays
type BirthdayMapper = HashMap<NaiveDate, HashSet<UserId>>;
struct BirthdayWrap;
impl TypeMapKey for BirthdayWrap {
    type Value = Arc<RwLock<BirthdayMapper>>;
}
// The next date when to wish a birthday
type NextBirthday = NaiveDate;
struct NextBirthdayWrap;
impl TypeMapKey for NextBirthdayWrap {
    type Value = Arc<RwLock<NextBirthday>>;
}

// The last poop joke date
type LastJoke = HashMap<String, Option<DateTime<Local>>>;
struct LastJokeWrap;
impl TypeMapKey for LastJokeWrap{
    type Value = Arc<RwLock<LastJoke>>;
}

#[group]
#[commands(angry, auto_react, monitor_gif)]
struct General;

async fn wish_birthday(ctx: &Context, guild: GuildId){
    // Check if we already wished happy birth day today
    let next_birthday_lock = {
        let data = ctx.data.read().await;
        data.get::<NextBirthdayWrap>()
            .expect("Did not find the next BD")
            .clone()
    };
    let mut next_birthday = next_birthday_lock.write().await;

    // If we did not wish happy BD today, get the BD map
    if Local::today().naive_local() == *next_birthday {
        let birthday_map_lock = {
            let data = ctx.data.read().await;
            data.get::<BirthdayWrap>()
                .expect("Did not find the next BD")
                .clone()
        };
        let birthday_map = birthday_map_lock.read().await;
        // Next check, tomorrow
        if let Some(target_people) = birthday_map.get(&next_birthday) {
            if target_people.len() != 0 {
                let noms: String = target_people.iter()
                    .map(|&id| id.mention())
                    .collect::<Vec<_>>()
                    .join(", ");

                let message: String = "Aujourd'hui c'est l'anniversaire de ".to_owned() + &noms + &" !!!".to_owned();

                find_birthday_chan(ctx, guild).await
                    .expect("Could not find birthday channel")
                    .say(ctx, message).await.unwrap();
            }
            *next_birthday = Local::today().naive_local().succ();
        }
    }


}

async fn find_birthday_chan(ctx: &Context, id: GuildId) -> Option<ChannelId>{
    let chans: HashMap<ChannelId, GuildChannel> =
            id.channels(ctx)
            .await
            .expect("Could not get channels");

    chans.values().find(|&chan| chan.name.contains("anniversaires"))
        .map_or(None, |c| Some(c.id))
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
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

        let joke_lock = {
            let data = ctx.data.read().await;
            data.get::<LastJokeWrap>().expect("Did not find LastJoke").clone()
        };
        {
            let mut jokes = joke_lock.write().await;
            let res = jokes.iter().find(|(key, _)| msg.content.contains(key.as_str())).to_owned();
            let update = match res{
                Some((key, old_time)) => {

                    if let Some(time) = old_time {
                        let delta = Local::now() - *time;
                        let _ = msg.reply(&ctx, format!("La dernière utilisation de ce gif date de {}", delta.to_string())).await;
                    }
                    Some(key.to_owned())
                }
                None => {None}
            };
            if let Some(key) = update{
                jokes.insert(key, Some(Local::now()));
            }
        }

        // wish_birthday(&ctx, msg.guild_id.expect("No guild ID")).await;
    }
}


// The main loop
#[tokio::main]
async fn main() {
    // First, build the framework
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "!"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = fs::read_to_string("DISCORD_TOKEN").expect("Error when reading the token");

    // Create a client for the bot, and add the Message handler
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // Populating the initial data for the client :

    // The emojis
    // Loading the file from serialized .yaml
    let emojis: EmojiMapper = match std::fs::File::open("auto_reacts.yaml") {
        Ok(f) => serde_yaml::from_reader(f).unwrap_or_default(),
        Err(_) => EmojiMapper::default(),
    };

    // The birthdays
    /*
    let birthdays: BirthdayMapper = match std::fs::File::open("birthdays.yaml") {
        Ok(f) => serde_yaml::from_reader(f).unwrap_or_default(),
        Err(_) => BirthdayMapper::default(),
    };

    // The next birthday to check
    let next_birthday: NextBirthday = Local::today().naive_local();
    */
    //Storing the data inside the client.data
    {
        let mut data = client.data.write().await;
        data.insert::<MapWrap>(Arc::new(RwLock::new(emojis.clone())));
        data.insert::<LastJokeWrap>( Arc::new(RwLock::new( LastJoke::default() )));
       // data.insert::<BirthdayWrap>( Arc::new(RwLock::new( birthdays.clone())));
       // data.insert::<NextBirthdayWrap>( Arc::new(RwLock::new(next_birthday.clone())));
    }


    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}


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

#[command]
async fn monitor_gif(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult{
    let joke_lock = {
            let data = ctx.data.read().await;
            data.get::<LastJokeWrap>().expect("Did not find LastJoke").clone()
    };
    {
        let mut jokes = joke_lock.write().await;
        let link = args.single::<String>().unwrap();
        jokes.insert(link.to_string(), None);
        msg.reply(&ctx, format!("Now monitoring {}", link)).await?;
    }
    Ok(())
}