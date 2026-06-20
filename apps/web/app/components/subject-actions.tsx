"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";

import {
  contestSynthesisReview,
  joinPublicEpisode,
  startPublicEpisode,
  submitChallengeResponse,
  submitCwePetition,
  submitFeaturePetition,
  submitSynthesisReview,
} from "../lib/csqd-api";
import { useSession } from "../lib/session";

function SignInLink({ subjectPath, label }: { subjectPath: string; label: string }) {
  return (
    <Link
      className="secondary-action gated"
      href={`/sign-in?return_to=${encodeURIComponent(subjectPath)}`}
    >
      {label}
      <span className="gated-hint">sign in</span>
    </Link>
  );
}

function useAction() {
  const router = useRouter();
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);

  const run = async (action: () => Promise<unknown>) => {
    setPending(true);
    setError(null);

    try {
      await action();
      setDone(true);
      router.refresh();
    } catch (actionError) {
      setError(
        actionError instanceof Error ? actionError.message : "Action failed.",
      );
    } finally {
      setPending(false);
    }
  };

  return { pending, error, done, run };
}

/// Start (or join) a public audit episode for this subject.
export function EpisodeParticipationActions({
  subjectPath,
  auditSubjectId,
  subjectTitle,
  openEpisodeId,
}: {
  subjectPath: string;
  auditSubjectId: string | null;
  subjectTitle: string;
  openEpisodeId: string | null;
}) {
  const { user } = useSession();
  const { pending, error, done, run } = useAction();

  if (!user) {
    return (
      <>
        <SignInLink subjectPath={subjectPath} label="Start public audit episode" />
        {openEpisodeId ? (
          <SignInLink subjectPath={subjectPath} label="Join public audit episode" />
        ) : null}
      </>
    );
  }

  if (done) {
    return <span className="action-confirmation">Participation recorded.</span>;
  }

  return (
    <>
      {auditSubjectId ? (
        <button
          className="secondary-action"
          disabled={pending}
          onClick={() =>
            void run(() =>
              startPublicEpisode(
                auditSubjectId,
                `Public review of ${subjectTitle}`.slice(0, 120),
              ),
            )
          }
          type="button"
        >
          Start public audit episode
        </button>
      ) : null}
      {openEpisodeId ? (
        <button
          className="secondary-action"
          disabled={pending}
          onClick={() => void run(() => joinPublicEpisode(openEpisodeId))}
          type="button"
        >
          Join public audit episode
        </button>
      ) : null}
      {error ? <span className="form-error">{error}</span> : null}
    </>
  );
}

/// Contest a synthesis review (challenge that preserves the record).
export function ContestReportForm({
  episodeId,
  subjectPath,
  reviewId,
}: {
  episodeId: string;
  subjectPath: string;
  reviewId: string;
}) {
  const { user } = useSession();
  const { pending, error, done, run } = useAction();
  const [rationale, setRationale] = useState("");
  const [scope, setScope] = useState<"partial" | "full">("partial");

  if (!user) {
    return <SignInLink subjectPath={subjectPath} label="Challenge this report" />;
  }

  if (done) {
    return (
      <span className="action-confirmation">
        Contestation recorded. The challenged report is preserved.
      </span>
    );
  }

  return (
    <details className="inline-action-form">
      <summary>Challenge this report</summary>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void run(async () => {
            const participation = await joinPublicEpisode(episodeId);

            if (!participation) {
              throw new Error("Could not join the public audit episode.");
            }

            const contestingReview = await submitSynthesisReview(episodeId, {
              summary: rationale,
              status: "current",
              sections: [],
              featured: false,
              unsolicited: true,
            });

            if (!contestingReview) {
              throw new Error("Could not record the contestation review.");
            }

            const relation = await contestSynthesisReview(
              contestingReview.id,
              reviewId,
              scope,
              rationale,
            );

            if (!relation) {
              throw new Error("Could not record the contestation relation.");
            }
          });
        }}
      >
        <label>
          Scope
          <select
            onChange={(event) => setScope(event.target.value as "partial" | "full")}
            value={scope}
          >
            <option value="partial">Partial — specific claims</option>
            <option value="full">Full — the overall interpretation</option>
          </select>
        </label>
        <label>
          Rationale
          <textarea
            onChange={(event) => setRationale(event.target.value)}
            placeholder="What does this report get wrong, and what evidence supports the contestation?"
            required
            rows={3}
            value={rationale}
          />
        </label>
        <button disabled={pending} type="submit">
          {pending ? "Submitting…" : "Submit contestation"}
        </button>
        {error ? <p className="form-error">{error}</p> : null}
      </form>
    </details>
  );
}

/// Challenge an ElementReview via a contesting submitter response.
export function ChallengeElementReviewForm({
  subjectPath,
  episodeId,
  factId,
}: {
  subjectPath: string;
  episodeId: string | null;
  factId: string;
}) {
  const { user } = useSession();
  const { pending, error, done, run } = useAction();
  const [content, setContent] = useState("");

  if (!user) {
    return <SignInLink subjectPath={subjectPath} label="Challenge this review" />;
  }

  if (!episodeId) {
    return null;
  }

  if (done) {
    return <span className="action-confirmation">Challenge recorded.</span>;
  }

  return (
    <details className="inline-action-form">
      <summary>Challenge this review</summary>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void run(() =>
            submitChallengeResponse(episodeId, [factId], "contests", content),
          );
        }}
      >
        <label>
          Contestation
          <textarea
            onChange={(event) => setContent(event.target.value)}
            placeholder="Contest the content, status, or interpretation of this review."
            required
            rows={3}
            value={content}
          />
        </label>
        <button disabled={pending} type="submit">
          {pending ? "Submitting…" : "Submit challenge"}
        </button>
        {error ? <p className="form-error">{error}</p> : null}
      </form>
    </details>
  );
}

/// Petition for someone else's ElementReview to be featured.
export function FeaturePetitionForm({
  subjectPath,
  episodeId,
  factId,
}: {
  subjectPath: string;
  episodeId: string | null;
  factId: string;
}) {
  const { user } = useSession();
  const { pending, error, done, run } = useAction();
  const [rationale, setRationale] = useState("");

  if (!user) {
    return <SignInLink subjectPath={subjectPath} label="Petition to feature" />;
  }

  if (!episodeId) {
    return null;
  }

  if (done) {
    return <span className="action-confirmation">Petition recorded.</span>;
  }

  return (
    <details className="inline-action-form">
      <summary>Petition to feature</summary>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void run(() => submitFeaturePetition(episodeId, factId, rationale));
        }}
      >
        <label>
          Why should this review be featured?
          <textarea
            onChange={(event) => setRationale(event.target.value)}
            required
            rows={2}
            value={rationale}
          />
        </label>
        <button disabled={pending} type="submit">
          {pending ? "Submitting…" : "Submit petition"}
        </button>
        {error ? <p className="form-error">{error}</p> : null}
      </form>
    </details>
  );
}

/// Petition for a new CRWE element or applicability of an existing one.
export function CwePetitionForm({
  subjectPath,
  episodeId,
  nodes,
}: {
  subjectPath: string;
  episodeId: string | null;
  nodes: Array<{ id: string; label: string }>;
}) {
  const { user } = useSession();
  const { pending, error, done, run } = useAction();
  const [kind, setKind] = useState<"new_element" | "applicability">("new_element");
  const [cweNode, setCweNode] = useState("");
  const [proposedLabel, setProposedLabel] = useState("");
  const [rationale, setRationale] = useState("");

  if (!user) {
    return <SignInLink subjectPath={subjectPath} label="Petition CRWE change" />;
  }

  if (!episodeId) {
    return (
      <span className="muted-copy">
        Start a public episode to petition CRWE changes.
      </span>
    );
  }

  if (done) {
    return <span className="action-confirmation">CRWE petition recorded.</span>;
  }

  return (
    <details className="inline-action-form">
      <summary>Petition CRWE change</summary>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void run(() =>
            submitCwePetition(episodeId, {
              kind,
              cwe_node: kind === "applicability" ? cweNode : undefined,
              proposed_label: kind === "new_element" ? proposedLabel : undefined,
              rationale,
            }),
          );
        }}
      >
        <label>
          Petition type
          <select
            onChange={(event) =>
              setKind(event.target.value as "new_element" | "applicability")
            }
            value={kind}
          >
            <option value="new_element">New CRWE element</option>
            <option value="applicability">Applicability of existing element</option>
          </select>
        </label>
        {kind === "applicability" ? (
          <label>
            Existing element
            <select
              onChange={(event) => setCweNode(event.target.value)}
              required
              value={cweNode}
            >
              <option value="">Choose a criterion…</option>
              {nodes.map((node) => (
                <option key={node.id} value={node.id}>
                  {node.label}
                </option>
              ))}
            </select>
          </label>
        ) : (
          <label>
            Proposed element label
            <input
              onChange={(event) => setProposedLabel(event.target.value)}
              required
              type="text"
              value={proposedLabel}
            />
          </label>
        )}
        <label>
          Why does the current taxonomy not cover this audit need?
          <textarea
            onChange={(event) => setRationale(event.target.value)}
            required
            rows={3}
            value={rationale}
          />
        </label>
        <button disabled={pending} type="submit">
          {pending ? "Submitting…" : "Submit petition"}
        </button>
        {error ? <p className="form-error">{error}</p> : null}
      </form>
    </details>
  );
}
