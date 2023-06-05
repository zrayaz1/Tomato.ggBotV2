use serde::Deserialize;

#[derive(Deserialize)]
struct ApiResponse {
    meta: MetaData,
    data: Vec<Tank>,
}

#[derive(Deserialize)]
struct MetaData {
    status: String,
}

#[derive(Deserialize, Debug)]
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = "https://api.tomato.gg/dev/api-v2/moe/com";
    let response = client
        .get(url)
        .send()
        .await?;
    let response_text = response
        .text()
        .await?;
    let response: ApiResponse = serde_json::from_str(&response_text)
        .unwrap();
    println!("status: {}", response.meta.status);
    for tank in response.data {
        println!("Tank Name: {}", tank.name);
        println!("ID: {}", tank.id);
        println!("Image URL: {}", tank.image);
        println!("Is Gift: {}", tank.is_gift);
        println!("Is Premium: {}", tank.is_premium);
        println!("Nation: {}", tank.nation);
        println!("Tier: {}", tank.tier);
        println!("Class: {}", tank.class);
        println!("50%: {}", tank.pct_50);
        println!("65%: {}", tank.pct_65);
        println!("85%: {}", tank.pct_85);
        println!("95%: {}", tank.pct_95);
        println!("100%: {}", tank.pct_100);
        println!("-------------------------");
    }
    Ok(())

}
   

























