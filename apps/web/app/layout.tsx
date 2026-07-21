import type { Metadata } from "next";
import { Inter, Newsreader } from "next/font/google";
import "./globals.css";
import "./public.css";

import { AdvancedModeProvider } from "./lib/advanced-mode";
import { SessionProvider } from "./lib/session";

/// Type pairing: Newsreader (editorial serif) carries headlines and
/// long-form report prose; Inter (neutral sans) carries chrome, labels,
/// and data. Exposed as CSS variables consumed by public.css.
const sans = Inter({
  subsets: ["latin"],
  variable: "--font-sans",
  display: "swap",
});

const serif = Newsreader({
  subsets: ["latin"],
  style: ["normal", "italic"],
  variable: "--font-serif",
  display: "swap",
});

export const metadata: Metadata = {
  title: {
    default: "C-SQD · Public epistemic audit registry",
    template: "%s · C-SQD",
  },
  description:
    "Commissioned and public audits of scientific and technical claims — structured criterion-level reviews, synthesis reports, and challenges, all on the record.",
  icons: {
    icon: "/csqd-logo.png",
  },
  openGraph: {
    title: "C-SQD · Public epistemic audit registry",
    description:
      "Structured, decomposed audits of important scientific and technical claims: criterion-level reviews, synthesis reports, and challenge trails with full provenance.",
    siteName: "C-SQD",
    type: "website",
    images: ["/csqd-logo.png"],
  },
  twitter: {
    card: "summary",
    title: "C-SQD · Public epistemic audit registry",
    description:
      "Structured, decomposed audits of important scientific and technical claims.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${sans.variable} ${serif.variable}`}>
        <SessionProvider>
          <AdvancedModeProvider>{children}</AdvancedModeProvider>
        </SessionProvider>
      </body>
    </html>
  );
}
