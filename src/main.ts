import { listPortfolios, createPortfolio, getDashboard, refreshPrices,
         getPortfolioHistory, getCoinHistory, listCoins } from "./api";
import { renderDashboard } from "./dashboard";
import { renderPortfolioChart, renderCoinChart } from "./charts";
import { mountForms } from "./forms";

const app = document.getElementById("app")!;
app.innerHTML = `
  <header>
    <h1>Privacy Coin Tracker</h1>
    <select id="portfolio-select"></select>
    <button id="refresh">Refresh prices</button>
  </header>
  <main id="dashboard"></main>
  <section class="charts">
    <canvas id="pf-chart" height="120"></canvas>
    <div><select id="coin-chart-select"></select><canvas id="coin-chart" height="120"></canvas></div>
  </section>`;

const select = document.getElementById("portfolio-select") as HTMLSelectElement;
const dashEl = document.getElementById("dashboard")!;
const coinSel = document.getElementById("coin-chart-select") as HTMLSelectElement;

const forms = mountForms({
  onChange: async () => { await bootstrap(); },
  currentPortfolioId: () => Number(select.value),
});
app.appendChild(forms);

async function bootstrap() {
  let portfolios = await listPortfolios();
  if (portfolios.length === 0) {
    await createPortfolio("Main");
    portfolios = await listPortfolios();
  }
  const current = select.value;
  select.innerHTML = portfolios.map((p) => `<option value="${p.id}">${p.name}</option>`).join("");
  if (current && portfolios.some((p) => String(p.id) === current)) {
    select.value = current;
  }
  await show(Number(select.value));
}

async function show(id: number) {
  renderDashboard(dashEl, await getDashboard(id));
  const pf = document.getElementById("pf-chart") as HTMLCanvasElement;
  renderPortfolioChart(pf, await getPortfolioHistory(id));
  if (!coinSel.dataset.filled) {
    const coins = await listCoins();
    coinSel.innerHTML = coins.map((c) => `<option value="${c.id}">${c.symbol}</option>`).join("");
    coinSel.dataset.filled = "1";
    coinSel.addEventListener("change", drawCoin);
  }
  await drawCoin();
}

async function drawCoin() {
  const cc = document.getElementById("coin-chart") as HTMLCanvasElement;
  renderCoinChart(cc, coinSel.value, await getCoinHistory(coinSel.value, 30));
}

select.addEventListener("change", () => show(Number(select.value)));
document.getElementById("refresh")!.addEventListener("click", async () => {
  await refreshPrices();
  await show(Number(select.value));
});

bootstrap();
