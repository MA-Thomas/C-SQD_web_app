"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";

import {
  attachEvidenceArtifact,
  retractEvidenceArtifact,
  searchScholarlyObjects,
  submitWarrantAssertion,
  type EvidenceArtifactSummary,
  type InferenceType,
  type ScholarlyObjectSummary,
} from "../lib/csqd-api";
import { useSession } from "../lib/session";

const INFERENCE_TYPES: Array<{ value: InferenceType; label: string }> = [
  { value: "statistical", label: "Statistical" },
  { value: "causal", label: "Causal" },
  { value: "mechanistic", label: "Mechanistic" },
  { value: "external_validity", label: "External validity" },
  { value: "other", label: "Other" },
];

/// Signed-in evidence management for a claim audit episode: attach papers as
/// evidence artifacts to be inspected, retract attachments, and assert
/// warrant links (why an artifact is supposed to bear on the target claim).
/// Attachment is neutral; bearing is derived from audited warrants.
export function EvidenceManager({
  artifacts,
  episodeId,
}: {
  artifacts: EvidenceArtifactSummary[];
  episodeId: string;
}) {
  const { user, loading } = useSession();
  const router = useRouter();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ScholarlyObjectSummary[]>([]);
  const [searching, setSearching] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [warrantTarget, setWarrantTarget] = useState<string | null>(null);
  const [artifactClaim, setArtifactClaim] = useState("");
  const [inferenceType, setInferenceType] = useState<InferenceType>("statistical");
  const [assumptions, setAssumptions] = useState("");
  const [rationale, setRationale] = useState("");

  if (loading || !user) {
    return null;
  }

  const attachedObjectIds = new Set(
    artifacts.map((artifact) => artifact.scholarly_object_id),
  );

  const search = async (event: React.FormEvent) => {
    event.preventDefault();
    setSearching(true);
    setError(null);

    try {
      const found = await searchScholarlyObjects(query);
      setResults(found.filter((object) => !attachedObjectIds.has(object.id)));
    } catch (searchError) {
      setError(
        searchError instanceof Error ? searchError.message : "Search failed.",
      );
    } finally {
      setSearching(false);
    }
  };

  const attach = async (scholarlyObjectId: string) => {
    setPending(true);
    setError(null);

    try {
      await attachEvidenceArtifact(episodeId, {
        scholarly_object_id: scholarlyObjectId,
      });
      setResults((current) =>
        current.filter((object) => object.id !== scholarlyObjectId),
      );
      router.refresh();
    } catch (attachError) {
      setError(
        attachError instanceof Error ? attachError.message : "Attach failed.",
      );
    } finally {
      setPending(false);
    }
  };

  const retract = async (artifactId: string, title: string) => {
    // Retraction stays on the record (status: retracted), but it removes the
    // artifact from active scrutiny — worth one deliberate click.
    if (
      !window.confirm(
        `Retract "${title}" from this episode? Its warrants stop counting toward the audit.`,
      )
    ) {
      return;
    }

    setPending(true);
    setError(null);

    try {
      await retractEvidenceArtifact(episodeId, artifactId);
      router.refresh();
    } catch (retractError) {
      setError(
        retractError instanceof Error ? retractError.message : "Retract failed.",
      );
    } finally {
      setPending(false);
    }
  };

  const assertWarrant = async (event: React.FormEvent) => {
    event.preventDefault();

    if (!warrantTarget) {
      return;
    }

    setPending(true);
    setError(null);

    try {
      await submitWarrantAssertion(episodeId, {
        evidence_artifact: warrantTarget,
        artifact_claim: artifactClaim,
        inference_type: inferenceType,
        assumptions: assumptions || null,
        rationale: rationale || null,
      });
      setWarrantTarget(null);
      setArtifactClaim("");
      setAssumptions("");
      setRationale("");
      router.refresh();
    } catch (warrantError) {
      setError(
        warrantError instanceof Error
          ? warrantError.message
          : "Warrant assertion failed.",
      );
    } finally {
      setPending(false);
    }
  };

  return (
    <div className="review-form">
      <form onSubmit={search}>
        <label>
          Attach evidence artifacts
          <input
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search registered works by title, author, or source"
            type="search"
            value={query}
          />
        </label>
        <button className="secondary-action" disabled={searching} type="submit">
          {searching ? "Searching…" : "Search works"}
        </button>
      </form>

      {results.length > 0 ? (
        <div className="pub-facts">
          {results.slice(0, 8).map((object) => (
            <div className="pub-location-row" key={object.id}>
              <div>
                <strong>{object.title}</strong>
                <span>
                  {object.authors.slice(0, 3).join(", ")}
                  {object.publication_year ? ` · ${object.publication_year}` : ""}
                </span>
              </div>
              <button
                className="secondary-action"
                disabled={pending}
                onClick={() => void attach(object.id)}
                type="button"
              >
                Attach
              </button>
            </div>
          ))}
        </div>
      ) : null}

      {artifacts.length > 0 ? (
        <div className="pub-facts">
          {artifacts.map((artifact) => (
            <div className="pub-location-row" key={artifact.artifact.id}>
              <div>
                <strong>{artifact.title}</strong>
                <span>
                  {artifact.warrant_count} warrants · {artifact.review_count}{" "}
                  reviews
                </span>
              </div>
              <div className="pub-auth-actions">
                <button
                  className="secondary-action"
                  disabled={pending}
                  onClick={() =>
                    setWarrantTarget(
                      warrantTarget === artifact.artifact.id
                        ? null
                        : artifact.artifact.id,
                    )
                  }
                  type="button"
                >
                  Assert warrant
                </button>
                <button
                  className="secondary-action"
                  disabled={pending}
                  onClick={() => void retract(artifact.artifact.id, artifact.title)}
                  type="button"
                >
                  Retract
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : null}

      {warrantTarget ? (
        <form onSubmit={assertWarrant}>
          <label>
            Artifact claim
            <textarea
              onChange={(event) => setArtifactClaim(event.target.value)}
              placeholder="What claim does this artifact actually make?"
              required
              rows={3}
              value={artifactClaim}
            />
          </label>
          <div className="review-form-row">
            <label>
              Inference type
              <select
                onChange={(event) =>
                  setInferenceType(event.target.value as InferenceType)
                }
                value={inferenceType}
              >
                {INFERENCE_TYPES.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Required assumptions (optional)
              <input
                onChange={(event) => setAssumptions(event.target.value)}
                placeholder="Assumptions needed for this claim to bear on the target"
                type="text"
                value={assumptions}
              />
            </label>
          </div>
          <label>
            Rationale (optional)
            <textarea
              onChange={(event) => setRationale(event.target.value)}
              placeholder="Why this inference is supposed to carry the artifact claim to the target claim."
              rows={2}
              value={rationale}
            />
          </label>
          <p className="muted-copy">
            A warrant records why this artifact is supposed to bear on the
            target claim. It carries no weight until element reviews
            scrutinize it.
          </p>
          <button className="primary-action" disabled={pending} type="submit">
            {pending ? "Submitting…" : "Assert warrant link"}
          </button>
        </form>
      ) : null}

      {error ? <p className="form-error">{error}</p> : null}
    </div>
  );
}
