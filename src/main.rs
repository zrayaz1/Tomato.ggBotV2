mod commands;
mod player_stats;
use std::collections::HashMap;
use tokio::sync::Mutex;
//use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use poise::serenity_prelude as serenity;
use commands::marks::{fetch_tank_data, Tank, marks,};
use commands::stats::stats;
 
#[derive(Debug, EnumIter, PartialEq, Eq, Hash, poise::ChoiceParameter)]
pub enum Region {
    NA,
    EU,
    ASIA,
}

impl Region {
    fn extension(&self) -> &str {
        match self {
            Region::NA => "com",
            Region::EU => "eu",
            Region::ASIA => "asia",
        }
    }
}


pub struct Data {
    tank_data: Mutex<HashMap<Region, Vec<Tank>>>,
    player_data: Mutex<HashMap<u32, String>>,
}


type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


#[tokio::main]
async fn main() {
    let mut tank_info = HashMap::new();
    let (na, eu, asia) = tokio::join!(
        fetch_tank_data(&Region::NA),
        fetch_tank_data(&Region::EU),
        fetch_tank_data(&Region::ASIA));
    
    tank_info.insert(Region::NA, na);
    tank_info.insert(Region::EU, eu);
    tank_info.insert(Region::ASIA, asia);

    let data = Data{
        tank_data: Mutex::new(tank_info),
        player_data: Mutex::new(HashMap::new()),
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![marks(), stats()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        }); 
    framework.run().await.unwrap();
}
   

























