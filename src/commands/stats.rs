use super::clanstats::{fetch_all_clan, generate_clan_embed, ClanData};
use crate::errors::{
    ClanInfoFetchError, CreateMainStatEmbedError, CreatePeriodEmbedError, FetchUserIDError,
};
use crate::player_stats::recents::{fetch_recent_data, RecentsData};
use crate::{get_short_position, get_wn8_color};
use crate::{
    player_stats::{
        overall::{fetch_overall_data, OverallData},
        recents::TimeFrame,
    },
    Context, Error, Region,
};
use poise::serenity_prelude::{
    ComponentType, CreateEmbed, CreateSelectMenu, CreateSelectMenuOption, CreateSelectMenuOptions,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::join;

#[derive(Deserialize)]
pub struct ClanInfoResponse {
    status: String,
    data: HashMap<String, Option<PlayerAccountInfo>>,
}

//TODO rename this so that its clear its for clan data
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

#[derive(Deserialize, Clone, Default)]
pub struct Emblems {
    pub x64: EmblemURL,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EmblemURL {
    pub portal: String,
}

#[derive(Deserialize)]
pub struct UserSearch {
    status: String,
    #[serde(default)]
    data: Vec<Player>,
}

#[derive(Deserialize, Clone, Default)]
pub struct Player {
    pub nickname: String,
    pub account_id: u32,
}

#[derive(Default)]
pub struct PlayerData {
    player_clan: Option<PlayerAccountInfo>,
    clan: Option<ClanData>,
    player: Player,
    region: Region,
    overall: Option<OverallData>,
    recents: Option<RecentsData>,
    is_in_clan: bool,
}

impl PlayerData {
    pub fn get_period_data(&self, period: Period) -> Option<&TimeFrame> {
        if let Some(recents) = &self.recents {
            match period {
                Period::R24HR => {
                    return Some(&recents.recent24hr);
                }
                Period::R3DAYS => {
                    return Some(&recents.recent3days);
                }
                Period::R7DAYS => {
                    return Some(&recents.recent7days);
                }
                Period::R30DAYS => {
                    return Some(&recents.recent30days);
                }
                Period::R60DAYS => {
                    return Some(&recents.recent60days);
                }
                Period::R1000BATTLES => {
                    return Some(&recents.recent1000battles);
                }
                Period::R100BATTLES => {
                    return Some(&recents.recent100battles);
                }
            }
        }
        None
    }
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
}

pub async fn fetch_user_id(
    input: &str,
    region: Region,
) -> Result<Option<Player>, FetchUserIDError> {
    let wot_user_url = format!(
        "https://api.worldoftanks.{}/wot/account/list/?language=en&application_id=42d1c07ba19a98fcbfdf5f3492bff972&search={}",
        region.extension(),
        input
    );

    let response = reqwest::get(wot_user_url)
        .await?
        .json::<UserSearch>()
        .await?;

    let player = &response.data.get(0);
    Ok(player.cloned())
}

pub async fn fetch_clan_info(
    region: &Region,
    account_id: &u32,
) -> Result<Option<PlayerAccountInfo>, ClanInfoFetchError> {
    let clan_info_url = format!(
        "https://api.worldoftanks.{}/wot/clans/accountinfo/?application_id=20e1e0e4254d98635796fc71f2dfe741&account_id={}",
        region.extension(),
        account_id
    );

    let response = reqwest::get(clan_info_url)
        .await?
        .json::<ClanInfoResponse>()
        .await?;

    if let Some(output) = response.data.get(&account_id.to_string()) {
        return Ok(output.clone());
    }

    Ok(None)
}

pub async fn find_user_server(user: &str) -> Option<(Region, Player)> {
    let (na, eu, asia) = join!(
        fetch_user_id(user, Region::NA),
        fetch_user_id(user, Region::EU),
        fetch_user_id(user, Region::ASIA)
    );

    if let Ok(na_response) = na {
        if let Some(player) = na_response {
            return Some((Region::NA, player));
        }
    }

    if let Ok(eu_response) = eu {
        if let Some(player) = eu_response {
            return Some((Region::EU, player));
        }
    }

    if let Ok(asia_response) = asia {
        if let Some(player) = asia_response {
            return Some((Region::ASIA, player));
        }
    }

    None
}

pub async fn generate_period_embed(
    player_data: &PlayerData,
    period: Period,
) -> Result<CreateEmbed, CreatePeriodEmbedError> {
    const DISPLAY_AMMOUNT: usize = 5;
    let mut embed = CreateEmbed::default();
    let data;

    match player_data.get_period_data(period) {
        Some(player_data) => {data = player_data;}
        None => {
            return Err(CreatePeriodEmbedError::MissingRecentsError);
        }
    }

    let mut tanks = data.tank_stats.clone();

    embed
        .title(format!("{}'s Stats", player_data.player.nickname))
        .description(format!("**Last {} Stats**", period.nice_name()))
        .field(
            "Totals",
            format!(
                "Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
                data.overall.battles, data.overall.wn8, data.overall.winrate, data.overall.tier
            ),
            true,
        )
        .color(get_wn8_color(data.overall.wn8));

    tanks.sort_by_key(|tank| tank.battles);
    tanks.reverse();
    tanks.truncate(DISPLAY_AMMOUNT);

    for tank in tanks {
        embed.field(
            format!("{}", tank.name),
            format!(
                "Battles: `{}`\nWin Rate: `{}%`\n WN8: `{}`\n DPG: `{}`",
                tank.battles, tank.win_rate, tank.wn8, tank.dpg
            ),
            true,
        );
    }
    Ok(embed)
}

pub async fn generate_main_stat_embed(
    data: &PlayerData,
) -> Result<CreateEmbed, CreateMainStatEmbedError> {
    let overall;
    let recents;

    match &data.overall {
        Some(data) => {
            overall = data;
        }
        None => return Err(CreateMainStatEmbedError::MissingOverallError),
    }

    match &data.recents {
        Some(data) => {
            recents = data;
        }
        None => return Err(CreateMainStatEmbedError::MissingRecentsError),
    }

    let mut embed = CreateEmbed::default();

    embed.title(format!("{}'s Stats", data.player.nickname));
    embed.url(format!(
        "https://tomato.gg/stats/{}/{}={}",
        data.region.name(),
        data.player.nickname,
        data.player.account_id
    ));

    embed.field(
        "**Overall**",
        format!(
            "Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
            overall.battles, overall.wn8, overall.win_rate, overall.tier
        ),
        true,
    );
    embed.field(
        "**24 Hours**",
        format!(
            "Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
            recents.recent24hr.overall.battles,
            recents.recent24hr.overall.wn8,
            recents.recent24hr.overall.winrate,
            recents.recent24hr.overall.tier
        ),
        true,
    );

    embed.field(
        "**7 Days**",
        format!(
            "Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
            recents.recent7days.overall.battles,
            recents.recent7days.overall.wn8,
            recents.recent7days.overall.winrate,
            recents.recent7days.overall.tier
        ),
        true,
    );

    embed.field(
        "**30 Days**",
        format!(
            "Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
            recents.recent30days.overall.battles,
            recents.recent30days.overall.wn8,
            recents.recent30days.overall.winrate,
            recents.recent30days.overall.tier
        ),
        true,
    );

    embed.field(
        "**60 Days**",
        format!(
            "Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
            recents.recent60days.overall.battles,
            recents.recent60days.overall.wn8,
            recents.recent60days.overall.winrate,
            recents.recent60days.overall.tier
        ),
        true,
    );

    embed.field(
        "**1000 Battles**",
        format!(
            "Battles: `{}`\nWN8: `{}`\nWinRate: `{}%`\nAvgTier: `{}`",
            recents.recent1000battles.overall.battles,
            recents.recent1000battles.overall.wn8,
            recents.recent1000battles.overall.winrate,
            recents.recent1000battles.overall.tier
        ),
        true,
    );
    embed.footer(|f| {
        f.text("Powered by Tomato.gg");
        f.icon_url("https://tomato.gg/_next/image?url=%2Ftomato.png&w=48&q=75");
        f
    });
    embed.color(get_wn8_color(overall.wn8));
    match &data.player_clan {
        Some(clan_info) => {
            embed.thumbnail(&clan_info.clan.emblems.x64.portal);
            embed.description(format!(
                "**{} at [{}]**",
                get_short_position(&clan_info.role),
                &clan_info.clan.tag
            ));
        }
        None => {}
    }
    Ok(embed)
}

pub fn create_select_menu(id: u64) -> CreateSelectMenu {
    let mut options = CreateSelectMenuOptions::default();

    for period in Period::iter() {
        let mut option = CreateSelectMenuOption::default();
        option.label(period.nice_name());
        option.value(period);
        options.add_option(option);
    }

    CreateSelectMenu::default()
        .custom_id(id)
        .min_values(1)
        .max_values(1)
        .options(|o| {
            o.clone_from(&options);
            o
        })
        .to_owned()
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
    let user_region: Region;

    match region {
        Some(region) => {
            user_region = region;

            // this cluster fuck is becuase fetch_user_id is a Result<Option>
            // so have to handle both the error case of the Result and the
            // None case from the option cant just propagate the error up
            // because this function is handled by poise

            match fetch_user_id(user.as_str(), user_region).await {
                Ok(player_response) => match player_response {
                    Some(player) => {
                        user_info = player;
                    }
                    None => {
                        ctx.say("No player found with that name").await?;
                        return Ok(());
                    }
                },
                Err(e) => {
                    ctx.say(format!("Error Fetching User Id {}", e)).await?;
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

    let mut all_data = PlayerData {
        player_clan: None,
        clan: None,
        player: user_info,
        region: user_region,
        overall: None,
        recents: None,
        is_in_clan: false,
    };

    let (cached_overalls_fetch, cached_recents_fetch, player_clan_fetch) = join!(
        fetch_overall_data(&user_region, &all_data.player, true),
        fetch_recent_data(&user_region, &all_data.player, true),
        fetch_clan_info(&user_region, &all_data.player.account_id)
    );

    match player_clan_fetch {
        Ok(data) => {
            all_data.player_clan = data;
        }

        Err(e) => {
            println!("{}", e);
        }
    }

    match cached_overalls_fetch {
        Ok(data) => all_data.overall = data,

        Err(e) => {
            println!("{}", e);
        }
    }

    match cached_recents_fetch {
        Ok(data) => all_data.recents = data,

        Err(e) => {
            println!("{}", e);
        }
    }

    let mut embed: CreateEmbed;

    match period {
        Some(period) => match generate_period_embed(&all_data, period).await {
            Ok(period_embed) => {
                embed = period_embed;
            }
            Err(_) => {
                embed = CreateEmbed::default()
                    .title("Not in Cache... Please wait")
                    .to_owned();
            }
        },

        None => match generate_main_stat_embed(&all_data).await {
            Ok(stat_embed) => {
                embed = stat_embed;
            }
            Err(_) => {
                embed = CreateEmbed::default()
                    .title("Not in Cache... Please wait")
                    .to_owned();
            }
        },
    }

    let message = ctx
        .send(|f| {
            f.embed(|f| {
                f.clone_from(&embed);
                f
            })
        })
        .await?;

    match &all_data.player_clan {
        Some(clan) => {
            all_data.is_in_clan = true;
            let (clan_data, overalls_fetch, recents_fetch) = join!(
                fetch_all_clan(user_region, clan.clan.clan_id),
                fetch_overall_data(&all_data.region, &all_data.player, false),
                fetch_recent_data(&all_data.region, &all_data.player, false),
            );

            match clan_data {
                Ok(data) => {
                    all_data.clan = Some(data);
                }

                Err(e) => {
                    println!("{}", e);
                }
            }

            match overalls_fetch {
                Ok(data) => {
                    if data.is_some() {
                        all_data.overall = data;
                    }
                }

                Err(e) => {
                    println!("{}", e);
                }
            }

            match recents_fetch {
                Ok(data) => {
                    if data.is_some() {
                        all_data.recents = data;
                    }
                }

                Err(e) => {
                    println!("{}", e);
                }
            }
        }
        None => {
            let (overalls_fetch, recents_fetch) = join!(
                fetch_overall_data(&all_data.region, &all_data.player, false),
                fetch_recent_data(&all_data.region, &all_data.player, false),
            );

            match overalls_fetch {
                Ok(data) => {
                    if data.is_some() {
                        all_data.overall = data;
                    }
                }

                Err(e) => {
                    println!("{}", e);
                }
            }

            match recents_fetch {
                Ok(data) => {
                    if data.is_some() {
                        all_data.recents = data;
                    }
                }

                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    }

    match period {
        Some(period) => match generate_period_embed(&all_data, period).await {
            Ok(period_embed) => {embed = period_embed;}

            Err(_) => {
                embed = CreateEmbed::default()
                    .title("User Not Found on Tomato.gg").to_owned();
                message
                    .edit(ctx, |f| {
                        f.embeds.push(embed);
                        f}
                    ).await?;
                return Ok(());
            }
        }

        None => match generate_main_stat_embed(&all_data).await {
            Ok(stat_embed) => { embed = stat_embed;}

            Err(_) => {
                embed = CreateEmbed::default()
                    .title("User Not Found on Tomato.gg").to_owned();
                message
                    .edit(ctx, |f| {
                        f.embeds.push(embed);
                        f}).await?;
                return Ok(());
            }
        }
    }
    

    if all_data.is_in_clan {
        message
            .edit(ctx, |f| {
                f.embed(|f| {
                    f.clone_from(&embed);
                    f
                })
                .components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_select_menu(|sm| {
                            sm.clone_from(&create_select_menu(uuid));
                            sm
                        })
                    })
                    .create_action_row(|ar| {
                        ar.create_button(|b| {
                            b.custom_id(uuid * 2)
                                .style(poise::serenity_prelude::ButtonStyle::Primary)
                                .label("Player Stats")
                        })
                        .create_button(|b| {
                            b.custom_id(uuid * 3)
                                .style(poise::serenity_prelude::ButtonStyle::Success)
                                .label("Clan Stats")
                        })
                    })
                })
            })
            .await?;
    } else {
        message
            .edit(ctx, |f| {
                f.embed(|f| {
                    f.clone_from(&embed);
                    f
                })
                .components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_select_menu(|sm| {
                            sm.clone_from(&create_select_menu(uuid));
                            sm
                        })
                    })
                })
            })
            .await?;
    }

    while let Some(mci) = poise::serenity_prelude::CollectComponentInteraction::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| {
            mci.data.custom_id == uuid.to_string()
                || mci.data.custom_id == (uuid * 2).to_string()
                || mci.data.custom_id == (uuid * 3).to_string()
        })
        .await
    {
        match mci.data.component_type {
            ComponentType::SelectMenu => {
                let period = mci.data.values.first().unwrap();
                embed = generate_period_embed(&all_data, Period::from_str(period).unwrap())
                    .await
                    .unwrap();
                if all_data.is_in_clan {
                    message
                        .edit(ctx, |f| {
                            f.embed(|f| {
                                f.clone_from(&embed);
                                f
                            })
                            .components(|c| {
                                c.create_action_row(|ar| {
                                    ar.create_select_menu(|sm| {
                                        sm.clone_from(&create_select_menu(uuid));
                                        sm
                                    })
                                })
                                .create_action_row(|ar| {
                                    ar.create_button(|b| {
                                        b.custom_id(uuid * 2)
                                            .style(poise::serenity_prelude::ButtonStyle::Primary)
                                            .label("Player Stats")
                                    })
                                    .create_button(|b| {
                                        b.custom_id(uuid * 3)
                                            .style(poise::serenity_prelude::ButtonStyle::Success)
                                            .label("Clan Stats")
                                    })
                                })
                            })
                        })
                        .await?;
                } else {
                    message
                        .edit(ctx, |f| {
                            f.embed(|f| {
                                f.clone_from(&embed);
                                f
                            })
                            .components(|c| {
                                c.create_action_row(|ar| {
                                    ar.create_select_menu(|sm| {
                                        sm.clone_from(&create_select_menu(uuid));
                                        sm
                                    })
                                })
                            })
                        })
                        .await?;
                }
            }
            ComponentType::Button => {
                let player_id = (uuid * 2).to_string();
                let clan_id = (uuid * 3).to_string();
                if mci.data.custom_id.clone() == player_id {
                    embed = generate_main_stat_embed(&all_data).await.unwrap();
                    message
                        .edit(ctx, |f| {
                            f.embed(|f| {
                                f.clone_from(&embed);
                                f
                            })
                            .components(|c| {
                                c.create_action_row(|ar| {
                                    ar.create_select_menu(|sm| {
                                        sm.clone_from(&create_select_menu(uuid));
                                        sm
                                    })
                                })
                                .create_action_row(|ar| {
                                    ar.create_button(|b| {
                                        b.custom_id(uuid * 2)
                                            .style(poise::serenity_prelude::ButtonStyle::Primary)
                                            .label("Player Stats")
                                    })
                                    .create_button(|b| {
                                        b.custom_id(uuid * 3)
                                            .style(poise::serenity_prelude::ButtonStyle::Success)
                                            .label("Clan Stats")
                                    })
                                })
                            })
                        })
                        .await?;
                } else if mci.data.custom_id.clone() == clan_id {
                    embed = generate_clan_embed(all_data.clan.as_ref().unwrap()).await;

                    message
                        .edit(ctx, |f| {
                            f.embed(|f| {
                                f.clone_from(&embed);
                                f
                            })
                            .components(|c| {
                                c.create_action_row(|ar| {
                                    ar.create_button(|b| {
                                        b.custom_id(uuid * 2)
                                            .style(poise::serenity_prelude::ButtonStyle::Primary)
                                            .label("Player Stats")
                                    })
                                    .create_button(|b| {
                                        b.custom_id(uuid * 3)
                                            .style(poise::serenity_prelude::ButtonStyle::Success)
                                            .label("Clan Stats")
                                    })
                                })
                            })
                        })
                        .await?;
                }
            }
            _ => {
                println!("cock");
            }
        }

        mci.create_interaction_response(ctx, |ir| {
            ir.kind(poise::serenity_prelude::InteractionResponseType::DeferredUpdateMessage)
        })
        .await?;
    }
    //removes buttons and select after timeout
    println!("sanity check");
    message.edit(ctx, |f| {
    f.components(|c| c);
    f.embeds.push(embed);
    f
    }).await?;

    Ok(())
}
