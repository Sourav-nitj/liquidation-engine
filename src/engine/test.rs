#[cfg(test)]
mod tests {
    use crate::engine::models::Position;
    use uuid::Uuid;

    #[test]
    fn test_margin_ratio_calculation() {
        // Construct a Position matching the types in `models::Position` (scaled integers)
        let pos = Position {
            id: Uuid::new_v4(),
            owner: "user123".to_string(),
            symbol: "BTC-USD".to_string(),
            size: 100, // units
            entry_price: 30_000_000_000i64, // 30,000 * 1e6
            margin: 20_000_000_000i64, // scaled margin
            is_long: true,
            leverage: 50,
            open: true,
        };

        // mark price slightly below entry to create negative unrealized PnL
        let mark = 29_500_000_000i64; // 29,500 * 1e6

        // compute unrealized pnl using same logic as the engine
        let unrealized = if pos.is_long {
            (pos.size as i128) * (mark as i128 - pos.entry_price as i128)
        } else {
            (pos.size as i128) * (pos.entry_price as i128 - mark as i128)
        };

        let pos_value = (pos.size as i128) * (mark as i128);
        assert!(pos_value > 0);

        let margin_ratio = (pos.margin as i128 + unrealized) as f64 / pos_value as f64;

        // For these numbers margin_ratio should be negative (thus below maintenance)
        assert!(margin_ratio < 0.01);
    }
}
