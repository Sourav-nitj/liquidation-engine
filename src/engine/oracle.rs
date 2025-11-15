use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use log::info;

#[derive(Clone)]
pub struct PriceOracle {
    prices: Arc<RwLock<HashMap<String, i64>>>, // scaled price (1e6)
}

impl PriceOracle {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert("BTC-USD".to_string(), 50_000_000_000i64); // 60,000 * 1e6
        m.insert("ETH-USD".to_string(), 2_800_000_000i64);  // 2,800 * 1e6
        Self { prices: Arc::new(RwLock::new(m)) }
    }

    pub async fn get_mark_price(&self, symbol: &str) -> Option<i64> {
        let map = self.prices.read().await;
        map.get(symbol).copied()
    }

    pub async fn start(self: Arc<Self>) {
        loop {
            sleep(Duration::from_millis(1500)).await;
            let mut w = self.prices.write().await;
            if let Some(btc) = w.get_mut("BTC-USD") {

                *btc = (*btc).saturating_sub(500_000);
                info!("Oracle: BTC-USD price updated to {}", *btc);
            }
            if let Some(eth) = w.get_mut("ETH-USD") {
                *eth = (*eth).saturating_sub(20_000);
                info!("Oracle: ETH-USD price updated to {}", *eth);
            }
        }
    }
}
