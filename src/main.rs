use std::collections::HashMap;
use std::str::FromStr;
use chrono::{Date, DateTime, Utc};


use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::Client;
use serenity::client::Context;
use serenity::framework::standard::StandardFramework;
use serenity::model::guild::Role;
use serenity::model::prelude::{GuildId, Interaction, InteractionResponseType, Ready};
use serenity::model::prelude::application_command::{ApplicationCommandInteraction, ApplicationCommandOptionType};
use serenity::model::user::User;
use serenity::prelude::{EventHandler, TypeMapKey};
use uuid::Uuid;

mod commands;
mod utils;

#[derive(Serialize, Deserialize)]
struct Config {
    discord: DiscordConfig,
    post_setup_msg: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct DiscordConfig {
    token: String,
    admin_role_id: Option<u64>,
    channel_id: Option<u64>,
    application_id: u64,
    guild_id: u64,
}

#[derive(PartialEq)]
struct StateContainer {
    state: State,
}

#[derive(Serialize, Deserialize)]
struct SeriesMap {
    map: String,
    picked_by: Role,
    start_attack: Role,
    start_defense: Role,
}

#[derive(Serialize, Deserialize)]
struct Veto {
    map: String,
    vetoed_by: Role,
}

#[derive(Serialize, Deserialize)]
enum MatchState {
    Entered,
    Scheduled,
    Completed,
}

#[derive(Serialize, Deserialize)]
struct Match {
    id: Uuid,
    team_one: Option<Role>,
    team_two: Option<Role>,
    note: Option<String>,
    date_added: DateTime<Utc>,
    match_state: MatchState,
}

#[derive(Serialize, Deserialize)]
struct Setup {
    captain_a: Option<User>,
    captain_b: Option<User>,
    team_a: Option<Role>,
    team_b: Option<Role>,
    current_picker: Option<User>,
    current_vetoer: Option<User>,
    maps_remaining: Vec<String>,
    maps: Vec<SeriesMap>,
    vetos: Vec<Veto>,
}

#[derive(PartialEq)]
enum State {
    Idle,
    MapVeto,
    SidePick,
}

struct Handler;

struct RiotIdCache;

struct BotState;

struct Maps;

struct Matches;


impl TypeMapKey for Config {
    type Value = Config;
}

impl TypeMapKey for RiotIdCache {
    type Value = HashMap<u64, String>;
}

impl TypeMapKey for BotState {
    type Value = StateContainer;
}

impl TypeMapKey for Maps {
    type Value = Vec<String>;
}

impl TypeMapKey for Setup {
    type Value = Setup;
}

impl TypeMapKey for Matches {
    type Value = Vec<Match>;
}

enum Command {
    Setup,
    Schedule,
    Addmatch,
    RiotID,
    Maps,
    Cancel,
    Defense,
    Attack,
    Help,
}


impl FromStr for Command {
    type Err = ();
    fn from_str(input: &str) -> Result<Command, Self::Err> {
        match input {
            "start" => Ok(Command::Setup),
            "schedule" => Ok(Command::Schedule),
            "addmatch" => Ok(Command::Addmatch),
            "riotid" => Ok(Command::RiotID),
            "maps" => Ok(Command::Maps),
            "cancel" => Ok(Command::Cancel),
            "defense" => Ok(Command::Defense),
            "attack" => Ok(Command::Attack),
            "help" => Ok(Command::Help),
            _ => Err(()),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        let config = read_config().await.unwrap();
        let guild_id = GuildId(config.discord.guild_id);
        let commands = GuildId::set_application_commands(&guild_id, &context.http, |commands| {
            return commands
                .create_application_command(|command| {
                    command.name("maps").description("Lists the current map pool")
                })
                .create_application_command(|command| {
                    command.name("help").description("DM yourself help info")
                })
                .create_application_command(|command| {
                    command.name("riotid").description("Set your Riot ID").create_option(|option| {
                        option
                            .name("riotid")
                            .description("Your Riot ID, i.e. Martige#0123")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
                })
                .create_application_command(|command| {
                    command.name("addmatch").description("Add match to schedule (admin required)").create_option(|option| {
                        option
                            .name("teamone")
                            .description("Team 1 (Home)")
                            .kind(ApplicationCommandOptionType::Role)
                            .required(true)
                    }).create_option(|option| {
                        option
                            .name("teamtwo")
                            .description("Team 2 (Away)")
                            .kind(ApplicationCommandOptionType::Role)
                            .required(true)
                    }).create_option(|option| {
                        option
                            .name("note")
                            .description("Note")
                            .kind(ApplicationCommandOptionType::String)
                            .required(false)
                    })
                })
                .create_application_command(|command| {
                    command.name("schedule").description("Schedule your next match").create_option(|option| {
                        option
                            .name("date")
                            .description("Date (MM/DD/YYYY)")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    }).create_option(|option| {
                        option
                            .name("time")
                            .description("Time (include timezone) i.e. 10EST")
                            .kind(ApplicationCommandOptionType::String)
                            .required(false)
                    })
                })
            ;
        }).await;
        println!("{} is connected!", ready.user.name);
        println!("Added these guild slash commands: {:#?}", commands);
    }
    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(inc_command) = interaction {
            let command = Command::from_str(&inc_command.data.name.as_str().to_lowercase()).expect("Expected valid command");
            {
                let data = context.data.write().await;
                let config: &Config = data.get::<Config>().unwrap();
                let content = String::from("Please use the assigned channel for bot commands");
                if &config.discord.channel_id.unwrap() != inc_command.channel_id.as_u64() {
                    if let Err(why) = create_int_resp(&context, &inc_command, content).await {
                        eprintln!("Cannot respond to slash command: {}", why);
                    }
                }
            }
            let content: String = match command {
                Command::Setup => commands::handle_setup(&context, &inc_command).await,
                Command::Addmatch => commands::handle_add_match(&context, &inc_command).await,
                Command::Schedule => commands::handle_schedule(&context, &inc_command).await,
                Command::Maps => commands::handle_map_list(&context).await,
                Command::RiotID => commands::handle_riotid(&context, &inc_command).await,
                Command::Defense => commands::handle_defense_option(&context, &inc_command).await,
                Command::Attack => commands::handle_attack_option(&context, &inc_command).await,
                Command::Cancel => commands::handle_cancel(&context, &inc_command).await,
                Command::Help => commands::handle_help(&context, &inc_command).await,
            };
            if let Err(why) = create_int_resp(&context, &inc_command, content).await {
                eprintln!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

async fn create_int_resp(context: &Context, inc_command: &ApplicationCommandInteraction, content: String) -> serenity::Result<()> {
    return inc_command
        .create_interaction_response(&context.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        }).await;
}

#[tokio::main]
async fn main() {
    let config = read_config().await.unwrap();
    let token = &config.discord.token;
    let framework = StandardFramework::new();
    let mut client = Client::builder(&token)
        .event_handler(Handler {})
        .framework(framework)
        .application_id(config.discord.application_id)
        .await
        .expect("Error creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<Config>(config);
        data.insert::<RiotIdCache>(read_riot_ids().await.unwrap());
        data.insert::<BotState>(StateContainer { state: State::Idle });
        data.insert::<Maps>(read_maps().await.unwrap());
        data.insert::<Matches>(Vec::new());
        data.insert::<Setup>(Setup {
            captain_a: None,
            captain_b: None,
            current_picker: None,
            current_vetoer: None,
            team_a: None,
            team_b: None,
            maps: Vec::new(),
            vetos: Vec::new(),
            maps_remaining: Vec::new(),
        });
    }
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

async fn read_config() -> Result<Config, serde_yaml::Error> {
    let yaml = std::fs::read_to_string("config.yaml").unwrap();
    let config: Config = serde_yaml::from_str(&yaml)?;
    Ok(config)
}

async fn read_riot_ids() -> Result<HashMap<u64, String>, serde_json::Error> {
    if std::fs::read("riot_ids.json").is_ok() {
        let json_str = std::fs::read_to_string("riot_ids.json").unwrap();
        let json = serde_json::from_str(&json_str).unwrap();
        Ok(json)
    } else {
        Ok(HashMap::new())
    }
}

async fn read_maps() -> Result<Vec<String>, serde_json::Error> {
    if std::fs::read("maps.json").is_ok() {
        let json_str = std::fs::read_to_string("maps.json").unwrap();
        let json = serde_json::from_str(&json_str).unwrap();
        Ok(json)
    } else {
        Ok(Vec::new())
    }
}


