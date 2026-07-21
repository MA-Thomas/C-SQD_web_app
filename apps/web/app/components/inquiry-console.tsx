"use client";

import { useEffect, useState } from "react";

import {
  formatLabel,
  getCommissionInquiries,
  updateCommissionInquiry,
  type CommissionInquiry,
} from "../lib/csqd-api";
import { formatDate } from "../lib/public-audit";

const STATUS_ACTIONS: Array<{
  status: CommissionInquiry["status"];
  label: string;
}> = [
  { status: "in_conversation", label: "Mark in conversation" },
  { status: "declined", label: "Decline" },
];

/// Operator triage for stage-one commission inquiries. Conversion into a
/// real commission happens through the full commission form; the operator
/// then links the episode id here so the trail stays connected.
export function InquiryConsole() {
  const [inquiries, setInquiries] = useState<CommissionInquiry[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pendingId, setPendingId] = useState<string | null>(null);
  const [episodeIds, setEpisodeIds] = useState<Record<string, string>>({});

  const load = async () => {
    setInquiries(await getCommissionInquiries());
  };

  useEffect(() => {
    void load();
  }, []);

  const transition = async (
    inquiry: CommissionInquiry,
    status: CommissionInquiry["status"],
  ) => {
    setPendingId(inquiry.id);
    setError(null);

    try {
      await updateCommissionInquiry(inquiry.id, {
        status,
        converted_episode_id:
          status === "converted" ? (episodeIds[inquiry.id] ?? "").trim() : null,
      });
      await load();
    } catch (updateError) {
      setError(
        updateError instanceof Error
          ? updateError.message
          : "Could not update the inquiry.",
      );
    } finally {
      setPendingId(null);
    }
  };

  return (
    <section className="workspace-section">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Stage-one intake</p>
          <h2>Commission Inquiries</h2>
        </div>
      </div>
      {error ? <p className="form-error">{error}</p> : null}
      {inquiries === null ? (
        <p className="muted-copy">Loading inquiries…</p>
      ) : inquiries.length === 0 ? (
        <div className="empty-state">
          <h2>No inquiries yet</h2>
          <p>Public commission inquiries land here for triage and scoping.</p>
        </div>
      ) : (
        <table className="console-table">
          <thead>
            <tr>
              <th>Received</th>
              <th>Contact</th>
              <th>Wants audited</th>
              <th>Budget</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {inquiries.map((inquiry) => (
              <tr key={inquiry.id}>
                <td>{formatDate(inquiry.created_at)}</td>
                <td>
                  <strong>{inquiry.contact_name}</strong>
                  <br />
                  <a href={`mailto:${inquiry.contact_email}`}>
                    {inquiry.contact_email}
                  </a>
                  {inquiry.organization_name ? (
                    <>
                      <br />
                      {inquiry.organization_name} (
                      {formatLabel(inquiry.organization_type)})
                    </>
                  ) : null}
                </td>
                <td>
                  {inquiry.subject_description}
                  {inquiry.decision_context ? (
                    <p className="muted-copy">
                      Context: {inquiry.decision_context}
                    </p>
                  ) : null}
                </td>
                <td>{formatLabel(inquiry.budget_band)}</td>
                <td>{formatLabel(inquiry.status)}</td>
                <td>
                  {inquiry.status === "converted" ? (
                    inquiry.converted_episode_id ? (
                      <a href={`/audit-episodes/${inquiry.converted_episode_id}`}>
                        Episode
                      </a>
                    ) : (
                      "—"
                    )
                  ) : (
                    <div className="inquiry-actions">
                      {STATUS_ACTIONS.filter(
                        (action) => action.status !== inquiry.status,
                      ).map((action) => (
                        <button
                          disabled={pendingId === inquiry.id}
                          key={action.status}
                          onClick={() => void transition(inquiry, action.status)}
                          type="button"
                        >
                          {action.label}
                        </button>
                      ))}
                      <input
                        onChange={(event) =>
                          setEpisodeIds((previous) => ({
                            ...previous,
                            [inquiry.id]: event.target.value,
                          }))
                        }
                        placeholder="Episode id"
                        type="text"
                        value={episodeIds[inquiry.id] ?? ""}
                      />
                      <button
                        disabled={
                          pendingId === inquiry.id ||
                          !(episodeIds[inquiry.id] ?? "").trim()
                        }
                        onClick={() => void transition(inquiry, "converted")}
                        type="button"
                      >
                        Mark converted
                      </button>
                    </div>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
