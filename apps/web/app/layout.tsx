import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "C-SQD",
  description: "Epistemic audit infrastructure",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
