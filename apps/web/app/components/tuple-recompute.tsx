"use client";

import { useState } from "react";

import { getEvalTupleWithParams, type AuditEpisode, type EvalTuple } from "../lib/csqd-api";
import { TupleBadge } from "./tuple-badge";

/// "Recompute as…" panel: the clearest demonstration that the evaluation
/// tuple is a derived view over immutable inputs, not a stored score. Filters
/// by reviewer community tags and reference time, recomputing live.
export function TupleRecomputePanel({ episodes }: { episodes: AuditEpisode[] }) {
  const [tEval, setTEval] = useState("");
  const [tags, setTags] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<EvalTuple[] | null>(null);

  if (episodes.length === 0) {
    return null;
  }

  const recompute = async (event: React.FormEvent) => {
    event.preventDefault();
    setPending(true);
    setError(null);

    try {
      const tuples = await Promise.all(
        episodes.map((episode) =>
          getEvalTupleWithParams(episode.id, {
            tEval: tEval ? new Date(tEval).toISOString() : undefined,
            tags: tags
              .split(",")
              .map((tag) => tag.trim())
              .filter(Boolean),
          }),
        ),
      );
      setResult(tuples.filter((tuple): tuple is EvalTuple => tuple !== null));
    } catch (recomputeError) {
      setError(
        recomputeError instanceof Error
          ? recomputeError.message
          : "Recomputation failed.",
      );
    } finally {
      setPending(false);
    }
  };

  const aggregated = result
    ? result.reduce(
        (summary, tuple) => ({
          problems: summary.problems + tuple.n,
          ethicalConcerns: summary.ethicalConcerns + tuple.m,
          stakes: Math.max(summary.stakes, tuple.s),
          scrutinyDepth: summary.scrutinyDepth + tuple.l,
          uptake: summary.uptake + tuple.u,
        }),
        { problems: 0, ethicalConcerns: 0, stakes: 0, scrutinyDepth: 0, uptake: 0 },
      )
    : null;

  return (
    <details className="recompute-panel">
      <summary>Recompute as…</summary>
      <p className="muted-copy">
        The evaluation tuple is a derived view, not a stored score: it is a
        pure function over the immutable audit record, recomputable for any
        reviewer community and reference time.
      </p>
      <form className="recompute-form" onSubmit={recompute}>
        <label>
          As of date
          <input
            onChange={(event) => setTEval(event.target.value)}
            type="date"
            value={tEval}
          />
        </label>
        <label>
          Reviewer community tags (comma-separated)
          <input
            onChange={(event) => setTags(event.target.value)}
            placeholder="e.g. statistics, genomics"
            type="text"
            value={tags}
          />
        </label>
        <button disabled={pending} type="submit">
          {pending ? "Recomputing…" : "Recompute tuple"}
        </button>
      </form>
      {error ? <p className="form-error">{error}</p> : null}
      {aggregated ? (
        <div className="recompute-result">
          <TupleBadge tuple={aggregated} size="compact" />
        </div>
      ) : null}
    </details>
  );
}
