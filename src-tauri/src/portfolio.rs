use std::collections::HashMap;
use rust_decimal::Decimal;
use crate::models::{Transaction, TxType};

#[derive(Debug, Clone)]
pub struct Holding {
    pub coin_id: String,
    pub quantity: Decimal,
    pub avg_cost_usd: Decimal,
    pub realized_pnl_usd: Decimal,
}

pub fn compute_holdings(txs: &[Transaction]) -> Vec<Holding> {
    use std::collections::BTreeMap;
    let mut sorted: Vec<&Transaction> = txs.iter().collect();
    sorted.sort_by_key(|t| t.ts);

    let mut map: BTreeMap<String, Holding> = BTreeMap::new();
    for t in sorted {
        let h = map.entry(t.coin_id.clone()).or_insert(Holding {
            coin_id: t.coin_id.clone(),
            quantity: Decimal::ZERO,
            avg_cost_usd: Decimal::ZERO,
            realized_pnl_usd: Decimal::ZERO,
        });
        match t.tx_type {
            TxType::Buy => {
                let new_qty = h.quantity + t.quantity;
                if new_qty > Decimal::ZERO {
                    let cost = h.avg_cost_usd * h.quantity + t.price_usd * t.quantity + t.fee_usd;
                    h.avg_cost_usd = cost / new_qty;
                }
                h.quantity = new_qty;
            }
            TxType::Sell => {
                h.realized_pnl_usd += (t.price_usd - h.avg_cost_usd) * t.quantity - t.fee_usd;
                h.quantity -= t.quantity;
                if h.quantity < Decimal::ZERO {
                    h.quantity = Decimal::ZERO;
                }
            }
        }
    }

    map.into_values()
        .filter(|h| h.quantity > Decimal::ZERO || h.realized_pnl_usd != Decimal::ZERO)
        .collect()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValuedCoin {
    pub coin_id: String,
    pub quantity: Decimal,
    pub avg_cost_usd: Decimal,
    pub price_usd: Decimal,
    pub value_usd: Decimal,
    pub unrealized_pnl_usd: Decimal,
    pub unrealized_pnl_pct: Decimal,
    pub realized_pnl_usd: Decimal,
    pub allocation_pct: Decimal,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Valuation {
    pub total_value_usd: Decimal,
    pub total_cost_usd: Decimal,
    pub total_unrealized_pnl_usd: Decimal,
    pub coins: Vec<ValuedCoin>,
}

pub fn value_holdings(holdings: &[Holding], prices: &HashMap<String, Decimal>) -> Valuation {
    let mut coins = Vec::new();
    let mut total_value = Decimal::ZERO;
    let mut total_cost = Decimal::ZERO;

    for h in holdings {
        let price = prices.get(&h.coin_id).copied().unwrap_or(Decimal::ZERO);
        let value = price * h.quantity;
        let cost = h.avg_cost_usd * h.quantity;
        let unrealized = value - cost;
        let unrealized_pct = if cost > Decimal::ZERO {
            unrealized / cost * Decimal::from(100)
        } else {
            Decimal::ZERO
        };
        total_value += value;
        total_cost += cost;
        coins.push(ValuedCoin {
            coin_id: h.coin_id.clone(),
            quantity: h.quantity,
            avg_cost_usd: h.avg_cost_usd,
            price_usd: price,
            value_usd: value,
            unrealized_pnl_usd: unrealized,
            unrealized_pnl_pct: unrealized_pct.round_dp(2),
            realized_pnl_usd: h.realized_pnl_usd,
            allocation_pct: Decimal::ZERO,
        });
    }

    for c in coins.iter_mut() {
        c.allocation_pct = if total_value > Decimal::ZERO {
            (c.value_usd / total_value * Decimal::from(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };
    }

    Valuation {
        total_value_usd: total_value,
        total_cost_usd: total_cost,
        total_unrealized_pnl_usd: total_value - total_cost,
        coins,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn buy(coin: &str, qty: &str, price: &str, ts: i64) -> Transaction {
        Transaction { id: 0, portfolio_id: 1, coin_id: coin.into(), tx_type: TxType::Buy,
            quantity: d(qty), price_usd: d(price), fee_usd: Decimal::ZERO, ts, note: String::new() }
    }
    fn sell(coin: &str, qty: &str, price: &str, ts: i64) -> Transaction {
        Transaction { id: 0, portfolio_id: 1, coin_id: coin.into(), tx_type: TxType::Sell,
            quantity: d(qty), price_usd: d(price), fee_usd: Decimal::ZERO, ts, note: String::new() }
    }

    #[test]
    fn single_buy_sets_qty_and_avg_cost() {
        let h = compute_holdings(&[buy("zcash", "2", "100", 1)]);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].quantity, d("2"));
        assert_eq!(h[0].avg_cost_usd, d("100"));
    }

    #[test]
    fn two_buys_average_cost() {
        let h = compute_holdings(&[buy("zcash", "1", "100", 1), buy("zcash", "1", "200", 2)]);
        assert_eq!(h[0].quantity, d("2"));
        assert_eq!(h[0].avg_cost_usd, d("150"));
    }

    #[test]
    fn sell_records_realized_pnl_and_keeps_avg_cost() {
        let txs = [buy("zcash", "2", "100", 1), sell("zcash", "1", "180", 2)];
        let h = compute_holdings(&txs);
        assert_eq!(h[0].quantity, d("1"));
        assert_eq!(h[0].avg_cost_usd, d("100"));
        assert_eq!(h[0].realized_pnl_usd, d("80"));
    }

    #[test]
    fn value_holdings_computes_unrealized_and_allocation() {
        let h = compute_holdings(&[buy("zcash", "2", "100", 1)]);
        let mut prices = HashMap::new();
        prices.insert("zcash".to_string(), d("150"));
        let v = value_holdings(&h, &prices);
        assert_eq!(v.total_value_usd, d("300"));
        assert_eq!(v.total_unrealized_pnl_usd, d("100"));
        assert_eq!(v.coins[0].allocation_pct, d("100"));
        assert_eq!(v.coins[0].unrealized_pnl_pct, d("50"));
    }
}
