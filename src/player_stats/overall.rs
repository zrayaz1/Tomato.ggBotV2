use serde::Deserialize;
use crate::Region;
use crate::commands::stats::Player;
use crate::errors::FetchOverallDataError;

#[derive(Deserialize)]
struct OverallResponse {
    data: OverallData,
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



pub async fn fetch_overall_data(region: Region, user: Player, cached: bool) -> Result<OverallData, FetchOverallDataError> {

    let url = format!("{}{}{}{}",
        "https://api.tomato.gg/dev/api-v2/overall/{}/{}",
        match cached { true => "?cache=true", false => "" },
        region.extension(), 
        user.account_id
    );

    let data = reqwest::get(url)
        .await?
        .json::<OverallResponse>()
        .await?
        .data;

    Ok(data)
}
