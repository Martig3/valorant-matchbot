use std::collections::HashMap;
use std::time::Duration;

use async_std::task;
use rand::Rng;
use regex::Regex;
use serenity::client::Context;
use serenity::futures::StreamExt;
use serenity::model::channel::{Message, ReactionType};
use serenity::model::guild::{GuildContainer, Guild};
use serenity::model::user::User;
use serenity::prelude::TypeMap;
use serenity::utils::MessageBuilder;
use tokio::sync::RwLockWriteGuard;

use crate::{BotState, Config, Setup, Maps, QueueMessages, RiotIdCache, State, StateContainer, TeamNameCache, UserQueue};
use crate::utils::{admin_check, send_simple_msg, send_simple_tagged_msg};

struct ReactionResult {
    count: u64,
    map: String,
}


pub(crate) async fn handle_list(context: Context, msg: Message) {
    let data = context.data.write().await;
    let user_queue: &Vec<User> = data.get::<UserQueue>().unwrap();
    let mut user_name = String::new();
    for u in user_queue {
        user_name.push_str(format!("\n- @{}", u.name).as_str());
    }
    let response = MessageBuilder::new()
        .push("Current queue size: ")
        .push(&user_queue.len())
        .push("/10")
        .push(user_name)
        .build();

    if let Err(why) = msg.channel_id.say(&context.http, &response).await {
        eprintln!("Error sending message: {:?}", why);
    }
}

pub(crate) async fn handle_clear(context: Context, msg: Message) {
    if !admin_check(&context, &msg, true).await { return; }
    let mut data = context.data.write().await;
    let user_queue: &mut Vec<User> = &mut data.get_mut::<UserQueue>().unwrap();
    user_queue.clear();
    let response = MessageBuilder::new()
        .mention(&msg.author)
        .push(" cleared queue")
        .build();
    if let Err(why) = msg.channel_id.say(&context.http, &response).await {
        eprintln!("Error sending message: {:?}", why);
    }
}

pub(crate) async fn handle_help(context: Context, msg: Message) {
    let mut commands = String::from("
`.join` - Join the queue, add a message in quotes (max 50 char) i.e. `.join \"available at 9pm\"`
`.leave` - Leave the queue
`.queue` - List all users in the queue
`.riotid` - Set your riotid i.e. `.riotid Martige#NA1`
`.maps` - Lists all maps available for map vote
`.teamname` - Sets a custom team name when you are a captain i.e. `.teamname Your Team Name`
_These are commands used during the `.setup` process:_
`.captain` - Add yourself as a captain.
`.pick` - If you are a captain, this is used to pick a player by tagging them i.e. `.pick @Martige`
");
    let admin_commands = String::from("
_These are privileged admin commands:_
`.setup` - Start the match setup process
`.kick` - Kick a player by mentioning them i.e. `.kick @user`
`.addmap` - Add a map to the map vote i.e. `.addmap mapname`
`.removemap` - Remove a map from the map vote i.e. `.removemap mapname`
`.recoverqueue` - Manually set a queue, tag all users to add after the command
`.clear` - Clear the queue
`.cancel` - Cancels `.setup` process & retains current queue
    ");
    if admin_check(&context, &msg, false).await {
        commands.push_str(&admin_commands)
    }
    let response = MessageBuilder::new()
        .push(commands)
        .build();
    if let Ok(channel) = &msg.author.create_dm_channel(&context.http).await {
        if let Err(why) = channel.say(&context.http, &response).await {
            eprintln!("Error sending message: {:?}", why);
        }
    } else {
        eprintln!("Error sending .help dm");
    }
}

pub(crate) async fn handle_start(context: Context, msg: Message) {
    let admin_check = admin_check(&context, &msg, true).await;
    if !&admin_check { return; }
    let mut data = context.data.write().await;
    let bot_state: &StateContainer = data.get::<BotState>().unwrap();
    if bot_state.state != State::Idle {
        send_simple_tagged_msg(&context, &msg, " `.setup` command has already been entered", &msg.author).await;
        return;
    }
    let user_queue: &mut Vec<User> = data.get_mut::<UserQueue>().unwrap();
    if !user_queue.contains(&msg.author) && !admin_check {
        send_simple_tagged_msg(&context, &msg, " non-admin users that are not in the queue cannot start the match", &msg.author).await;
        return;
    }
    if user_queue.len() != 10 {
        let response = MessageBuilder::new()
            .mention(&msg.author)
            .push(" the queue is not full yet")
            .build();
        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            eprintln!("Error sending message: {:?}", why);
        }
        return;
    }
    // let user_queue_mention: String = user_queue
    //     .iter()
    //     .map(|user| format!("- <@{}>\n", user.id))
    //     .collect();
    // let response = MessageBuilder::new()
    //     .push(user_queue_mention)
    //     .push_bold_line("Match setup is starting...")
    //     .build();
    // if let Err(why) = msg.channel_id.say(&context.http, &response).await {
    //     eprintln!("Error sending message: {:?}", why);
    // }

}

pub(crate) async fn handle_defense_option(context: Context, msg: Message) {
    {
        let mut data: RwLockWriteGuard<TypeMap> = context.data.write().await;
        let bot_state: &mut StateContainer = &mut data.get_mut::<BotState>().unwrap();
        if bot_state.state != State::SidePick {
            send_simple_tagged_msg(&context, &msg, " it is not currently the side pick phase", &msg.author).await;
            return;
        }
        let draft: &mut Setup = &mut data.get_mut::<Setup>().unwrap();
        if &msg.author != draft.captain_b.as_ref().unwrap() {
            send_simple_tagged_msg(&context, &msg, " you are not Captain B", &msg.author).await;
            return;
        }
        draft.team_b_start_side = String::from("ct");
        send_simple_msg(&context, &msg, "Setup is completed.").await;
    }
    handle_ready(&context, &msg).await;
}

pub(crate) async fn handle_attack_option(context: Context, msg: Message) {
    {
        let mut data = context.data.write().await;
        let bot_state: &mut StateContainer = &mut data.get_mut::<BotState>().unwrap();
        if bot_state.state != State::SidePick {
            send_simple_tagged_msg(&context, &msg, " it is not currently the side pick phase", &msg.author).await;
            return;
        }
        let draft: &mut Setup = &mut data.get_mut::<Setup>().unwrap();
        if &msg.author != draft.captain_b.as_ref().unwrap() {
            send_simple_tagged_msg(&context, &msg, " you are not Captain B", &msg.author).await;
            return;
        }
        draft.team_b_start_side = String::from("t");
        send_simple_msg(&context, &msg, "Setup is completed.").await;
    }
    handle_ready(&context, &msg).await;
}

pub(crate) async fn handle_riotid(context: Context, msg: Message) {
    let mut data = context.data.write().await;
    let riot_id_cache: &mut HashMap<u64, String> = &mut data.get_mut::<RiotIdCache>().unwrap();
    let split_content = msg.content.trim().split(' ').take(2).collect::<Vec<_>>();
    if split_content.len() == 1 {
        send_simple_tagged_msg(&context, &msg, " please check the command formatting. There must be a space in between `.riotid` and your Riot id. \
        Example: `.riotid Martige#NA1`", &msg.author).await;
        return;
    }
    let riot_id_str: String = String::from(split_content[1]);
    let riot_id_regex = Regex::new("\\w+#\\w+").unwrap();
    if !riot_id_regex.is_match(&riot_id_str) {
        send_simple_tagged_msg(&context, &msg, " invalid Riot id formatting. Please follow this example: `.riotid Martige#NA1`", &msg.author).await;
        return;
    }
    riot_id_cache.insert(*msg.author.id.as_u64(), String::from(&riot_id_str));
    write_to_file(String::from("riot_ids.json"), serde_json::to_string(riot_id_cache).unwrap()).await;
    let response = MessageBuilder::new()
        .push("Updated Riot id for ")
        .mention(&msg.author)
        .push(" to `")
        .push(&riot_id_str)
        .push("`")
        .build();
    if let Err(why) = msg.channel_id.say(&context.http, &response).await {
        eprintln!("Error sending message: {:?}", why);
    }
}

pub(crate) async fn handle_map_list(context: Context, msg: Message) {
    let data = context.data.write().await;
    let maps: &Vec<String> = data.get::<Maps>().unwrap();
    let map_str: String = maps.iter().map(|map| format!("- `{}`\n", map)).collect();
    let response = MessageBuilder::new()
        .push_line("Current map pool:")
        .push(map_str)
        .build();
    if let Err(why) = msg.channel_id.say(&context.http, &response).await {
        eprintln!("Error sending message: {:?}", why);
    }
}

pub(crate) async fn handle_kick(context: Context, msg: Message) {
    if !admin_check(&context, &msg, true).await { return; }
    let mut data = context.data.write().await;
    let state: &mut StateContainer = data.get_mut::<BotState>().unwrap();
    if state.state != State::Idle {
        send_simple_tagged_msg(&context, &msg, " cannot `.kick` the queue after `.setup`, use `.cancel` to start over if needed.", &msg.author).await;
        return;
    }
    let user_queue: &mut Vec<User> = data.get_mut::<UserQueue>().unwrap();
    let user = &msg.mentions[0];
    if !user_queue.contains(&user) {
        let response = MessageBuilder::new()
            .mention(&msg.author)
            .push(" is not in the queue.")
            .build();
        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            eprintln!("Error sending message: {:?}", why);
        }
        return;
    }
    let index = user_queue.iter().position(|r| r.id == user.id).unwrap();
    user_queue.remove(index);
    let response = MessageBuilder::new()
        .mention(user)
        .push(" has been kicked. Queue size: ")
        .push(user_queue.len().to_string())
        .push("/10")
        .build();
    if let Err(why) = msg.channel_id.say(&context.http, &response).await {
        eprintln!("Error sending message: {:?}", why);
    }
}

pub(crate) async fn handle_add_map(context: Context, msg: Message) {
    if !admin_check(&context, &msg, true).await { return; }
    let mut data = context.data.write().await;
    let maps: &mut Vec<String> = data.get_mut::<Maps>().unwrap();
    if maps.len() >= 26 {
        let response = MessageBuilder::new()
            .mention(&msg.author)
            .push(" unable to add map, max amount reached.")
            .build();
        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            eprintln!("Error sending message: {:?}", why);
        }
        return;
    }
    let map_name: String = String::from(msg.content.trim().split(" ").take(2).collect::<Vec<_>>()[1]);
    if maps.contains(&map_name) {
        let response = MessageBuilder::new()
            .mention(&msg.author)
            .push(" unable to add map, already exists.")
            .build();
        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            eprintln!("Error sending message: {:?}", why);
        }
        return;
    }
    maps.push(String::from(&map_name));
    write_to_file(String::from("maps.json"), serde_json::to_string(maps).unwrap()).await;
    let response = MessageBuilder::new()
        .mention(&msg.author)
        .push(" added map: `")
        .push(&map_name)
        .push("`")
        .build();
    if let Err(why) = msg.channel_id.say(&context.http, &response).await {
        eprintln!("Error sending message: {:?}", why);
    }
}

pub(crate) async fn handle_remove_map(context: Context, msg: Message) {
    if !admin_check(&context, &msg, true).await { return; }
    let mut data = context.data.write().await;
    let maps: &mut Vec<String> = data.get_mut::<Maps>().unwrap();
    let map_name: String = String::from(msg.content.trim().split(" ").take(2).collect::<Vec<_>>()[1]);
    if !maps.contains(&map_name) {
        let response = MessageBuilder::new()
            .mention(&msg.author)
            .push(" this map doesn't exist in the list.")
            .build();
        if let Err(why) = msg.channel_id.say(&context.http, &response).await {
            eprintln!("Error sending message: {:?}", why);
        }
        return;
    }
    let index = maps.iter().position(|m| m == &map_name).unwrap();
    maps.remove(index);
    write_to_file(String::from("maps.json"), serde_json::to_string(maps).unwrap()).await;
    let response = MessageBuilder::new()
        .mention(&msg.author)
        .push(" removed map: `")
        .push(&map_name)
        .push("`")
        .build();
    if let Err(why) = msg.channel_id.say(&context.http, &response).await {
        eprintln!("Error sending message: {:?}", why);
    }
}

pub(crate) async fn handle_unknown(context: Context, msg: Message) {
    let response = MessageBuilder::new()
        .push("Unknown command, type `.help` for list of commands.")
        .build();
    if let Err(why) = msg.channel_id.say(&context.http, &response).await {
        eprintln!("Error sending message: {:?}", why);
    }
}

pub(crate) async fn write_to_file(path: String, content: String) {
    let mut error_string = String::from("Error writing to ");
    error_string.push_str(&path);
    std::fs::write(path, content)
        .expect(&error_string);
}

pub(crate) async fn handle_ready(context: &Context, msg: &Message) {
    let mut data = context.data.write().await;
    // reset to Idle state
    let draft: &mut Setup = &mut data.get_mut::<Setup>().unwrap();
    draft.team_a = vec![];
    draft.team_b = vec![];
    draft.captain_a = None;
    draft.captain_b = None;
    draft.current_picker = None;
    let bot_state: &mut StateContainer = &mut data.get_mut::<BotState>().unwrap();
    bot_state.state = State::Idle;
}

pub(crate) async fn handle_cancel(context: Context, msg: Message) {
    if !admin_check(&context, &msg, true).await { return; }
    let mut data = context.data.write().await;
    let bot_state: &StateContainer = &data.get::<BotState>().unwrap();
    if bot_state.state == State::Idle {
        send_simple_tagged_msg(&context, &msg, " command only valid during `.setup` process", &msg.author).await;
        return;
    }
    let draft: &mut Setup = &mut data.get_mut::<Setup>().unwrap();
    draft.team_a = vec![];
    draft.team_b = vec![];
    draft.captain_a = None;
    draft.captain_b = None;
    draft.current_picker = None;
    let bot_state: &mut StateContainer = &mut data.get_mut::<BotState>().unwrap();
    bot_state.state = State::Idle;
    send_simple_tagged_msg(&context, &msg, " `.setup` process cancelled.", &msg.author).await;
}
