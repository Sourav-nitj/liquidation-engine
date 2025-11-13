use std::sync::Arc;
use tokio::time::{sleep, Duration};
use log::info;
use crate::engine::{EngineState, models::Position};

pub struct PositionMonitor {
    state: Arc<EngineState>,
}

impl PositionMonitor {
    pub fn new(state: Arc<EngineState>) -> Self { Self { state } }

    pub async fn run(self) {
        loop {
            sleep(Duration::from_millis(1000)).await;

            let positions = { self.state.positions.lock().await.clone() };
            for pos in positions {
                if !pos.open { continue; }
                if let Some(mark) = self.state.oracle.get_mark_price(&pos.symbol).await {
                    // unrealized pnl = size * (mark - entry)  (note: both scaled)
                    let unrealized = if pos.is_long {
                        (pos.size as i128) * (mark as i128 - pos.entry_price as i128)
                    } else {
                        (pos.size as i128) * (pos.entry_price as i128 - mark as i128)
                    };

                    let pos_value = (pos.size as i128) * (mark as i128);
                    if pos_value <= 0 { continue; }

                    // margin ratio as float
                    let margin_ratio = (pos.margin as i128 + unrealized) as f64 / pos_value as f64;
                    let maintenance = maintenance_margin_ratio(pos.leverage);

                    if margin_ratio < maintenance {
                        info!("Position {} liquidatable (ratio {:.6} < {:.6})", pos.id, margin_ratio, maintenance);
                        // we simply log here; executor will pick up and act
                    }
                }
            }
        }
    }
}

fn maintenance_margin_ratio(leverage: u16) -> f64 {
    match leverage {
        1..=20 => 0.025,
        21..=50 => 0.01,
        51..=100 => 0.005,
        101..=500 => 0.0025,
        501..=1000 => 0.001,
        _ => 0.025,
    }
}
