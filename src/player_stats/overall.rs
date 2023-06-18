use serde::Deserialize;
use crate::Region;
use crate::commands::stats::Player;
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

#[derive(Deserialize, Clone, Default)]
pub struct OverallData {
    pub server: String,
    pub id: u32,
    pub battles: u32,
    #[serde(rename = "overallWN8")]
    pub wn8: u32,
    #[serde(rename = "avgTier")]
    pub tier: f32,
    #[serde(rename = "winrate")]
    pub win_rate: f32,
    pub dpg: u32,
}



pub async fn fetch_overall_data(region: &Region, user: &Player, cached: bool) -> OverallData {
    let start = Instant::now();
    let overall_url;
    match cached {
        true => { 
        overall_url = 
            format!("https://api.tomato.gg/dev/api-v2/overall/{}/{}?cache=true", region.extension(), user.account_id);
        }
        false => {
        overall_url = 
            format!("https://api.tomato.gg/dev/api-v2/overall/{}/{}", region.extension(), user.account_id);
        }
    }

    let overall_response: OverallResponse = 
        reqwest::get(overall_url).await.unwrap().json().await.unwrap();
    let duration = start.elapsed();
    println!("fetched overall starts for {} in {:?}", user.account_id, duration);
    overall_response.data
}

















