import { type ArtifactBearing } from "../lib/csqd-api";

/// Color-coded pill for an artifact's derived bearing, reusing the status-pill
/// palette so "problems found" vs "survives scrutiny" reads at a glance.
/// Labels and colors only — the bearing itself is derived in Rust
/// (`derive_artifact_bearing`) and never adjusted here.
const BEARING_DISPLAY: Record<ArtifactBearing, { label: string; variant: string }> = {
  not_yet_inspected: { label: "Not yet inspected", variant: "status-neutral" },
  warrants_unaudited: { label: "Warrants unaudited", variant: "status-info" },
  problems_found: { label: "Problems found", variant: "status-warning" },
  inconclusive: { label: "Inconclusive", variant: "status-progress" },
  survives_scrutiny: { label: "Survives scrutiny", variant: "status-positive" },
};

export function BearingPill({ bearing }: { bearing: ArtifactBearing }) {
  const display = BEARING_DISPLAY[bearing] ?? BEARING_DISPLAY.not_yet_inspected;

  return (
    <span
      className={`status-pill-badge ${display.variant}`}
      title="Derived from warrant assertions and the element reviews that scrutinize them. Attachment never confers support."
    >
      {display.label}
    </span>
  );
}

export function bearingLabel(bearing: ArtifactBearing) {
  return (BEARING_DISPLAY[bearing] ?? BEARING_DISPLAY.not_yet_inspected).label;
}
