use crate::errors::TankEconomicsFetchError;
use crate::errors::RecentTankStatsFetchError;
use crate::get_wn8_color;
use crate::{Context, Error, Region};
use fuzzywuzzy::fuzz;
use fuzzywuzzy::process;
use fuzzywuzzy::utils;
use poise::serenity_prelude::CreateEmbed;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Deserialize)]
pub struct MarkResponse {
    meta: MetaData,
    data: Vec<Tank>,
}

#[derive(Deserialize)]
pub struct MetaData {
    status: String,
}

// I FUCKING HATE JSON WHY WHY WHY
// WHY THE FUCK DO THE INTS BECOME STRINGS WHEN THE DATA IS MISSING
// WHO THE FUCK DESIGNED THIS SYSTEM
// GARGLE MY FUCKING NUTS
// DYNAMICALLY TYPED LANGUAGES WERE A FUCKING MISTAKE
fn deserialize_str_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Value = serde::Deserialize::deserialize(deserializer)?;

    match value {
        Value::Number(n) => {
            if let Some(v) = n.as_u64() {
                Ok(v as u32)
            } else {
                Err(serde::de::Error::custom("Expected u32 for third"))
            }
        }
        Value::String(_) => {
            println!("Missing Value in Tanks");

            Ok(0) // treat any string as a zero
        }
        _ => Err(serde::de::Error::custom("Expected u32 or String for third")),
    }
}




#[derive(Deserialize,Debug,Clone)]
pub struct RecentTankStats {
    tank_id: u32,
    name: String,
    nation: String,
    tier: u32,
    class: String,
    image: String,
    big_image: String,
    battles: u32,
    winrate: f64,
    player_winrate: f64,
    winrate_differential: f64,
    damage: u32,
    sniper_damage: u32,
    frags: f64,
    shots_fired: f64,
    direct_hits: f64,
    penetrations: f64,
    hit_rate: f64,
    pen_rate: f64,
    spotting_assist: u32,
    tracking_assist: u32,
    spots: f64,
    damage_blocked: u32,
    damage_received: u32,
    potential_damage_received: u32,
    base_capture_points: f64,
    base_defense_points: f64,
    life_time: u32,
    survival: f64,
    distance_traveled: u32,
    wn8: u32,
    #[serde(rename = "isPrem")]
    is_prem: bool,
}

pub async fn fetch_recent_tank_stats(region: &Region) 
    -> Result<Vec<RecentTankStats>, RecentTankStatsFetchError> {
    let start = Instant::now();
    let recent_tank_stats_url = format!("https://api.tomato.gg/dev/api-v2/all-tanks-server-stats-wr-range/{}/0/100?cache=true",region.extension());
    let response = reqwest::get(recent_tank_stats_url)
        .await?
        .json::<Vec<RecentTankStats>>()
        .await?;
    let duration = start.elapsed();
    println!("Fetched Recent Tank Stats from {} in {:?}",region.name() ,duration);
    return Ok(response);
}


#[derive(Deserialize, Debug, Clone)]
pub struct TankEconomicsResponse {
    data: Vec<TankEconomics>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TankEconomics {
    #[serde(rename = "tank_id")]
    pub id: u32,
    pub battles: u32,
    pub avg_earnings: u32,
    pub avg_profit: i32,
    pub avg_ammo_cost: u32,
    pub cost_per_shot: u32,
    pub earnings_per_minute: u32,
    pub profit_per_minute: i32,
}

pub async fn fetch_tank_economics() -> Result<Vec<TankEconomics>, TankEconomicsFetchError> {
    let start = Instant::now();
    let economics_url = "https://api.tomato.gg/dev/api-v2/tank-economics";
    let response = reqwest::get(economics_url)
        .await?
        .json::<TankEconomicsResponse>()
        .await?;

    let duration = start.elapsed();
    println!("Fetched Tank Economics in {:?}", duration);
    return Ok(response.data);
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tank {
    pub id: u32,
    pub nation: String,
    #[serde(rename = "isPrem")]
    pub is_prem: bool,
    pub class: String,
    pub name: String,
    pub tier: u32,
    #[serde(default)]
    #[serde(rename = "65")]
    pub pct_65: u32,
    #[serde(default)]
    #[serde(rename = "85")]
    pub pct_85: u32,
    #[serde(default)]
    #[serde(rename = "95")]
    pub pct_95: u32,
    #[serde(default)]
    #[serde(rename = "100")]
    pub pct_100: u32,
    #[serde(default)]
    #[serde(rename = "1st")]
    pub first: u32,
    #[serde(default)]
    #[serde(rename = "2nd")]
    pub second: u32,
    #[serde(default, deserialize_with = "deserialize_str_to_u32")]
    #[serde(rename = "3rd")]
    pub third: u32,
    #[serde(default)]
    pub ace: u32,
    #[serde(default)]
    pub images: Images,
}
impl Default for Tank {
    fn default() -> Tank {
        Tank {
            id: 16897,
            name: String::from("Obj. 140"),
            tier: 10,
            class: String::from("ussr"),
            is_prem: false,
            nation: String::from("Big Dick Land"),
            pct_65: 6969,
            pct_85: 6969,
            pct_95: 6969,
            pct_100: 6969,
            first: 6969,
            second:6969,
            third: 6969,
            ace: 6969,
            images: Images {
                big_icon: String::from("https://api.worldoftanks.com/static/2.71.0/wot/encyclopedia/vehicle/ussr-R97_Object_140.png"),
            }
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Images {
    big_icon: String,
}

pub async fn fetch_tank_data(region: &Region) -> Vec<Tank> {
    let start = Instant::now();
    let moe_url = format!(
        "https://api.tomato.gg/dev/api-v2/moe/{}",
        region.extension()
    );
    let mastery_url = format!(
        "https://api.tomato.gg/dev/api-v2/mastery/{}",
        region.extension()
    );
    let moe: MarkResponse = reqwest::get(moe_url)
        .await
        .expect("MOE Api Failed")
        .json()
        .await
        .expect("MOE Json Conversion Failed");
    let mastery: MarkResponse = reqwest::get(mastery_url)
        .await
        .expect("Mastery Api Failed")
        .json()
        .await
        .expect("Mastery Json Conversion Failed");
    if moe.meta.status != "ok" || mastery.meta.status != "ok" {
        println!("MoE or Mastery Status Not Ok, missing data?");
    }

    let tanks: Vec<Tank> = moe
        .data
        .into_iter()
        .filter_map(|tank1| {
            let matching_tank2 = mastery.data.iter().find(|tank2| tank2.id == tank1.id);
            matching_tank2.map(|tank2| Tank {
                id: tank1.id,
                name: tank1.name,
                tier: tank1.tier,
                pct_65: tank1.pct_65,
                pct_85: tank1.pct_85,
                nation: tank1.nation,
                is_prem: tank1.is_prem,
                class: tank1.class,
                pct_95: tank1.pct_95,
                pct_100: tank1.pct_100,
                first: tank2.first,
                second: tank2.second,
                third: tank2.third,
                ace: tank2.ace,
                images: tank2.images.clone(),
            })
        })
        .collect();
    let duration = start.elapsed();
    println!("Fetched Tanks from {} in {:?}", region.name(), duration);
    return tanks;
}

pub fn fuzzy_find_tank(input: &str, tanks: &Vec<Tank>) -> String {
    let tank_name_list: Vec<String> = tanks.iter().map(|t| t.name.clone()).collect();
    let expected_name = process::extract_one(
        &input,
        &tank_name_list,
        &utils::full_process,
        &fuzz::wratio,
        0,
    )
    .unwrap()
    .0;
    println!("Passed: {}, Found: {}", input, &expected_name);
    expected_name
}

pub fn create_tank_embed_description(tank: &Tank) -> String {
    let class_emoji = get_class_emoji(tank.is_prem, &tank.class);
    let nation_emoji = get_nation_emoji(&tank.nation);
    return "".to_owned() + nation_emoji + " " + class_emoji;

}

pub fn get_class_emoji(is_prem: bool, class: &str) -> &str {
    if is_prem {
        match class {
            "MT" => {return "<:premMT:1188064479020339241>"},
            "LT" => {return "<:premLT:1188064478282137602>"},
            "HT" => {return "<:premHT:1188064474989592647>"},
            "SPG" => {return "<:premSPG:1188064475870416896>"},
            "TD" => {return "<:premTD:1188064477405528134>"},
            _ => {return "Error"},
        }
    }

    match class {
        "MT" => {return "<:MT:1188064483134951474>"},
        "LT" => {return "<:LT:1188064513061290035>"},
        "HT" => {return "<:HT:1188064482153467954>"},
        "SPG" => {return "<:SPG:1188064481050370048>"},
        "TD" => {return "<:TD:1188064486490378260>"},
        _ => {return "Error"},
    }
}

pub fn get_nation_emoji(nation: &str) -> &str{

    match nation {
        "france" => {"<:France:1188231544800813106>"},
        "ussr" => {"<:USSR:1188231683397406772>"},
        "germany" => {"<:Germany:1188231546340122654>"},
        "china" => {"<:China:1188231542452011038>"},
        "poland" => {"<:Poland:1188231549938827365>"},
        "uk" => {"<:UK:1188231586550915163>"},
        "usa" => {"<:USA:1188231554502250596>"},
        "sweden" => {"<:Sweden:1188231551285219348>"},
        "japan" => {"<:Japan:1188231548751855696>"},
        "italy" => {"<:Italy:1188231547246088222>"},
        "czech" => {"<:Czech:1188231543328604191>"},
        _ => {"Emoji Error"},
    }


}

pub async fn generate_mark_embed(
        tank: &Tank, 
        region: &Region, 
        tank_economics: &TankEconomics,
        recent_tank_stats: &RecentTankStats,) 
    -> CreateEmbed {
    CreateEmbed::default().title(format!("{} {}",tank.name,region.name()))
        .url(format!("https://tomato.gg/tanks/{}/{}",region.name(),tank.id))
        .description(create_tank_embed_description(tank))
        .field("MoE Reqs",
            format!("100: `{}`\n<:mark_3:1188009637291765801>: `{}`\n <:mark_2:1188009640777236514>: `{}`\n <:mark_1:1188009633772736563>: `{}`",
            tank.pct_100,tank.pct_95,tank.pct_85,tank.pct_65),true)
        .field("Mastery(XP)",
            format!("<:masteryIcon:1188009638420037652>: `{}`\n<:firstClassIcon:1188009639820935240>: `{}`\n<:2ndClassIcon:1188009636398387260>: `{}`\n<:3rdClassIcon:1188009635014246441>: `{}`",
            tank.ace,tank.first,tank.second,tank.third),true)
        .field(
            "Economics",
            format!(
                "Avg. Profit: `{}`<:credits:1188059891395477585>\nAvg. Revenue: `{}`<:credits:1188059891395477585>\n Avg. Ammo Cost: `{}`<:credits:1188059891395477585>\n Profit/Min: `{}`<:credits:1188059891395477585>",
                tank_economics.avg_profit,
                tank_economics.avg_earnings,
                tank_economics.avg_ammo_cost,
                tank_economics.profit_per_minute
            ),
            true,
        )
        .field(
            "30 Days Stats",
            format!(
                "WN8: `{}`\nWinRate: `{}%`\n Damage: `{}`\n Assist: `{}`",
                recent_tank_stats.wn8,
                recent_tank_stats.winrate,
                recent_tank_stats.damage,
                recent_tank_stats.spotting_assist 
                + recent_tank_stats.tracking_assist,
            ),
            true,
        )
        .color(get_wn8_color(recent_tank_stats.wn8))
        .thumbnail(&tank.images.big_icon)
        .footer(|f| {
            f.text("Powered by Tomato.gg");
            f.icon_url("https://tomato.gg/_next/image?url=%2Ftomato.png&w=48&q=75");
            f
        }).to_owned()
}


pub async fn generate_tank_map() -> HashMap<Region, Vec<Tank>> {
    let mut tank_map = HashMap::new();
    let (na, eu, asia) = tokio::join!(
        fetch_tank_data(&Region::NA),
        fetch_tank_data(&Region::EU),
        fetch_tank_data(&Region::ASIA)
    );

    tank_map.insert(Region::NA, na);
    tank_map.insert(Region::EU, eu);
    tank_map.insert(Region::ASIA, asia);
    tank_map
}

pub async fn generate_recent_tank_map() -> HashMap<Region, Vec<RecentTankStats>> {
    let mut tank_map = HashMap::new();
    let (na, eu, asia) = tokio::join!(
        fetch_recent_tank_stats(&Region::NA),
        fetch_recent_tank_stats(&Region::EU),
        fetch_recent_tank_stats(&Region::ASIA)
    );
    //TODO proper error handling here
    tank_map.insert(Region::NA, na.unwrap());
    tank_map.insert(Region::EU, eu.unwrap());
    tank_map.insert(Region::ASIA, asia.unwrap());
    tank_map
}

#[poise::command(slash_command)]
pub async fn marks(
    ctx: Context<'_>,
    #[description = "Tank Name"] input: String,
    #[description = "Select a Region"] region: Option<Region>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;

    let parsed_region = region.unwrap_or(Region::NA); // default region NA
    let tank_map = ctx.data().tank_data.lock().await;
    let tanks = tank_map.get(&parsed_region).unwrap();
    let tank_name = fuzzy_find_tank(&input, &tanks);
    let tank = tanks.iter().find(|tank| tank.name == tank_name).unwrap();
    let economics = ctx.data().tank_economics.lock().await;
    let tank_economics = economics.iter().find(|t| t.id == tank.id).unwrap();
    let recent_tank_map = ctx.data().recent_tank_stats.lock().await;
    let recent_tanks = recent_tank_map.get(&parsed_region).unwrap();
    let recent_tank_stats = recent_tanks.iter().find(|t| t.tank_id == tank.id).unwrap();
    let embed = generate_mark_embed(tank, 
        &parsed_region, 
        &tank_economics,
        &recent_tank_stats,).await;
    ctx.send(|f| {f.embed(|f| {f.clone_from(&embed);f})}).await?;
    Ok(())
}
