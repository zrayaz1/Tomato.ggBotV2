use crate::{player_stats::overall::fetch_overall_data, Region, Error, Context};
use std::time::Instant;
use crate::player_stats::recents::fetch_recent_data;
use serde::Deserialize;
use tokio::join;
use std::collections::HashMap;


#[derive(Deserialize)]
pub struct ClanInfoResponse {
    status: String,
    meta: MetaData,
    data: HashMap<String, PlayerAccountInfo>,
}

#[derive(Deserialize, Clone)]
pub struct PlayerAccountInfo {
    clan: PlayerClanInfo,
    account_id: u32,
    role_i18n: String,
    joined_at: u64,
    role: String,
    account_name: String,
}

#[derive(Deserialize, Clone)]
pub struct PlayerClanInfo {
    members_count: u32,
    name: String,
    color: String,
    created_at: u64,
    tag: String,
    emblems: Emblems,
    clan_id: u32,
}


#[derive(Deserialize, Clone)]
pub struct Emblems {
    x64: EmblemURL,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmblemURL {
    portal: String,
}

#[derive(Deserialize)]
pub struct UserSearch {
    status: String,
    meta: MetaData,
    data: Vec<Player>,
}

#[derive(Deserialize)]
pub struct MetaData {
    count: u32,
}

#[derive(Deserialize)]
pub struct Player {
    nickname: String,
    account_id: u32,
}




pub async fn fetch_user_id(input: &str, region: &Region) -> Option<u32>{
    let start = Instant::now();
    let wot_user_url = format!("https://api.worldoftanks.{}/wot/account/list/?language=en&application_id=42d1c07ba19a98fcbfdf5f3492bff972&search={}",
                           region.extension(), input);
    let response: UserSearch = reqwest::get(wot_user_url).await.unwrap().json().await.unwrap();
    let duration = start.elapsed();
    println!("Fetched user_id for {} in {:?}",input ,duration);
    if response.data.len() > 0 {
        println!("{}", response.data[0].account_id);
        return Some(response.data[0].account_id);
    }
    None
}

pub async fn fetch_clan_info(region: &Region, account_id: &u32) -> PlayerAccountInfo {
    let start = Instant::now();
    let clan_info_url = format!("https://api.worldoftanks.{}/wot/clans/accountinfo/?application_id=20e1e0e4254d98635796fc71f2dfe741&account_id={}",
                                region.extension(), account_id);
    let response: ClanInfoResponse = reqwest::get(clan_info_url).await.unwrap().json().await.unwrap();
    let duration = start.elapsed();
    println!("Fetched Clan Info for {}, in {:?}", account_id, duration);
    let output = response.data.get(&account_id.to_string()).unwrap();
    return output.clone();
}

pub async fn find_user_server(user: &str) -> Option<(Region, u32)> {
    let (na, eu, asia) = join!(
        fetch_user_id(&user, &Region::NA),
        fetch_user_id(&user, &Region::EU),
        fetch_user_id(&user, &Region::ASIA));         
    match na {
        Some(id) => {return Some((Region::NA, id));}
        None => {}
    }
    match eu {
        Some(id) => {return Some((Region::EU, id));}
        None => {}
    }
    match asia {
        Some(id) => {return Some((Region::ASIA, id));}
        None => {}
    }
    None

}

#[poise::command(slash_command)]
pub async fn stats(
    ctx: Context<'_>,
    #[description = "Players Username"] user: String,
    #[description = "Select a Region"] region: Option<Region>,
    ) -> Result<(), Error> {
    let start = Instant::now();
    ctx.defer().await?;
    let user_id;
    let user_region;
    match region{
        Some(region) => {
            user_region = region;
            match fetch_user_id(&user, &user_region).await {
                Some(id) => {user_id = id}
                None => {
                    ctx.say("No user found with that name").await?;
                    return Ok(());
                }
            }
        }
        None => {
             let response = find_user_server(&user).await;
             match response {
                Some((server, id)) => {
                    user_id = id;
                    user_region = server;
                }
                None => {
                    ctx.say("No Player found with that name").await?;
                    return Ok(());
                }
             }
        } 
    }
    
    let (overalls, recents, clan_info) = join!(
        fetch_overall_data(&user_region, &user_id),
        fetch_recent_data(&user_region, &user_id),
        fetch_clan_info(&user_region, &user_id));

    let duration = start.elapsed();

    ctx.say(format!("cock balls {:?}", duration)).await?;
    Ok(())
}











