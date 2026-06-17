# Privacy-Coin Portfolio Tracker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a local-first Windows desktop portfolio tracker for privacy coins (Zcash, Monero, Dash, Firo, Zano) with manual transaction entry, P&L, charts, and multiple portfolios.

**Architecture:** Tauri 2 app. A Rust core owns all logic (SQLite storage, CoinGecko price client, P&L calculator, scheduler) and exposes a narrow set of IPC commands. A web frontend (Vite + vanilla TypeScript + Chart.js) renders the dashboard and only talks to the core through those commands. All user data stays local; only public coin-price requests leave the machine.

**Tech Stack:** Tauri 2, Rust (edition 2021), `rusqlite` (bundled SQLite), `reqwest` (async HTTP), `rust_decimal` (money), `tokio`; frontend: Vite, TypeScript, Chart.js. Tests: `cargo test`, `mockito` for HTTP.

## Global Constraints

- Tauri **2.x**; Rust **edition 2021**, MSRV 1.77+.
- All monetary amounts and coin quantities use `rust_decimal::Decimal`. **Never** use `f64` for money.
- SQLite values for money/quantity are stored as **TEXT** (Decimal `to_string()`), timestamps as **INTEGER** Unix seconds (`i64`).
- Database file lives in the OS app-data dir (`tauri::Manager::path().app_data_dir()`), filename `tracker.db`. Tests use an in-memory DB (`:memory:`).
- **Privacy coins only.** Supported set is the curated list in `coins.rs`; nothing else is selectable.
- **No telemetry / no analytics.** The only outbound network calls are to the CoinGecko public API and carry only coin IDs — never holdings.
- Must never panic on network failure: on error, serve last cached prices and surface `price_as_of`.
- CoinGecko free tier ~30 req/min: batch price requests; exponential backoff on HTTP 429.

---

## File Structure

```
privacy-coin-tracker/
  package.json                 # frontend deps + scripts (dev/build)
  vite.config.ts               # Vite config (fixed port 1420)
  tsconfig.json
  index.html                   # app shell
  src/                         # FRONTEND (reused later for the website)
    main.ts                    # bootstrap, tab/portfolio switching
    api.ts                     # typed wrappers around Tauri invoke()
    dashboard.ts               # render holdings table + totals
    charts.ts                  # Chart.js: portfolio history + coin price
    forms.ts                   # add-transaction + create-portfolio forms
    styles.css
  src-tauri/
    Cargo.toml
    build.rs
    tauri.conf.json
    src/
      main.rs                  # entry; builds State; registers commands
      models.rs                # shared structs (Portfolio, Transaction, …)
      coins.rs                 # curated privacy-coin list + lookups
      db.rs                    # SQLite: schema + CRUD + snapshots + cache
      prices.rs                # CoinGecko client (current + history, backoff)
      portfolio.rs             # pure calculator: holdings, P&L, allocation
      commands.rs              # #[tauri::command] handlers (wire it together)
      scheduler.rs             # periodic refresh + value snapshot
  docs/superpowers/...
```

Responsibilities are split so each Rust module has one job and can be tested in isolation. `portfolio.rs` is pure (no I/O) and carries the heaviest test suite. `db.rs` and `prices.rs` own all I/O. `commands.rs` is thin glue.

---

### Task 1: Scaffold the Tauri project (builds + empty test green)

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.ts`, `src/styles.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`
- Create: `.gitignore`

**Interfaces:**
- Consumes: nothing.
- Produces: a buildable Tauri app and a working `cargo test` harness in `src-tauri/`.

- [ ] **Step 1: Create `.gitignore`**

```gitignore
node_modules/
dist/
src-tauri/target/
*.db
```

- [ ] **Step 2: Create frontend manifest `package.json`**

```json
{
  "name": "privacy-coin-tracker",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "chart.js": "^4"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "typescript": "^5",
    "vite": "^5"
  }
}
```

- [ ] **Step 3: Create `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.ts`, `src/styles.css`**

`vite.config.ts`:
```ts
import { defineConfig } from "vite";
export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: "es2021", outDir: "dist" },
});
```

`tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2021",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "types": ["vite/client"]
  },
  "include": ["src"]
}
```

`index.html`:
```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>Privacy Coin Tracker</title>
    <link rel="stylesheet" href="/src/styles.css" />
  </head>
  <body>
    <div id="app"><h1>Privacy Coin Tracker</h1></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

`src/main.ts`:
```ts
console.log("Privacy Coin Tracker booting…");
```

`src/styles.css`:
```css
:root { color-scheme: dark; }
body { font-family: system-ui, sans-serif; margin: 0; background: #14161a; color: #e6e6e6; }
#app { padding: 16px; }
```

- [ ] **Step 4: Create `src-tauri/Cargo.toml`**

```toml
[package]
name = "privacy-coin-tracker"
version = "0.1.0"
edition = "2021"

[lib]
name = "app_lib"
path = "src/lib.rs"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
reqwest = { version = "0.12", features = ["json"] }
rust_decimal = { version = "1", features = ["serde-str"] }
tokio = { version = "1", features = ["time", "rt", "macros"] }

[dev-dependencies]
mockito = "1"
```

Note: there is no separate `lib.rs` yet; create a stub in Step 6 so `[lib]` resolves.

- [ ] **Step 5: Create `src-tauri/build.rs` and `src-tauri/tauri.conf.json`**

`build.rs`:
```rust
fn main() {
    tauri_build::build();
}
```

`tauri.conf.json`:
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Privacy Coin Tracker",
  "version": "0.1.0",
  "identifier": "com.privacycoin.tracker",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      { "title": "Privacy Coin Tracker", "width": 1100, "height": 720 }
    ],
    "security": { "csp": null }
  },
  "bundle": { "active": true, "targets": "all" }
}
```

- [ ] **Step 6: Create `src-tauri/src/lib.rs` and `src-tauri/src/main.rs`**

`src/lib.rs`:
```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod smoke {
    #[test]
    fn it_builds() {
        assert_eq!(2 + 2, 4);
    }
}
```

`src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    app_lib::run();
}
```

- [ ] **Step 7: Verify the Rust test harness runs**

Run: `cd src-tauri && cargo test`
Expected: compiles; `smoke::it_builds` PASSES (1 passed).

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri 2 project skeleton"
```

---

### Task 2: Domain models

**Files:**
- Create: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/lib.rs` (add `pub mod models;`)

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `enum TxType { Buy, Sell }` (serde: lowercase `"buy"`/`"sell"`).
  - `struct Transaction { id: i64, portfolio_id: i64, coin_id: String, tx_type: TxType, quantity: Decimal, price_usd: Decimal, fee_usd: Decimal, ts: i64, note: String }`
  - `struct Portfolio { id: i64, name: String, created_at: i64 }`
  - `struct NewTransaction { portfolio_id: i64, coin_id: String, tx_type: TxType, quantity: Decimal, price_usd: Decimal, fee_usd: Decimal, ts: i64, note: String }` (no `id`).

- [ ] **Step 1: Write the failing test**

In `src-tauri/src/models.rs`:
```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn txtype_serializes_lowercase() {
        let j = serde_json::to_string(&TxType::Buy).unwrap();
        assert_eq!(j, "\"buy\"");
    }

    #[test]
    fn transaction_roundtrips_json_with_decimal() {
        let t = Transaction {
            id: 1,
            portfolio_id: 2,
            coin_id: "zcash".into(),
            tx_type: TxType::Buy,
            quantity: Decimal::from_str("1.5").unwrap(),
            price_usd: Decimal::from_str("250.00").unwrap(),
            fee_usd: Decimal::ZERO,
            ts: 1_700_000_000,
            note: String::new(),
        };
        let j = serde_json::to_string(&t).unwrap();
        let back: Transaction = serde_json::from_str(&j).unwrap();
        assert_eq!(back.quantity, Decimal::from_str("1.5").unwrap());
        assert_eq!(back.coin_id, "zcash");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test models`
Expected: FAIL — `TxType` / `Transaction` not found.

- [ ] **Step 3: Write minimal implementation**

Above the `tests` module in `models.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub portfolio_id: i64,
    pub coin_id: String,
    pub tx_type: TxType,
    pub quantity: Decimal,
    pub price_usd: Decimal,
    pub fee_usd: Decimal,
    pub ts: i64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransaction {
    pub portfolio_id: i64,
    pub coin_id: String,
    pub tx_type: TxType,
    pub quantity: Decimal,
    pub price_usd: Decimal,
    pub fee_usd: Decimal,
    pub ts: i64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
}
```

Add to `lib.rs`: `pub mod models;`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test models`
Expected: PASS (2 passed).

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: add domain models"
```

---

### Task 3: Curated privacy-coin list

**Files:**
- Create: `src-tauri/src/coins.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod coins;`)

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `struct Coin { id: &'static str, symbol: &'static str, name: &'static str, coingecko_id: &'static str }`
  - `fn all() -> &'static [Coin]`
  - `fn by_id(id: &str) -> Option<&'static Coin>`
  - `fn coingecko_ids() -> Vec<&'static str>`
  - Coin `id` values used everywhere else: `"zcash"`, `"monero"`, `"dash"`, `"firo"`, `"zano"`.

- [ ] **Step 1: Write the failing test**

In `coins.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_five_privacy_coins() {
        assert_eq!(all().len(), 5);
    }

    #[test]
    fn lookup_by_id_works() {
        assert_eq!(by_id("zcash").unwrap().symbol, "ZEC");
        assert!(by_id("bitcoin").is_none());
    }

    #[test]
    fn firo_maps_to_zcoin_on_coingecko() {
        assert_eq!(by_id("firo").unwrap().coingecko_id, "zcoin");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test coins`
Expected: FAIL — `all` / `by_id` not found.

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Copy)]
pub struct Coin {
    pub id: &'static str,
    pub symbol: &'static str,
    pub name: &'static str,
    pub coingecko_id: &'static str,
}

static COINS: &[Coin] = &[
    Coin { id: "zcash",  symbol: "ZEC",  name: "Zcash",  coingecko_id: "zcash" },
    Coin { id: "monero", symbol: "XMR",  name: "Monero", coingecko_id: "monero" },
    Coin { id: "dash",   symbol: "DASH", name: "Dash",   coingecko_id: "dash" },
    Coin { id: "firo",   symbol: "FIRO", name: "Firo",   coingecko_id: "zcoin" },
    Coin { id: "zano",   symbol: "ZANO", name: "Zano",   coingecko_id: "zano" },
];

pub fn all() -> &'static [Coin] { COINS }

pub fn by_id(id: &str) -> Option<&'static Coin> {
    COINS.iter().find(|c| c.id == id)
}

pub fn coingecko_ids() -> Vec<&'static str> {
    COINS.iter().map(|c| c.coingecko_id).collect()
}
```

Add to `lib.rs`: `pub mod coins;`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test coins`
Expected: PASS (3 passed).

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: add curated privacy-coin list"
```

---

### Task 4: Portfolio calculator (holdings, P&L, allocation)

This is the core. Pure functions, no I/O.

**Files:**
- Create: `src-tauri/src/portfolio.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod portfolio;`)

**Interfaces:**
- Consumes: `models::{Transaction, TxType}`.
- Produces:
  - `struct Holding { coin_id: String, quantity: Decimal, avg_cost_usd: Decimal, realized_pnl_usd: Decimal }`
  - `fn compute_holdings(txs: &[Transaction]) -> Vec<Holding>` — processes txs in ascending `ts` order; running average-cost method. Buy increases qty and recomputes `avg_cost_usd = (avg_cost*qty + price*buy_qty + fee) / (qty + buy_qty)`. Sell: `realized += (price - avg_cost) * sell_qty - fee`; qty decreases; `avg_cost` unchanged. Coins ending at quantity `0` are omitted, but their `realized_pnl_usd` is preserved by keeping a zero-qty holding only if realized != 0.
  - `struct ValuedCoin { coin_id: String, quantity: Decimal, avg_cost_usd: Decimal, price_usd: Decimal, value_usd: Decimal, unrealized_pnl_usd: Decimal, unrealized_pnl_pct: Decimal, realized_pnl_usd: Decimal, allocation_pct: Decimal }`
  - `struct Valuation { total_value_usd: Decimal, total_cost_usd: Decimal, total_unrealized_pnl_usd: Decimal, coins: Vec<ValuedCoin> }`
  - `fn value_holdings(holdings: &[Holding], prices: &HashMap<String, Decimal>) -> Valuation` — coins with no price use `price_usd = 0` and contribute `0` value (caller decides how to flag staleness).

- [ ] **Step 1: Write the failing tests**

In `portfolio.rs`:
```rust
use std::collections::HashMap;
use rust_decimal::Decimal;
use crate::models::{Transaction, TxType};

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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test portfolio`
Expected: FAIL — `compute_holdings` / `value_holdings` not found.

- [ ] **Step 3: Implement `compute_holdings`**

```rust
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
```

- [ ] **Step 4: Implement `value_holdings`**

```rust
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
```

Add to `lib.rs`: `pub mod portfolio;`

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test portfolio`
Expected: PASS (4 passed).

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: add portfolio P&L and allocation calculator"
```

---

### Task 5: SQLite data layer

**Files:**
- Create: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod db;`)

**Interfaces:**
- Consumes: `models::{Portfolio, Transaction, NewTransaction, TxType}`.
- Produces a `Db` wrapper:
  - `fn open_in_memory() -> rusqlite::Result<Db>` (tests)
  - `fn open_at(path: &std::path::Path) -> rusqlite::Result<Db>` (app)
  - `fn create_portfolio(&self, name: &str, now: i64) -> rusqlite::Result<Portfolio>`
  - `fn list_portfolios(&self) -> rusqlite::Result<Vec<Portfolio>>`
  - `fn add_transaction(&self, tx: &NewTransaction) -> rusqlite::Result<Transaction>`
  - `fn list_transactions(&self, portfolio_id: i64) -> rusqlite::Result<Vec<Transaction>>`
  - `fn upsert_price(&self, coin_id: &str, price_usd: Decimal, price_btc: Decimal, updated_at: i64) -> rusqlite::Result<()>`
  - `fn cached_prices(&self) -> rusqlite::Result<(HashMap<String, Decimal>, Option<i64>)>` — returns (coin_id→price_usd, newest updated_at)
  - `fn insert_snapshot(&self, portfolio_id: i64, total_usd: Decimal, ts: i64) -> rusqlite::Result<()>`
  - `fn list_snapshots(&self, portfolio_id: i64) -> rusqlite::Result<Vec<(i64, Decimal)>>` — `(ts, total_usd)` ascending
- Internals: `Db` holds `Mutex<rusqlite::Connection>`. Decimal stored/read as TEXT.

- [ ] **Step 1: Write the failing tests**

In `db.rs`:
```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test db`
Expected: FAIL — `Db` not found.

- [ ] **Step 3: Implement the `Db` wrapper and schema**

```rust
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
```

Add to `lib.rs`: `pub mod db;`

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test db`
Expected: PASS (4 passed).

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: add SQLite data layer"
```

---

### Task 6: CoinGecko price client

**Files:**
- Create: `src-tauri/src/prices.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod prices;`)

**Interfaces:**
- Consumes: nothing from our modules (takes coingecko ids as `&[&str]`).
- Produces:
  - `struct PriceClient { base_url: String, http: reqwest::Client }`
  - `fn new() -> PriceClient` (base `https://api.coingecko.com/api/v3`)
  - `fn with_base_url(base: impl Into<String>) -> PriceClient` (tests)
  - `async fn fetch_prices(&self, cg_ids: &[&str]) -> Result<HashMap<String, (Decimal, Decimal)>, String>` — calls `/simple/price?ids=…&vs_currencies=usd,btc`; returns `cg_id → (usd, btc)`. On HTTP 429, retries with exponential backoff (100ms, 200ms, 400ms; max 3 tries) then errors.
  - `async fn fetch_history(&self, cg_id: &str, days: u32) -> Result<Vec<(i64, Decimal)>, String>` — calls `/coins/{id}/market_chart?vs_currency=usd&days={days}`; maps the `prices` array (`[ms_timestamp, price]`) to `(unix_seconds, Decimal)`.

- [ ] **Step 1: Write the failing test (mocked HTTP)**

In `prices.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn fetch_prices_parses_usd_and_btc() {
        let mut server = mockito::Server::new_async().await;
        let body = r#"{"zcash":{"usd":300.5,"btc":0.004},"monero":{"usd":250.0,"btc":0.0033}}"#;
        let _m = server.mock("GET", "/simple/price")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create_async().await;

        let client = PriceClient::with_base_url(server.url());
        let out = client.fetch_prices(&["zcash", "monero"]).await.unwrap();
        assert_eq!(out.get("zcash").unwrap().0, Decimal::from_str("300.5").unwrap());
        assert_eq!(out.get("monero").unwrap().1, Decimal::from_str("0.0033").unwrap());
    }

    #[tokio::test]
    async fn fetch_history_maps_ms_to_seconds() {
        let mut server = mockito::Server::new_async().await;
        let body = r#"{"prices":[[1700000000000,300.0],[1700086400000,310.0]]}"#;
        let _m = server.mock("GET", "/coins/zcash/market_chart")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body(body)
            .create_async().await;

        let client = PriceClient::with_base_url(server.url());
        let hist = client.fetch_history("zcash", 30).await.unwrap();
        assert_eq!(hist[0].0, 1_700_000_000);
        assert_eq!(hist[1].1, Decimal::from_str("310.0").unwrap());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test prices`
Expected: FAIL — `PriceClient` not found.

- [ ] **Step 3: Implement the client**

```rust
use std::collections::HashMap;
use rust_decimal::Decimal;

pub struct PriceClient {
    base_url: String,
    http: reqwest::Client,
}

fn num_to_decimal(v: &serde_json::Value) -> Decimal {
    v.as_f64()
        .and_then(|f| Decimal::from_f64_retain(f))
        .unwrap_or(Decimal::ZERO)
}

impl PriceClient {
    pub fn new() -> PriceClient {
        PriceClient::with_base_url("https://api.coingecko.com/api/v3")
    }

    pub fn with_base_url(base: impl Into<String>) -> PriceClient {
        PriceClient { base_url: base.into(), http: reqwest::Client::new() }
    }

    pub async fn fetch_prices(&self, cg_ids: &[&str]) -> Result<HashMap<String, (Decimal, Decimal)>, String> {
        let ids = cg_ids.join(",");
        let url = format!("{}/simple/price", self.base_url);
        let mut delay_ms = 100u64;
        for attempt in 0..3 {
            let resp = self.http.get(&url)
                .query(&[("ids", ids.as_str()), ("vs_currencies", "usd,btc")])
                .send().await.map_err(|e| e.to_string())?;
            if resp.status().as_u16() == 429 {
                if attempt < 2 {
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2;
                    continue;
                }
                return Err("rate limited".into());
            }
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let mut out = HashMap::new();
            if let Some(obj) = json.as_object() {
                for (id, v) in obj {
                    let usd = num_to_decimal(&v["usd"]);
                    let btc = num_to_decimal(&v["btc"]);
                    out.insert(id.clone(), (usd, btc));
                }
            }
            return Ok(out);
        }
        Err("rate limited".into())
    }

    pub async fn fetch_history(&self, cg_id: &str, days: u32) -> Result<Vec<(i64, Decimal)>, String> {
        let url = format!("{}/coins/{}/market_chart", self.base_url, cg_id);
        let resp = self.http.get(&url)
            .query(&[("vs_currency", "usd"), ("days", &days.to_string())])
            .send().await.map_err(|e| e.to_string())?;
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        if let Some(arr) = json["prices"].as_array() {
            for pair in arr {
                if let Some(p) = pair.as_array() {
                    let ms = p[0].as_i64().unwrap_or(0);
                    out.push((ms / 1000, num_to_decimal(&p[1])));
                }
            }
        }
        Ok(out)
    }
}
```

Add to `lib.rs`: `pub mod prices;`

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test prices`
Expected: PASS (2 passed).

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: add CoinGecko price client with backoff"
```

---

### Task 7: Application state + Tauri commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod commands;`, build `AppState`, register handlers)

**Interfaces:**
- Consumes: `db::Db`, `prices::PriceClient`, `portfolio::{compute_holdings, value_holdings, Valuation}`, `coins`, `models::*`.
- Produces:
  - `struct AppState { db: Db, prices: PriceClient }`
  - `struct Dashboard { portfolio_id: i64, valuation: Valuation, price_as_of: Option<i64> }` (Serialize)
  - `struct CoinMeta { id: String, symbol: String, name: String }` (Serialize)
  - Commands (all return `Result<T, String>`):
    - `list_coins() -> Vec<CoinMeta>`
    - `list_portfolios(state) -> Vec<Portfolio>`
    - `create_portfolio(state, name, now) -> Portfolio`
    - `add_transaction(state, tx: NewTransaction) -> Transaction`
    - `list_transactions(state, portfolio_id) -> Vec<Transaction>`
    - `get_dashboard(state, portfolio_id) -> Dashboard`
    - `refresh_prices(state, now) -> Option<i64>` (returns newest cache ts)
    - `get_coin_history(state, coin_id, days) -> Vec<(i64, Decimal)>`
    - `get_portfolio_history(state, portfolio_id) -> Vec<(i64, Decimal)>`
- Notes: `now`/timestamps are passed **from the frontend** (`Date.now()/1000`) to keep the core deterministic and testable. `refresh_prices` fetches via `PriceClient`, writes `price_cache`, then writes a `value_snapshots` row per portfolio using freshly valued totals.

- [ ] **Step 1: Write the failing test (core logic, no Tauri runtime)**

Tests target a free function `build_dashboard` so they don't need a Tauri `State`. In `commands.rs`:
```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test commands`
Expected: FAIL — `build_dashboard` not found.

- [ ] **Step 3: Implement state, `build_dashboard`, and commands**

```rust
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
    let cg_ids = coins::coingecko_ids();
    let fetched = state.prices.fetch_prices(&cg_ids).await?;
    for c in coins::all() {
        if let Some((usd, btc)) = fetched.get(c.coingecko_id) {
            state.db.upsert_price(c.id, *usd, *btc, now).map_err(|e| e.to_string())?;
        }
    }
    // snapshot each portfolio's total value
    let (prices, as_of) = state.db.cached_prices().map_err(|e| e.to_string())?;
    for p in state.db.list_portfolios().map_err(|e| e.to_string())? {
        let txs = state.db.list_transactions(p.id).map_err(|e| e.to_string())?;
        let holdings = compute_holdings(&txs);
        let v = value_holdings(&holdings, &prices);
        state.db.insert_snapshot(p.id, v.total_value_usd, now).map_err(|e| e.to_string())?;
    }
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
```

- [ ] **Step 4: Wire state + handlers into `lib.rs`**

Replace `run()` in `lib.rs`:
```rust
pub mod models;
pub mod coins;
pub mod portfolio;
pub mod db;
pub mod prices;
pub mod commands;

use tauri::Manager;
use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("app data dir");
            std::fs::create_dir_all(&dir).ok();
            let db = db::Db::open_at(&dir.join("tracker.db")).expect("open db");
            app.manage(AppState { db, prices: prices::PriceClient::new() });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_coins,
            commands::list_portfolios,
            commands::create_portfolio,
            commands::add_transaction,
            commands::list_transactions,
            commands::get_dashboard,
            commands::refresh_prices,
            commands::get_coin_history,
            commands::get_portfolio_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Remove the old `smoke` test module from `lib.rs` (the `commands` test now covers integration).

- [ ] **Step 5: Run tests + build**

Run: `cargo test commands` → Expected: PASS (1 passed).
Run: `cargo build` → Expected: compiles cleanly.

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: add app state and Tauri commands"
```

---

### Task 8: Background price scheduler

**Files:**
- Create: `src-tauri/src/scheduler.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod scheduler;`, spawn in `setup`)

**Interfaces:**
- Consumes: `tauri::AppHandle`, `commands::AppState`, `prices`, `coins`, `db`.
- Produces: `fn spawn(app: tauri::AppHandle, interval_secs: u64)` — every `interval_secs`, runs the same fetch+upsert+snapshot logic as `refresh_prices`, using `now` from `std::time::SystemTime`. Failures are logged (`eprintln!`) and swallowed — the loop never dies.

- [ ] **Step 1: Write the failing test**

The refresh body is extracted into a testable free function `refresh_once`. In `scheduler.rs`:
```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test scheduler`
Expected: FAIL — `refresh_once` not found.

- [ ] **Step 3: Implement `refresh_once` and `spawn`**

```rust
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
```

Refactor `commands::refresh_prices` to call `scheduler::refresh_once` (DRY) instead of duplicating the body:
```rust
#[tauri::command]
pub async fn refresh_prices(state: State<'_, AppState>, now: i64) -> Result<Option<i64>, String> {
    crate::scheduler::refresh_once(&state.db, &state.prices, now).await?;
    let (_, as_of) = state.db.cached_prices().map_err(|e| e.to_string())?;
    Ok(as_of)
}
```

In `lib.rs` `setup`, after `app.manage(...)`, add: `scheduler::spawn(app.handle().clone(), 120);` and `pub mod scheduler;` at top.

- [ ] **Step 4: Run tests + build**

Run: `cargo test scheduler` → Expected: PASS (1 passed).
Run: `cargo test` → Expected: ALL pass.
Run: `cargo build` → Expected: compiles.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: add background price scheduler"
```

---

### Task 9: Frontend — typed API layer + dashboard render

**Files:**
- Create: `src/api.ts`, `src/dashboard.ts`
- Modify: `src/main.ts`, `src/styles.css`, `index.html`

**Interfaces:**
- Consumes (via `invoke`): the Task 7 commands.
- Produces:
  - `api.ts` exports typed functions: `listCoins()`, `listPortfolios()`, `createPortfolio(name)`, `addTransaction(tx)`, `listTransactions(id)`, `getDashboard(id)`, `refreshPrices()`, `getCoinHistory(coinId, days)`, `getPortfolioHistory(id)` plus the TS interfaces `Portfolio`, `Transaction`, `Dashboard`, `ValuedCoin`, `CoinMeta`. (Decimals arrive as strings — keep them as `string` in TS and format for display.)
  - `dashboard.ts` exports `renderDashboard(el: HTMLElement, dash: Dashboard): void`.

- [ ] **Step 1: Implement `api.ts`**

```ts
import { invoke } from "@tauri-apps/api/core";

export interface CoinMeta { id: string; symbol: string; name: string; }
export interface Portfolio { id: number; name: string; created_at: number; }
export interface Transaction {
  id: number; portfolio_id: number; coin_id: string;
  tx_type: "buy" | "sell"; quantity: string; price_usd: string;
  fee_usd: string; ts: number; note: string;
}
export interface ValuedCoin {
  coin_id: string; quantity: string; avg_cost_usd: string; price_usd: string;
  value_usd: string; unrealized_pnl_usd: string; unrealized_pnl_pct: string;
  realized_pnl_usd: string; allocation_pct: string;
}
export interface Valuation {
  total_value_usd: string; total_cost_usd: string;
  total_unrealized_pnl_usd: string; coins: ValuedCoin[];
}
export interface Dashboard {
  portfolio_id: number; valuation: Valuation; price_as_of: number | null;
}

const now = () => Math.floor(Date.now() / 1000);

export const listCoins = () => invoke<CoinMeta[]>("list_coins");
export const listPortfolios = () => invoke<Portfolio[]>("list_portfolios");
export const createPortfolio = (name: string) =>
  invoke<Portfolio>("create_portfolio", { name, now: now() });
export const addTransaction = (tx: Omit<Transaction, "id">) =>
  invoke<Transaction>("add_transaction", { tx });
export const listTransactions = (portfolioId: number) =>
  invoke<Transaction[]>("list_transactions", { portfolioId });
export const getDashboard = (portfolioId: number) =>
  invoke<Dashboard>("get_dashboard", { portfolioId });
export const refreshPrices = () =>
  invoke<number | null>("refresh_prices", { now: now() });
export const getCoinHistory = (coinId: string, days: number) =>
  invoke<[number, string][]>("get_coin_history", { coinId, days });
export const getPortfolioHistory = (portfolioId: number) =>
  invoke<[number, string][]>("get_portfolio_history", { portfolioId });
```

- [ ] **Step 2: Implement `dashboard.ts`**

```ts
import type { Dashboard } from "./api";

const fmtUsd = (s: string) => `$${Number(s).toLocaleString(undefined, { maximumFractionDigits: 2 })}`;
const fmtPct = (s: string) => `${Number(s).toFixed(2)}%`;
const cls = (s: string) => (Number(s) >= 0 ? "pos" : "neg");

export function renderDashboard(el: HTMLElement, dash: Dashboard): void {
  const v = dash.valuation;
  const asOf = dash.price_as_of
    ? new Date(dash.price_as_of * 1000).toLocaleTimeString()
    : "—";
  const rows = v.coins.map((c) => `
    <tr>
      <td>${c.coin_id}</td>
      <td>${Number(c.quantity)}</td>
      <td>${fmtUsd(c.avg_cost_usd)}</td>
      <td>${fmtUsd(c.price_usd)}</td>
      <td>${fmtUsd(c.value_usd)}</td>
      <td class="${cls(c.unrealized_pnl_usd)}">${fmtUsd(c.unrealized_pnl_usd)} (${fmtPct(c.unrealized_pnl_pct)})</td>
      <td>${fmtPct(c.allocation_pct)}</td>
    </tr>`).join("");

  el.innerHTML = `
    <div class="totals">
      <span>Total: <b>${fmtUsd(v.total_value_usd)}</b></span>
      <span class="${cls(v.total_unrealized_pnl_usd)}">P&L: ${fmtUsd(v.total_unrealized_pnl_usd)}</span>
      <span class="muted">prices as of ${asOf}</span>
    </div>
    <table>
      <thead><tr>
        <th>Coin</th><th>Qty</th><th>Avg cost</th><th>Price</th>
        <th>Value</th><th>Unrealized P&L</th><th>Alloc</th>
      </tr></thead>
      <tbody>${rows || `<tr><td colspan="7" class="muted">No holdings yet</td></tr>`}</tbody>
    </table>`;
}
```

- [ ] **Step 3: Wire `main.ts` to load portfolios and render**

```ts
import { listPortfolios, createPortfolio, getDashboard, refreshPrices } from "./api";
import { renderDashboard } from "./dashboard";

const app = document.getElementById("app")!;
app.innerHTML = `
  <header><h1>Privacy Coin Tracker</h1>
    <select id="portfolio-select"></select>
    <button id="refresh">Refresh prices</button>
  </header>
  <main id="dashboard"></main>`;

const select = document.getElementById("portfolio-select") as HTMLSelectElement;
const dashEl = document.getElementById("dashboard")!;

async function bootstrap() {
  let portfolios = await listPortfolios();
  if (portfolios.length === 0) {
    await createPortfolio("Main");
    portfolios = await listPortfolios();
  }
  select.innerHTML = portfolios.map((p) => `<option value="${p.id}">${p.name}</option>`).join("");
  await show(Number(select.value));
}

async function show(id: number) {
  renderDashboard(dashEl, await getDashboard(id));
}

select.addEventListener("change", () => show(Number(select.value)));
document.getElementById("refresh")!.addEventListener("click", async () => {
  await refreshPrices();
  await show(Number(select.value));
});

bootstrap();
```

- [ ] **Step 4: Add styles**

Append to `src/styles.css`:
```css
header { display: flex; gap: 12px; align-items: center; }
table { width: 100%; border-collapse: collapse; margin-top: 12px; }
th, td { text-align: left; padding: 6px 10px; border-bottom: 1px solid #2a2d33; }
.totals { display: flex; gap: 18px; margin: 12px 0; font-size: 15px; }
.pos { color: #4 caf50; } .neg { color: #ef5350; } .muted { color: #888; }
button, select { background: #21262d; color: #e6e6e6; border: 1px solid #2a2d33; padding: 6px 10px; border-radius: 6px; }
```
(Fix the `.pos` value to `#4caf50` — no space.)

- [ ] **Step 5: Verify the app runs end-to-end**

Run (from project root): `npm install` then `npm run tauri dev`
Expected: window opens, shows "Main" portfolio, empty holdings table, "prices as of —". Clicking **Refresh prices** populates the price cache (no holdings yet, so totals stay $0). No console errors.

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: add frontend api layer and dashboard"
```

---

### Task 10: Frontend — add-transaction & create-portfolio forms

**Files:**
- Create: `src/forms.ts`
- Modify: `src/main.ts`

**Interfaces:**
- Consumes: `api.ts` (`listCoins`, `addTransaction`, `createPortfolio`).
- Produces: `mountForms(opts: { onChange: () => void; currentPortfolioId: () => number }): HTMLElement` — returns a panel with a coin dropdown (from `listCoins`), buy/sell toggle, quantity, price, date, and an "Add transaction" button, plus a "New portfolio" control. Calls `onChange()` after a successful write.

- [ ] **Step 1: Implement `forms.ts`**

```ts
import { listCoins, addTransaction, createPortfolio, type CoinMeta } from "./api";

export function mountForms(opts: { onChange: () => void; currentPortfolioId: () => number }): HTMLElement {
  const panel = document.createElement("section");
  panel.className = "forms";
  panel.innerHTML = `
    <fieldset>
      <legend>Add transaction</legend>
      <select id="f-coin"></select>
      <select id="f-type"><option value="buy">Buy</option><option value="sell">Sell</option></select>
      <input id="f-qty" type="number" step="any" placeholder="Quantity" />
      <input id="f-price" type="number" step="any" placeholder="Price USD" />
      <input id="f-date" type="date" />
      <button id="f-add">Add</button>
      <span id="f-err" class="neg"></span>
    </fieldset>
    <fieldset>
      <legend>New portfolio</legend>
      <input id="f-pname" placeholder="Name" />
      <button id="f-pcreate">Create</button>
    </fieldset>`;

  listCoins().then((coins: CoinMeta[]) => {
    (panel.querySelector("#f-coin") as HTMLSelectElement).innerHTML =
      coins.map((c) => `<option value="${c.id}">${c.symbol} — ${c.name}</option>`).join("");
  });

  const err = panel.querySelector("#f-err") as HTMLElement;

  panel.querySelector("#f-add")!.addEventListener("click", async () => {
    err.textContent = "";
    const coin = (panel.querySelector("#f-coin") as HTMLSelectElement).value;
    const type = (panel.querySelector("#f-type") as HTMLSelectElement).value as "buy" | "sell";
    const qty = (panel.querySelector("#f-qty") as HTMLInputElement).value;
    const price = (panel.querySelector("#f-price") as HTMLInputElement).value;
    const dateStr = (panel.querySelector("#f-date") as HTMLInputElement).value;
    if (!qty || !price) { err.textContent = "qty and price required"; return; }
    const ts = dateStr ? Math.floor(new Date(dateStr).getTime() / 1000) : Math.floor(Date.now() / 1000);
    try {
      await addTransaction({
        portfolio_id: opts.currentPortfolioId(), coin_id: coin, tx_type: type,
        quantity: qty, price_usd: price, fee_usd: "0", ts, note: "",
      });
      opts.onChange();
    } catch (e) { err.textContent = String(e); }
  });

  panel.querySelector("#f-pcreate")!.addEventListener("click", async () => {
    const name = (panel.querySelector("#f-pname") as HTMLInputElement).value.trim();
    if (!name) return;
    await createPortfolio(name);
    opts.onChange();
  });

  return panel;
}
```

- [ ] **Step 2: Wire forms into `main.ts`**

After `dashEl` is defined, add:
```ts
import { mountForms } from "./forms";

const forms = mountForms({
  onChange: async () => { await bootstrap(); },
  currentPortfolioId: () => Number(select.value),
});
document.getElementById("app")!.appendChild(forms);
```
(Ensure `bootstrap()` re-reads portfolios and re-renders. Keep the single `bootstrap()` call at the end.)

- [ ] **Step 3: Verify end-to-end**

Run: `npm run tauri dev`
Steps: add a Buy of 2 ZEC @ 100 → holdings table shows Zcash qty 2, avg $100. Click **Refresh prices** → value and P&L populate from live price. Create a new portfolio → it appears in the dropdown and is empty.
Expected: all behaviors work, no console errors.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: add transaction and portfolio forms"
```

---

### Task 11: Frontend — charts (portfolio history + coin price)

**Files:**
- Create: `src/charts.ts`
- Modify: `src/main.ts`, `index.html` (add `<canvas>` mounts)

**Interfaces:**
- Consumes: `api.ts` (`getPortfolioHistory`, `getCoinHistory`), `chart.js`.
- Produces:
  - `renderPortfolioChart(canvas: HTMLCanvasElement, points: [number, string][]): void`
  - `renderCoinChart(canvas: HTMLCanvasElement, coinId: string, points: [number, string][]): void`
  - Both destroy any prior Chart instance on the canvas before drawing (guard against leaks on re-render).

- [ ] **Step 1: Implement `charts.ts`**

```ts
import { Chart, registerables } from "chart.js";
Chart.register(...registerables);

const existing = new WeakMap<HTMLCanvasElement, Chart>();

function draw(canvas: HTMLCanvasElement, label: string, points: [number, string][]) {
  existing.get(canvas)?.destroy();
  const chart = new Chart(canvas, {
    type: "line",
    data: {
      labels: points.map((p) => new Date(p[0] * 1000).toLocaleDateString()),
      datasets: [{ label, data: points.map((p) => Number(p[1])), borderColor: "#4caf50", tension: 0.2 }],
    },
    options: { responsive: true, plugins: { legend: { display: true } } },
  });
  existing.set(canvas, chart);
}

export function renderPortfolioChart(canvas: HTMLCanvasElement, points: [number, string][]) {
  draw(canvas, "Portfolio value (USD)", points);
}
export function renderCoinChart(canvas: HTMLCanvasElement, coinId: string, points: [number, string][]) {
  draw(canvas, `${coinId} price (USD)`, points);
}
```

- [ ] **Step 2: Add canvases + wiring in `main.ts`**

Extend the `#app` markup to include:
```html
<section class="charts">
  <canvas id="pf-chart" height="120"></canvas>
  <div><select id="coin-chart-select"></select><canvas id="coin-chart" height="120"></canvas></div>
</section>
```
Then in `show(id)`:
```ts
import { renderPortfolioChart, renderCoinChart } from "./charts";
import { getPortfolioHistory, getCoinHistory, listCoins } from "./api";

// inside show(id):
const pf = document.getElementById("pf-chart") as HTMLCanvasElement;
renderPortfolioChart(pf, await getPortfolioHistory(id));

const coinSel = document.getElementById("coin-chart-select") as HTMLSelectElement;
if (!coinSel.dataset.filled) {
  const coins = await listCoins();
  coinSel.innerHTML = coins.map((c) => `<option value="${c.id}">${c.symbol}</option>`).join("");
  coinSel.dataset.filled = "1";
  coinSel.addEventListener("change", drawCoin);
}
async function drawCoin() {
  const cc = document.getElementById("coin-chart") as HTMLCanvasElement;
  renderCoinChart(cc, coinSel.value, await getCoinHistory(coinSel.value, 30));
}
await drawCoin();
```

- [ ] **Step 3: Verify end-to-end**

Run: `npm run tauri dev`
Steps: with at least one Buy and one **Refresh prices** done, the portfolio chart shows ≥1 snapshot point; the coin selector switches the price chart between coins (30-day history from CoinGecko).
Expected: charts render; switching coins redraws without console errors or canvas-reuse warnings.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: add portfolio and coin price charts"
```

---

### Task 12: Production build smoke test

**Files:** none (verification only).

- [ ] **Step 1: Full test suite**

Run: `cd src-tauri && cargo test`
Expected: ALL tests across models/coins/portfolio/db/prices/commands/scheduler pass.

- [ ] **Step 2: Release build**

Run (project root): `npm run tauri build`
Expected: completes; produces a Windows installer under `src-tauri/target/release/bundle/`.

- [ ] **Step 3: Launch the built app**

Install/run the produced executable. Confirm: window opens, a default "Main" portfolio exists, adding a transaction and refreshing prices works, data persists across restart (SQLite in AppData).

- [ ] **Step 4: Commit any config fixes**

```bash
git add -A && git commit -m "chore: production build verification"
```

---

## Self-Review

**1. Spec coverage**
- Manual entry + CoinGecko prices → Tasks 5, 6, 7, 9, 10. ✅
- Desktop Windows, local data → Tasks 1, 5 (AppData SQLite), 12. ✅
- Privacy coins only → Task 3 + `add_transaction` guard (Task 7). ✅
- Tauri (Rust core + web front + SQLite) → Tasks 1–11. ✅
- Transaction-based holdings → Tasks 2, 4, 5. ✅
- P&L (unrealized + realized, avg cost) → Task 4. ✅
- Charts (portfolio history via snapshots + coin price via API) → Tasks 7, 8, 11. ✅
- Multiple portfolios → Tasks 5, 7, 9, 10. ✅
- Offline/rate-limit resilience → Task 6 backoff + Task 7/9 `price_as_of` display. ✅
- Decimal money, TEXT storage, INTEGER ts → Global Constraints, Tasks 2/5. ✅
- Out-of-MVP (alerts, website, address watch) → intentionally excluded. ✅

**2. Placeholder scan:** No TBD/TODO; every code step contains full code. One inline typo to fix during Task 9 Step 4 (`#4 caf50` → `#4caf50`) is flagged in the step itself. ✅

**3. Type consistency:** `compute_holdings`/`value_holdings`/`Valuation`/`ValuedCoin` names match across Tasks 4, 7, 8. `NewTransaction` (no id) used by `add_transaction` in Tasks 5/7/10. Command names match `invoke()` calls in `api.ts` (Task 9). `refresh_prices` refactored once (Task 8) to call `scheduler::refresh_once` — single source of truth. ✅
