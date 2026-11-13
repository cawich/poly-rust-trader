use reqwest::Client;
use serde::Deserialize;

pub struct PriceFeed {
    client: Client,
}

impl PriceFeed {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn get_binance_spot_price(&self, asset: &str) -> Option<f64> {
        let symbol = format!("{}USDT", asset.to_uppercase());
        let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", symbol);

        #[derive(Deserialize)]
        struct BinanceTicker {
            price: String,
        }

        let resp: BinanceTicker = self.client.get(&url).send().await.ok()?.json().await.ok()?;
        resp.price.parse::<f64>().ok()
    }
}
