# Website SEO & Design Upgrade Plan

**Date:** 2026-06-18
**Branch:** feature/mvp
**Goal:** Increase organic installs by adding scalable per-coin SEO pages, completing technical SEO/schema, and polishing landing design with real app screenshots.

**Builds on:** the shipped Astro site (`website/`). Reuses `BaseLayout`, `DownloadButton`, and the tested `seo.ts`.

## Global Constraints (carry over)

- Astro 5, zero client JS on content pages, unique title/description/canonical/OG per page.
- Download link stays `…/releases/latest/download/PrivacyCoinTracker-Setup.exe`.
- Product name "Privacy Coin Tracker". Privacy brand: no third-party trackers.
- All new SEO logic in `seo.ts` is unit-tested (vitest).

---

## Track 2 — Technical SEO & schema (do first; pure code)

**Files:** `website/src/lib/seo.ts` (+ `seo.test.ts`), `website/src/layouts/BaseLayout.astro`, `website/src/pages/index.astro`

- Add tested builders:
  - `organizationJsonLd()` → `Organization` (name, url, logo).
  - `webSiteJsonLd()` → `WebSite` (name, url).
- Inject Organization + WebSite JSON-LD on the homepage.
- Add a default OG image: create `website/public/og-default.png` (1200×630, branded) and set it as the BaseLayout `ogImage` default.
- Verify: `npm test` green; built `dist/index.html` contains `"Organization"` and `"WebSite"`; OG image referenced.

## Track 1 — Programmatic per-coin pages (`/coins/<coin>`)

**Files:** `website/src/data/coins.ts`, `website/src/pages/coins/[coin].astro`, `website/src/pages/coins/index.astro`, link from `index.astro` + `Header.astro`

- `data/coins.ts`: array of 5 coins `{ id, name, symbol, privacyModel, blurb, keywords[] }` (Zcash, Monero, Dash, Firo, Zano).
- `seo.ts`: add tested `coinPageJsonLd(coin)` → `SoftwareApplication` specialized per coin + reuse `faqJsonLd`/`breadcrumbJsonLd`.
- `/coins/<coin>` page (getStaticPaths over coins): H1 "<Coin> Portfolio Tracker", intro, "how to track <Coin> privately", features, per-coin FAQ, DownloadButton, JSON-LD (SoftwareApplication + FAQ + Breadcrumb), canonical, meta targeting "<coin> portfolio tracker".
- `/coins` index linking all five. Add "Coins" to header nav.
- Verify: 5 pages build under `dist/coins/`; each has canonical + SoftwareApplication; appears in sitemap.

## Track 3 — Design polish + real screenshots

**Files:** `website/src/styles/global.css`, `website/src/pages/index.astro`, `website/public/screenshots/*`, optionally a `Screenshot.astro`

- Use `ui-ux-pro-max` skill to define a cohesive visual system (palette, type scale, spacing, hero, cards, buttons, responsive) fitting a privacy/fintech tone; apply to `global.css`.
- Real screenshots: seed the app's local SQLite DB with sample data (Node 24 `node:sqlite`), launch the built app, capture its window (PowerShell .NET window capture) → `website/public/screenshots/dashboard.png` (+ 1–2 more). Fallback: high-fidelity HTML/CSS mockup screenshot if window capture is unreliable.
- Show the hero screenshot on the landing and a relevant one on coin pages.
- Verify: landing renders screenshot; Lighthouse-style checks (build clean, images optimized).

## Acceptance

- `npm test` and `npm run check` green.
- `npm run build` produces: home with Organization+WebSite JSON-LD, 5 `/coins/*` pages in sitemap, screenshots present, all pages with unique meta + canonical + OG.
- Visual: landing looks polished (not default), with a real product screenshot above or near the fold.

## Out of scope (next wave)

- Distribution (Track 5): launch checklist + directory submissions (AlternativeTo, Product Hunt, GitHub topics, communities).
- Track 4: more articles + alternative pages.
