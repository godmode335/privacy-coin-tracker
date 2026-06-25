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
