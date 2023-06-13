mod commands;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use poise::serenity_prelude as serenity;
use commands::marks::{fetch_tank_data, Tank, marks};

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
    tank_data: HashMap<Region, Vec<Tank>>
}


type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;



#[tokio::main]
async fn main() {
    let mut tank_info = HashMap::new();
    let mut tank_data;
    for region in Region::iter() {
        tank_data = fetch_tank_data(&region).await;
        tank_info.insert(region, tank_data);
    }

    let data = Data{
        tank_data: tank_info,
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![marks()],
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
   

























