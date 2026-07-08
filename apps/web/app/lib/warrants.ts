/// Read-side helpers for warrant-assertion facts (claim-scoped audits memo).
///
/// Presentation-layer only: warrants are created, stored, validated, and
/// scored in the Rust backend. This module merely extracts the
/// `warrant_assertion` payloads already present in an episode's fact list so
/// pages can display them and review forms can offer them as targets.

import { type Fact } from "./csqd-api";
import { payloadRecord, stringValue } from "./public-audit";

export type WarrantSummary = {
  factId: string;
  /// The `episode_evidence_artifacts` link this warrant runs through.
  evidenceArtifactId: string | null;
  /// The claim the artifact itself actually makes.
  artifactClaim: string;
  /// statistical | causal | mechanistic | external_validity | other
  inferenceType: string;
  assumptions: string | null;
  rationale: string | null;
};

/// Extract active warrant assertions from a fact list (subject- or
/// episode-scoped). Superseded and retracted warrants are excluded, matching
/// the backend's `derive_artifact_bearing` which only counts active facts.
export function warrantsFromFacts(facts: Fact[]): WarrantSummary[] {
  return facts
    .filter((fact) => fact.status === "active")
    .flatMap((fact) => {
      const payload = payloadRecord(fact, "warrant_assertion");

      if (!payload) {
        return [];
      }

      return [
        {
          factId: fact.id,
          evidenceArtifactId: stringValue(payload.evidence_artifact) || null,
          artifactClaim: stringValue(payload.artifact_claim),
          inferenceType: stringValue(payload.inference_type) || "other",
          assumptions: stringValue(payload.assumptions) || null,
          rationale: stringValue(payload.rationale) || null,
        },
      ];
    });
}

/// Group warrants by the evidence-artifact link they run through. Warrants
/// asserted without an artifact link land under the empty-string key.
export function warrantsByArtifact(
  warrants: WarrantSummary[],
): Map<string, WarrantSummary[]> {
  const byArtifact = new Map<string, WarrantSummary[]>();

  for (const warrant of warrants) {
    const key = warrant.evidenceArtifactId ?? "";
    const existing = byArtifact.get(key);

    if (existing) {
      existing.push(warrant);
    } else {
      byArtifact.set(key, [warrant]);
    }
  }

  return byArtifact;
}
