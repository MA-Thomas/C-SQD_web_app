"use client";

import Link from "next/link";
import { usePathname, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";

import { useAdvancedMode } from "../lib/advanced-mode";
import { useSession } from "../lib/session";

/// Five tabs: the visitor's core loop (read → explore → understand → buy).
/// Claims, Criteria, and Domains remain first-class pages, reachable from
/// Discover, Method, and the footer — they are reference depth, not
/// primary navigation.
const TABS = [
  { href: "/", label: "Home", exact: true },
  { href: "/discover", label: "Discover" },
  { href: "/audits", label: "Audit Reports" },
  { href: "/method", label: "Method" },
  { href: "/commission", label: "Commission" },
];

/// Public chrome: brand + persistent search + session row, then section
/// tabs. Reading is public; role surfaces appear only with the role.
export function SiteHeader() {
  const pathname = usePathname();
  const { user, loading, signOut, hasRole } = useSession();
  const { advanced, toggle } = useAdvancedMode();
  const condensed = useCondensedOnScroll();

  return (
    <header className={`pub-header${condensed ? " condensed" : ""}`}>
      <div className="pub-header-top">
        <Link className="pub-brand" href="/">
          <img alt="" src="/csqd-logo.png" />
          <span className="pub-brand-name">
            <strong>C-SQD</strong>
            <small>Epistemic audit registry</small>
          </span>
        </Link>

        <Suspense fallback={<div className="pub-search" />}>
          <SearchBox />
        </Suspense>

        <div className="pub-header-side">
          <button
            className={`pub-advanced-toggle${advanced ? " on" : ""}`}
            onClick={toggle}
            title="Toggle expert notation and provenance detail"
            type="button"
          >
            {advanced ? "Advanced: on" : "Advanced"}
          </button>
          {loading ? null : user ? (
            <div className="pub-session">
              <Link href="/account" title="Account settings">
                {user.display_name}
              </Link>
              <Link href="/library">Library</Link>
              {hasRole("sponsor") ? <Link href="/sponsor-console">Sponsor</Link> : null}
              {hasRole("reviewer") ? <Link href="/reviewer-queue">Reviewer</Link> : null}
              {hasRole("operator") ? <Link href="/operations">Operations</Link> : null}
              <button onClick={() => void signOut()} type="button">
                Sign out
              </button>
            </div>
          ) : (
            <Link className="pub-signin" href="/sign-in">
              Sign in
            </Link>
          )}
        </div>
      </div>

      <nav className="pub-tabs" aria-label="Primary">
        {TABS.map((tab) => {
          const active = tab.exact
            ? pathname === tab.href
            : pathname?.startsWith(tab.href);

          return (
            <Link
              aria-current={active ? "page" : undefined}
              className={active ? "active" : undefined}
              href={tab.href}
              key={tab.href}
            >
              {tab.label}
            </Link>
          );
        })}
        <span className="pub-tabs-spacer" />
        <Link
          className={`pub-tab-quiet${pathname?.startsWith("/register") ? " active" : ""}`}
          href="/register"
        >
          Register a work
        </Link>
      </nav>
    </header>
  );
}

/// Condenses the sticky header once the reader has scrolled past the top
/// of the page: the brand row tightens and the tagline collapses, keeping
/// the navigation tabs while the chrome recedes.
function useCondensedOnScroll(threshold = 64) {
  const [condensed, setCondensed] = useState(false);

  useEffect(() => {
    let ticking = false;

    const onScroll = () => {
      if (ticking) {
        return;
      }
      ticking = true;
      window.requestAnimationFrame(() => {
        setCondensed(window.scrollY > threshold);
        ticking = false;
      });
    };

    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });

    return () => window.removeEventListener("scroll", onScroll);
  }, [threshold]);

  return condensed;
}

/// Persistent header search. Submits to Discover; keeps the current query
/// visible while on Discover so refinement feels continuous.
function SearchBox() {
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const currentQuery =
    pathname === "/discover" ? (searchParams.get("q") ?? "") : "";

  return (
    <form action="/discover" className="pub-search" role="search">
      <div className="pub-search-box">
        <svg
          aria-hidden
          fill="none"
          height="16"
          stroke="currentColor"
          strokeLinecap="round"
          strokeWidth="2"
          viewBox="0 0 24 24"
          width="16"
        >
          <circle cx="11" cy="11" r="7" />
          <path d="m20 20-3.5-3.5" />
        </svg>
        <input
          aria-label="Search public audit subjects"
          defaultValue={currentQuery}
          key={currentQuery}
          name="q"
          placeholder="Search works, DOIs, authors, venues…"
          type="search"
        />
      </div>
    </form>
  );
}
