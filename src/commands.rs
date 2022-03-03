use std::collections::HashMap;


use regex::Regex;
use serenity::client::Context;
use serenity::model::channel::{Message};
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::prelude::TypeMap;
use serenity::utils::MessageBuilder;
use tokio::sync::RwLockWriteGuard;

use crate::{BotState, Setup, Maps, RiotIdCache, State, StateContainer};
use crate::utils::{admin_check, write_to_file};


// pub(crate) async fn handle_list(context: &Context, msg: Message) {
//     let data = context.data.write().await;
//     let user_queue: &Vec<User> = data.get::<UserQueue>().unwrap();
//     let mut user_name = String::new();
//     for u in user_queue {
//         user_name.push_str(format!("\n- @{}", u.name).as_str());
//     }
//     let response = MessageBuilder::new()
//         .push("Current queue size: ")
//         .push(&user_queue.len())
//         .push("/10")
//         .push(user_name)
//         .build();
//
//     if let Err(why) = msg.channel_id.say(&context.http, &response).await {
//         eprintln!("Error sending message: {:?}", why);
//     }
// }

pub(crate) async fn handle_help(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let mut commands = String::from("
`/riotid` - Set your riotid i.e. `.riotid Martige#NA1`
`/maps` - Lists all maps available for map vote
`/setup` - Start the match setup process (captains only)
_These are commands used during the `.setup` process:_
//TODO
");
    let admin_commands = String::from("
_These are privileged admin commands:_
`/addmap` - Add a map to the map vote i.e. `.addmap mapname`
`/removemap` - Remove a map from the map vote i.e. `.removemap mapname`
`/recoverqueue` - Manually set a queue, tag all users to add after the command
`/cancel` - Cancels `.setup` process & retains current queue
    ");
    let admin_check = admin_check(&context, &msg).await;
    if let Ok(_result_str) = admin_check {
        commands.push_str(&admin_commands)
    }
    let response = MessageBuilder::new()
        .push(commands)
        .build();
    if let Ok(channel) = &msg.user.create_dm_channel(&context.http).await {
        if let Err(why) = channel.say(&context.http, &response).await {
            eprintln!("Error sending message: {:?}", why);
        }
    } else {
        eprintln!("Error sending .help dm");
    }
    return String::from("Help info sent via DM");
}

pub(crate) async fn handle_setup(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let admin_check = admin_check(&context, &msg).await;
    if let Err(error) = admin_check { return error; }
    let data = context.data.write().await;
    let bot_state: &StateContainer = data.get::<BotState>().unwrap();
    if bot_state.state != State::Idle {
        return String::from(" `/setup` command has already been entered");
    }
    String::from("")
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

pub(crate) async fn handle_defense_option(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    {
        let mut data: RwLockWriteGuard<TypeMap> = context.data.write().await;
        let bot_state: &mut StateContainer = &mut data.get_mut::<BotState>().unwrap();
        if bot_state.state != State::SidePick {
            return String::from(" it is not currently the side pick phase");
        }
        let draft: &mut Setup = &mut data.get_mut::<Setup>().unwrap();
        if &msg.user != draft.captain_b.as_ref().unwrap() {
            return String::from(" you are not Captain B");
        }
        draft.team_b_start_side = String::from("ct");
        // TODO: more elaborate printout here
        return String::from("Setup is completed.");
    }
}

pub(crate) async fn handle_attack_option(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    {
        let mut data = context.data.write().await;
        let bot_state: &mut StateContainer = &mut data.get_mut::<BotState>().unwrap();
        if bot_state.state != State::SidePick {
            return String::from(" it is not currently the side pick phase");
        }
        let draft: &mut Setup = &mut data.get_mut::<Setup>().unwrap();
        if &msg.user != draft.captain_b.as_ref().unwrap() {
            return String::from(" you are not Captain B");
        }
        draft.team_b_start_side = String::from("t");
        return String::from("Setup is completed.");
    }
}

pub(crate) async fn handle_riotid(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let mut data = context.data.write().await;
    let riot_id_cache: &mut HashMap<u64, String> = &mut data.get_mut::<RiotIdCache>().unwrap();
    // TODO: impl this
    let split_content = [""];
    let riot_id_str: String = String::from(split_content[1]);
    let riot_id_regex = Regex::new("\\w+#\\w+").unwrap();
    if !riot_id_regex.is_match(&riot_id_str) {
        return String::from(" invalid RiotId formatting");
    }
    riot_id_cache.insert(*msg.user.id.as_u64(), String::from(&riot_id_str));
    write_to_file(String::from("riot_ids.json"), serde_json::to_string(riot_id_cache).unwrap()).await;
    return MessageBuilder::new()
        .push("Updated Riot id for ")
        .mention(&msg.user)
        .push(" to `")
        .push(&riot_id_str)
        .push("`")
        .build();
}

pub(crate) async fn handle_map_list(context: &Context) -> String {
    let data = context.data.write().await;
    let maps: &Vec<String> = data.get::<Maps>().unwrap();
    let map_str: String = maps.iter().map(|map| format!("- `{}`\n", map)).collect();
    return MessageBuilder::new()
        .push_line("Current map pool:")
        .push(map_str)
        .build();
}

pub(crate) async fn handle_add_map(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let admin_check = admin_check(&context, &msg).await;
    if let Err(error) = admin_check { return error; }
    let mut data = context.data.write().await;
    let maps: &mut Vec<String> = data.get_mut::<Maps>().unwrap();
    if maps.len() >= 26 {
        return MessageBuilder::new()
            .mention(&msg.user)
            .push(" unable to add map, max amount reached.")
            .build();
    }
    // TODO: impl this
    let map_name = String::from("");
    if maps.contains(&map_name) {
        return MessageBuilder::new()
            .mention(&msg.user)
            .push(" unable to add map, already exists.")
            .build();
    }
    maps.push(String::from(&map_name));
    write_to_file(String::from("maps.json"), serde_json::to_string(maps).unwrap()).await;
    return MessageBuilder::new()
        .mention(&msg.user)
        .push(" added map: `")
        .push(&map_name)
        .push("`")
        .build();
}

pub(crate) async fn handle_remove_map(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let admin_check = admin_check(&context, &msg).await;
    if let Err(error) = admin_check { return error; }
    let mut data = context.data.write().await;
    let maps: &mut Vec<String> = data.get_mut::<Maps>().unwrap();
    // TODO: impl this
    let map_name = String::from("");
    if !maps.contains(&map_name) {
        return MessageBuilder::new()
            .mention(&msg.user)
            .push(" this map doesn't exist in the list.")
            .build();
    }
    let index = maps.iter().position(|m| m == &map_name).unwrap();
    maps.remove(index);
    write_to_file(String::from("maps.json"), serde_json::to_string(maps).unwrap()).await;
    return MessageBuilder::new()
        .mention(&msg.user)
        .push(" removed map: `")
        .push(&map_name)
        .push("`")
        .build();
}

pub(crate) async fn handle_ready(context: &Context, _msg: &Message) {
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

pub(crate) async fn handle_cancel(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let admin_check = admin_check(&context, &msg).await;
    if let Err(error) = admin_check { return error; }
    let mut data = context.data.write().await;
    let bot_state: &StateContainer = &data.get::<BotState>().unwrap();
    if bot_state.state == State::Idle {
        return String::from(" command only valid during `.setup` process");
    }
    let draft: &mut Setup = &mut data.get_mut::<Setup>().unwrap();
    draft.team_a = vec![];
    draft.team_b = vec![];
    draft.captain_a = None;
    draft.captain_b = None;
    draft.current_picker = None;
    let bot_state: &mut StateContainer = &mut data.get_mut::<BotState>().unwrap();
    bot_state.state = State::Idle;
    String::from("`.setup` process cancelled.")
}
