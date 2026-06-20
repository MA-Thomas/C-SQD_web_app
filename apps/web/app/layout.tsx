import type { Metadata } from "next";
import "./globals.css";

import { AdvancedModeProvider } from "./lib/advanced-mode";
import { SessionProvider } from "./lib/session";

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
      <body>
        <SessionProvider>
          <AdvancedModeProvider>{children}</AdvancedModeProvider>
        </SessionProvider>
      </body>
    </html>
  );
}
