use std::net::SocketAddr;
use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::broadcast;
use log::info;

mod engine;
mod api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    // Connect to Postgres (falls back to default)
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dev:dev@localhost:5432/goquant".to_string());
    let db = PgPool::connect(&db_url).await?;

    // Run migrations (migrations folder)
    sqlx::migrate!("./migrations").run(&db).await?;

    // broadcast channel for liquidation events
    let (tx, _rx) = broadcast::channel::<engine::models::LiquidationEvent>(256);
    let tx = Arc::new(tx);

    // create engine state
    let state: Arc<engine::EngineState> =
        Arc::new(engine::EngineState::new(db.clone(), tx.clone()).await?);

    // start engine background tasks
    {
        let state = state.clone();
        tokio::spawn(async move {
            state.start().await;
        });
    }

    // HTTP + WS server
    let app = Router::new()
        .route("/health", get(api::http::health))
        .route("/insurance", get(api::http::get_insurance))
        .route("/liquidations", get(api::http::get_liquidations))
        .route("/positions/pending", get(api::http::get_pending))
        .route("/ws", get(api::websocket::ws_handler))
        .with_state(state.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
