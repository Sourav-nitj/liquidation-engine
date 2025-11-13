-GoQuant Liquidation Engine 

This is a basic liquidation engine built in Rust.  
It simulates mark prices, checks positions, executes partial/full liquidations,  
and stores liquidation history in PostgreSQL.

This project is built only for demonstration as required in the assignment.


## How It Works (Simple Explanation)

### 1. Oracle (Fake Price Feed)
- Updates BTC & ETH prices every ~1.5 seconds.
- Prices keep going down slowly.

### 2. Position Monitor
- Checks all positions every 1 second.
- Calculates:
  - unrealized PnL  
  - margin ratio  
  - if liquidation is needed  

### 3. Liquidation Executor
- If margin ratio < maintenance margin:
  - reduces position by 50% (partial liquidation)
  - calculates liquidator reward
  - if still not sufficient → full liquidation
- Saves liquidation record in the database
- Sends event to WebSocket clients

### 4. API Endpoints
- `GET /health` — check if server is running  
- `GET /liquidations` — recent liquidation history  
- `GET /insurance` — insurance fund balance  
- `GET /positions/pending` — open positions  
- `ws://localhost:8080/ws` — live liquidation events  

---

## How to Run

### 1. Start PostgreSQL
```
docker compose up -d
```

### 2. Export database URL
```
export DATABASE_URL=postgres://dev:dev@localhost:5432/goquant
```

### 3. Run migrations
```
cargo sqlx migrate run
```

### 4. Start engine
```
cargo run
```

Server runs at:
```
http://localhost:8080
```

That’s it.



- Positions are seeded (predefined).
- Liquidation threshold depends on leverage.
- Margin ratio = (margin + PnL) / (position value)
- Partial liquidation = 50% size cut.
- Records stored in `liquidation_history`.
- Prices are integers (scaled).
- Oracle is simulated (no external feed).


