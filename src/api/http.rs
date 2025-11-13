use axum::{extract::State, Json};
use serde_json::json;
use std::sync::Arc;
use axum::response::IntoResponse;
use crate::engine::EngineState;

pub async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

pub async fn get_insurance(State(state): State<Arc<EngineState>>) -> impl IntoResponse {
    let insurance = state.insurance.lock().await.clone();
    Json(insurance)
}

pub async fn get_pending(State(state): State<Arc<EngineState>>) -> impl IntoResponse {
    let positions = state.positions.lock().await.clone();
    Json(positions)
}

pub async fn get_liquidations(State(state): State<Arc<EngineState>>) -> impl IntoResponse {
    let rows = sqlx::query!(
        r#"SELECT id, position_id, position_owner, liquidator, symbol,
           liquidated_size, liquidation_price, margin_before, margin_after,
           liquidator_reward, bad_debt, created_at
           FROM liquidation_history
           ORDER BY created_at DESC
           LIMIT 50"#
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Convert rows to JSON-friendly values
    let mapped: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "position_id": r.position_id,
                "position_owner": r.position_owner,
                "liquidator": r.liquidator,
                "symbol": r.symbol,
                "liquidated_size": r.liquidated_size,
                "liquidation_price": r.liquidation_price,
                "margin_before": r.margin_before,
                "margin_after": r.margin_after,
                "liquidator_reward": r.liquidator_reward,
                "bad_debt": r.bad_debt,
                "created_at": r.created_at,
            })
        })
        .collect();

    Json(mapped)
}
