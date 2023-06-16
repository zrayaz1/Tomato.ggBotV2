use serde::Deserialize;
use crate::Region;
use crate::commands::stats::fetch_user_id;
use tokio::time::Instant;


#[derive(Deserialize)]
struct OverallResponse {
    meta: Meta,
    data: OverallData,
}

#[derive(Deserialize)]
struct Meta {
    status: String,
    id: String,
    cached: bool,
}

#[derive(Deserialize)]
pub struct OverallData {
    server: String,
    id: u32,
    battles: u32,
    #[serde(rename = "overallWN8")]
    wn8: u32,
    #[serde(rename = "avgTier")]
    tier: f32,
    #[serde(rename = "winrate")]
    win_rate: f32,
    dpg: u32,
}



pub async fn fetch_overall_data(region: &Region, user_id: &u32) -> OverallData {
    let start = Instant::now();

    let overall_url = 
        format!("https://api.tomato.gg/dev/api-v2/overall/{}/{}?cache=true", region.extension(), user_id);
    let overall_response: OverallResponse = 
        reqwest::get(overall_url).await.unwrap().json().await.unwrap();
    let duration = start.elapsed();
    println!("fetched overall starts for {} in {:?}", user_id, duration);
    overall_response.data
}

















