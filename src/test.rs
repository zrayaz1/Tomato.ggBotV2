use serde::Deserialize;
use reqwest::Client;

#[derive(Deserialize)]
struct ApiResponse {
    meta: MetaData,
    data: Vec<Tank>,
}

#[derive(Deserialize)]
struct MetaData {
    status: String,
}

#[derive(Deserialize)]
struct Tank {
    id: u32,
    image: String,
    #[serde(rename = "isGift")]
    is_gift: bool,
    #[serde(rename = "isPrem")]
    is_premium: bool,
    name: String,
    nation: String,
    tier: u32,
    class: String,
    statistics: TankStatistics,
}

#[derive(Deserialize)]
struct TankStatistics {
    #[serde(rename = "50")]
    pct_50: u32,
    #[serde(rename = "65")]
    pct_65: u32,
    #[serde(rename = "85")]
    pct_85: u32,
    #[serde(rename = "95")]
    pct_95: u32,
    #[serde(rename = "100")]
    pct_100: u32,
}

#[tokio::main]
async fn main() {
    let url = "https://api.tomato.gg/dev/api-v2/moe/com";

    let client = Client::new();
    let response = client.get(url).send().await.unwrap();
    let response_text = response.text().await.unwrap();

    let response: ApiResponse = serde_json::from_str(&response_text).unwrap();

    // Accessing the data
    println!("Status: {}", response.meta.status);
    for tank in response.data {
        println!("Tank Name: {}", tank.name);
        println!("ID: {}", tank.id);
        println!("Image URL: {}", tank.image);
        println!("Is Gift: {}", tank.is_gift);
        println!("Is Premium: {}", tank.is_premium);
        println!("Nation: {}", tank.nation);
        println!("Tier: {}", tank.tier);
        println!("Class: {}", tank.class);
        println!("Statistics: {:?}", tank.statistics);
        println!("-------------------------");
    }
}

