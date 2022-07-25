use std::collections::HashMap;
use std::{env, fs};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg
            .content
            .chars()
            .take_while(|c| *c != ' ')
            .collect::<String>()
            == "!set"
        {
            let words: Vec<&str> = msg.content.split(' ').collect();
            if words.len() >= 3 {
                let mut replies = read_user_replies();
                replies.insert(
                    words[1].to_string(),
                    words[2..].iter().map(|s| s.to_string() + " ").collect(),
                );
                write_user_replies(replies);
            }
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

fn write_user_replies(replies: HashMap<String, String>) {
    fs::write(
        env::var("USER_REPLY_FILE").expect("USER_REPLY_FILE is not set"),
        replies
            .iter()
            .map(|(k, v)| format!("{}|{}\n", k.to_lowercase(), v))
            .collect::<String>(),
    )
    .expect("Failed to write file");
}

fn read_user_replies() -> HashMap<String, String> {
    fs::read_to_string(env::var("USER_REPLY_FILE").expect("USER_REPLY_FILE is not set"))
        .expect("Something went wrong reading the file")
        .split('\n')
        .map(|s| {
            let mut parts = s.split('|');
            (parts.next().unwrap_or("").to_string(), parts.next().unwrap_or("").to_string())
        })
        .collect()
}
fn get_user_reply(user: &str) -> Option<String> {
    read_user_replies().get(&user.to_lowercase()).cloned()
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    // let msg_reply_file = "msg_reply.txt";
    // let msg_replies: HashMap<String, String> = fs::read_to_string(msg_reply_file)
    //     .expect("Something went wrong reading the file")
    //     .split('\n')
    //     .map(|s| {
    //         (
    //             s.chars().take_while(|c| *c != ' ').collect(),
    //             s.chars().skip_while(|c| *c != ' ').collect(),
    //         )
    //     })
    //     .collect();

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
