use serde::Deserialize;
use serde::Deserializer;
use crate::Region;
use tokio::time::Instant;


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
    meta: Meta,
    data: RecentsData,
}

#[derive(Debug, Deserialize)]
struct Meta {
    id: String,
    cached: bool,
}

#[derive(Debug, Deserialize)]
pub struct RecentsData {
    recent24hr: TimeFrame,
    recent3days: TimeFrame,
    recent7days: TimeFrame,
    recent30days: TimeFrame,
    recent1000battles: TimeFrame,
    recent100battles: TimeFrame,
}

#[derive(Debug, Deserialize)]
pub struct TimeFrame {
    overall: OverallStats,
    #[serde(default)]
    tank_stats: Vec<TankStats>,
}

#[derive(Debug, Deserialize)]
pub struct OverallStats {
    battles: u32,
    #[serde(deserialize_with = "deserialize_default")]
    wn8: u32,
    #[serde(deserialize_with = "deserialize_default")]
    tier: f32,
    #[serde(deserialize_with = "deserialize_default")]
    winrate: f32,
    #[serde(deserialize_with = "deserialize_default")]
    dpg: u32,
}

#[derive(Debug, Deserialize)]
pub struct TankStats {
    id: u32,
    name: String,
    tier: u32,
    battles: u32,
    wn8: u32,
    dpg: u32,
    kpg: u32,
}


pub async fn fetch_recent_data(region: &Region, user_id: &u32) -> RecentsData {
    let start = Instant::now();
    let recents_url = 
        format!("https://api.tomato.gg/dev/api-v2/recents/{}/{}?cache=true", region.extension(), user_id);
    let recents_response: RecentsResponse = 
        reqwest::get(recents_url).await.unwrap().json().await.unwrap();
    let duration = start.elapsed();
    println!("fetched recent stats for {} in {:?}", user_id, duration);
    recents_response.data
    
}

