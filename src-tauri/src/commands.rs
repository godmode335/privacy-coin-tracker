use serde::Serialize;
use rust_decimal::Decimal;
use tauri::State;
use crate::coins;
use crate::db::Db;
use crate::models::{NewTransaction, Portfolio, Transaction};
use crate::portfolio::{compute_holdings, value_holdings, Valuation};
use crate::prices::PriceClient;

pub struct AppState {
    pub db: Db,
    pub prices: PriceClient,
}

#[derive(Serialize)]
pub struct Dashboard {
    pub portfolio_id: i64,
    pub valuation: Valuation,
    pub price_as_of: Option<i64>,
}

#[derive(Serialize)]
pub struct CoinMeta {
    pub id: String,
    pub symbol: String,
    pub name: String,
}

pub fn build_dashboard(db: &Db, portfolio_id: i64) -> Result<Dashboard, String> {
    let txs = db.list_transactions(portfolio_id).map_err(|e| e.to_string())?;
    let holdings = compute_holdings(&txs);
    let (prices, as_of) = db.cached_prices().map_err(|e| e.to_string())?;
    let valuation = value_holdings(&holdings, &prices);
    Ok(Dashboard { portfolio_id, valuation, price_as_of: as_of })
}

#[tauri::command]
pub fn list_coins() -> Vec<CoinMeta> {
    coins::all().iter().map(|c| CoinMeta {
        id: c.id.to_string(), symbol: c.symbol.to_string(), name: c.name.to_string(),
    }).collect()
}

#[tauri::command]
pub fn list_portfolios(state: State<AppState>) -> Result<Vec<Portfolio>, String> {
    state.db.list_portfolios().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_portfolio(state: State<AppState>, name: String, now: i64) -> Result<Portfolio, String> {
    state.db.create_portfolio(&name, now).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_transaction(state: State<AppState>, tx: NewTransaction) -> Result<Transaction, String> {
    if coins::by_id(&tx.coin_id).is_none() {
        return Err(format!("unsupported coin: {}", tx.coin_id));
    }
    state.db.add_transaction(&tx).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_transactions(state: State<AppState>, portfolio_id: i64) -> Result<Vec<Transaction>, String> {
    state.db.list_transactions(portfolio_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_dashboard(state: State<AppState>, portfolio_id: i64) -> Result<Dashboard, String> {
    build_dashboard(&state.db, portfolio_id)
}

#[tauri::command]
pub async fn refresh_prices(state: State<'_, AppState>, now: i64) -> Result<Option<i64>, String> {
    crate::scheduler::refresh_once(&state.db, &state.prices, now).await?;
    let (_, as_of) = state.db.cached_prices().map_err(|e| e.to_string())?;
    Ok(as_of)
}

#[tauri::command]
pub async fn get_coin_history(state: State<'_, AppState>, coin_id: String, days: u32) -> Result<Vec<(i64, Decimal)>, String> {
    let coin = coins::by_id(&coin_id).ok_or_else(|| format!("unknown coin: {coin_id}"))?;
    state.prices.fetch_history(coin.coingecko_id, days).await
}

#[tauri::command]
pub fn get_portfolio_history(state: State<AppState>, portfolio_id: i64) -> Result<Vec<(i64, Decimal)>, String> {
    state.db.list_snapshots(portfolio_id).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use crate::models::{NewTransaction, TxType};
    use std::str::FromStr;
    fn d(s: &str) -> rust_decimal::Decimal { rust_decimal::Decimal::from_str(s).unwrap() }

    #[test]
    fn build_dashboard_values_holdings_from_cache() {
        let db = Db::open_in_memory().unwrap();
        let p = db.create_portfolio("HODL", 1).unwrap();
        db.add_transaction(&NewTransaction {
            portfolio_id: p.id, coin_id: "zcash".into(), tx_type: TxType::Buy,
            quantity: d("2"), price_usd: d("100"), fee_usd: rust_decimal::Decimal::ZERO,
            ts: 10, note: String::new(),
        }).unwrap();
        db.upsert_price("zcash", d("150"), d("0.002"), 999).unwrap();

        let dash = build_dashboard(&db, p.id).unwrap();
        assert_eq!(dash.valuation.total_value_usd, d("300"));
        assert_eq!(dash.price_as_of, Some(999));
    }
}
