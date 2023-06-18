use crate::{player_stats::{overall::{fetch_overall_data, OverallData}, recents::TimeFrame}, Region, Error, Context};
use std::{time::Instant, str::FromStr};
use crate::player_stats::recents::{fetch_recent_data, RecentsData};
use crate::{get_wn8_color, get_short_position};
use serde::Deserialize;
use strum_macros::EnumIter;
use tokio::join;
use std::collections::HashMap;
use poise::{serenity_prelude::{CreateEmbed, CreateSelectMenuOption, CreateSelectMenuOptions, CreateSelectMenu}, CreateReply};
use strum::{IntoEnumIterator};

#[derive(Deserialize)]
pub struct ClanInfoResponse {
    status: String,
    data: HashMap<String, Option<PlayerAccountInfo>>,
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
    pub x64: EmblemURL,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmblemURL {
    pub portal: String,
}

#[derive(Deserialize)]
pub struct UserSearch {
    status: String,
    data: Vec<Player>,
}


#[derive(Deserialize, Clone, Default)]
pub struct Player {
    pub nickname: String,
    pub account_id: u32,
}

#[derive(Default)]
pub struct PlayerData {
    wot_clan: Option<PlayerAccountInfo>,
    player: Player,
    region: Region,
    overall: OverallData,
    recents: RecentsData,
}

#[derive(poise::ChoiceParameter, Clone, Copy, EnumIter)]
pub enum Period {
    R24HR,
    R3DAYS,
    R7DAYS,
    R30DAYS,
    R60DAYS,
    R1000BATTLES,
    R100BATTLES,
}


impl Period {
    fn nice_name(&self) -> &str {
        match self {
            Period::R24HR => "24 Hours",
            Period::R3DAYS => "3 Days",
            Period::R7DAYS => "7 Days",
            Period::R30DAYS => "30 Days",
            Period::R60DAYS => "60 Days",
            Period::R1000BATTLES => "1000 Battles",
            Period::R100BATTLES => "100 Battles",
        }
    }
    fn get_period_data(&self, data: &PlayerData) -> TimeFrame{
        match self {
            Period::R24HR => data.recents.recent24hr.clone(),
            Period::R3DAYS => data.recents.recent3days.clone(),
            Period::R7DAYS => data.recents.recent3days.clone(),
            Period::R30DAYS => data.recents.recent30days.clone(),
            Period::R60DAYS => data.recents.recent60days.clone(),
            Period::R1000BATTLES => data.recents.recent1000battles.clone(),
            Period::R100BATTLES => data.recents.recent100battles.clone(),
        }
    }
}

pub async fn fetch_user_id(input: &str, region: &Region) -> Option<Player>{
    let start = Instant::now();
    let wot_user_url = format!("https://api.worldoftanks.{}/wot/account/list/?language=en&application_id=42d1c07ba19a98fcbfdf5f3492bff972&search={}",
                           region.extension(), input);
    let response: UserSearch = reqwest::get(wot_user_url).await.unwrap().json().await.unwrap();
    let duration = start.elapsed();
    println!("Fetched user_id for {} in {:?}",input ,duration);
    if response.data.len() > 0 {
        let player = &response.data[0];
        return Some(player.clone());
    }
    None
}

pub async fn fetch_clan_info(region: &Region, player: &Player) -> Option<PlayerAccountInfo> {
    let start = Instant::now();
    let clan_info_url = format!("https://api.worldoftanks.{}/wot/clans/accountinfo/?application_id=20e1e0e4254d98635796fc71f2dfe741&account_id={}",
                                region.extension(), player.account_id);
    let response: ClanInfoResponse = reqwest::get(clan_info_url).await.unwrap().json().await.unwrap();
    let duration = start.elapsed();
    println!("Fetched Clan Info for {}, in {:?}", player.account_id, duration);
    let output = response.data.get(&player.account_id.to_string()).unwrap();
    match output {
        Some(clan_info) => {Some(clan_info.clone())}
        None => {None}
    }
}

pub async fn find_user_server(user: &str) -> Option<(Region, Player)> {
    let (na, eu, asia) = join!(
        fetch_user_id(&user, &Region::NA),
        fetch_user_id(&user, &Region::EU),
        fetch_user_id(&user, &Region::ASIA));         
    match na {
        Some(player) => {return Some((Region::NA, player));}
        None => {}
    }
    match eu {
        Some(player) => {return Some((Region::EU, player));}
        None => {}
    }
    match asia {
        Some(player) => {return Some((Region::ASIA, player));}
        None => {}
    }
    None
}


pub async fn generate_period_embed(player_data: &PlayerData, period: Period) -> CreateEmbed {
    const DISPLAY_AMMOUNT: usize = 5;
    let mut embed = CreateEmbed::default();
    let data = period.get_period_data(player_data);
    let mut tanks = data.tank_stats;
    embed.title(format!("{}'s Stats", player_data.player.nickname));
    embed.description(format!("**Last {} Stats**",period.nice_name()));
    embed.field("Totals", 
                format!("Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
                        data.overall.battles,
                        data.overall.wn8,
                        data.overall.winrate,
                        data.overall.tier),true);

    tanks.sort_by_key(|tank| tank.battles);
    tanks.reverse();
    println!("{}",&tanks.len());
    if tanks.len() > DISPLAY_AMMOUNT {
        tanks.truncate(DISPLAY_AMMOUNT);
    }
    for tank in tanks {
        embed.field(format!("{}",tank.name),
        format!("Battles: `{}`\nWin Rate: `{}%`\n WN8: `{}`\n DPG: `{}`",
                tank.battles,
                tank.win_rate,
                tank.wn8,
                tank.dpg),true);
    }

    embed
}

pub async fn generate_main_stat_embed(data: &PlayerData) -> CreateEmbed{
    let mut embed = CreateEmbed::default();
    embed.title(format!("{}'s Stats", data.player.nickname));
    embed.url(format!("https://tomato.gg/stats/{}/{}={}",
              data.region.name(),
              data.player.nickname,
              data.player.account_id));
    
    embed.field("**Overall**", 
                format!("Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
                        data.overall.battles,
                        data.overall.wn8,
                        data.overall.win_rate,
                        data.overall.tier),true);
    embed.field("**24 Hours**",
                format!("Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
                        data.recents.recent24hr.overall.battles,
                        data.recents.recent24hr.overall.wn8,
                        data.recents.recent24hr.overall.winrate,
                        data.recents.recent24hr.overall.tier),true);

    embed.field("**7 Days**",
                format!("Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
                        data.recents.recent7days.overall.battles,
                        data.recents.recent7days.overall.wn8,
                        data.recents.recent7days.overall.winrate,
                        data.recents.recent7days.overall.tier),true);

    embed.field("**30 Days**",
                format!("Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
                        data.recents.recent30days.overall.battles,
                        data.recents.recent30days.overall.wn8,
                        data.recents.recent30days.overall.winrate,
                        data.recents.recent30days.overall.tier),true);

    embed.field("**60 Days**",
                format!("Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
                        data.recents.recent60days.overall.battles,
                        data.recents.recent60days.overall.wn8,
                        data.recents.recent60days.overall.winrate,
                        data.recents.recent60days.overall.tier),true);

    embed.field("**1000 Battles**",
                format!("Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
                        data.recents.recent1000battles.overall.battles,
                        data.recents.recent1000battles.overall.wn8,
                        data.recents.recent1000battles.overall.winrate,
                        data.recents.recent1000battles.overall.tier),true);
    embed.footer(|f| {
        f.text("Powered by Tomato.gg");
        f.icon_url("https://tomato.gg/_next/image?url=%2Ftomato.png&w=48&q=75");
        f});
    embed.color(get_wn8_color(data.overall.wn8));
    match &data.wot_clan {
        Some(clan_info) => {
            embed.thumbnail(&clan_info.clan.emblems.x64.portal);
            embed.description(format!("**{} at [{}]**",get_short_position(&clan_info.role),&clan_info.clan.tag));
        }
        None => {}
    }
    embed
}

pub fn create_select_menu(id: u64) -> CreateSelectMenu {
    let mut options = CreateSelectMenuOptions::default();
    let mut select_menu = CreateSelectMenu::default(); 

    for period in Period::iter() {
        let mut option = CreateSelectMenuOption::default();
        option.label(period.nice_name());
        option.value(period);
        options.add_option(option);
    }
    select_menu.custom_id(id);
    select_menu.min_values(1);
    select_menu.max_values(1);
    select_menu.options(|o| {o.clone_from(&options); o});
    select_menu
}

#[poise::command(slash_command)]
pub async fn stats(
    ctx: Context<'_>,
    #[description = "Players Username"] user: String,
    #[description = "Select a Region"] region: Option<Region>,
    #[description = "Detailed Stats for a Period"] period: Option<Period>,
    ) -> Result<(), Error> {
    ctx.defer().await?;
    let uuid = ctx.id();
    let user_info;
    let user_region;
    match region{
        Some(region) => {
            user_region = region;
            match fetch_user_id(&user, &user_region).await {
                Some(player) => {user_info = player}
                None => {
                    ctx.say("No user found with that name").await?;
                  return Ok(());
                }
            }
        }
        None => {
             let response = find_user_server(&user).await;
             match response {
                Some((server, player)) => {
                    user_info = player;
                    user_region = server;
                }
                None => {
                    ctx.say("No Player found with that name").await?;
                    return Ok(());
                }
             }
        } 
    }
    
    let (mut overalls, mut recents, clan_info) = join!(
        fetch_overall_data(&user_region, &user_info, true),
        fetch_recent_data(&user_region, &user_info, true),
        fetch_clan_info(&user_region, &user_info));


    let mut all_data = PlayerData {
        wot_clan: clan_info.clone(),
        player: user_info.clone(),
        region: user_region.clone(),
        overall: overalls.clone(),
        recents: recents.clone(),
    };

    let mut embed;
    match period {
        Some(period) => {
            embed = generate_period_embed(&all_data, period).await;
        }
        None => {
            embed = generate_main_stat_embed(&all_data).await;
        }
    }
    let message = ctx.send(|f| {f.embed(|f| {f.clone_from(&embed);f})}).await?;
    (overalls, recents) = join!(
        fetch_overall_data(&user_region,&user_info, false),
        fetch_recent_data(&user_region, &user_info, false),
        );
    all_data.recents = recents;
    all_data.overall = overalls;
    match period {
        Some(period) => {
            embed = generate_period_embed(&all_data, period).await;
        }
        None => {
            embed = generate_main_stat_embed(&all_data).await;
        }
    }
   message.edit(ctx, |f| {f.embed(|f| {f.clone_from(&embed);f})
        .components(|c| {
            c.create_action_row(|ar| {
                ar.create_select_menu(|sm| {
                    sm.clone_from(&create_select_menu(uuid)); sm
                })
            })
        })
    }).await?;
    while let Some(mci) = poise::serenity_prelude::CollectComponentInteraction::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == uuid.to_string())
        .await 
        {    
            let period = mci.data.values.first().unwrap();
            println!("{}",period);
            embed = generate_period_embed(&all_data, Period::from_str(period).unwrap()).await;
            
            message.edit(ctx, |f| {f.embed(|f| {f.clone_from(&embed);f})
                .components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_select_menu(|sm| {
                            sm.clone_from(&create_select_menu(uuid)); sm
                        })
                    })
                })
            }).await?;
            mci.create_interaction_response(ctx, |ir| {
            ir.kind(poise::serenity_prelude::InteractionResponseType::DeferredUpdateMessage)
            })
            .await?; 
            
        }
    Ok(())
}















