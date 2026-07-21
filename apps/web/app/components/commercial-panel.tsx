"use client";

import { useCallback, useEffect, useState } from "react";

import {
  getFactsForEpisode,
  recordInvoiceIssued,
  recordPaymentReceived,
  recordReviewerPayout,
  type Fact,
} from "../lib/csqd-api";
import { formatDate } from "../lib/public-audit";

type CommercialKind = "invoice_issued" | "payment_received" | "reviewer_payout";

type CommercialFact = {
  fact: Fact;
  kind: CommercialKind;
  amount: string;
  detail: string | null;
};

const KIND_LABELS: Record<CommercialKind, string> = {
  invoice_issued: "Invoice issued",
  payment_received: "Payment received",
  reviewer_payout: "Reviewer payout",
};

function asRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null
    ? (value as Record<string, unknown>)
    : null;
}

/// The payload is `{ kind: { ...fields } }` (serde external tagging).
function parseCommercialFact(fact: Fact): CommercialFact | null {
  const payload = asRecord(fact.payload);

  if (!payload) {
    return null;
  }

  for (const kind of Object.keys(KIND_LABELS) as CommercialKind[]) {
    const body = asRecord(payload[kind]);

    if (!body) {
      continue;
    }

    const amount = asRecord(body.amount);
    const amountText = amount
      ? `${Number(amount.amount).toLocaleString("en")} ${String(amount.currency ?? "")}`
      : "—";
    const detail =
      typeof body.invoice_ref === "string"
        ? `ref ${body.invoice_ref}`
        : typeof body.note === "string"
          ? body.note
          : null;

    return { fact, kind, amount: amountText, detail };
  }

  return null;
}

/// Operator ledger panel for one episode: the commercial facts on the
/// record, plus forms to add invoice / payment / payout facts. Money is
/// append-only administrative record-keeping here — it never touches the
/// evaluation tuple, and "funded" is derived from an active
/// payment_received fact.
export function CommercialPanel({ episodeId }: { episodeId: string }) {
  const [facts, setFacts] = useState<CommercialFact[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [kind, setKind] = useState<CommercialKind>("payment_received");
  const [amount, setAmount] = useState("");
  const [currency, setCurrency] = useState("USD");
  const [reference, setReference] = useState("");
  const [paidTo, setPaidTo] = useState("");
  const [note, setNote] = useState("");

  const load = useCallback(async () => {
    const all = await getFactsForEpisode(episodeId);
    setFacts(
      all
        .map(parseCommercialFact)
        .filter((entry): entry is CommercialFact => entry !== null),
    );
  }, [episodeId]);

  useEffect(() => {
    void load();
  }, [load]);

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setPending(true);
    setError(null);

    const money = {
      amount: Number(amount),
      currency: currency.trim().toUpperCase() || "USD",
    };

    try {
      if (kind === "invoice_issued") {
        await recordInvoiceIssued(episodeId, {
          amount: money,
          invoice_ref: reference.trim() || null,
          note: note.trim() || null,
        });
      } else if (kind === "payment_received") {
        await recordPaymentReceived(episodeId, {
          amount: money,
          note: note.trim() || null,
        });
      } else {
        await recordReviewerPayout(episodeId, {
          paid_to: paidTo.trim(),
          amount: money,
          note: note.trim() || null,
        });
      }

      setAmount("");
      setReference("");
      setPaidTo("");
      setNote("");
      await load();
    } catch (submitError) {
      setError(
        submitError instanceof Error
          ? submitError.message
          : "Could not record the commercial fact.",
      );
    } finally {
      setPending(false);
    }
  };

  const funded = facts?.some((entry) => entry.kind === "payment_received") ?? false;

  return (
    <section className="workspace-section">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Commercial record</p>
          <h2>Funding &amp; Payouts</h2>
        </div>
        <span className="muted-copy">
          {facts === null ? "" : funded ? "Funded" : "Funding pending"}
        </span>
      </div>

      {facts === null ? (
        <p className="muted-copy">Loading commercial facts…</p>
      ) : facts.length === 0 ? (
        <p className="muted-copy">
          No invoices, payments, or payouts on this episode&apos;s record yet.
        </p>
      ) : (
        <table className="console-table">
          <thead>
            <tr>
              <th>When</th>
              <th>Act</th>
              <th>Amount</th>
              <th>Detail</th>
            </tr>
          </thead>
          <tbody>
            {facts.map((entry) => (
              <tr key={entry.fact.id}>
                <td>{formatDate(entry.fact.occurred_at)}</td>
                <td>{KIND_LABELS[entry.kind]}</td>
                <td>{entry.amount}</td>
                <td>{entry.detail ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <form className="element-review-form" onSubmit={submit}>
        <label>
          <span>Record</span>
          <select
            onChange={(event) => setKind(event.target.value as CommercialKind)}
            value={kind}
          >
            <option value="payment_received">Payment received (marks funded)</option>
            <option value="invoice_issued">Invoice issued</option>
            <option value="reviewer_payout">Reviewer payout</option>
          </select>
        </label>
        <label>
          <span>Amount</span>
          <input
            min="0.01"
            onChange={(event) => setAmount(event.target.value)}
            required
            step="0.01"
            type="number"
            value={amount}
          />
        </label>
        <label>
          <span>Currency</span>
          <input
            onChange={(event) => setCurrency(event.target.value)}
            required
            type="text"
            value={currency}
          />
        </label>
        {kind === "invoice_issued" ? (
          <label>
            <span>Invoice reference</span>
            <input
              onChange={(event) => setReference(event.target.value)}
              placeholder="Accounting system id"
              type="text"
              value={reference}
            />
          </label>
        ) : null}
        {kind === "reviewer_payout" ? (
          <label>
            <span>Reviewer user id</span>
            <input
              onChange={(event) => setPaidTo(event.target.value)}
              required
              type="text"
              value={paidTo}
            />
          </label>
        ) : null}
        <label>
          <span>Note</span>
          <input
            onChange={(event) => setNote(event.target.value)}
            placeholder="Optional"
            type="text"
            value={note}
          />
        </label>
        {error ? <p className="form-error">{error}</p> : null}
        <button className="primary-action" disabled={pending} type="submit">
          {pending ? "Recording…" : "Record on the audit trail"}
        </button>
      </form>
    </section>
  );
}
