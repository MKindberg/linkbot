#![feature(iter_intersperse)]
use rand::Rng;
use std::collections::HashMap;
use std::{env, fs};
use yaml_rust::YamlLoader;

use serenity::async_trait;
use serenity::model::channel::{Message, ReactionType};
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[derive(Debug, Clone)]
struct Reply {
    message: String,
    percent: i64,
}
#[derive(Debug, Clone)]
struct React {
    emoji: String,
}
#[derive(Debug)]
struct Actions {
    reaction: Option<React>,
    reply: Option<Reply>,
}

impl Actions {
    fn get_message(&self) -> Option<String> {
        let reply = self.reply.clone()?;
        (rand::thread_rng().gen_range(0..=100) < reply.percent).then_some(reply.message)
    }
    fn get_reaction(&self) -> Option<String> {
        let reaction = self.reaction.clone()?;
        Some(reaction.emoji)
    }
}

type ActionMap = HashMap<String, Actions>;

fn parse_actions() -> ActionMap {
    let data = fs::read_to_string(env::var("USER_REPLY_FILE").expect("USER_REPLY_FILE is not set"))
        .expect("Something went wrong reading the file");
    let doc = YamlLoader::load_from_str(&data).unwrap();

    let mut map = ActionMap::new();
    let users = &doc[0]["users"];
    for user in users.as_vec().unwrap() {
        let name = user["name"]
            .as_str()
            .expect("Each user in config must contain a name")
            .to_lowercase();
        let react: Option<React> = user["reaction"].as_str().map(|reaction| React {
            emoji: reaction.to_string(),
        });
        let reply: Option<Reply> = user["reply"].as_str().map(|message| Reply {
            message: message.to_string(),
            percent: user["reply_percent"].as_i64().unwrap_or(100),
        });
        let actions = Actions {
            reaction: react,
            reply,
        };
        map.insert(name, actions);
    }
    map
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Some(actions) = parse_actions().get(&msg.author.name.to_lowercase()) {
            if let Some(s) = actions.get_message() {
                if let Err(why) = msg
                    .channel_id
                    .send_message(&ctx.http, |m| m.content(s).tts(false))
                    .await
                {
                    println!("Error sending message: {:?}", why);
                }
            }
            if let Some(s) = actions.get_reaction() {
                if let Err(why) = msg
                    .react(&ctx.http, ReactionType::try_from(s).unwrap())
                    .await
                {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
