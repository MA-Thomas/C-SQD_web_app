"use client";

import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";

import { completeMagicLink } from "../../../lib/csqd-api";
import { useSession } from "../../../lib/session";

function CompleteSignIn() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const { refresh } = useSession();
  const [state, setState] = useState<"working" | "failed">("working");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const token = searchParams.get("token");

    if (!token) {
      setState("failed");
      setError("Missing sign-in token.");
      return;
    }

    let cancelled = false;

    (async () => {
      try {
        const result = await completeMagicLink(token);

        if (cancelled) {
          return;
        }

        if (result?.user) {
          await refresh();
          let returnTo = "/discover";

          try {
            returnTo = window.localStorage.getItem("csqd_return_to") ?? returnTo;
            window.localStorage.removeItem("csqd_return_to");
          } catch {
            // best-effort
          }

          // Onboarding: a display name still equal to the derived
          // email-local-part guess means the user never chose one — route
          // through account setup first, preserving their destination.
          const derivedName = result.user.email
            .split("@")[0]
            ?.replace(/[._-]/g, " ")
            .trim();

          if (
            derivedName &&
            result.user.display_name.trim().toLowerCase() ===
              derivedName.toLowerCase()
          ) {
            try {
              window.localStorage.setItem("csqd_return_to", returnTo);
            } catch {
              // best-effort
            }

            router.replace("/account?welcome=1");
            return;
          }

          router.replace(returnTo);
        } else {
          setState("failed");
          setError("The sign-in link is invalid or has expired.");
        }
      } catch (completeError) {
        if (!cancelled) {
          setState("failed");
          setError(
            completeError instanceof Error
              ? completeError.message
              : "The sign-in link is invalid or has expired.",
          );
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [refresh, router, searchParams]);

  return (
    <article className="pub-auth">
      <p className="pub-kicker">Sign in</p>
      {state === "working" ? (
        <h1>Completing sign-in…</h1>
      ) : (
        <>
          <h1>Sign-in failed</h1>
          <p>{error}</p>
          <div className="pub-auth-actions">
            <Link className="primary-action" href="/sign-in">
              Request a new link
            </Link>
          </div>
        </>
      )}
    </article>
  );
}

export default function CompleteSignInPage() {
  return (
    <Suspense fallback={null}>
      <CompleteSignIn />
    </Suspense>
  );
}
