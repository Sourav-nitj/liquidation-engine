use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub owner: String,
    pub symbol: String,
    pub size: i64,           
    pub entry_price: i64,    
    pub margin: i64,
    pub is_long: bool,
    pub leverage: u16,
    pub open: bool,
}

impl Position {
    pub fn seed_defaults() -> Vec<Self> {
        vec![
            Position {
                id: Uuid::new_v4(),
                owner: "alice".into(),
                symbol: "BTC-USD".into(),
                size: 100,
                entry_price: 65_000_000_000i64,
                margin: 5_000,
                is_long: true,
                leverage: 100,
                open: true,
            },
            Position {
                id: Uuid::new_v4(),
                owner: "bob".into(),
                symbol: "ETH-USD".into(),
                size: 200,
                entry_price: 3_000_000_00i64, // 3,000 * 1e6 (note: consistent scaling)
                margin: 3_000,
                is_long: false,
                leverage: 50,
                open: true,
            },
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InsuranceFund {
    pub balance: i64,
    pub total_contributions: i64,
    pub total_bad_debt_covered: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiquidationRecord {
    pub id: Uuid,
    pub position_id: Uuid,
    pub position_owner: String,
    pub liquidator: String,
    pub symbol: String,
    pub liquidated_size: i64,
    pub liquidation_price: i64,
    pub margin_before: i64,
    pub margin_after: i64,
    pub liquidator_reward: i64,
    pub bad_debt: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiquidationEvent {
    pub record: LiquidationRecord,
}
