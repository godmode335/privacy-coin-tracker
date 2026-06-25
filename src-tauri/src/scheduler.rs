use crate::commands::AppState;
use crate::db::Db;
use crate::prices::PriceClient;
use crate::{coins, portfolio};
use tauri::Manager;

pub async fn refresh_once(db: &Db, prices: &PriceClient, now: i64) -> Result<(), String> {
    let cg_ids = coins::coingecko_ids();
    let fetched = prices.fetch_prices(&cg_ids).await?;
    for c in coins::all() {
        if let Some((usd, btc)) = fetched.get(c.coingecko_id) {
            db.upsert_price(c.id, *usd, *btc, now).map_err(|e| e.to_string())?;
        }
    }
    let (prices_map, _) = db.cached_prices().map_err(|e| e.to_string())?;
    for p in db.list_portfolios().map_err(|e| e.to_string())? {
        let txs = db.list_transactions(p.id).map_err(|e| e.to_string())?;
        let holdings = portfolio::compute_holdings(&txs);
        let v = portfolio::value_holdings(&holdings, &prices_map);
        db.insert_snapshot(p.id, v.total_value_usd, now).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn spawn(app: tauri::AppHandle, interval_secs: u64) {
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        loop {
            ticker.tick().await;
            let state = app.state::<AppState>();
            if let Err(e) = refresh_once(&state.db, &state.prices, unix_now()).await {
                eprintln!("price refresh failed: {e}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use crate::prices::PriceClient;

    #[tokio::test]
    async fn refresh_once_writes_cache_and_snapshot() {
        let mut server = mockito::Server::new_async().await;
        let body = r#"{"zcash":{"usd":300.0,"btc":0.004}}"#;
        let _m = server.mock("GET", "/simple/price")
            .match_query(mockito::Matcher::Any)
            .with_body(body).create_async().await;

        let db = Db::open_in_memory().unwrap();
        let p = db.create_portfolio("HODL", 1).unwrap();
        let prices = PriceClient::with_base_url(server.url());

        refresh_once(&db, &prices, 1234).await.unwrap();

        let (map, as_of) = db.cached_prices().unwrap();
        assert!(map.contains_key("zcash"));
        assert_eq!(as_of, Some(1234));
        let snaps = db.list_snapshots(p.id).unwrap();
        assert_eq!(snaps.len(), 1);
    }
}
