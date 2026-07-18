import {
  PRODUCT_NAME, SITE_URL, GITHUB_OWNER, GITHUB_REPO, DOWNLOAD_ASSET,
} from "../config";

export function downloadUrl(): string {
  return `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest/download/${DOWNLOAD_ASSET}`;
}

// Normalize a page path to the canonical trailing-slash form so that
// <link rel="canonical">, og:url, the sitemap and internal links all agree.
// Asset paths (with a file extension, e.g. /og-default.png) are left untouched.
export function withTrailingSlash(path: string): string {
  if (path === "/") return path;
  const [pathname, query] = path.split("?");
  const lastSegment = pathname.split("/").pop() ?? "";
  if (lastSegment.includes(".")) return path; // asset, keep as-is
  const normalized = pathname.endsWith("/") ? pathname : `${pathname}/`;
  return query ? `${normalized}?${query}` : normalized;
}

export function canonical(path: string): string {
  return new URL(withTrailingSlash(path), SITE_URL).href;
}

export function softwareAppJsonLd(): object {
  return {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: PRODUCT_NAME,
    applicationCategory: "FinanceApplication",
    operatingSystem: "Windows 10, Windows 11",
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    downloadUrl: downloadUrl(),
    screenshot: canonical("/screenshots/dashboard.png"),
    image: canonical("/og-default.png"),
  };
}

export function blogPostingJsonLd(a: {
  title: string; description: string; date: string; slug: string;
}): object {
  return {
    "@context": "https://schema.org",
    "@type": "BlogPosting",
    headline: a.title,
    description: a.description,
    datePublished: a.date,
    url: canonical(`/blog/${a.slug}`),
    author: { "@type": "Organization", name: PRODUCT_NAME },
    image: canonical("/og-default.png"),
  };
}

export function faqJsonLd(items: { q: string; a: string }[]): object {
  return {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: items.map((it) => ({
      "@type": "Question",
      name: it.q,
      acceptedAnswer: { "@type": "Answer", text: it.a },
    })),
  };
}

export function breadcrumbJsonLd(items: { name: string; path: string }[]): object {
  return {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: items.map((it, i) => ({
      "@type": "ListItem",
      position: i + 1,
      name: it.name,
      item: canonical(it.path),
    })),
  };
}

export function organizationJsonLd(): object {
  return {
    "@context": "https://schema.org",
    "@type": "Organization",
    name: PRODUCT_NAME,
    url: SITE_URL,
    logo: canonical("/favicon.svg"),
  };
}

export function webSiteJsonLd(): object {
  return {
    "@context": "https://schema.org",
    "@type": "WebSite",
    name: PRODUCT_NAME,
    url: SITE_URL,
  };
}

export function coinPageJsonLd(coin: { id: string; name: string }): object {
  return {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: `${coin.name} Portfolio Tracker — ${PRODUCT_NAME}`,
    applicationCategory: "FinanceApplication",
    operatingSystem: "Windows 10, Windows 11",
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    downloadUrl: downloadUrl(),
    url: canonical(`/coins/${coin.id}`),
  };
}
