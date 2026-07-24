export const PRODUCT_NAME = "Privacy Coin Tracker";
// Production domain — keep identical to `site` in astro.config.mjs
export const SITE_URL = "https://privacycointracker.com";
export const GITHUB_OWNER = "godmode335";
export const GITHUB_REPO = "privacy-coin-tracker";
// Stable release asset names uploaded to every GitHub Release by
// .github/workflows/release.yml. These are a public contract: the download
// buttons link to releases/latest/download/<name>, so renaming one here
// without renaming it in the workflow breaks the button silently.
export const DOWNLOAD_ASSETS = {
  windows: "PrivacyCoinTracker-Setup.exe",
  linuxAppImage: "PrivacyCoinTracker-x86_64.AppImage",
  linuxDeb: "PrivacyCoinTracker-amd64.deb",
} as const;

export type DownloadPlatform = keyof typeof DOWNLOAD_ASSETS;

/** Operating systems the app ships for — used in copy and JSON-LD. */
export const SUPPORTED_OS = "Windows 10, Windows 11, Linux";
