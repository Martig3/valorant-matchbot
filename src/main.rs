use std::collections::HashMap;
use std::str::FromStr;


use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::Client;
use serenity::client::Context;
use serenity::framework::standard::StandardFramework;
use serenity::model::prelude::{Interaction, InteractionResponseType, Ready};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::user::User;
use serenity::prelude::{EventHandler, TypeMapKey};

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
}

#[derive(PartialEq)]
struct StateContainer {
    state: State,
}

struct Setup {
    captain_a: Option<User>,
    captain_b: Option<User>,
    team_a: Vec<User>,
    team_b: Vec<User>,
    team_b_start_side: String,
    current_picker: Option<User>,
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

enum Command {
    SETUP,
    RIOTID,
    MAPS,
    ADDMAP,
    CANCEL,
    REMOVEMAP,
    DEFENSE,
    ATTACK,
    CLEAR,
    HELP,
    UNKNOWN,
}

async fn interaction_create(context: Context, interaction: Interaction) {
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
            Command::SETUP => commands::handle_setup(&context, &inc_command).await,
            Command::MAPS => commands::handle_map_list(&context).await,
            Command::CANCEL => commands::handle_cancel(&context, &inc_command).await,
            Command::ADDMAP => commands::handle_add_map(&context, &inc_command).await,
            Command::REMOVEMAP => commands::handle_remove_map(&context, &inc_command).await,
            Command::CLEAR => commands::handle_clear(&context, &inc_command).await,
            _ => {String::from("Unknown command, use `/help` for list of commands.")}
        };
        if let Err(why) = create_int_resp(&context, &inc_command, content).await {
            eprintln!("Cannot respond to slash command: {}", why);
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
impl FromStr for Command {
    type Err = ();
    fn from_str(input: &str) -> Result<Command, Self::Err> {
        match input {
            "start" => Ok(Command::SETUP),
            "riotid" => Ok(Command::RIOTID),
            "maps" => Ok(Command::MAPS),
            "addmap" => Ok(Command::ADDMAP),
            "cancel" => Ok(Command::CANCEL),
            "defense" => Ok(Command::DEFENSE),
            "attack" => Ok(Command::ATTACK),
            "removemap" => Ok(Command::REMOVEMAP),
            "help" => Ok(Command::HELP),
            _ => Err(()),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _context: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> () {
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
        data.insert::<Setup>(Setup {
            captain_a: None,
            captain_b: None,
            current_picker: None,
            team_a: Vec::new(),
            team_b: Vec::new(),
            team_b_start_side: String::from(""),
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


