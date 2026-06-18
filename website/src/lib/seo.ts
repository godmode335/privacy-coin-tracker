import {
  PRODUCT_NAME, SITE_URL, GITHUB_OWNER, GITHUB_REPO, DOWNLOAD_ASSET,
} from "../config";

export function downloadUrl(): string {
  return `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest/download/${DOWNLOAD_ASSET}`;
}

export function canonical(path: string): string {
  return new URL(path, SITE_URL).href;
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
