export interface CoinInfo {
  id: string;
  name: string;
  symbol: string;
  privacyModel: string;
  blurb: string;
  trackingNote: string;
  keywords: string[];
}

export const COINS: CoinInfo[] = [
  {
    id: "zcash",
    name: "Zcash",
    symbol: "ZEC",
    privacyModel: "Optional shielded transactions using zk-SNARKs (z-addresses).",
    blurb:
      "Zcash lets you transact privately with zero-knowledge proofs. Shielded balances can't be read from the public chain — so tracking your ZEC reliably means recording your own transactions.",
    trackingNote:
      "Shielded (z-address) balances are encrypted and unreadable on-chain by design. The private way to track Zcash is to log your buys and sells locally and let the app compute holdings and P&L.",
    keywords: ["zcash portfolio tracker", "track zcash", "zec tracker", "zcash holdings"],
  },
  {
    id: "monero",
    name: "Monero",
    symbol: "XMR",
    privacyModel: "Private by default — ring signatures, stealth addresses, confidential amounts.",
    blurb:
      "Monero hides every transaction automatically. Because balances aren't public (and view keys don't reliably show spends), a local transaction-based tracker is the dependable way to follow your XMR.",
    trackingNote:
      "Monero is private by default, so there is no public balance to read and view keys miss outgoing spends. Record transactions locally for an accurate Monero portfolio.",
    keywords: ["monero portfolio tracker", "track monero", "xmr tracker", "monero holdings"],
  },
  {
    id: "dash",
    name: "Dash",
    symbol: "DASH",
    privacyModel: "Payments coin with optional PrivateSend mixing.",
    blurb:
      "Dash focuses on fast payments with optional privacy. Track your DASH holdings, cost basis and profit/loss locally, alongside your other privacy coins.",
    trackingNote:
      "Track DASH the simple way: enter your transactions and the app computes quantity, average cost and P&L — all stored locally on your PC.",
    keywords: ["dash portfolio tracker", "track dash", "dash holdings"],
  },
  {
    id: "firo",
    name: "Firo",
    symbol: "FIRO",
    privacyModel: "Lelantus Spark — burn-and-redeem to break transaction history.",
    blurb:
      "Firo uses a burn-and-redeem model to wipe transaction links. Keep tabs on your FIRO holdings and performance with a private, local-first tracker.",
    trackingNote:
      "Record your FIRO buys and sells locally to see holdings, average cost and profit/loss without exposing anything online.",
    keywords: ["firo portfolio tracker", "track firo", "firo holdings"],
  },
  {
    id: "zano",
    name: "Zano",
    symbol: "ZANO",
    privacyModel: "Ring signatures with confidential assets and hidden amounts.",
    blurb:
      "Zano combines ring signatures with confidential assets. Track your ZANO position privately, with cost basis and P&L computed on your own machine.",
    trackingNote:
      "Log ZANO transactions locally and the app handles the rest — quantity, average entry price and unrealized/realized P&L.",
    keywords: ["zano portfolio tracker", "track zano", "zano holdings"],
  },
];

export function getCoin(id: string): CoinInfo | undefined {
  return COINS.find((c) => c.id === id);
}
