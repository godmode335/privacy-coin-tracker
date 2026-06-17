use std::collections::HashMap;
use std::str::FromStr;
use rust_decimal::Decimal;

pub struct PriceClient {
    base_url: String,
    http: reqwest::Client,
}

// Parse a JSON number via its canonical string form, not via f64, so values
// like 0.0033 stay exact (avoids float rounding in money/price data).
fn num_to_decimal(v: &serde_json::Value) -> Decimal {
    match v {
        serde_json::Value::Number(n) => Decimal::from_str(&n.to_string()).unwrap_or(Decimal::ZERO),
        _ => Decimal::ZERO,
    }
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
