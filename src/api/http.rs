use axum::{extract::State, Json};
use serde_json::json;
use std::sync::Arc;
use axum::response::IntoResponse;
use crate::engine::EngineState;
use sqlx::Row;

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
    let rows = sqlx::query(
        r#"SELECT id, position_id, position_owner, liquidator, symbol,
           liquidated_size, liquidation_price, margin_before, margin_after,
           liquidator_reward, bad_debt, created_at
           FROM liquidation_history
           ORDER BY created_at DESC
           LIMIT 50"#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Convert rows to JSON-friendly values using dynamic row getters
    let mapped: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.get::<uuid::Uuid, _>("id"),
                "position_id": r.get::<uuid::Uuid, _>("position_id"),
                "position_owner": r.get::<String, _>("position_owner"),
                "liquidator": r.get::<String, _>("liquidator"),
                "symbol": r.get::<String, _>("symbol"),
                "liquidated_size": r.get::<i64, _>("liquidated_size"),
                "liquidation_price": r.get::<i64, _>("liquidation_price"),
                "margin_before": r.get::<i64, _>("margin_before"),
                "margin_after": r.get::<i64, _>("margin_after"),
                "liquidator_reward": r.get::<i64, _>("liquidator_reward"),
                "bad_debt": r.get::<i64, _>("bad_debt"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            })
        })
        .collect();

    Json(mapped)
}
