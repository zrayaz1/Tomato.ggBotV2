use poise::serenity_prelude::CreateEmbed;
use serde::Deserialize;
use std::time::Instant;
use fuzzywuzzy::fuzz;
use fuzzywuzzy::utils;
use fuzzywuzzy::process;
use std::collections::HashMap;
use crate::errors::TankEconomicsFetchError;
use crate::{Region,Context, Error};
use serde_json::Value;


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
        }, 
        _ => Err(serde::de::Error::custom("Expected u32 or String for third")),
    }
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
    pub avg_earnings: i32,
    pub avg_profit: u32,
    pub avg_ammo_cost: u32,
    pub cost_per_shot: u32,
    pub earnings_per_minute: u32,
    pub profit_per_minute: i32,
}

pub async fn fetch_tank_economics() 
    -> Result<Vec<TankEconomics>, TankEconomicsFetchError> {

    let economics_url = "https://api.tomato.gg/dev/api-v2/tank-economics";
    let response = reqwest::get(economics_url)
        .await?
        .json::<TankEconomicsResponse>()
        .await?;
    return Ok(response.data);
}


#[derive(Deserialize, Debug, Clone)]
pub struct Tank {
    pub id: u32,
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
    #[serde(default,deserialize_with = "deserialize_str_to_u32")]
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
    let moe_url = format!("https://api.tomato.gg/dev/api-v2/moe/{}",
                          region.extension());
    let mastery_url = format!("https://api.tomato.gg/dev/api-v2/mastery/{}",
                              region.extension());
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
            let matching_tank2 = mastery
                .data
                .iter()
                .find(|tank2| tank2.id == tank1.id);
            matching_tank2.map(|tank2| Tank {
                id: tank1.id,
                name: tank1.name,
                tier: tank1.tier,
                pct_65: tank1.pct_65,
                pct_85: tank1.pct_85,
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
    return tanks 
}


pub async fn fuzzy_find_tank(input: &str, tanks: &Vec<Tank>) -> String {
    let tank_name_list: Vec<String> = tanks.iter().map(|t| t.name.clone()).collect();
    let expected_name = process::extract_one(
        &input,
        &tank_name_list, 
        &utils::full_process,
        &fuzz::wratio,
        0,
        ).unwrap().0;
    println!("Passed: {}, Found: {}",input, &expected_name);
    expected_name
}

pub async fn generate_mark_embed(tank: &Tank, region: &Region) -> CreateEmbed{
    CreateEmbed::default().title(format!("{} {}",tank.name, region.name()))
        .url(format!("https://tomato.gg/tanks/{}/{}",region.name(),tank.id))
        .field("MoE Reqs",
            format!("100%: `{}`\n[★★★]: `{}`\n [★★]: `{}`\n [★]: `{}`",
            tank.pct_100,tank.pct_95,tank.pct_85,tank.pct_65),true)
        .field("Mastery(XP)",
            format!("Mastery: `{}`\n1st Class: `{}`\n2nd Class: `{}`\n3rd Class: `{}`",
            tank.ace,tank.first,tank.second,tank.third),true)
        .thumbnail(&tank.images.big_icon)
        .footer(|f| {
            f.text("Powered by Tomato.gg");
            f.icon_url("https://tomato.gg/_next/image?url=%2Ftomato.png&w=48&q=75");
            f
        }).to_owned()
}

pub async fn generate_economics_embed(
        tank: &Tank,
        tank_economics: &TankEconomics,
        region: &Region,
        ) -> CreateEmbed {
    
    CreateEmbed::default().title(format!("{} Details",tank.name))
        .url(format!("https://tomato.gg/tanks/{}/{}",region.name(),tank.id))
        .field("Economics",
            format!("Avg. Profit: `{}`\n
                Avg. Revenue`{}`\n 
                Avg. Ammo Cost: `{}`\n 
                Profit/Min: `{}`",
            tank_economics.avg_profit,
            tank_economics.avg_earnings,
            tank_economics.avg_ammo_cost,
            tank_economics.profit_per_minute),true)

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
        fetch_tank_data(&Region::ASIA));
    
    tank_map.insert(Region::NA, na);
    tank_map.insert(Region::EU, eu);
    tank_map.insert(Region::ASIA, asia);
    tank_map
}

#[poise::command(slash_command)]
pub async fn marks(
    ctx: Context<'_>,
    #[description = "Tank Name"] input: String,
    #[description = "Select a Region"]
    region: Option<Region>,
    ) -> Result<(), Error> {
    let _ = ctx.defer().await;

    let parsed_region = region.unwrap_or(Region::NA); // default region NA
    let tank_map = ctx.data().tank_data.lock().await;
    let tanks = tank_map.get(&parsed_region).unwrap(); 
    let tank_name = fuzzy_find_tank(&input, &tanks).await;
    let tank = tanks.iter().find(|tank| tank.name == tank_name).unwrap();
    let embed = generate_mark_embed(tank, &parsed_region).await;
    ctx.send(|f| {f.embed(|f| {f.clone_from(&embed);f})}).await?;
     Ok(())
}

