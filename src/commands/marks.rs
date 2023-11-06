use poise::serenity_prelude::CreateEmbed;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use fuzzywuzzy::fuzz;
use fuzzywuzzy::utils;
use fuzzywuzzy::process;
use crate::{Region,Context, Error};
use tokio::{task, time};


#[derive(Deserialize)]
pub struct MarkResponse {
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
        );
    expected_name.unwrap().0
}

pub async fn generate_mark_embed(tank: &Tank, region: &Region) -> CreateEmbed{
    CreateEmbed::default().title(format!("{} {}",tank.name, region.name()))
        .url(format!("https://tomato.gg/tanks/NA/{}",tank.id))
        .field("Marks(Dmg + Track/Spot)",
            format!("1 Mark: `{}`\n2 Mark: `{}`\n3 Mark: `{}`\n100% MoE: `{}`",
            tank.pct_65,tank.pct_85,tank.pct_95,tank.pct_100),true)
        .field("Mastery(XP)",
            format!("3rd Class: `{}`\n2nd Class: `{}`\n1st Class: `{}`\nMastery: `{}`",
            tank.third,tank.second,tank.first,tank.ace),true)
        .thumbnail(&tank.images.big_icon)
        .footer(|f| {
            f.text("Powered by Tomato.gg");
            f.icon_url("https://tomato.gg/_next/image?url=%2Ftomato.png&w=48&q=75");
            f
        }).to_owned()
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
            let tank_map = ctx.data().tank_data.lock().await;
            let tanks = tank_map.get(&region).unwrap();
            let tank_name = fuzzy_find_tank(&input, &tanks).await;
            let tank = tanks.iter().find(|tank| tank.name == tank_name).unwrap();
            let embed = generate_mark_embed(tank, &region).await;
            ctx.send(|f| {f.embed(|f| {f.clone_from(&embed);f})}).await?;
        }         
        None => {
            let region = Region::NA;
            let tank_map = ctx.data().tank_data.lock().await;
            let tanks = tank_map.get(&region).unwrap();
            let tank_name = fuzzy_find_tank(&input, &tanks).await;
            let tank = tanks.iter().find(|tank| tank.name == tank_name).unwrap();
            let embed = generate_mark_embed(tank, &region).await;
            ctx.send(|f| {f.embed(|f| {f.clone_from(&embed);f})}).await?;
        }
    }
    if !*ctx.data().loop_running.lock().await {
        let mut interval = time::interval(Duration::from_secs(36000));
        *ctx.data().loop_running.lock().await = true;
        loop {
            interval.tick().await;
            let (na, eu, asia) = tokio::join!(
                fetch_tank_data(&Region::NA),
                fetch_tank_data(&Region::EU),
                fetch_tank_data(&Region::ASIA));
            let mut old_map = ctx.data().tank_data.lock().await;
            let old_na = old_map.entry(Region::NA).or_default();
            *old_na = na;
            let old_eu = old_map.entry(Region::EU).or_default();
            *old_eu = eu;
            let old_asia = old_map.entry(Region::ASIA).or_default();
            *old_asia = asia;
        }
    }


    


    Ok(())
}

