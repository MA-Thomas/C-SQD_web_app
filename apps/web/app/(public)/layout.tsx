import Link from "next/link";

import { SiteHeader } from "../components/site-header";

/// Public registry shell: sticky two-row header (brand + search, section
/// tabs), full-width reading canvas, quiet footer.
export default function PublicLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className="pub-shell">
      <SiteHeader />
      <main className="pub-main">{children}</main>
      <footer className="pub-footer">
        <div className="pub-footer-inner">
          <span>C-SQD · public registry and method for epistemic audits</span>
          <nav aria-label="Footer">
            <Link href="/method">Method</Link>
            <Link href="/domains">Domains</Link>
            <Link href="/criteria">Criteria</Link>
            <Link href="/register">Register a work</Link>
            <Link href="/commission">Commission an audit</Link>
          </nav>
        </div>
      </footer>
    </div>
  );
}
