use std::collections::HashMap;
use serenity::model::prelude::{GuildContainer, Message, RoleId, User};
use serenity::prelude::Context;
use serenity::utils::MessageBuilder;
use crate::Config;

pub(crate) async fn send_simple_msg(context: &Context, msg: &Message, text: &str) {
    let response = MessageBuilder::new()
        .push(text)
        .build();
    if let Err(why) = msg.channel_id.say(&context.http, &response).await {
        eprintln!("Error sending message: {:?}", why);
    }
}

pub(crate) async fn send_simple_tagged_msg(context: &Context, msg: &Message, text: &str, mentioned: &User) -> Option<Message> {
    let response = MessageBuilder::new()
        .mention(mentioned)
        .push(text)
        .build();
    if let Ok(m) = msg.channel_id.say(&context.http, &response).await {
        Some(m)
    } else {
        eprintln!("Error sending message");
        None
    }
}

pub(crate) async fn admin_check(context: &Context, msg: &Message, print_msg: bool) -> bool {
    let data = context.data.write().await;
    let config: &Config = data.get::<Config>().unwrap();
    if let Some(admin_role_id) = &config.discord.admin_role_id {
        let role_name = context.cache.role(msg.guild_id.unwrap(), RoleId::from(*admin_role_id)).await.unwrap().name;
        return if msg.author.has_role(&context.http, GuildContainer::from(msg.guild_id.unwrap()), RoleId::from(*admin_role_id)).await.unwrap_or_else(|_| false) {
            true
        } else {
            if print_msg {
                let response = MessageBuilder::new()
                    .mention(&msg.author)
                    .push(" this command requires the '")
                    .push(role_name)
                    .push("' role.")
                    .build();
                if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                    eprintln!("Error sending message: {:?}", why);
                }
            }
            false
        };
    }
    true
}

pub(crate) async fn populate_unicode_emojis() -> HashMap<char, String> {
// I hate this implementation and I deserve to be scolded
// in my defense however, you have to provide unicode emojis to the api
// if Discord's API allowed their shortcuts i.e. ":smile:" instead that would have been more intuitive
    let mut map = HashMap::new();
    map.insert('a', String::from("🇦"));
    map.insert('b', String::from("🇧"));
    map.insert('c', String::from("🇨"));
    map.insert('d', String::from("🇩"));
    map.insert('e', String::from("🇪"));
    map.insert('f', String::from("🇫"));
    map.insert('g', String::from("🇬"));
    map.insert('h', String::from("🇭"));
    map.insert('i', String::from("🇮"));
    map.insert('j', String::from("🇯"));
    map.insert('k', String::from("🇰"));
    map.insert('l', String::from("🇱"));
    map.insert('m', String::from("🇲"));
    map.insert('n', String::from("🇳"));
    map.insert('o', String::from("🇴"));
    map.insert('p', String::from("🇵"));
    map.insert('q', String::from("🇶"));
    map.insert('r', String::from("🇷"));
    map.insert('s', String::from("🇸"));
    map.insert('t', String::from("🇹"));
    map.insert('u', String::from("🇺"));
    map.insert('v', String::from("🇻"));
    map.insert('w', String::from("🇼"));
    map.insert('x', String::from("🇽"));
    map.insert('y', String::from("🇾"));
    map.insert('z', String::from("🇿"));
    map
}
