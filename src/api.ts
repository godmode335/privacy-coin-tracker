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
