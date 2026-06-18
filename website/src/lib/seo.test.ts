import { test, expect } from "vitest";
import {
  downloadUrl, canonical, softwareAppJsonLd, blogPostingJsonLd,
  faqJsonLd, breadcrumbJsonLd,
} from "./seo";

test("downloadUrl points at the stable latest-release asset", () => {
  expect(downloadUrl()).toBe(
    "https://github.com/godmode335/privacy-coin-tracker/releases/latest/download/PrivacyCoinTracker-Setup.exe"
  );
});

test("canonical builds absolute URLs from a path", () => {
  expect(canonical("/blog/zcash-vs-monero")).toBe(
    "https://privacycointracker.com/blog/zcash-vs-monero"
  );
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
  expect(j.url).toBe("https://privacycointracker.com/blog/zcash-vs-monero");
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
  expect(j.itemListElement[1].item).toBe("https://privacycointracker.com/blog");
});
