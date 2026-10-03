import type { Metadata } from "next";
import Script from "next/script";
import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import { getMetadataBaseUrl } from "@/lib/site-url";
import "@fontsource-variable/dm-sans/wght.css";
import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";
import "./globals.css";

const themeInitialization = `(() => {
  let theme = "system";
  try {
    const saved = window.localStorage.getItem("princi-theme");
    if (saved === "light" || saved === "dark" || saved === "system") theme = saved;
  } catch {}
  document.documentElement.dataset.theme = theme;
})();`;
export const metadata: Metadata = {
  title: "PrinciPL — Princi Programming Language",
  description:
    "Explore Princi, a statically typed language compiled to native Windows x86-64 programs. Read the v0.1 language guide, installation steps, examples, and compiler architecture.",
  applicationName: "PrinciPL",
  metadataBase: getMetadataBaseUrl(),
  keywords: [
    "PrinciPL",
    "Princi Programming Language",
    "native Windows compiler",
    ".prnc",
    ".princi",
  ],
  openGraph: {
    type: "website",
    siteName: "PrinciPL",
    title: "PrinciPL — Princi Programming Language",
    description:
      "A statically typed language and Rust compiler for native Windows x86-64 programs.",
  },
  twitter: {
    card: "summary_large_image",
    title: "PrinciPL — Princi Programming Language",
    description: "A statically typed language and Rust compiler for native Windows x86-64 programs.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html data-scroll-behavior="smooth" lang="en" suppressHydrationWarning>
      <head>
        <Script
          dangerouslySetInnerHTML={{ __html: themeInitialization }}
          id="princi-theme-initialization"
          strategy="beforeInteractive"
        />
      </head>
      <body>
        <a className="skip-link" href="#main-content">
          Skip to content
        </a>
        <SiteHeader />
        {children}
        <SiteFooter />
      </body>
    </html>
  );
}
