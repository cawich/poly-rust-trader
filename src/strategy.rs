pub struct StrategyEngine {
    max_buy_price: f64, // e.g., only buy shares priced <= 0.60 (60 cents) to ensure good profit margin
}

impl StrategyEngine {
    pub fn new(max_buy_price: f64) -> Self {
        Self { max_buy_price }
    }

    /// Evaluates whether current market conditions warrant a $1 micro-bet.
    /// Returns Some("UP" or "DOWN") if criteria are met, or None.
    pub fn evaluate_market(
        &self,
        asset: &str,
        up_price: Option<f64>,
        down_price: Option<f64>,
        spot_price: f64,
    ) -> Option<String> {
        println!("[Strategy] Analyzing {} | Spot: ${:.2}", asset, spot_price);

        // Example Strategy Rule for High Probability:
        // If "Up" shares are cheap (e.g. <= 0.55) AND we want to test execution logic, 
        // or you can add momentum thresholds here once you track strike prices.
        
        if let Some(up) = up_price {
            if up <= self.max_buy_price && up > 0.05 {
                // For demonstration, if Up is within our favorable pricing window:
                println!("[Strategy] -> Signal Triggered: Buying UP for ${:.2} shares", up);
                return Some("UP".to_string());
            }
        }

        if let Some(down) = down_price {
            if down <= self.max_buy_price && down > 0.05 {
                println!("[Strategy] -> Signal Triggered: Buying DOWN for ${:.2} shares", down);
                return Some("DOWN".to_string());
            }
        }

        None
    }
}
