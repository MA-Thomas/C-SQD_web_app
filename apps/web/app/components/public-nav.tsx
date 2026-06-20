"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { useAdvancedMode } from "../lib/advanced-mode";
import { useSession } from "../lib/session";

const NAV_ITEMS = [
  { href: "/discover", label: "Discover" },
  { href: "/public-audits", label: "Public Audits" },
  { href: "/domains", label: "Domains" },
  { href: "/method", label: "Method" },
  { href: "/commission", label: "Commission an Audit" },
];

/// Public registry chrome: slim top navbar. No sponsor/reviewer/operations
/// surfaces here — the public face is the registry, not the console.
export function PublicNav() {
  const pathname = usePathname();
  const { user, loading, signOut, hasRole } = useSession();
  const { advanced, toggle } = useAdvancedMode();

  return (
    <header className="public-nav">
      <div className="public-nav-inner">
        <Link className="public-brand" href="/">
          <img alt="" className="public-brand-logo" src="/csqd-logo.png" />
          <span>
            <strong>C-SQD</strong>
            <small>Public epistemic audit registry</small>
          </span>
        </Link>

        <nav className="public-nav-links" aria-label="Primary">
          {NAV_ITEMS.map((item) => (
            <Link
              aria-current={pathname?.startsWith(item.href) ? "page" : undefined}
              className={pathname?.startsWith(item.href) ? "active" : undefined}
              href={item.href}
              key={item.href}
            >
              {item.label}
            </Link>
          ))}
        </nav>

        <div className="public-nav-side">
          <button
            className={`advanced-toggle${advanced ? " on" : ""}`}
            onClick={toggle}
            title="Toggle expert notation and provenance detail"
            type="button"
          >
            {advanced ? "Advanced: on" : "Advanced"}
          </button>
          {loading ? null : user ? (
            <div className="public-nav-session">
              <Link href="/library">{user.display_name}</Link>
              {hasRole("sponsor") ? <Link href="/sponsor-console">Sponsor</Link> : null}
              {hasRole("reviewer") ? <Link href="/reviewer-queue">Reviewer</Link> : null}
              {hasRole("operator") ? <Link href="/operations">Operations</Link> : null}
              <button onClick={() => void signOut()} type="button">
                Sign out
              </button>
            </div>
          ) : (
            <Link className="public-nav-signin" href="/sign-in">
              Sign in
            </Link>
          )}
        </div>
      </div>
    </header>
  );
}
