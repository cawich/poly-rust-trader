/*pub struct StrategyEngine {
    max_buy_price: f64, }

impl StrategyEngine {
    pub fn new(max_buy_price: f64) -> Self {
        Self { max_buy_price }
    }

 
 
    pub fn evaluate_market(
        &self,
        asset: &str,
        up_price: Option<f64>,
        down_price: Option<f64>,
        spot_price: f64,
    ) -> Option<String> {
        println!("[Strategy] Analyzing {} | Spot: ${:.2}", asset, spot_price);

       
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
}*/

use std::collections::VecDeque;

#[derive(Debug)]
struct SimulatedPosition {
    asset: String,
    side: String,      // "UP" or "DOWN"
    entry_price: f64,  // What you paid for the share (e.g., 0.45)
    entry_spot: f64,   // The spot price when you entered
    expires_at: u64,   // Unix timestamp when the window closes
}

pub struct StrategyEngine {
    max_buy_price: f64,
    simulated_bankroll: f64,
    open_positions: VecDeque<SimulatedPosition>,
}

impl StrategyEngine {
    pub fn new(max_buy_price: f64) -> Self {
        Self {
            max_buy_price,
            simulated_bankroll: 10.00, // $100 starting paper bankroll
            open_positions: VecDeque::new(),
        }
    }

    pub fn evaluate_market(
        &mut self,
        asset: &str,
        up_price: Option<f64>,
        down_price: Option<f64>,
        spot_price: f64,
        _current_time: u64,
    ) {
        // 1. RESOLVE EXPIRED POSITIONS FIRST
        self.resolve_positions(asset, spot_price, _current_time);

        println!(
            "[Simulation Engine] Analyzing {} | Spot: ${:.2} | Virtual Bankroll: ${:.2} | Active Positions: {}",
            asset.to_uppercase(),
            spot_price,
            self.simulated_bankroll,
            self.open_positions.len()
        );

        // 2. SELECT STRICTLY ONE SIDE (Mutually Exclusive - No Hedging)
        let chosen_action = match (up_price, down_price) {
            (Some(up), Some(down)) => {
                if up <= self.max_buy_price && down <= self.max_buy_price {
                    // Both under threshold: pick the cheaper one to maximize value
                    if up < down {
                        Some(("UP", up))
                    } else {
                        Some(("DOWN", down))
                    }
                } else if up <= self.max_buy_price {
                    Some(("UP", up))
                } else if down <= self.max_buy_price {
                    Some(("DOWN", down))
                } else {
                    None
                }
            }
            (Some(up), None) if up <= self.max_buy_price => Some(("UP", up)),
            (None, Some(down)) if down <= self.max_buy_price => Some(("DOWN", down)),
            _ => None,
        };

        // 3. EXECUTE SINGLE SIMULATED TRADE
        if let Some((side, price)) = chosen_action {
            let bet_size = 1.00; // $1.00 micro-bet
            if self.simulated_bankroll >= bet_size {
                self.simulated_bankroll -= bet_size;
                let expires_at = _current_time + 900; // 15 minutes ahead

                println!(
                    "  [SIMULATED TRADE] 🛒 Placed $1.00 BUY on {} ({}) at price ${:.3} | Bankroll: ${:.2}",
                    asset.to_uppercase(), side, price, self.simulated_bankroll
                );

                self.open_positions.push_back(SimulatedPosition {
                    asset: asset.to_string(),
                    side: side.to_string(),
                    entry_price: price,
                    entry_spot: spot_price,
                    expires_at,
                });
            }
        }
    }

    /// Checks if any open positions have reached their expiration time and settles them
    fn resolve_positions(&mut self, current_asset: &str, current_spot: f64, current_time: u64) {
        let mut remaining_positions = VecDeque::new();

        while let Some(pos) = self.open_positions.pop_front() {
            if pos.asset == current_asset && current_time >= pos.expires_at {
                let spot_went_up = current_spot > pos.entry_spot;
                let spot_went_down = current_spot < pos.entry_spot;

                let won = (pos.side == "UP" && spot_went_up) || (pos.side == "DOWN" && spot_went_down);

                if won {
                    let shares = 1.00 / pos.entry_price;
                    let payout = shares * 1.00;
                    self.simulated_bankroll += payout;

                    println!(
                        "  [SETTLEMENT] 🎉 WIN! {} {} position expired. Spot moved from ${:.2} to ${:.2}. Payout: +${:.2} | Bankroll: ${:.2}",
                        pos.asset.to_uppercase(), pos.side, pos.entry_spot, current_spot, payout, self.simulated_bankroll
                    );
                } else {





                println!(
                        "  [SETTLEMENT] ❌ LOSS. {} {} position expired. Spot moved from ${:.2} to ${:.2}. Payout: $0.00 | Bankroll: ${:.2}",
                        pos.asset.to_uppercase(), pos.side, pos.entry_spot, current_spot, self.simulated_bankroll
                    );}



            } else {
                remaining_positions.push_back(pos);
            }
        }

        self.open_positions = remaining_positions;
    }
}
