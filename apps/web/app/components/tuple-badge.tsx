"use client";

import { useAdvancedMode } from "../lib/advanced-mode";
import { TUPLE_ITEMS, type TupleValues } from "../lib/tuple-items";

/// Re-exported for existing importers of this module's types.
export type { TupleItemSpec, TupleValues } from "../lib/tuple-items";

const DOT_COUNT = 5;

function formatValue(value: number) {
  return value.toLocaleString("en", {
    maximumFractionDigits: 2,
    minimumFractionDigits: 0,
  });
}

/// Small at-a-glance magnitude cue: five dots, filled up to the value
/// (clamped). The exact number sits right above it, so the dots only need
/// to be directionally honest, not precise.
function MagnitudeDots({ value }: { value: number }) {
  const filled = Math.max(0, Math.min(DOT_COUNT, Math.round(value)));

  return (
    <span aria-hidden className="tuple-dots">
      {Array.from({ length: DOT_COUNT }, (_, index) => (
        <i className={index < filled ? "filled" : undefined} key={index} />
      ))}
    </span>
  );
}

/// The platform's visual signature: one compact, consistent rendering of the
/// evaluation tuple. Friendly labels by default; symbolic notation in
/// advanced mode. Concern criteria (problems, ethical concerns) shift to the
/// danger hue when nonzero so the badge reads at a glance; every cell
/// carries its criterion definition as a tooltip.
export function TupleBadge({
  tuple,
  size = "regular",
}: {
  tuple: TupleValues;
  size?: "compact" | "regular";
}) {
  const { advanced } = useAdvancedMode();

  if (!tuple) {
    return (
      <div
        aria-label="Claim audit tuple: not yet evaluated"
        className={`tuple-badge tuple-none tuple-${size}`}
        role="group"
      >
        <span className="tuple-none-chip">Not yet evaluated</span>
      </div>
    );
  }

  return (
    <div
      className={`tuple-badge tuple-${size}`}
      role="group"
      aria-label="Claim audit tuple"
      title={advanced ? "E(A | R, T_eval) -> (N, M, S, L, U)" : undefined}
    >
      {TUPLE_ITEMS.map((item) => {
        const value = tuple[item.key];
        const concerning = item.valence === "concern" && value > 0;

        return (
          <span
            className={`tuple-item${concerning ? " tuple-concern" : ""}`}
            key={item.key}
            title={item.definition}
          >
            {/* The friendly name stays available to assistive tech even when
                the visual rendering is symbolic (advanced mode). */}
            <span className="tuple-label" aria-hidden={advanced || undefined}>
              {advanced ? item.symbol : item.label}
            </span>
            {advanced ? <span className="sr-only">{item.label}</span> : null}
            <strong className="tuple-value">{formatValue(value)}</strong>
            {size === "regular" ? <MagnitudeDots value={value} /> : null}
          </span>
        );
      })}
      {advanced ? (
        <span className="tuple-notation">E(A | R, T_eval)</span>
      ) : null}
    </div>
  );
}
