use goquant_liquidation_backend::engine::liquidation_engine::LiquidationEngine;

#[tokio::test]
async fn test_liquidation_trigger_flow() {
    let engine = LiquidationEngine::new();

    let pos = engine.position_manager.add_mock_position(
        "BTC",
        100.0,
        30000.0,
        true,
        50,
        200.0
    ).await.unwrap();

    engine.oracle.set_mock_price("BTC", 29500.0).await.unwrap();

    let result = engine.check_all_positions().await;

    assert!(result.is_ok());
}

