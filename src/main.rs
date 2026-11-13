mod scanner;
mod pricing;
mod strategy;

use reqwest::Client;
use scanner::PolymarketScanner;
use pricing::PriceFeed;
use strategy::StrategyEngine;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Initialize our modular components
    let scanner = PolymarketScanner::new(client.clone());
    let price_feed = PriceFeed::new(client.clone());
    let  mut strategy = StrategyEngine::new(0.60); // Max buy price of 60 cents

    let poll_interval_secs = 15;
    println!("Starting Modular Polymarket Autonomous Bot (Multi-Asset)...\n");

    loop {
        println!("==================================================");
        println!("Running scan cycle at timestamp: {}", 
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );

        // Pass both strategy and price_feed into the scanner
        if let Err(e) = scanner.scan_short_term(&mut strategy, &price_feed).await {
            println!("Scanner error: {}", e);
        }

        println!("Sleeping for {} seconds...\n", poll_interval_secs);
        tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval_secs)).await;
    }
}
