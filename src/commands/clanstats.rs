use std::collections::HashMap;

use serde::Deserialize;
use poise::serenity_prelude::CreateEmbed;
use crate::{Region, Error, Context};

use super::stats::Emblems;


#[derive(Deserialize)]
pub struct ClanIdResponse {
    data: Option<Vec<ClanId>>,
}


#[derive(Deserialize)]
pub struct ClanId {
    clan_id: u32,
}


#[derive(Deserialize)]
pub struct TomatoClan {
    color: String,
    motto: String,
    emblems: Emblems,
    #[serde(rename = "overallWN8")]
    overall_wn8: f32,
    #[serde(rename = "overallWinrate")]
    overall_winrate: f32,
    #[serde(rename = "recentWN8")]
    recent_wn8: f32,
    #[serde(rename = "recentWinrate")]
    recent_winrate: f32,
    members_count: u32,
}


#[derive(Deserialize)]
pub struct GlobalResponse {
    data: HashMap<String, GlobalClanData>,
}

#[derive(Deserialize, Clone)]
pub struct GlobalClanData {
    statistics: GlobalStatistics,
    tag: String,
    name: String,
}

#[derive(Deserialize, Clone)]
pub struct GlobalStatistics {
    battles_10_level: u32,
    wins_10_level: u32,
    provinces_count: u32,
}


#[derive(Deserialize)]
pub struct RatingResponse {
    data: HashMap<String, RatingClanData>,
}

#[derive(Deserialize, Clone)]
pub struct RatingClanData {
    efficiency: Value,
    battles_count_avg_daily: Value,
    global_rating_weighted_avg: Value,
    fb_elo_rating_10: Value,
    fb_elo_rating_8: Value,
    fb_elo_rating_6: Value,
    gm_elo_rating_10: Value,
}

#[derive(Deserialize, Clone)]
pub struct Value {
    value: f32,
}

pub struct ClanData {
    rating: RatingClanData,
    global: GlobalClanData,
    tomato: TomatoClan,
}


pub async fn fetch_global_map(region: &Region, clan_id: u32) -> GlobalClanData {
    let global_map_url = format!("https://api.worldoftanks.{}/wot/globalmap/claninfo/?application_id=20e1e0e4254d98635796fc71f2dfe741&clan_id={}",
                                 region.extension(), clan_id);
    let response: GlobalResponse = reqwest::get(global_map_url).await.unwrap().json().await.unwrap();
    let output = response.data.get(&clan_id.to_string()).unwrap();
    return output.clone();
}

pub async fn fetch_clan_rating(region: &Region, clan_id: u32) -> RatingClanData{

    let rating_url = format!("https://api.worldoftanks.{}/wot/clanratings/clans/?application_id=20e1e0e4254d98635796fc71f2dfe741&clan_id={}",
                                 region.extension(), clan_id);
    let response: RatingResponse = reqwest::get(rating_url).await.unwrap().json().await.unwrap();
    let output = response.data.get(&clan_id.to_string()).unwrap();
    return output.clone();
}

pub async fn fetch_tomato_clan(region: &Region, clan_id: u32) -> TomatoClan{
    let tomato_url = format!("https://api.tomato.gg/api/clan/{}/{}",
                             region.extension(), clan_id);
    let response: TomatoClan = reqwest::get(tomato_url).await.unwrap().json().await.unwrap();
    response
}

pub async fn fetch_all_clan(region: &Region, clan_id: u32) -> ClanData{
    let (global_map, clan_rating, tomato_clan) = tokio::join!(
            fetch_global_map(&region, clan_id),
            fetch_clan_rating(&region, clan_id),
            fetch_tomato_clan(&region, clan_id)
        );

    ClanData {
        rating: clan_rating,
        global: global_map,
        tomato: tomato_clan,
    }
}

pub async fn fetch_clan_id(region: &Region, clan: &str) -> Option<u32> {
    let id_url = format!("https://api.worldoftanks.{}/wot/clans/list/?application_id=20e1e0e4254d98635796fc71f2dfe741&search={}",
                         region.extension(), clan);
    let response: ClanIdResponse = reqwest::get(id_url).await.unwrap().json().await.unwrap();
    match response.data {
        Some(data) => {Some(data[0].clan_id)},
        None => {None}
    }
}


pub async fn generate_clan_embed(data: &ClanData) -> CreateEmbed{
    let mut embed = CreateEmbed::default();
    embed.title(format!("[{}] {}", data.global.tag.to_uppercase(), data.global.name));
    embed.thumbnail(&data.tomato.emblems.x64.portal);
    embed.description(&data.tomato.motto);
    embed.field("Player Stats", 
                format!("Overall WN8: `{}`\nOverall WR: `{:.1}%`\nRecent WN8: `{}`\nRecent WR: `{:.1}`",
                        data.tomato.overall_wn8.round(),
                        data.tomato.overall_winrate,
                        data.tomato.recent_wn8.round(),
                        data.tomato.recent_winrate,
                        ),true);
    embed.field("General Stats",
                format!(
                    "Clan Rating: `{}`\nAvg. Daily Battles: `{}`\nAvg. PR: `{}`\nPlayers: `{}`",
                    data.rating.efficiency.value.round(),
                    data.rating.battles_count_avg_daily.value.round(),
                    data.rating.global_rating_weighted_avg.value.round(),
                    data.tomato.members_count
                    ),true);

    embed.field("Stronghold Stats",
                format!(
                    "SH Tier X ELO: `{}`\nSH Tier VIII ELO: `{}`\nSH Tier VI ELO: `{}`",
                     data.rating.fb_elo_rating_10.value.round(),
                     data.rating.fb_elo_rating_8.value.round(),
                     data.rating.fb_elo_rating_6.value.round(),
                    ),true);
    embed.field("Global Map Stats",
                format!(
                    "Global Map ELO: `{}`\nGlobal Map WR: `{}`\nProvinces: `{}`",
                    data.rating.gm_elo_rating_10.value.round(),
                    format!("{:.1}%", (data.global.statistics.wins_10_level as f32/
                            data.global.statistics.battles_10_level as f32)*100.0),
                    data.global.statistics.provinces_count,
                       ),true);
    let mut color_str = data.tomato.color.clone();
    color_str.remove(0);
    embed.color(i32::from_str_radix(&color_str,16).unwrap());
    embed
}

#[poise::command(slash_command)]
pub async fn clanstats(
    ctx: Context<'_>,
    #[description = "Clan Tag"] clan: String,
    #[description = "Select a Region"] region: Region,
    ) -> Result<(), Error> {
        ctx.defer().await?;
        let clan_id;
        let try_clan_id = fetch_clan_id(&region, &clan).await;
        match try_clan_id {
            Some(id) => {clan_id = id;},
            None => {
                ctx.say("Couldn't find a clan with that name").await?;
                return Ok(());
             }
        } 
        let data = fetch_all_clan(&region, clan_id).await;
        let embed = generate_clan_embed(&data).await;
        ctx.send(|f| f.embed(|f| {f.clone_from(&embed); f})).await?;
        Ok(())
}



