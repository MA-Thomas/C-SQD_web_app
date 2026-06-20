"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { useSession } from "../lib/session";

/// Visible-but-auth-gated action. Signed-in users go straight to `href`;
/// signed-out users go to sign-in with the destination preserved.
export function GatedAction({
  href,
  className = "secondary-action",
  children,
  explain,
}: {
  href: string;
  className?: string;
  children: React.ReactNode;
  explain?: string;
}) {
  const { user } = useSession();
  const pathname = usePathname();

  if (user) {
    return (
      <Link className={className} href={href}>
        {children}
      </Link>
    );
  }

  const params = new URLSearchParams({ return_to: href });

  if (explain) {
    params.set("explain", explain);
  }

  return (
    <Link
      className={`${className} gated`}
      href={`/sign-in?${params.toString()}`}
      title={explain ?? "Sign in required"}
    >
      {children}
      <span className="gated-marker" aria-hidden>
        ·
      </span>
      <span className="gated-hint">sign in</span>
    </Link>
  );
}
