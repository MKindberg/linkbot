#![feature(iter_intersperse)]
use std::collections::HashMap;
use std::{env, fs};
use rand::seq::SliceRandom;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

type ReplyMap = HashMap<String, Vec<String>>;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!set") {
            set_user_reply(&msg.content["!set ".len()..]);
        } else if msg.content.starts_with("!unset") {
            let mut replies = read_user_replies();
            replies.remove(&msg.content["!unset ".len()..].trim().to_lowercase());
            write_user_replies(replies);
        } else if msg.content.starts_with("!add") {
            add_user_reply(&msg.content["!add ".len()..]);
        } else if let Some(s) = get_user_reply(&msg.author.name) {
            if let Err(why) = msg
                .channel_id
                .send_message(&ctx.http, |m| m.content(s).tts(false))
                .await
            {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn split_input(input: &str) -> Vec<&str> {
    if input.contains('|') {
        return input.split('|').collect();
    }

    let idx = input.find(' ').unwrap_or(0);
    if idx != 0 {
        return vec![&input[..idx], &input[idx + 1..]];
    }

    vec![]
}

fn set_user_reply(input: &str) {
    let parts = split_input(input);
    if parts.is_empty() {
        return;
    }

    let mut replies = read_user_replies();
    replies.insert(
        parts[0].trim().to_lowercase(),
        parts.iter().skip(1).cloned().map(String::from).collect(),
    );

    write_user_replies(replies);
}

fn add_user_reply(input: &str) {
    let parts = split_input(input);
    if parts.is_empty() {
        return;
    }

    let mut replies = read_user_replies();

    let user = parts[0].to_lowercase();
    let mut reply: Vec<String> = parts.iter().skip(1).cloned().map(String::from).collect();
    if let Some(v) = replies.get_mut(&user) {
        v.append(&mut reply);
    } else {
        replies.insert(
            user,
            reply,
        );
    }

    write_user_replies(replies);
}

fn write_user_replies(replies: ReplyMap) {
    fs::write(
        env::var("USER_REPLY_FILE").expect("USER_REPLY_FILE is not set"),
        replies
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}|{}\n",
                    k.to_lowercase(),
                    v.iter()
                        .cloned()
                        .intersperse("|".to_string())
                        .collect::<String>()
                )
            })
            .collect::<String>(),
    )
    .expect("Failed to write file");
}

fn read_user_replies() -> ReplyMap {
    fs::read_to_string(env::var("USER_REPLY_FILE").expect("USER_REPLY_FILE is not set"))
        .expect("Something went wrong reading the file")
        .split('\n')
        .map(|s| {
            let mut parts = s.split('|');
            (
                parts.next().unwrap_or("").to_string(),
                parts.map(String::from).collect(),
            )
        })
        .collect()
}
fn get_user_reply(user: &str) -> Option<String> {
    read_user_replies()
        .get(&user.to_lowercase())?
        .choose(&mut rand::thread_rng())
        .cloned()
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
