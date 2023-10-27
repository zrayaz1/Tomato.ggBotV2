mod errors;

mod commands;
mod player_stats;
use std::collections::HashMap;
use tokio::sync::Mutex;
use strum_macros::EnumIter;
use poise::serenity_prelude as serenity;
use commands::marks::{fetch_tank_data, Tank, marks,};
use commands::stats::stats;
use commands::clanstats::clanstats; 


#[derive(Debug, EnumIter, PartialEq, Eq, Hash, poise::ChoiceParameter, Clone, Copy, Default)]
pub enum Region {
    #[default]
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
    loop_running: Mutex<bool>,
}


type Error = Box<dyn std::error::Error + Send + Sync>; //DO NOT OPEN PANDORA'S BOX
type Context<'a> = poise::Context<'a, Data, Error>;

pub fn get_short_position(position: &str) -> &str {
    match position {
        "commander" => {"CDR"}
        "executive_officer" => {"XO"}
        "personnel_officer" => {"PO"}
        "combat_officer" => {"CO"}
        "recruitment_officer" => {"RO"}
        "intelligence_officer" => {"IO"}
        "quartermaster" => {"QM"}
        "junior_officer" => {"JO"}
        "private" => {"PVT"}
        "recruit" => {"RCT"}
        "reservist" => {"RES"}
        _ => {"Err"}
    }
                    
}

pub fn get_wn8_color(wn8: u32) -> i32 {
    match wn8 {
        0 => {0x808080}
        1..=300 => {0x930D0D}
        301..=450 => {0xCD3333}
        451..=650 => {0xCC7A00}
        651..=900 => {0xCCB800}
        901..=1200 => {0x849B24}
        1201..=1600 => {0x4D7326}
        1601..=2000 => {0x4099BF}
        2001..=2450 => {0x3972C6}
        2451..=2900 => {0x6844d4}
        2901..=3400 => {0x522b99}
        3401..=4000 => {0x411d73}
        4001..=4700 => {0x310d59}
        4701..=u32::MAX => {0x24073d}
    }
}

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
        loop_running: Mutex::new(false),
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![marks(), stats(), clanstats()],
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

    if let Err(e) = framework.run().await {
        eprintln!("{}", e);
    }
}
   

























