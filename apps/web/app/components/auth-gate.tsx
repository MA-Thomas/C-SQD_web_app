"use client";

import Link from "next/link";

import { useSession } from "../lib/session";

type AuthGateProps = {
  eyebrow: string;
  title: string;
  body: string;
  returnTo: string;
  role?: "member" | "sponsor" | "reviewer" | "operator";
  children?: React.ReactNode;
};

/// Backstage gate: renders children for an authorized session, otherwise a
/// sign-in prompt that preserves the destination. Content-only — the
/// backstage layout supplies the shell.
export function AuthGate({
  body,
  children,
  eyebrow,
  returnTo,
  role = "member",
  title,
}: AuthGateProps) {
  const { user, loading, hasRole } = useSession();

  if (loading) {
    return (
      <article className="auth-panel">
        <p className="eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p>Checking your session…</p>
      </article>
    );
  }

  if (user && (role === "member" || hasRole(role))) {
    return <>{children}</>;
  }

  const params = new URLSearchParams({ return_to: returnTo, role });

  return (
    <article className="auth-panel">
      <p className="eyebrow">{eyebrow}</p>
      <h1>{title}</h1>
      <p>{body}</p>
      {user ? (
        <p className="auth-note">
          You are signed in as {user.display_name}, but this area requires the{" "}
          {role} role.
        </p>
      ) : null}
      <div className="source-actions">
        {user ? null : (
          <Link className="primary-action" href={`/sign-in?${params.toString()}`}>
            Sign in
          </Link>
        )}
        <Link className="secondary-action" href="/discover">
          Browse public registry
        </Link>
      </div>
    </article>
  );
}
