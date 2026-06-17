use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
