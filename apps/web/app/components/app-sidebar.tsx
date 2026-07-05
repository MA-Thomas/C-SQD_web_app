"use client";

import Link from "next/link";

import { useSession } from "../lib/session";

export type ActiveItem =
  | "console"
  | "commission"
  | "discover"
  | "domains"
  | "method"
  | "audits"
  | "element-reviews"
  | "synthesis-reviews"
  | "challenges"
  | "sponsor-console"
  | "reviewer-queue"
  | "register"
  | "criteria"
  | "library"
  | "sign-in";

type AppSidebarProps = {
  activeItem?: ActiveItem;
};

type NavItem = { id: ActiveItem; href: string; label: string };

const registryNavItems: NavItem[] = [
  { id: "discover", href: "/discover", label: "Discover" },
  { id: "audits", href: "/audits", label: "Audit Reports" },
  { id: "method", href: "/method", label: "Method" },
];

const workspaceNavItems: NavItem[] = [
  { id: "library", href: "/library", label: "Library / Watchlist" },
];

const sponsorNavItems: NavItem[] = [
  { id: "sponsor-console", href: "/sponsor-console", label: "Sponsor Console" },
];

const reviewerNavItems: NavItem[] = [
  { id: "reviewer-queue", href: "/reviewer-queue", label: "Reviewer Queue" },
];

const operationsNavItems: NavItem[] = [
  { id: "console", href: "/operations", label: "Audit Operations" },
];

/// Backstage chrome only. Sections render per the session's actual roles —
/// an authenticated member without sponsor/reviewer/operator roles sees only
/// their account workspace and the way back to the public registry.
export function AppSidebar({ activeItem }: AppSidebarProps) {
  const { user, loading, hasRole } = useSession();

  return (
    <aside className="sidebar" aria-label="Backstage">
      <div className="brand">
        <span className="brand-mark">
          <img alt="" className="brand-logo" src="/csqd-logo.png" />
        </span>
        <div>
          <strong>C-SQD</strong>
          <span>Audit operations</span>
        </div>
      </div>

      <nav className="nav-list">
        <NavSection activeItem={activeItem} items={registryNavItems} label="Public registry" />
        {user ? (
          <NavSection activeItem={activeItem} items={workspaceNavItems} label="Account" />
        ) : null}
        {hasRole("sponsor") ? (
          <NavSection activeItem={activeItem} items={sponsorNavItems} label="Sponsor" />
        ) : null}
        {hasRole("reviewer") ? (
          <NavSection activeItem={activeItem} items={reviewerNavItems} label="Reviewer" />
        ) : null}
        {hasRole("operator") ? (
          <NavSection activeItem={activeItem} items={operationsNavItems} label="Operations" />
        ) : null}
        {!loading && !user ? (
          <>
            <p className="nav-section-label">Account</p>
            <Link className="nav-item" href="/sign-in">
              Sign in
            </Link>
          </>
        ) : null}
      </nav>
    </aside>
  );
}

function NavSection({
  activeItem,
  items,
  label,
}: {
  activeItem?: ActiveItem;
  items: NavItem[];
  label: string;
}) {
  return (
    <>
      <p className="nav-section-label">{label}</p>
      {items.map((item) => (
        <Link
          aria-current={item.id === activeItem ? "page" : undefined}
          className={`nav-item${item.id === activeItem ? " active" : ""}`}
          href={item.href}
          key={item.id}
        >
          {item.label}
        </Link>
      ))}
    </>
  );
}
