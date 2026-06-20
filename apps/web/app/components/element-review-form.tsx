"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";

import {
  joinPublicEpisode,
  startPublicEpisode,
  submitElementReview,
  submitSynthesisReview,
  type AuditEpisode,
} from "../lib/csqd-api";
import { useSession } from "../lib/session";

type NodeOption = {
  id: string;
  label: string;
  description: string;
  existingReviewCount: number;
};

/// The unsolicited contribution flow. Ensures a public episode exists (start
/// one when needed), then submits the review with the session identity. The
/// backend marks the result unsolicited (no solicitation reference).
export function ElementReviewForm({
  auditSubjectId,
  episodes,
  nodes,
  preselectedCriterion,
  subjectPath,
  subjectTitle,
  synthesisMode,
}: {
  auditSubjectId: string | null;
  episodes: Array<Pick<AuditEpisode, "id" | "label" | "status">>;
  nodes: NodeOption[];
  preselectedCriterion: string | null;
  subjectPath: string;
  subjectTitle: string;
  synthesisMode: boolean;
}) {
  const { user, loading } = useSession();
  const router = useRouter();
  const [criterion, setCriterion] = useState(preselectedCriterion ?? "");
  const [finding, setFinding] = useState("inconclusive");
  const [severity, setSeverity] = useState("");
  const [confidence, setConfidence] = useState("");
  const [content, setContent] = useState("");
  const [limitations, setLimitations] = useState("");
  const [recommendations, setRecommendations] = useState("");
  const [summary, setSummary] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);

  if (loading) {
    return <p className="muted-copy">Checking your session…</p>;
  }

  if (!user) {
    const returnTo = `${subjectPath}/review${
      preselectedCriterion ? `?criterion=${encodeURIComponent(preselectedCriterion)}` : ""
    }`;

    return (
      <article className="auth-panel">
        <p className="eyebrow">Identity required</p>
        <h1>
          {synthesisMode
            ? "Submitting a SynthesisReview requires sign in"
            : "Review one criterion requires sign in"}
        </h1>
        <p>
          {synthesisMode
            ? "Unsolicited SynthesisReviews are integrative interpretations of a public AuditEpisode. You must start or join the episode first, which requires identity."
            : "ElementReviews are focused reviews of one CRWE criterion. Identity gives the review provenance, moderation state, and a durable relationship to the public audit subject."}
        </p>
        <div className="source-actions">
          <Link
            className="primary-action"
            href={`/sign-in?return_to=${encodeURIComponent(returnTo)}`}
          >
            Sign in
          </Link>
          <Link className="secondary-action" href={subjectPath}>
            Back to subject
          </Link>
        </div>
      </article>
    );
  }

  if (done) {
    return (
      <article className="auth-panel">
        <p className="eyebrow">Submitted</p>
        <h1>
          {synthesisMode
            ? "SynthesisReview submitted"
            : "Unsolicited ElementReview submitted"}
        </h1>
        <p>
          Your contribution is now part of the public audit record. It may be
          cited by later SynthesisReviews, challenged, moderated, or
          superseded — the record preserves all of it.
        </p>
        <div className="source-actions">
          <Link className="primary-action" href={subjectPath}>
            Back to the public audit subject
          </Link>
        </div>
      </article>
    );
  }

  const ensureParticipatingEpisode = async (): Promise<string> => {
    const open = episodes.find((episode) => episode.status === "active");

    if (open) {
      if (synthesisMode) {
        const participation = await joinPublicEpisode(open.id);

        if (!participation) {
          throw new Error("Could not join the public audit episode.");
        }
      }

      return open.id;
    }

    if (!auditSubjectId) {
      throw new Error(
        "This work is not yet registered as an audit subject. Register it from Scholarly Works first.",
      );
    }

    const started = await startPublicEpisode(
      auditSubjectId,
      `Public review of ${subjectTitle}`.slice(0, 120),
    );

    if (!started) {
      throw new Error("Could not start a public audit episode.");
    }

    return started.episode.id;
  };

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setPending(true);
    setError(null);

    try {
      const episodeId = await ensureParticipatingEpisode();

      if (synthesisMode) {
        await submitSynthesisReview(episodeId, {
          summary,
          status: "current",
          sections: [],
          featured: false,
          unsolicited: true,
        });
      } else {
        await submitElementReview(episodeId, {
          cwe_node_id: criterion,
          finding,
          severity: severity || null,
          confidence: confidence || null,
          limitations: limitations || null,
          recommendations: recommendations || null,
          content,
          featured: false,
          solicitation: null,
          submitted_by: null,
        });
      }

      setDone(true);
      router.refresh();
    } catch (submitError) {
      setError(
        submitError instanceof Error ? submitError.message : "Submission failed.",
      );
    } finally {
      setPending(false);
    }
  };

  const selectedNode = nodes.find((node) => node.id === criterion);

  if (synthesisMode) {
    return (
      <form className="review-form" onSubmit={submit}>
        <label>
          Integrative summary
          <textarea
            onChange={(event) => setSummary(event.target.value)}
            placeholder="Integrate the episode's ElementReviews and facts into a higher-level interpretation: key findings, recommendations, open questions."
            required
            rows={8}
            value={summary}
          />
        </label>
        <p className="muted-copy">
          Submitting will join the public episode if needed. Your review is
          marked unsolicited in the audit record.
        </p>
        <button className="primary-action" disabled={pending} type="submit">
          {pending ? "Submitting…" : "Submit unsolicited SynthesisReview"}
        </button>
        {error ? <p className="form-error">{error}</p> : null}
      </form>
    );
  }

  return (
    <form className="review-form" onSubmit={submit}>
      <label>
        CRWE criterion
        <select
          onChange={(event) => setCriterion(event.target.value)}
          required
          value={criterion}
        >
          <option value="">Choose a criterion…</option>
          {nodes.map((node) => (
            <option key={node.id} value={node.id}>
              {node.label}
              {node.existingReviewCount > 0
                ? ` (${node.existingReviewCount} existing)`
                : ""}
            </option>
          ))}
        </select>
      </label>
      {selectedNode ? (
        <p className="criterion-description">{selectedNode.description}</p>
      ) : null}

      <div className="review-form-row">
        <label>
          Finding
          <select
            onChange={(event) => setFinding(event.target.value)}
            value={finding}
          >
            <option value="no_problems">No problems</option>
            <option value="non_ethical_problem">Non-ethical problem</option>
            <option value="ethical_problem">Ethical problem</option>
            <option value="inconclusive">Inconclusive</option>
          </select>
        </label>
        <label>
          Severity
          <select
            onChange={(event) => setSeverity(event.target.value)}
            value={severity}
          >
            <option value="">Unspecified</option>
            <option value="minor">Minor</option>
            <option value="moderate">Moderate</option>
            <option value="major">Major</option>
            <option value="critical">Critical</option>
          </select>
        </label>
        <label>
          Confidence
          <select
            onChange={(event) => setConfidence(event.target.value)}
            value={confidence}
          >
            <option value="">Unspecified</option>
            <option value="low">Low</option>
            <option value="moderate">Moderate</option>
            <option value="high">High</option>
          </select>
        </label>
      </div>

      <label>
        Review content
        <textarea
          onChange={(event) => setContent(event.target.value)}
          placeholder="Evidence, reasoning, and assessment for this one criterion."
          required
          rows={6}
          value={content}
        />
      </label>
      <label>
        Limitations (optional)
        <textarea
          onChange={(event) => setLimitations(event.target.value)}
          rows={2}
          value={limitations}
        />
      </label>
      <label>
        Recommendations (optional)
        <textarea
          onChange={(event) => setRecommendations(event.target.value)}
          rows={2}
          value={recommendations}
        />
      </label>

      <p className="muted-copy">
        Submitting will start or join a public audit episode for this subject.
        Unsolicited reviews are uncompensated, contribute to scrutiny depth,
        and may be cited by later SynthesisReviews.
      </p>
      <button className="primary-action" disabled={pending} type="submit">
        {pending ? "Submitting…" : "Submit unsolicited ElementReview"}
      </button>
      {error ? <p className="form-error">{error}</p> : null}
    </form>
  );
}
