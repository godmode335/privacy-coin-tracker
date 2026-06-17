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

pub fn all() -> &'static [Coin] {
    COINS
}

pub fn by_id(id: &str) -> Option<&'static Coin> {
    COINS.iter().find(|c| c.id == id)
}

pub fn coingecko_ids() -> Vec<&'static str> {
    COINS.iter().map(|c| c.coingecko_id).collect()
}

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
