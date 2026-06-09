import Link from "next/link";

type ActiveItem =
  | "domains"
  | "browse"
  | "search"
  | "library"
  | "assignments"
  | "review-episodes"
  | "bounties"
  | "payments";

type AppSidebarProps = {
  activeItem: ActiveItem;
};

const navItems: Array<{
  id: ActiveItem;
  href: string;
  label: string;
}> = [
  { id: "domains", href: "/domains", label: "Domains" },
  { id: "library", href: "/library", label: "Library" },
];

const domainNavItems: Array<{
  id: ActiveItem;
  href: string;
  label: string;
}> = [
  { id: "browse", href: "/browse", label: "Browse" },
  { id: "search", href: "/", label: "Scholarly Search" },
  { id: "assignments", href: "/assignments", label: "Assignments" },
  { id: "review-episodes", href: "/", label: "Review episodes" },
  { id: "bounties", href: "/", label: "Bounties" },
  { id: "payments", href: "/", label: "Payments" },
];

export function AppSidebar({ activeItem }: AppSidebarProps) {
  return (
    <aside className="sidebar" aria-label="Primary">
      <div className="brand">
        <span className="brand-mark">
          <img alt="" className="brand-logo" src="/csqd-logo.png" />
        </span>
        <div>
          <strong>C-SQD</strong>
          <span>Epistemic audit infrastructure</span>
        </div>
      </div>

      <section className="domain-switcher" aria-label="Active domain">
        <span className="domain-switcher-label">Active domain</span>
        <Link className="domain-switcher-current" href="/domains">
          <strong>Academic Peer Review</strong>
          <span>Scholarly works, reviews, synthesis</span>
        </Link>
        <div className="domain-switcher-planned" aria-label="Planned domains">
          <span>Clinical Trial Protocol Review</span>
          <em>Planned</em>
        </div>
      </section>

      <nav className="nav-list">
        <p className="nav-section-label">C-SQD</p>
        {navItems.map((item) => (
          <Link
            aria-current={item.id === activeItem ? "page" : undefined}
            className={`nav-item${item.id === activeItem ? " active" : ""}`}
            href={item.href}
            key={item.id}
          >
            {item.label}
          </Link>
        ))}
        <p className="nav-section-label">Academic Peer Review</p>
        {domainNavItems.map((item) => (
          <Link
            aria-current={item.id === activeItem ? "page" : undefined}
            className={`nav-item${item.id === activeItem ? " active" : ""}`}
            href={item.href}
            key={item.id}
          >
            {item.label}
          </Link>
        ))}
      </nav>
    </aside>
  );
}
