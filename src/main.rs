use reqwest::Client;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Market {
    question: Option<String>,
    condition_id: Option<String>,
    end_date: Option<String>,
    clob_token_ids: Option<String>, // JSON array stored as a string
    outcomes: Option<String>,       // JSON array stored as a string, e.g. ["Up","Down"]
}

#[derive(Deserialize, Debug)]
struct Event {
    title: Option<String>,
    slug: Option<String>,
    markets: Option<Vec<Market>>,
}

// Whole-word match so "method" doesn't match "eth"
fn is_crypto(question: &str) -> bool {
    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| {
            matches!(
                w,
                "btc" | "bitcoin" | "eth" | "ethereum" | "sol" | "solana" | "xrp" | "crypto"
            )
        })
}

// Midpoint price (average of best bid and ask) for one outcome token
async fn midpoint(client: &Client, token_id: &str) -> Option<String> {
    let url = format!("https://clob.polymarket.com/midpoint?token_id={}", token_id);
    let v: serde_json::Value = client.get(&url).send().await.ok()?.json().await.ok()?;
    v.get("mid").and_then(|m| m.as_str()).map(|s| s.to_string())
}

// Part 1: long-dated crypto markets from the general listing
async fn scan_general(client: &Client) -> Result<usize, reqwest::Error> {
    println!("=== Part 1: General crypto markets ===");

    let page_size: usize = 100;
    let max_pages: usize = 5;
    let mut total = 0;
    let mut count = 0;

    for page in 0..max_pages {
        let url = format!(
            "https://gamma-api.polymarket.com/markets?active=true&closed=false&limit={}&offset={}",
            page_size,
            page * page_size
        );

        let markets: Vec<Market> = client.get(&url).send().await?.json().await?;
        let n = markets.len();
        total += n;

        for m in markets {
            if let Some(q) = &m.question {
                if is_crypto(q) {
                    count += 1;
                    println!("--------------------------------------------------");
                    println!("Market #{}: {}", count, q);
                    println!("Condition ID: {}", m.condition_id.clone().unwrap_or_default());
                }
            }
        }

        if n < page_size {
            break;
        }
    }

    println!("\nScanned {} markets, found {} crypto matches.\n", total, count);
    Ok(count)
}

// Part 2: recurring 5m / 15m up-down markets, fetched by slug, with live prices
async fn scan_short_term(client: &Client) -> Result<usize, reqwest::Error> {
    println!("=== Part 2: Short-term Up/Down markets ===");

    let assets = ["btc", "eth", "sol", "xrp"];
    let intervals = [("5m", 300u64), ("15m", 900u64)];

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut found = 0;

    for asset in assets {
        for (label, secs) in intervals {
            let current_start = now - (now % secs);

            // current window and the next one
            for start in [current_start, current_start + secs] {
                let slug = format!("{}-updown-{}-{}", asset, label, start);
                let url = format!("https://gamma-api.polymarket.com/events?slug={}", slug);

                let events: Vec<Event> = match client.get(&url).send().await {
                    Ok(resp) => match resp.json().await {
                        Ok(e) => e,
                        Err(e) => {
                            println!("{}: parse error: {}", slug, e);
                            continue;
                        }
                    },
                    Err(e) => {
                        println!("{}: request error: {}", slug, e);
                        continue;
                    }
                };

                // Empty list = that asset/interval/window doesn't exist
                for event in events {
                    found += 1;
                    println!("--------------------------------------------------");
                    println!("{}", event.title.unwrap_or_default());
                    println!("Slug: {}", event.slug.unwrap_or_default());

                    if let Some(markets) = event.markets {
                        for m in markets {
                            println!("Condition ID: {}", m.condition_id.clone().unwrap_or_default());
                            println!("Ends: {}", m.end_date.clone().unwrap_or_default());

                            let tokens: Vec<String> = m
                                .clob_token_ids
                                .as_deref()
                                .and_then(|s| serde_json::from_str(s).ok())
                                .unwrap_or_default();
                            let outcomes: Vec<String> = m
                                .outcomes
                                .as_deref()
                                .and_then(|s| serde_json::from_str(s).ok())
                                .unwrap_or_default();

                            for (i, t) in tokens.iter().enumerate() {
                                let name = outcomes
                                    .get(i)
                                    .cloned()
                                    .unwrap_or_else(|| format!("Outcome {}", i));
                                let price = midpoint(client, t)
                                    .await
                                    .unwrap_or_else(|| "n/a".to_string());
                                println!("  {}: {}", name, price);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\nFound {} short-term markets.\n", found);
    Ok(found)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = Client::new();

    let general = scan_general(&client).await?;
    let short = scan_short_term(&client).await?;

    println!("=== Summary ===");
    println!("General crypto markets: {}", general);
    println!("Short-term Up/Down markets: {}", short);

    Ok(())
}
