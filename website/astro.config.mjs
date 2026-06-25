import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";

// Keep this in sync with SITE_URL in src/config.ts
export default defineConfig({
  site: "https://privacycointracker.com",
  integrations: [sitemap()],
});
