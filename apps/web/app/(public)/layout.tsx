import Link from "next/link";

import { SiteHeader } from "../components/site-header";

/// Public registry shell: sticky two-row header (brand + search, section
/// tabs), full-width reading canvas, quiet footer.
export default function PublicLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const demoMode = process.env.NEXT_PUBLIC_DEMO_MODE === "1";

  return (
    <div className="pub-shell">
      {demoMode ? (
        <div className="pub-demo-banner" role="note">
          Demonstration environment — the audits, sponsors, and reports shown
          are illustrative seed data, not real audit activity.
        </div>
      ) : null}
      <SiteHeader />
      <main className="pub-main">{children}</main>
      <footer className="pub-footer">
        <div className="pub-footer-inner">
          <span>C-SQD · public registry and method for epistemic audits</span>
          <nav aria-label="Footer">
            <Link href="/method">Method</Link>
            <Link href="/method#vocabulary">Vocabulary</Link>
            <Link href="/claims">Claims under audit</Link>
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
