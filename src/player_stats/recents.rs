use crate::commands::stats::Player;
use crate::errors::FetchRecentsDataError;
use crate::Region;
use serde::Deserialize;
use serde::Deserializer;

fn deserialize_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct RecentsResponse {
    data: RecentsData,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RecentsData {
    pub recent24hr: TimeFrame,
    pub recent3days: TimeFrame,
    pub recent7days: TimeFrame,
    pub recent30days: TimeFrame,
    pub recent60days: TimeFrame,
    pub recent1000battles: TimeFrame,
    pub recent100battles: TimeFrame,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TimeFrame {
    pub overall: OverallStats,
    #[serde(rename = "tankStats")]
    #[serde(default)]
    pub tank_stats: Vec<TankStats>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct OverallStats {
    pub battles: u32,
    #[serde(deserialize_with = "deserialize_default")]
    pub wn8: u32,
    #[serde(deserialize_with = "deserialize_default")]
    pub tier: f32,
    #[serde(deserialize_with = "deserialize_default")]
    pub winrate: f32,
    #[serde(deserialize_with = "deserialize_default")]
    pub dpg: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TankStats {
    pub id: u32,
    pub name: String,
    pub tier: u32,
    pub battles: u32,
    pub wn8: u32,
    pub dpg: u32,
    pub kpg: f32,
    #[serde(rename = "winrate")]
    pub win_rate: f32,
}

pub async fn fetch_recent_data(
    region: &Region,
    user: &Player,
    cached: bool,
) -> Result<Option<RecentsData>, FetchRecentsDataError> {
    let url = format!(
        "{}{}",
        format!(
            "https://api.tomato.gg/dev/api-v2/recents/{}/{}",
            region.extension(),
            user.account_id
        ),
        match cached {
            true => "?cache=true",
            false => "",
        }
    );

    let parsed_data = reqwest::get(url).await?.json::<RecentsResponse>().await;

    match parsed_data {
        Ok(data) => Ok(Some(data.data)),
        Err(_) => Ok(None),
    }
}
