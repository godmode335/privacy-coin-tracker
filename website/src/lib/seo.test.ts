import { test, expect } from "vitest";
import {
  downloadUrl, canonical, softwareAppJsonLd, blogPostingJsonLd,
  faqJsonLd, breadcrumbJsonLd, organizationJsonLd, webSiteJsonLd, coinPageJsonLd,
} from "./seo";
import { DOWNLOAD_ASSETS } from "../config";

const RELEASES =
  "https://github.com/godmode335/privacy-coin-tracker/releases/latest/download";

test("downloadUrl defaults to the Windows installer", () => {
  expect(downloadUrl()).toBe(`${RELEASES}/PrivacyCoinTracker-Setup.exe`);
});

test("downloadUrl resolves each Linux asset", () => {
  expect(downloadUrl("linuxAppImage")).toBe(`${RELEASES}/PrivacyCoinTracker-x86_64.AppImage`);
  expect(downloadUrl("linuxDeb")).toBe(`${RELEASES}/PrivacyCoinTracker-amd64.deb`);
});

// The workflow copies bundles to these exact names; if the two ever drift the
// download buttons 404 while the site still builds and the tests still pass.
test("asset names match the ones release.yml publishes", () => {
  expect(Object.values(DOWNLOAD_ASSETS).sort()).toEqual([
    "PrivacyCoinTracker-Setup.exe",
    "PrivacyCoinTracker-amd64.deb",
    "PrivacyCoinTracker-x86_64.AppImage",
  ].sort());
});

test("softwareAppJsonLd advertises Linux alongside Windows", () => {
  const j = softwareAppJsonLd() as any;
  expect(j.operatingSystem).toContain("Windows");
  expect(j.operatingSystem).toContain("Linux");
});

test("canonical builds absolute trailing-slash URLs from a page path", () => {
  expect(canonical("/blog/zcash-vs-monero")).toBe(
    "https://privacycointracker.com/blog/zcash-vs-monero/"
  );
});

test("canonical leaves asset paths (with extension) untouched", () => {
  expect(canonical("/og-default.png")).toBe(
    "https://privacycointracker.com/og-default.png"
  );
});

test("canonical keeps the site root as a single slash", () => {
  expect(canonical("/")).toBe("https://privacycointracker.com/");
});

test("softwareAppJsonLd is a free Windows SoftwareApplication", () => {
  const j = softwareAppJsonLd() as any;
  expect(j["@type"]).toBe("SoftwareApplication");
  expect(j.operatingSystem).toContain("Windows");
  expect(j.offers.price).toBe("0");
});

test("blogPostingJsonLd carries headline and absolute url", () => {
  const j = blogPostingJsonLd({
    title: "Zcash vs Monero", description: "compare", date: "2026-06-18", slug: "zcash-vs-monero",
  }) as any;
  expect(j["@type"]).toBe("BlogPosting");
  expect(j.headline).toBe("Zcash vs Monero");
  expect(j.url).toBe("https://privacycointracker.com/blog/zcash-vs-monero/");
});

test("faqJsonLd maps items to Question/Answer", () => {
  const j = faqJsonLd([{ q: "Is it free?", a: "Yes." }]) as any;
  expect(j["@type"]).toBe("FAQPage");
  expect(j.mainEntity[0].name).toBe("Is it free?");
  expect(j.mainEntity[0].acceptedAnswer.text).toBe("Yes.");
});

test("breadcrumbJsonLd numbers positions from 1", () => {
  const j = breadcrumbJsonLd([
    { name: "Home", path: "/" }, { name: "Blog", path: "/blog" },
  ]) as any;
  expect(j.itemListElement[0].position).toBe(1);
  expect(j.itemListElement[1].item).toBe("https://privacycointracker.com/blog/");
});

test("organizationJsonLd is an Organization with absolute url and logo", () => {
  const j = organizationJsonLd() as any;
  expect(j["@type"]).toBe("Organization");
  expect(j.url).toBe("https://privacycointracker.com");
  expect(j.logo).toBe("https://privacycointracker.com/favicon.svg");
});

test("webSiteJsonLd is a WebSite", () => {
  const j = webSiteJsonLd() as any;
  expect(j["@type"]).toBe("WebSite");
  expect(j.name).toBe("Privacy Coin Tracker");
});

test("coinPageJsonLd is a per-coin SoftwareApplication with coin url", () => {
  const j = coinPageJsonLd({ id: "zcash", name: "Zcash" }) as any;
  expect(j["@type"]).toBe("SoftwareApplication");
  expect(j.name).toContain("Zcash");
  expect(j.url).toBe("https://privacycointracker.com/coins/zcash/");
  expect(j.offers.price).toBe("0");
});
