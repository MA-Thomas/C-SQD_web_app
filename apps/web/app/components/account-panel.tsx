"use client";

import { useEffect, useState } from "react";

import { updateDisplayName } from "../lib/csqd-api";
import { useSession } from "../lib/session";

/// Self-service account settings: identity readout + display-name update.
/// The display name is how authorship appears on the audit record, so the
/// onboarding path routes here right after first sign-in.
export function AccountPanel({ welcome = false }: { welcome?: boolean }) {
  const { user, refresh } = useSession();
  const [displayName, setDisplayName] = useState("");
  const [pending, setPending] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (user) {
      setDisplayName((current) => (current ? current : user.display_name));
    }
  }, [user]);

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setPending(true);
    setError(null);
    setSaved(false);

    try {
      await updateDisplayName(displayName);
      await refresh();
      setSaved(true);
    } catch (updateError) {
      setError(
        updateError instanceof Error
          ? updateError.message
          : "Could not update your display name.",
      );
    } finally {
      setPending(false);
    }
  };

  if (!user) {
    return null;
  }

  return (
    <section className="workspace-section first-workspace-section">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Your identity on the audit record</p>
          <h2>{welcome ? "Welcome — Set Your Name" : "Account"}</h2>
        </div>
      </div>

      {welcome ? (
        <p className="muted-copy">
          Reviews, challenges, and reports you author are attributed to this
          name with full provenance. Set it to how you want to be cited.
        </p>
      ) : null}

      <form className="element-review-form" onSubmit={submit}>
        <label>
          <span>Display name</span>
          <input
            onChange={(event) => setDisplayName(event.target.value)}
            required
            type="text"
            value={displayName}
          />
        </label>
        <label>
          <span>Email</span>
          <input disabled type="email" value={user.email} />
        </label>
        <label>
          <span>Roles</span>
          <input disabled type="text" value={user.roles.join(", ")} />
        </label>
        {error ? <p className="form-error">{error}</p> : null}
        {saved ? <p className="muted-copy">Saved.</p> : null}
        <button className="primary-action" disabled={pending} type="submit">
          {pending ? "Saving…" : "Save"}
        </button>
      </form>

      <p className="muted-copy">
        Sponsor, reviewer, and operator roles are granted by an operator.
        Signing out revokes this session; sign-in links are single-use.
      </p>
    </section>
  );
}
