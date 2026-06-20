import Link from "next/link";

import { PublicNav } from "../components/public-nav";

/// Public registry shell: top navbar, full-width reading column, quiet
/// footer. No backstage chrome.
export default function PublicLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className="public-shell">
      <PublicNav />
      <main className="public-main">{children}</main>
      <footer className="public-footer">
        <div className="public-footer-inner">
          <span>
            C-SQD · public registry and method for epistemic audits
          </span>
          <nav aria-label="Footer">
            <Link href="/method">Method</Link>
            <Link href="/domains">Domains</Link>
            <Link href="/commission">Commission an Audit</Link>
            <Link href="/intake">Scholarly Works</Link>
            <Link href="/browse">CRWE</Link>
          </nav>
        </div>
      </footer>
    </div>
  );
}
