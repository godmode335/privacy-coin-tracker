import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";

// Keep this in sync with SITE_URL in src/config.ts
export default defineConfig({
  site: "https://privacycointracker.com",
  // Canonical URL shape: always a trailing slash (e.g. /about/, /coins/dash/).
  // Must match what Cloudflare Pages serves and what the sitemap emits.
  trailingSlash: "always",
  integrations: [sitemap()],
});
