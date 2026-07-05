/// Canonical evaluation-tuple criteria (kept in sync with /method).
/// Lives outside any "use client" module so both server components (the
/// homepage explainer) and client components (TupleBadge) can import it.

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

export const TUPLE_ITEMS: TupleItemSpec[] = [
  {
    key: "problems",
    label: "Problems",
    symbol: "N",
    definition: "Non-ethical problems surfaced by ElementReviews. Zero is best.",
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
    definition: "How consequential the subject is for the domain.",
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
