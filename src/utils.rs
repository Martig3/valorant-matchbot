use std::collections::HashMap;
use serenity::model::prelude::{GuildContainer, Message, Role, RoleId, User};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use serenity::utils::MessageBuilder;
use crate::{Config, Setup, State};

pub(crate) async fn write_to_file(path: &str, content: String) {
    let mut error_string = String::from("Error writing to ");
    error_string.push_str(&path);
    std::fs::write(path, content)
        .expect(&error_string);
}

pub(crate) async fn find_user_team_role(all_guild_roles: Vec<Role>, user: &User, context: &&Context) -> Result<Role, String> {
    let team_roles: Vec<Role> = all_guild_roles.into_iter().filter(|r| r.name.starts_with("Team")).collect();
    for team_role in team_roles {
        if let Ok(has_role) = user.has_role(&context.http, team_role.guild_id, team_role.id).await {
            if !has_role { continue; }
            return Ok(team_role);
        }
    } 
    Err(String::from("User does not have a team role"))
}
pub(crate) async fn map_veto_allowed(context: &Context, msg: &ApplicationCommandInteraction) -> Result<(), String> {
    let mut data = context.data.write().await;
    let setup: &mut Setup = data.get_mut::<Setup>().unwrap();
    if setup.current_phase != State::MapVeto {
        return Err(String::from("It is not the map veto phase"));
    }
    if let Ok(has_role_one) = msg.user.has_role(&context.http, msg.guild_id.unwrap(), setup.clone().team_one.unwrap().id).await {
        if let Ok(has_role_two) = msg.user.has_role(&context.http, msg.guild_id.unwrap(), setup.clone().team_two.unwrap().id).await {
            if !has_role_one && !has_role_two {
                return Err(String::from("You are not part of either team currently running `/setup`"));
            }
        }
    } 
    Ok(())
}

pub(crate) async fn admin_check(context: &Context, inc_command: &ApplicationCommandInteraction) -> Result<String, String> {
    let data = context.data.write().await;
    let config: &Config = data.get::<Config>().unwrap();
    if let Some(admin_role_id) = &config.discord.admin_role_id {
        let role_name = context.cache.role(inc_command.guild_id.unwrap(), RoleId::from(*admin_role_id)).await.unwrap().name;
        return if inc_command.user.has_role(&context.http, GuildContainer::from(inc_command.guild_id.unwrap()), RoleId::from(*admin_role_id)).await.unwrap_or_else(|_| false) {
            Ok(String::from("User has role"))
        } else {
            Err(MessageBuilder::new()
                .mention(&inc_command.user)
                .push(" this command requires the '")
                .push(role_name)
                .push("' role.")
                .build())
        };
    }
    Ok(String::from("Admin Role not set, allowed"))
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
