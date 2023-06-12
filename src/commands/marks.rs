use poise::serenity_prelude::CreateEmbed;
use serde::Deserialize;
use std::time::Instant;
use fuzzywuzzy::fuzz;
use fuzzywuzzy::utils;
use fuzzywuzzy::process;
use crate::{Region,Context, Error};

#[derive(Deserialize)]
pub struct ApiResponse {
    meta: MetaData,
    data: Vec<Tank>,
}

#[derive(Deserialize)]
pub struct MetaData {
    status: String,
}

#[derive(Deserialize, Debug)]
pub struct Tank {
    id: u32,
    image: String,
    #[serde(rename = "isGift")]
    is_gift: bool,
    #[serde(rename = "isPrem")]
    is_premium: bool,
    name: String,
    nation: String,
    tier: u32,
    class: String,
    #[serde(default)]
    #[serde(rename = "50")]
    pct_50: u32,
    #[serde(default)]
    #[serde(rename = "65")]
    pct_65: u32,
    #[serde(default)]
    #[serde(rename = "85")]
    pct_85: u32,
    #[serde(default)]
    #[serde(rename = "95")]
    pct_95: u32,
    #[serde(default)]
    #[serde(rename = "100")]
    pct_100: u32,
    #[serde(default)] 
    #[serde(rename = "1st")]
    first: u32,
    #[serde(default)]
    #[serde(rename = "2nd")]
    second: u32,
    #[serde(default)]
    #[serde(rename = "3rd")]
    third: u32,
    #[serde(default)]
    ace: u32,
    #[serde(default)]
    images: Images,
}


#[derive(Deserialize, Debug, Default, Clone)]
pub struct Images {
    small_icon: String,
    contour_icon: String,
    big_icon: String,
}

pub async fn fetch_tank_data(region: &Region) -> Vec<Tank> {
    let start = Instant::now();
    let moe_url = format!("https://api.tomato.gg/dev/api-v2/moe/{}",
                          region.get_extension());
    let mastery_url = format!("https://api.tomato.gg/dev/api-v2/mastery/{}",
                              region.get_extension());
    let moe: ApiResponse = reqwest::get(moe_url)
        .await
        .expect("MOE Api Failed")
        .json()
        .await
        .expect("MOE Json Conversion Failed");
    let mastery: ApiResponse = reqwest::get(mastery_url)
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
                image: tank1.image,
                is_gift: tank1.is_gift,
                is_premium: tank1.is_premium,
                name: tank1.name,
                nation: tank1.nation,
                tier: tank1.tier,
                class: tank1.class,
                pct_50: tank1.pct_50,
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
    println!("Fetched Tanks from {} in {:?}", region.get_name(), duration);
    return tanks 
}


pub async fn fuzzy_find_tank(input: &str, tanks: &Vec<Tank>) {
    let tank_name_list: Vec<String> = tanks.iter().map(|t| t.name.clone()).collect();
    let expected_name = process::extract_one(
        &input,
        &tank_name_list, 
        &utils::full_process,
        &fuzz::wratio,
        0,
        );
    println!("think it's {}", expected_name.unwrap().0.as_str());
}

pub async fn generate_mark_embed(tank_name: &str, tanks: Vec<Tank>) {
    let tank = tanks.iter().find(|tank| tank.name == tank_name); 
    let mut embed = CreateEmbed::default();
    embed.title("test");
    
}


#[poise::command(slash_command)]
pub async fn marks(
    ctx: Context<'_>,
    #[description = "Tank Name"] input: String,
    #[description = "Select a Region"]
    region: Option<Region>,
    ) -> Result<(), Error> {
    match region{
        Some(region) => {
            let region_data = ctx.data().tank_data.get(&region).unwrap();
            let region_name = &region.get_name();
            fuzzy_find_tank(&input, region_data).await;
            ctx.say(format!("{} pee pee {}",region_name,input)).await?;
        }
        None => {
            let region_data = ctx.data().tank_data.get(&Region::NA).unwrap();
            let region_name = &Region::NA.get_name();
            ctx.say(format!("{} pee pee {}",region_name,input)).await?;
        }
    }
    Ok(())
}

