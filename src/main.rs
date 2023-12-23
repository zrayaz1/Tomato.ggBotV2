mod commands;
mod errors;
mod player_stats;
use commands::clanstats::clanstats;
use commands::marks::{RecentTankStats,fetch_tank_economics, generate_tank_map, marks, Tank, TankEconomics, generate_recent_tank_map};
use commands::stats::stats;
use poise::serenity_prelude as serenity;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use strum_macros::EnumIter;
use tokio::sync::Mutex;
use tokio::time;

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
    tank_data: Arc<Mutex<HashMap<Region, Vec<Tank>>>>,
    tank_economics: Arc<Mutex<Vec<TankEconomics>>>,
    recent_tank_stats: Arc<Mutex<HashMap<Region, Vec<RecentTankStats>>>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub fn get_short_position(position: &str) -> &str {
    match position {
        "commander" => "CDR",
        "executive_officer" => "XO",
        "personnel_officer" => "PO",
        "combat_officer" => "CO",
        "recruitment_officer" => "RO",
        "intelligence_officer" => "IO",
        "quartermaster" => "QM",
        "junior_officer" => "JO",
        "private" => "PVT",
        "recruit" => "RCT",
        "reservist" => "RES",
        _ => "Err",
    }
}

pub fn get_wn8_color(wn8: u32) -> i32 {
    match wn8 {
        0 => 0x808080,
        1..=300 => 0x930D0D,
        301..=450 => 0xCD3333,
        451..=650 => 0xCC7A00,
        651..=900 => 0xCCB800,
        901..=1200 => 0x849B24,
        1201..=1600 => 0x4D7326,
        1601..=2000 => 0x4099BF,
        2001..=2450 => 0x3972C6,
        2451..=2900 => 0x6844d4,
        2901..=3400 => 0x522b99,
        3401..=4000 => 0x411d73,
        4001..=4700 => 0x310d59,
        4701..=u32::MAX => 0x24073d,
    }
}

async fn update_recent_tank_data(data: Arc<Mutex<HashMap<Region, Vec<RecentTankStats>>>>) {
    let mut interval = time::interval(Duration::from_secs(36000000));
    loop {
        interval.tick().await;
        let mut old = data.lock().await;
        *old = generate_recent_tank_map().await;
    }
}

async fn update_tank_data(data: Arc<Mutex<HashMap<Region, Vec<Tank>>>>) {
    let mut interval = time::interval(Duration::from_secs(36000));
    loop {
        interval.tick().await;
        let mut old = data.lock().await;
        *old = generate_tank_map().await;
    }
}

async fn update_tank_economics(economics: Arc<Mutex<Vec<TankEconomics>>>) {
    let mut interval = time::interval(Duration::from_secs(128000));
    loop {
        interval.tick().await;
        let mut old = economics.lock().await;
        match fetch_tank_economics().await {
            Ok(tanks) => {
                *old = tanks;
            }
            Err(e) => {
                println!("Error in tank economics: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let data = Data {
        tank_data: Arc::new(Mutex::new(HashMap::new())),
        tank_economics: Arc::new(Mutex::new(Vec::new())),
        recent_tank_stats: Arc::new(Mutex::new(HashMap::new())),
    };
    tokio::spawn(update_tank_data(Arc::clone(&data.tank_data)));
    tokio::spawn(update_tank_economics(Arc::clone(&data.tank_economics)));
    tokio::spawn(update_recent_tank_data(Arc::clone(&data.recent_tank_stats)));

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
