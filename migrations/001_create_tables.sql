CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS liquidation_history (
  id uuid PRIMARY KEY,
  position_id uuid,
  position_owner text,
  liquidator text,
  symbol text,
  liquidated_size bigint,
  liquidation_price bigint,
  margin_before bigint,
  margin_after bigint,
  liquidator_reward bigint,
  bad_debt bigint,
  created_at timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS insurance_fund (
  id serial PRIMARY KEY,
  balance bigint DEFAULT 0,
  total_contributions bigint DEFAULT 0,
  total_bad_debt_covered bigint DEFAULT 0,
  updated_at timestamptz DEFAULT now()
);
