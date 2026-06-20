"use client";

import { useAdvancedMode } from "../lib/advanced-mode";

export type TupleValues = {
  problems: number;
  ethicalConcerns: number;
  stakes: number;
  scrutinyDepth: number;
  uptake: number;
} | null;

const ITEMS: Array<{
  key: keyof NonNullable<TupleValues>;
  label: string;
  symbol: string;
}> = [
  { key: "problems", label: "Problems", symbol: "N" },
  { key: "ethicalConcerns", label: "Ethical concerns", symbol: "M" },
  { key: "stakes", label: "Stakes", symbol: "S" },
  { key: "scrutinyDepth", label: "Scrutiny depth", symbol: "L" },
  { key: "uptake", label: "Uptake", symbol: "U" },
];

function formatValue(value: number) {
  return value.toLocaleString("en", {
    maximumFractionDigits: 2,
    minimumFractionDigits: 0,
  });
}

/// The platform's visual signature: one compact, consistent rendering of the
/// evaluation tuple. Friendly labels by default; symbolic notation in
/// advanced mode.
export function TupleBadge({
  tuple,
  size = "regular",
}: {
  tuple: TupleValues;
  size?: "compact" | "regular";
}) {
  const { advanced } = useAdvancedMode();

  return (
    <div
      className={`tuple-badge tuple-${size}`}
      role="group"
      aria-label={
        tuple ? "Evaluation tuple" : "Evaluation tuple: not yet evaluated"
      }
      title={advanced ? "E(A | R, T_eval) -> (N, M, S, L, U)" : undefined}
    >
      {ITEMS.map((item) => (
        <span className="tuple-item" key={item.key}>
          {/* The friendly name stays available to assistive tech even when
              the visual rendering is symbolic (advanced mode). */}
          <span className="tuple-label" aria-hidden={advanced || undefined}>
            {advanced ? item.symbol : item.label}
          </span>
          {advanced ? <span className="sr-only">{item.label}</span> : null}
          <strong className="tuple-value">
            {tuple ? (
              formatValue(tuple[item.key])
            ) : (
              <>
                <span aria-hidden>–</span>
                <span className="sr-only">not yet evaluated</span>
              </>
            )}
          </strong>
        </span>
      ))}
      {advanced ? (
        <span className="tuple-notation">E(A | R, T_eval)</span>
      ) : null}
    </div>
  );
}
