use serenity::model::prelude::{GuildContainer, Role, RoleId, User};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use serenity::utils::MessageBuilder;
use crate::{Config, Maps, RolePartial, Setup, State};

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

pub(crate) async fn is_phase_allowed(context: &Context, msg: &ApplicationCommandInteraction, state: State) -> Result<(), String> {
    let mut data = context.data.write().await;
    let setup: &mut Setup = data.get_mut::<Setup>().unwrap();
    if setup.current_phase != state {
        return Err(String::from("It is not the correct phase"));
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


pub(crate) async fn user_team(context: &Context, msg: &ApplicationCommandInteraction) -> Result<RolePartial, String> {
    let mut data = context.data.write().await;
    let setup: &mut Setup = data.get_mut::<Setup>().unwrap();
    if let Ok(has_role_one) = msg.user.has_role(&context.http, msg.guild_id.unwrap(), setup.clone().team_one.unwrap().id).await {
        if has_role_one { return Ok(setup.clone().team_one.unwrap()); }
        if let Ok(has_role_two) = msg.user.has_role(&context.http, msg.guild_id.unwrap(), setup.clone().team_two.unwrap().id).await {
            if has_role_two { return Ok(setup.clone().team_two.unwrap()); }
        }
    }
    return Err(String::from("You are not part of either team currently running `/setup`"));
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

pub(crate) async fn get_maps(context: &Context) -> Vec<String> {
    let data = context.data.write().await;
    let maps: &Vec<String> = data.get::<Maps>().unwrap();
    maps.clone()
}


pub(crate) fn eos_printout(setup: Setup) -> String {
    let mut resp = String::from("\n\nSetup is completed. GLHF!\n\n");
    for (i, el) in setup.maps.iter().enumerate() {
        resp.push_str(format!("**{}. {}** - picked by: <@&{}>\n    _Defense start:_ <@&{}>\n    _Attack start:_ <@&{}>\n\n", i + 1, el.map.to_uppercase(), &el.picked_by.id, el.start_defense.clone().unwrap().id, el.start_attack.clone().unwrap().id).as_str())
    }
    resp
}