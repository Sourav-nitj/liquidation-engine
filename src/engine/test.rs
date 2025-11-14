#[cfg(test)]
mod tests {
    use crate::engine::models::Position;

    #[test]
    fn test_margin_ratio_calculation() {
        let pos = Position {
            id: 1,
            user: "user123".to_string(),
            symbol: "BTC".to_string(),
            size: 100.0,
            entry_price: 30000.0,
            collateral: 500.0,
            is_long: true,
            leverage: 50,
        };

        let mark_price = 29500.0;
        let unrealized_pnl = pos.size * (mark_price - pos.entry_price); // -500

        let position_value = pos.size * mark_price; // 2,950,000
        let margin_ratio = (pos.collateral + unrealized_pnl) / position_value;

        assert!(margin_ratio < 0.005); // should trigger liquidation
    }
}
