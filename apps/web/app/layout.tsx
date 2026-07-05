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
  title: "C-SQD",
  description: "Public epistemic audit registry",
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
