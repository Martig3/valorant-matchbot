use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::{Deref, Index};
use std::str::FromStr;
use chrono::{Date, DateTime, Utc};
use rand::distributions::uniform::SampleBorrow;


use regex::Regex;
use serenity::client::Context;
use serenity::http::CacheHttp;
use serenity::model::channel::{Message};
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::prelude::Role;
use serenity::prelude::TypeMap;
use serenity::utils::MessageBuilder;
use tokio::sync::RwLockWriteGuard;
use uuid::Uuid;

use crate::{BotState, Setup, Maps, RiotIdCache, State, StateContainer, Match, Matches, MatchState, RolePartial, ScheduleInfo};
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
`/setup` - Start the match setup process
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
    let admin_check = admin_check(context, msg).await;
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
    String::from("Help info sent via DM")
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
    let mut data: RwLockWriteGuard<TypeMap> = context.data.write().await;
    let bot_state: &mut StateContainer = data.get_mut::<BotState>().unwrap();
    if bot_state.state != State::SidePick {
        return String::from(" it is not currently the side pick phase");
    }
    let draft: &mut Setup = data.get_mut::<Setup>().unwrap();
    if &msg.user != draft.captain_b.as_ref().unwrap() {
        return String::from(" you are not Captain B");
    }
    // TODO: more elaborate printout here
    String::from("Setup is completed.")
}

pub(crate) async fn handle_attack_option(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let mut data = context.data.write().await;
    let bot_state: &mut StateContainer = data.get_mut::<BotState>().unwrap();
    if bot_state.state != State::SidePick {
        return String::from(" it is not currently the side pick phase");
    }
    let draft: &mut Setup = data.get_mut::<Setup>().unwrap();
    if &msg.user != draft.captain_b.as_ref().unwrap() {
        return String::from(" you are not Captain B");
    }
    String::from("Setup is completed.")
}

pub(crate) async fn handle_riotid(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let mut data = context.data.write().await;
    let riot_id_cache: &mut HashMap<u64, String> = data.get_mut::<RiotIdCache>().unwrap();
    let option = msg.data
        .options
        .get(0)
        .expect("Expected steamid option")
        .resolved
        .as_ref()
        .expect("Expected object");
    if let ApplicationCommandInteractionDataOptionValue::String(riot_id_str) = option {
        let riot_id_regex = Regex::new("\\w+#\\w+").unwrap();
        if !riot_id_regex.is_match(riot_id_str) {
            return String::from(" invalid Riot ID formatting");
        }
        riot_id_cache.insert(*msg.user.id.as_u64(), String::from(riot_id_str));
        write_to_file("riot_ids.json", serde_json::to_string(riot_id_cache).unwrap()).await;
        return MessageBuilder::new()
            .push("Updated Riot id for ")
            .mention(&msg.user)
            .push(" to `")
            .push(&riot_id_str)
            .push("`")
            .build();
    }
    String::from("Discord API error")
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

pub(crate) async fn handle_schedule(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let option_one = msg.data
        .options
        .get(0)
        .expect("Expected date option")
        .resolved
        .as_ref()
        .expect("Expected object");
    let option_two = msg.data
        .options
        .get(1)
        .expect("Expected time option")
        .resolved
        .as_ref()
        .expect("Expected object");
    let mut date: Option<DateTime<Utc>> = None;
    let mut time: Option<String> = None;
    if let ApplicationCommandInteractionDataOptionValue::String(date_str) = option_one {
        if let Ok(date_result) = DateTime::parse_from_str(date_str, "MM/DD/yyyy") {
            date = Some(DateTime::from(date_result));
        } else {
            return String::from("Incorrect date format. Please use correct format (MM/DD/YYYY) i.e. `12/23/2022`");
        }
    }
    if let ApplicationCommandInteractionDataOptionValue::String(time_str) = option_two {
        time = Some(time_str.to_string());
    }
    if let Ok(roles) = context.http.get_guild_roles(*msg.guild_id.unwrap().as_u64()).await {
        let team_roles: Vec<Role> = roles.into_iter().filter(|r| r.name.starts_with("Team")).collect();
        let mut user_team_role: Option<Role> = None;
        for team_role in team_roles {
            if let Ok(_has_role) = msg.user.has_role(&context.http, team_role.guild_id, team_role.id).await {
                user_team_role = Some(team_role);
                break;
            }
        }
        if let Some(team_role) = user_team_role {
            let mut data = context.data.write().await;
            let matches: &mut Vec<Match> = data.get_mut::<Matches>().unwrap();
            for m in matches {
                if m.team_one.id != team_role.id && m.team_two.id != team_role.id { continue; }
                m.schedule_info = Some(ScheduleInfo { date: date.unwrap(), time_str: time.clone().unwrap() });
                return format!("Your next match ({} vs {}) is now scheduled", m.team_one.name, m.team_two.name);
            }
        }
    }
    String::from("You are not part of any team. Verify you have a role starting with `Team `")
}

pub(crate) async fn handle_matches(context: &Context, _msg: &ApplicationCommandInteraction) -> String {
    let data = context.data.write().await;
    let matches: &Vec<Match> = data.get::<Matches>().unwrap();
    if matches.is_empty() {
        return String::from("No matches have been added");
    }
    let matches_str: String = matches.iter()
        .map(|m| {
            if m.note.is_some() {
                let row = format!("- {} vs {} `{}`\n", m.team_one.name, m.team_two.name, m.note.clone().unwrap());
                row
            } else {
                let row = format!("- {} vs {} \n", m.team_one.name, m.team_two.name);
                row
            }
        })
        .collect();
    matches_str
}

pub(crate) async fn handle_add_match(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let option_one = msg.data
        .options
        .get(0)
        .expect("Expected teamone option")
        .resolved
        .as_ref()
        .expect("Expected object");
    let option_two = msg.data
        .options
        .get(1)
        .expect("Expected teamtwo option")
        .resolved
        .as_ref()
        .expect("Expected object");
    let option_three = msg.data
        .options
        .get(2);
    let mut team_one = None;
    let mut team_two = None;
    if let ApplicationCommandInteractionDataOptionValue::Role(team_one_role) = option_one {
        team_one = Some(RolePartial { id: team_one_role.id, name: team_one_role.name.to_string(), guild_id: team_one_role.guild_id });
    }
    if let ApplicationCommandInteractionDataOptionValue::Role(team_two_role) = option_two {
        team_two = Some(RolePartial { id: team_two_role.id, name: team_two_role.name.to_string(), guild_id: team_two_role.guild_id });
    }
    let mut new_match = Match { id: Uuid::new_v4(), team_one: team_one.unwrap(), team_two: team_two.unwrap(), note: None, date_added: Utc::now(), match_state: MatchState::Entered, schedule_info: None };
    if let Some(option) = option_three {
        if let Some(ApplicationCommandInteractionDataOptionValue::String(option_value)) = &option.resolved {
            new_match.note = Option::from(option_value.clone());
        }
    }
    let mut data = context.data.write().await;
    let matches: &mut Vec<Match> = data.get_mut::<Matches>().unwrap();
    matches.push(new_match);
    write_to_file("matches.json", serde_json::to_string_pretty(matches).unwrap()).await;
    String::from("Successfully added new match")
}

pub(crate) async fn handle_ready(context: &Context, _msg: &Message) {
    let mut data = context.data.write().await;
    // reset to Idle state
    let draft: &mut Setup = data.get_mut::<Setup>().unwrap();
    draft.team_a = None;
    draft.team_b = None;
    draft.maps_remaining = Vec::new();
    draft.vetos = Vec::new();
    draft.current_vetoer = None;
    draft.captain_a = None;
    draft.captain_b = None;
    draft.current_picker = None;
    let bot_state: &mut StateContainer = data.get_mut::<BotState>().unwrap();
    bot_state.state = State::Idle;
}

pub(crate) async fn handle_cancel(context: &Context, msg: &ApplicationCommandInteraction) -> String {
    let admin_check = admin_check(&context, &msg).await;
    if let Err(error) = admin_check { return error; }
    let mut data = context.data.write().await;
    let bot_state: &StateContainer = data.get::<BotState>().unwrap();
    if bot_state.state == State::Idle {
        return String::from(" command only valid during `.setup` process");
    }
    let draft: &mut Setup = data.get_mut::<Setup>().unwrap();
    draft.team_a = None;
    draft.team_b = None;
    draft.maps_remaining = Vec::new();
    draft.vetos = Vec::new();
    draft.current_vetoer = None;
    draft.captain_a = None;
    draft.captain_b = None;
    draft.current_picker = None;
    let bot_state: &mut StateContainer = data.get_mut::<BotState>().unwrap();
    bot_state.state = State::Idle;
    String::from("`.setup` process cancelled.")
}
