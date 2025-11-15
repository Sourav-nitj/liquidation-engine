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

    //  database
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dev:dev@localhost:5432/goquant".to_string());
    let db = PgPool::connect(&db_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;

    // Broadcast channel (for WS events)
    let (tx, _rx) = broadcast::channel::<engine::models::LiquidationEvent>(256);
    let tx = Arc::new(tx);

    // Create engine state
    let state: Arc<engine::EngineState> =
        Arc::new(engine::EngineState::new(db.clone(), tx.clone()).await?);

    // Start engine background tasks
    {
        let s = state.clone();
        tokio::spawn(async move {
            s.start().await;
        });
    }

    // Add a mock BTC position for demo liquidation
    {
        use uuid::Uuid;

        let s = state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let mut positions = s.positions.lock().await;

            positions.push(engine::models::Position {
                id: Uuid::new_v4(),
                owner: "demo-user".to_string(),
                symbol: "BTC-USD".to_string(),
                size: 5000,
                entry_price: 50_000_000_000, // $50,000 scaled
                margin: 1000,                // low margin so it liquidates
                is_long: true,
                leverage: 100,
                open: true,
            });

            println!("Added mock BTC position for demo.");
        });
    }

    // HTTP routes
    let app = Router::new()
        .route("/health", get(api::http::health))
        .route("/insurance", get(api::http::get_insurance))
        .route("/liquidations", get(api::http::get_liquidations))
        .route("/positions/pending", get(api::http::get_pending))
        .route("/ws", get(api::websocket::ws_handler))
        .with_state(state.clone());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
