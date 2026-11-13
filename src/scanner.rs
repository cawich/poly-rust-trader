use reqwest::Client;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::strategy::StrategyEngine;
use crate::pricing::PriceFeed; // Import PriceFeed

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    pub _condition_id: Option<String>,
    pub _end_date: Option<String>,
    pub clob_token_ids: Option<String>,
    pub outcomes: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Event {
    pub title: Option<String>,
    pub _slug: Option<String>,
    pub markets: Option<Vec<Market>>,
}

pub struct PolymarketScanner {
    client: Client,
}

impl PolymarketScanner {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn get_midpoint(&self, token_id: &str) -> Option<String> {
        let url = format!("https://clob.polymarket.com/midpoint?token_id={}", token_id);
        let v: serde_json::Value = self.client.get(&url).send().await.ok()?.json().await.ok()?;
        v.get("mid").and_then(|m| m.as_str()).map(|s| s.to_string())
    }

    pub async fn scan_short_term(
        &self,
        strategy: &StrategyEngine,
        price_feed: &PriceFeed, // Pass the price feed object here
    ) -> Result<usize, reqwest::Error> {
        println!("=== Scanning Short-term Up/Down Markets ===");
        let assets = ["btc", "eth", "sol", "xrp"];
        let intervals = [("5m", 300u64), ("15m", 900u64)];
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut found = 0;

        for asset in assets {
            // Fetch the live Binance spot price for THIS specific asset
            let spot_price = price_feed.get_binance_spot_price(asset).await.unwrap_or(0.0);
            println!("\n>>> Live Spot Price for {}: ${:.2}", asset.to_uppercase(), spot_price);

            for (label, secs) in intervals {
                let current_start = now - (now % secs);
                for start in [current_start, current_start + secs] {
                    let slug = format!("{}-updown-{}-{}", asset, label, start);
                    let url = format!("https://gamma-api.polymarket.com/events?slug={}", slug);

                    let events: Vec<Event> = match self.client.get(&url).send().await {
                        Ok(resp) => resp.json().await.unwrap_or_default(),
                        Err(_) => continue,
                    };

                    for event in events {
                        found += 1;
                        println!("--------------------------------------------------");
                        println!("{}", event.title.unwrap_or_default());
                        
                        if let Some(markets) = event.markets {
                            for m in markets {
                                let tokens: Vec<String> = m.clob_token_ids
                                    .as_deref()
                                    .and_then(|s| serde_json::from_str(s).ok())
                                    .unwrap_or_default();
                                let outcomes: Vec<String> = m.outcomes
                                    .as_deref()
                                    .and_then(|s| serde_json::from_str(s).ok())
                                    .unwrap_or_default();

                                let mut up_price: Option<f64> = None;
                                let mut down_price: Option<f64> = None;

                                for (i, t) in tokens.iter().enumerate() {
                                    let name = outcomes.get(i).cloned().unwrap_or_else(|| format!("Outcome {}", i));
                                    let price_str = self.get_midpoint(t).await.unwrap_or_else(|| "n/a".to_string());
                                    
                                    println!("  {}: {}", name, price_str);

                                    if let Ok(val) = price_str.parse::<f64>() {
                                        if name.eq_ignore_ascii_case("Up") {
                                            up_price = Some(val);
                                        } else if name.eq_ignore_ascii_case("Down") {
                                            down_price = Some(val);
                                        }
                                    }
                                }

                                strategy.evaluate_market(asset, up_price, down_price, spot_price);
                            }
                        }
                    }
                }
            }
        }
        Ok(found)
    }
}
