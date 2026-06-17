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
