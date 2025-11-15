use std::sync::Arc;
use tokio::time::{sleep, Duration};
use log::{info, error};
use sqlx::PgPool;
use crate::engine::{EngineState, models::{LiquidationRecord, LiquidationEvent}};
use chrono::Utc;
use uuid::Uuid;

pub struct LiquidationExecutor {
    state: Arc<EngineState>,
    db: PgPool,
}

impl LiquidationExecutor {
    pub fn new(state: Arc<EngineState>) -> Self {
        Self { db: state.db.clone(), state }
    }

    pub async fn run(self) {
        loop {
            sleep(Duration::from_millis(1200)).await;

            // We'll lock and iterate mutably
            let mut positions = self.state.positions.lock().await;
            for pos in positions.iter_mut() {
                if !pos.open { continue; }

                if let Some(mark) = self.state.oracle.get_mark_price(&pos.symbol).await {
                    // compute unrealized PnL (i128 to avoid overflow)
                    let unrealized = if pos.is_long {
                        (pos.size as i128) * (mark as i128 - pos.entry_price as i128)
                    } else {
                        (pos.size as i128) * (pos.entry_price as i128 - mark as i128)
                    };

                    let pos_value = (pos.size as i128) * (mark as i128);
                    if pos_value <= 0 { continue; }

                    let margin_ratio = (pos.margin as i128 + unrealized) as f64 / pos_value as f64;
                    let mm = maintenance_margin(pos.leverage);

                    if margin_ratio < mm {
                        // Partial liquidation: reduce by 50% (min 1)
                        let reduction = ((pos.size as i64) / 2).max(1) as i64;

                        // compute liquidated value and reward (2.5%)
                        let liquidated_value = (reduction as i128) * (mark as i128);
                        let reward = ((liquidated_value as i128 * 25) / 1000) as i64; // 2.5%

                        // store margin_before
                        let margin_before = (pos.margin as i128 + unrealized) as i64;

                        // apply reduction
                        pos.size = (pos.size as i64 - reduction) as i64;
                        if pos.size <= 0 {
                            pos.open = false;
                        }

                        // compute new unrealized & margin_after
                        let new_unrealized = if pos.is_long {
                            (pos.size as i128) * (mark as i128 - pos.entry_price as i128)
                        } else {
                            (pos.size as i128) * (pos.entry_price as i128 - mark as i128)
                        };
                        let margin_after = (pos.margin as i128 + new_unrealized) as i64;

                        // prepare liquidation record
                        let record = LiquidationRecord {
                            id: Uuid::new_v4(),
                            position_id: pos.id,
                            position_owner: pos.owner.clone(),
                            liquidator: "executor".into(),
                            symbol: pos.symbol.clone(),
                            liquidated_size: reduction,
                            liquidation_price: mark,
                            margin_before,
                            margin_after,
                            liquidator_reward: reward,
                            bad_debt: 0,
                            timestamp: Utc::now(),
                        };

                        // persist
                        if let Err(e) = self.insert_record(&record).await {
                            error!("DB insert failed: {:?}", e);
                        } else {
                            let _ = self.state.event_tx.send(LiquidationEvent { record: record.clone() });
                            info!("Executed partial liquidation for pos {} reduction {}", pos.id, reduction);
                        }

                        // if position now zero or margin after negative -> full liquidation handling
                        if pos.size <= 0 || margin_after < 0 {
                            // compute bad debt if any
                            let bd = if margin_after < 0 {
                                let deficit = (-margin_after) as i64;
                                let mut ins = self.state.insurance.lock().await;
                                let cover = deficit.min(ins.balance);
                                ins.balance = ins.balance.saturating_sub(cover);
                                ins.total_bad_debt_covered = ins.total_bad_debt_covered.saturating_add(cover);
                                cover
                            } else { 0i64 };

                            let record_full = LiquidationRecord {
                                id: Uuid::new_v4(),
                                position_id: pos.id,
                                position_owner: pos.owner.clone(),
                                liquidator: "executor".into(),
                                symbol: pos.symbol.clone(),
                                liquidated_size: pos.size,
                                liquidation_price: mark,
                                margin_before: margin_after,
                                margin_after: 0,
                                liquidator_reward: 0,
                                bad_debt: bd,
                                timestamp: Utc::now(),
                            };

                            if let Err(e) = self.insert_record(&record_full).await {
                                error!("DB insert failed (full): {:?}", e);
                            } else {
                                let _ = self.state.event_tx.send(LiquidationEvent { record: record_full.clone() });
                                info!("Executed full liquidation for pos {} bad_debt {}", pos.id, bd);
                            }

                            pos.open = false;
                        }
                    }
                }
            }
        }
    }

    async fn insert_record(&self, rec: &LiquidationRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO liquidation_history (
                    id, position_id, position_owner, liquidator, symbol,
                    liquidated_size, liquidation_price, margin_before, margin_after,
                    liquidator_reward, bad_debt, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
            )
            .bind(rec.id)
            .bind(rec.position_id)
            .bind(&rec.position_owner)
            .bind(&rec.liquidator)
            .bind(&rec.symbol)
            .bind(rec.liquidated_size)
            .bind(rec.liquidation_price)
            .bind(rec.margin_before)
            .bind(rec.margin_after)
            .bind(rec.liquidator_reward)
            .bind(rec.bad_debt)
            .bind(rec.timestamp)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

fn maintenance_margin(leverage: u16) -> f64 {
    match leverage {
        1..=20 => 0.025,
        21..=50 => 0.01,
        51..=100 => 0.005,
        101..=500 => 0.0025,
        501..=1000 => 0.001,
        _ => 0.025,
    }
}
