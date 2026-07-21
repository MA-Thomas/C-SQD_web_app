"use client";

import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";

import { requestMagicLink } from "../../lib/csqd-api";
import { useSession } from "../../lib/session";

function SignInForm() {
  const searchParams = useSearchParams();
  const { user } = useSession();
  const [email, setEmail] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [issued, setIssued] = useState<{
    email: string;
    sign_in_url?: string;
  } | null>(null);
  const returnTo = safeReturnPath(searchParams.get("return_to")) ?? "/discover";
  const explain = searchParams.get("explain");

  if (user) {
    return (
      <article className="pub-auth">
        <p className="pub-kicker">Signed in</p>
        <h1>You are signed in as {user.display_name}</h1>
        <div className="pub-auth-actions">
          <Link className="primary-action" href={returnTo}>
            Continue
          </Link>
        </div>
      </article>
    );
  }

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setPending(true);
    setError(null);

    try {
      window.localStorage.setItem("csqd_return_to", returnTo);
    } catch {
      // best-effort
    }

    try {
      const result = await requestMagicLink(email);

      if (result) {
        setIssued({ email: result.email, sign_in_url: result.sign_in_url });
      } else {
        setError("Could not issue a sign-in link. Is the API running?");
      }
    } catch (requestError) {
      setError(
        requestError instanceof Error
          ? requestError.message
          : "Could not issue a sign-in link.",
      );
    } finally {
      setPending(false);
    }
  };

  return (
    <article className="pub-auth">
      <p className="pub-kicker">Identity required</p>
      <h1>Sign In To C-SQD</h1>
      <p>
        Public audit records are readable without an account. Submitting
        reviews, watching subjects, challenging audit claims, and sponsor or
        reviewer operations require identity and role state.
      </p>
      {explain ? <p className="pub-auth-note">{explain}</p> : null}

      {issued ? (
        issued.sign_in_url ? (
          <div>
            <p>
              A sign-in link was issued for <strong>{issued.email}</strong>.
              In this preview environment no email is sent — use the link
              directly:
            </p>
            <div className="pub-auth-actions">
              <a className="primary-action" href={issued.sign_in_url}>
                Complete sign-in
              </a>
            </div>
          </div>
        ) : (
          <div>
            <p>
              A sign-in link was sent to <strong>{issued.email}</strong>.
              Check your inbox — the link works for 15 minutes and can be
              used once.
            </p>
            <div className="pub-auth-actions">
              <button
                className="secondary-action"
                onClick={() => setIssued(null)}
                type="button"
              >
                Use a different address
              </button>
            </div>
          </div>
        )
      ) : (
        <form onSubmit={submit}>
          <label htmlFor="sign-in-email">Email address</label>
          <div className="pub-auth-controls">
            <input
              autoComplete="email"
              id="sign-in-email"
              onChange={(event) => setEmail(event.target.value)}
              placeholder="you@example.org"
              required
              type="email"
              value={email}
            />
            <button className="primary-action" disabled={pending} type="submit">
              {pending ? "Issuing…" : "Email me a sign-in link"}
            </button>
          </div>
          {error ? <p className="form-error">{error}</p> : null}
        </form>
      )}

      <div className="pub-auth-actions">
        <Link className="secondary-action" href={returnTo}>
          Continue to public record
        </Link>
        <Link className="secondary-action" href="/discover">
          Browse public registry
        </Link>
      </div>
    </article>
  );
}

export default function SignInPage() {
  return (
    <Suspense fallback={null}>
      <SignInForm />
    </Suspense>
  );
}

function safeReturnPath(value: string | null) {
  if (!value?.startsWith("/") || value.startsWith("//")) {
    return null;
  }

  return value;
}
