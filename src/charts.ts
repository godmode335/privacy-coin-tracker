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
