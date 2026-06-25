use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use crate::models::{NewTransaction, Portfolio, Transaction, TxType};

pub struct Db {
    conn: Mutex<Connection>,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS portfolios (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS transactions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  portfolio_id INTEGER NOT NULL,
  coin_id TEXT NOT NULL,
  tx_type TEXT NOT NULL,
  quantity TEXT NOT NULL,
  price_usd TEXT NOT NULL,
  fee_usd TEXT NOT NULL,
  ts INTEGER NOT NULL,
  note TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS price_cache (
  coin_id TEXT PRIMARY KEY,
  price_usd TEXT NOT NULL,
  price_btc TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS value_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  portfolio_id INTEGER NOT NULL,
  total_usd TEXT NOT NULL,
  ts INTEGER NOT NULL
);
";

fn tx_type_str(t: TxType) -> &'static str {
    match t { TxType::Buy => "buy", TxType::Sell => "sell" }
}
fn tx_type_from(s: &str) -> TxType {
    if s == "sell" { TxType::Sell } else { TxType::Buy }
}

impl Db {
    pub fn open_in_memory() -> rusqlite::Result<Db> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Db { conn: Mutex::new(conn) })
    }

    pub fn open_at(path: &Path) -> rusqlite::Result<Db> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Db { conn: Mutex::new(conn) })
    }

    pub fn create_portfolio(&self, name: &str, now: i64) -> rusqlite::Result<Portfolio> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO portfolios (name, created_at) VALUES (?1, ?2)",
            params![name, now],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Portfolio { id, name: name.to_string(), created_at: now })
    }

    pub fn list_portfolios(&self) -> rusqlite::Result<Vec<Portfolio>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, created_at FROM portfolios ORDER BY id")?;
        let rows = stmt.query_map([], |r| Ok(Portfolio {
            id: r.get(0)?, name: r.get(1)?, created_at: r.get(2)?,
        }))?;
        rows.collect()
    }

    pub fn add_transaction(&self, tx: &NewTransaction) -> rusqlite::Result<Transaction> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO transactions
             (portfolio_id, coin_id, tx_type, quantity, price_usd, fee_usd, ts, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                tx.portfolio_id, tx.coin_id, tx_type_str(tx.tx_type),
                tx.quantity.to_string(), tx.price_usd.to_string(), tx.fee_usd.to_string(),
                tx.ts, tx.note
            ],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Transaction {
            id, portfolio_id: tx.portfolio_id, coin_id: tx.coin_id.clone(),
            tx_type: tx.tx_type, quantity: tx.quantity, price_usd: tx.price_usd,
            fee_usd: tx.fee_usd, ts: tx.ts, note: tx.note.clone(),
        })
    }

    pub fn list_transactions(&self, portfolio_id: i64) -> rusqlite::Result<Vec<Transaction>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, portfolio_id, coin_id, tx_type, quantity, price_usd, fee_usd, ts, note
             FROM transactions WHERE portfolio_id = ?1 ORDER BY ts, id")?;
        let rows = stmt.query_map(params![portfolio_id], |r| {
            let qty: String = r.get(4)?;
            let price: String = r.get(5)?;
            let fee: String = r.get(6)?;
            let tt: String = r.get(3)?;
            Ok(Transaction {
                id: r.get(0)?, portfolio_id: r.get(1)?, coin_id: r.get(2)?,
                tx_type: tx_type_from(&tt),
                quantity: Decimal::from_str(&qty).unwrap_or_default(),
                price_usd: Decimal::from_str(&price).unwrap_or_default(),
                fee_usd: Decimal::from_str(&fee).unwrap_or_default(),
                ts: r.get(7)?, note: r.get(8)?,
            })
        })?;
        rows.collect()
    }

    pub fn upsert_price(&self, coin_id: &str, price_usd: Decimal, price_btc: Decimal, updated_at: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO price_cache (coin_id, price_usd, price_btc, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(coin_id) DO UPDATE SET
               price_usd=excluded.price_usd, price_btc=excluded.price_btc, updated_at=excluded.updated_at",
            params![coin_id, price_usd.to_string(), price_btc.to_string(), updated_at],
        )?;
        Ok(())
    }

    pub fn cached_prices(&self) -> rusqlite::Result<(HashMap<String, Decimal>, Option<i64>)> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT coin_id, price_usd, updated_at FROM price_cache")?;
        let mut map = HashMap::new();
        let mut newest: Option<i64> = None;
        let rows = stmt.query_map([], |r| {
            let id: String = r.get(0)?;
            let price: String = r.get(1)?;
            let updated: i64 = r.get(2)?;
            Ok((id, price, updated))
        })?;
        for row in rows {
            let (id, price, updated) = row?;
            map.insert(id, Decimal::from_str(&price).unwrap_or_default());
            newest = Some(newest.map_or(updated, |n| n.max(updated)));
        }
        Ok((map, newest))
    }

    pub fn insert_snapshot(&self, portfolio_id: i64, total_usd: Decimal, ts: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO value_snapshots (portfolio_id, total_usd, ts) VALUES (?1, ?2, ?3)",
            params![portfolio_id, total_usd.to_string(), ts],
        )?;
        Ok(())
    }

    pub fn list_snapshots(&self, portfolio_id: i64) -> rusqlite::Result<Vec<(i64, Decimal)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ts, total_usd FROM value_snapshots WHERE portfolio_id = ?1 ORDER BY ts")?;
        let rows = stmt.query_map(params![portfolio_id], |r| {
            let ts: i64 = r.get(0)?;
            let total: String = r.get(1)?;
            Ok((ts, Decimal::from_str(&total).unwrap_or_default()))
        })?;
        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TxType;
    use std::str::FromStr;
    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    #[test]
    fn create_and_list_portfolios() {
        let db = Db::open_in_memory().unwrap();
        let p = db.create_portfolio("HODL", 100).unwrap();
        assert_eq!(p.name, "HODL");
        let all = db.list_portfolios().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, p.id);
    }

    #[test]
    fn add_and_list_transactions() {
        let db = Db::open_in_memory().unwrap();
        let p = db.create_portfolio("HODL", 100).unwrap();
        let nt = NewTransaction {
            portfolio_id: p.id, coin_id: "zcash".into(), tx_type: TxType::Buy,
            quantity: d("1.5"), price_usd: d("250"), fee_usd: Decimal::ZERO,
            ts: 1700, note: "first".into(),
        };
        let saved = db.add_transaction(&nt).unwrap();
        assert!(saved.id > 0);
        let list = db.list_transactions(p.id).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].quantity, d("1.5"));
    }

    #[test]
    fn prices_upsert_and_read_back() {
        let db = Db::open_in_memory().unwrap();
        db.upsert_price("zcash", d("300"), d("0.004"), 500).unwrap();
        db.upsert_price("zcash", d("310"), d("0.0041"), 600).unwrap();
        let (map, newest) = db.cached_prices().unwrap();
        assert_eq!(map.get("zcash").copied().unwrap(), d("310"));
        assert_eq!(newest, Some(600));
    }

    #[test]
    fn snapshots_roundtrip_ascending() {
        let db = Db::open_in_memory().unwrap();
        let p = db.create_portfolio("HODL", 100).unwrap();
        db.insert_snapshot(p.id, d("1000"), 20).unwrap();
        db.insert_snapshot(p.id, d("1200"), 10).unwrap();
        let snaps = db.list_snapshots(p.id).unwrap();
        assert_eq!(snaps[0].0, 10);
        assert_eq!(snaps[1].1, d("1000"));
    }
}
