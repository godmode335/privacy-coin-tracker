# Deploy runbook

## Cloudflare Pages
1. Push this repo to GitHub (done).
2. Cloudflare dashboard → Workers & Pages → Create → Pages → Connect to Git → select `privacy-coin-tracker`.
3. Build settings:
   - Framework preset: Astro
   - Build command: `npm run build`
   - Build output directory: `dist`
   - Root directory: `website`
4. Deploy. You get a `*.pages.dev` URL.

## Custom domain
1. Buy the domain (e.g. at Cloudflare Registrar or Namecheap).
2. Cloudflare Pages → your project → Custom domains → add the domain; follow DNS steps.
3. Update `SITE_URL` in `website/src/config.ts` and `site` in `website/astro.config.mjs` to the real domain (keep them identical), then commit and push.

## Analytics & Search Console
1. Cloudflare Pages → Web Analytics → enable for the domain (cookieless, no code changes).
2. Google Search Console → add the domain property → verify via DNS TXT → submit `https://<domain>/sitemap-index.xml`.

## Releasing a new app version
Re-run the release step with the new version tag, always attaching the asset named `PrivacyCoinTracker-Setup.exe` so the website button keeps working:

```
cp "src-tauri/target/release/bundle/nsis/Privacy Coin Tracker_<ver>_x64-setup.exe" "PrivacyCoinTracker-Setup.exe"
gh release create v<ver> "PrivacyCoinTracker-Setup.exe" --title "Privacy Coin Tracker v<ver>" --notes "..."
```
