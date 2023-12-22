use std::collections::HashMap;

use serde::Deserialize;
use poise::serenity_prelude::CreateEmbed;
use crate::errors::*;
use crate::{Region, Error, Context};

use super::stats::Emblems;


#[derive(Deserialize)]
pub struct ClanIdResponse {
    data: Option<Vec<ClanId>>,
}


#[derive(Deserialize, Default)]
pub struct ClanId {
    clan_id: u32,
}


#[derive(Deserialize, Default, Clone)]
pub struct TomatoClan {
    pub name: String,
    pub tag: String,
    pub color: String,
    motto: String,
    pub emblems: Emblems,
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

#[derive(Deserialize, Clone, Default)]
pub struct GlobalClanData {
    statistics: GlobalStatistics,
}

#[derive(Deserialize, Clone, Default)]
pub struct GlobalStatistics {
    battles_10_level: u32,
    wins_10_level: u32,
    provinces_count: u32,
}


#[derive(Deserialize)]
pub struct RatingResponse {
    data: HashMap<String, RatingClanData>,
}

#[derive(Deserialize, Clone, Default)]
pub struct RatingClanData {
    efficiency: Value,
    battles_count_avg_daily: Value,
    global_rating_weighted_avg: Value,
    fb_elo_rating_10: Value,
    fb_elo_rating_8: Value,
    fb_elo_rating_6: Value,
    gm_elo_rating_10: Value,
}

#[derive(Deserialize, Clone, Default)]
pub struct Value {
    value: f32,
}

#[derive(Clone)]
pub struct ClanData {
    pub rating: Option<RatingClanData>,
    pub global: Option<GlobalClanData>,
    pub tomato: TomatoClan,
}

pub async fn fetch_global_map(region: Region, clan_id: u32)
    -> Result<Option<GlobalClanData>, GlobalMapFetchError> {
    let global_map_url = format!(
        "https://api.worldoftanks.{}/wot/globalmap/claninfo/?application_id=20e1e0e4254d98635796fc71f2dfe741&clan_id={}",
        region.extension(), 
        clan_id
    );

    let response = reqwest::get(global_map_url)
        .await?
        .json::<GlobalResponse>()
        .await?;

    Ok(response
        .data
        .get(&clan_id.to_string())
        .cloned())
}

pub async fn fetch_clan_rating(region: Region, clan_id: u32) -> Result<Option<RatingClanData>, ClanRatingFetchError> {

    let rating_url = format!(
        "https://api.worldoftanks.{}/wot/clanratings/clans/?application_id=20e1e0e4254d98635796fc71f2dfe741&clan_id={}",
        region.extension(), 
        clan_id
    );

    let parsed_response = reqwest::get(rating_url)
        .await?
        .json::<RatingResponse>()
        .await?;

    Ok(parsed_response
        .data
        .get(&clan_id.to_string())
        .cloned()
    )
}

pub async fn fetch_tomato_clan(region: Region, clan_id: u32) -> Result<TomatoClan, TomatoClanFetchError> {
    let tomato_url = format!("https://api.tomato.gg/api/clan/{}/{}",
                             region.extension(), clan_id);

    let parsed_response = reqwest::get(tomato_url)
    .await?
    .json::<TomatoClan>()
    .await?;

    Ok(parsed_response)

}

pub async fn fetch_all_clan(region: Region, clan_id: u32) -> Result<ClanData, FetchAllClanDataError> {
    let (global_map_result, clan_rating_result, tomato_clan_result) = tokio::join!(
            fetch_global_map(region, clan_id),
            fetch_clan_rating(region, clan_id),
            fetch_tomato_clan(region, clan_id)
        );

        let (rating, global, tomato) = (clan_rating_result?, global_map_result?, tomato_clan_result?);

    Ok(ClanData {
        rating,
        global,
        tomato,
    })
}

pub async fn fetch_clan_id(region: Region, clan: &str) -> Result<u32, FetchClanIDError> {
    let id_url = format!("https://api.worldoftanks.{}/wot/clans/list/?application_id=20e1e0e4254d98635796fc71f2dfe741&search={}",
                         region.extension(), clan);
    let response = reqwest::get(id_url)
    .await?
    .json::<ClanIdResponse>()
    .await?;

    match response.data {
        Some(data) => Ok(data[0].clan_id),
        None => Err(FetchClanIDError::EmptyResponse)
    }
}


pub async fn generate_clan_embed(data: &ClanData) -> CreateEmbed {
    let tomato = &data.tomato;
    let rating = data.rating.clone().unwrap_or_default();
    let global = data.global.clone().unwrap_or_default();

    
    CreateEmbed::default()
        .title(format!("[{}] {}", tomato.tag, tomato.name))
        .thumbnail(&tomato.emblems.x64.portal)
        .description(&tomato.motto)
        
        .field("Player Stats", 
            format!("Overall WN8: `{}`\nOverall WR: `{:.1}%`\nRecent WN8: `{}`\nRecent WR: `{:.1}`",
                tomato.overall_wn8.round(),
                tomato.overall_winrate,
                tomato.recent_wn8.round(),
                tomato.recent_winrate,
                ),true)
        .field("General Stats",
            format!(
                "Clan Rating: `{}`\nAvg. Daily Battles: `{}`\nAvg. PR: `{}`\nPlayers: `{}`",
                rating.efficiency.value.round(),
                rating.battles_count_avg_daily.value.round(),
                rating.global_rating_weighted_avg.value.round(),
                tomato.members_count
            ),true)
        .field("Stronghold Stats",
            format!(
                "SH Tier X ELO: `{}`\nSH Tier VIII ELO: `{}`\nSH Tier VI ELO: `{}`",
                rating.fb_elo_rating_10.value.round(),
                rating.fb_elo_rating_8.value.round(),
                rating.fb_elo_rating_6.value.round(),
            ),true)
        .field("Global Map Stats",
            format!(
                "Global Map ELO: `{}`\nGlobal Map WR: `{}`\nProvinces: `{}`",
                rating.gm_elo_rating_10.value.round(),
                format!("{:.1}%", (global.statistics.wins_10_level as f32/
                    global.statistics.battles_10_level as f32)*100.0),
                    global.statistics.provinces_count,
                ),true
            ).color(i32::from_str_radix(&tomato.color[1..],16).unwrap())
        .to_owned()

}

#[poise::command(slash_command)]
pub async fn clanstats(
    ctx: Context<'_>,
    #[description = "Clan Tag"] clan: String,
    #[description = "Select a Region"] region: Region,
) -> Result<(), Error> {

    ctx.defer().await?;

    let clan_id_result = fetch_clan_id(region, clan.as_str()).await;

    if let Err(e) = clan_id_result {
        ctx.say("Couldn't find a clan with that name").await?;
        return Err(Box::new(e));
    }

    let embed = generate_clan_embed(
        &fetch_all_clan(region, clan_id_result.unwrap())
        .await?
    )
    .await;

    ctx.send(|f| 
        f.embed(|f| {
            f.clone_from(&embed); 
            f
        })).await?;

    Ok(())
}
