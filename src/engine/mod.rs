pub mod models;
pub mod oracle;
pub mod position_monitor;
pub mod liquidation_executor;

#[cfg(test)]
mod test;

use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use crate::engine::models::{InsuranceFund, Position};
use crate::engine::oracle::PriceOracle;
use crate::engine::position_monitor::PositionMonitor;
use crate::engine::liquidation_executor::LiquidationExecutor;

pub struct EngineState {
    pub db: PgPool,
    pub oracle: Arc<PriceOracle>,
    pub positions: Arc<Mutex<Vec<Position>>>,
    pub insurance: Arc<Mutex<InsuranceFund>>,
    pub event_tx: Arc<broadcast::Sender<crate::engine::models::LiquidationEvent>>,
}

impl EngineState {
    pub async fn new(
        db: PgPool,
        event_tx: Arc<broadcast::Sender<crate::engine::models::LiquidationEvent>>,
    ) -> anyhow::Result<Self> {
        let positions = Position::seed_defaults();

        let insurance = InsuranceFund {
            balance: 1_000_000,
            total_contributions: 1_000_000,
            total_bad_debt_covered: 0,
        };

        Ok(Self {
            db,
            oracle: Arc::new(PriceOracle::new()),
            positions: Arc::new(Mutex::new(positions)),
            insurance: Arc::new(Mutex::new(insurance)),
            event_tx,
        })
    }

    pub async fn start(self: Arc<Self>) {
        // clone values we'll move into join
        let oracle = self.oracle.clone();
        let monitor = PositionMonitor::new(self.clone());
        let executor = LiquidationExecutor::new(self.clone());

        tokio::join!(
            async move { oracle.start().await },
            async move { monitor.run().await },
            async move { executor.run().await }
        );
    }
}
