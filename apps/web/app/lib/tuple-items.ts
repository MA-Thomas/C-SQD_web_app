/// Canonical evaluation-tuple criteria (kept in sync with /method).
/// Lives outside any "use client" module so both server components (the
/// homepage explainer) and client components (TupleBadge) can import it.

import { type EvalTuple } from "./csqd-api";

export type TupleValues = {
  problems: number;
  ethicalConcerns: number;
  stakes: number;
  scrutinyDepth: number;
  uptake: number;
} | null;

export type TupleItemSpec = {
  key: keyof NonNullable<TupleValues>;
  label: string;
  symbol: string;
  definition: string;
  /// "concern" items are bad when nonzero (rendered in the danger hue);
  /// "record" items describe the audit record and stay in the audit teal.
  valence: "concern" | "record";
};

/// Display mapping from the backend's raw (N, M, S, L, U) tuple to the
/// friendly-keyed shape TupleBadge renders. Pure renaming — the values are
/// computed in Rust (`compute_eval_tuple`) and never adjusted here.
export function evalTupleValues(tuple: EvalTuple | null): TupleValues {
  if (!tuple) {
    return null;
  }

  return {
    problems: tuple.n,
    ethicalConcerns: tuple.m,
    stakes: tuple.s,
    scrutinyDepth: tuple.l,
    uptake: tuple.u,
  };
}

export const TUPLE_ITEMS: TupleItemSpec[] = [
  {
    key: "problems",
    label: "Problems",
    symbol: "N",
    definition:
      "Audited non-ethical problems in the claim's warrants, surfaced by ElementReviews. Zero is best.",
    valence: "concern",
  },
  {
    key: "ethicalConcerns",
    label: "Ethical concerns",
    symbol: "M",
    definition: "Ethical problems surfaced by ElementReviews. Zero is best.",
    valence: "concern",
  },
  {
    key: "stakes",
    label: "Stakes",
    symbol: "S",
    definition: "How consequential the target claim is for the domain.",
    valence: "record",
  },
  {
    key: "scrutinyDepth",
    label: "Scrutiny depth",
    symbol: "L",
    definition: "The amount and weight of focused review activity.",
    valence: "record",
  },
  {
    key: "uptake",
    label: "Uptake",
    symbol: "U",
    definition: "How much the audit record has been synthesized or used.",
    valence: "record",
  },
];
