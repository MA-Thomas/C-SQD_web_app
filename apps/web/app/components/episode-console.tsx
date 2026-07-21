"use client";

import Link from "next/link";
import { useEffect, useState } from "react";

import {
  getAuditEpisodes,
  isApiReachable,
  type AuditEpisodeSummary,
} from "../lib/csqd-api";
import { formatDate } from "../lib/public-audit";

type ConsoleVariant = "sponsor" | "reviewer" | "operations";

const VARIANT_COPY: Record<ConsoleVariant, { title: string; question: string }> = {
  sponsor: {
    title: "Commissioned Audits",
    question: "What did we fund, and what is the delivery state?",
  },
  reviewer: {
    title: "Review Workload",
    question: "What is assigned or open for synthesis?",
  },
  operations: {
    title: "Audit Operations",
    question: "Which episodes need solicitations, drafts, or delivery?",
  },
};

function deliveryState(episode: AuditEpisodeSummary) {
  if (episode.status === "delivered") {
    return "Delivered";
  }

  if (episode.status === "closed") {
    return "Closed";
  }

  if (episode.synthesis_review_count > 0) {
    return "Synthesis drafted";
  }

  if (episode.synthesis_ready) {
    return "Ready for synthesis";
  }

  if (episode.element_review_count > 0) {
    return "Reviews in progress";
  }

  return "Awaiting reviews";
}

/// Shared backstage console: dense episode table with delivery state. The
/// sponsor variant emphasizes funding/delivery; operations links into the
/// episode workspace.
export function EpisodeConsole({ variant }: { variant: ConsoleVariant }) {
  const [episodes, setEpisodes] = useState<AuditEpisodeSummary[] | null>(null);
  const [apiDown, setApiDown] = useState(false);
  const copy = VARIANT_COPY[variant];

  useEffect(() => {
    let cancelled = false;

    void (async () => {
      const result = await getAuditEpisodes();

      if (cancelled) {
        return;
      }

      // Empty could mean "no episodes" or "API unreachable" — check.
      if (result.length === 0 && !(await isApiReachable())) {
        setApiDown(true);
      }

      if (!cancelled) {
        setEpisodes(result);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="workspace-section first-workspace-section">
      <div className="section-heading">
        <div>
          <p className="eyebrow">{copy.question}</p>
          <h2>{copy.title}</h2>
        </div>
      </div>
      {episodes === null ? (
        <p className="muted-copy">Loading episodes…</p>
      ) : apiDown ? (
        <div className="empty-state" role="alert">
          <h2>The audit registry is unreachable</h2>
          <p>
            The C-SQD API could not be reached — this console cannot show
            episode state. Check that the API and database are running.
          </p>
        </div>
      ) : episodes.length === 0 ? (
        <div className="empty-state">
          <h2>No audit episodes yet</h2>
          <p>Commissioned and public episodes will appear here.</p>
        </div>
      ) : (
        <table className="console-table">
          <thead>
            <tr>
              <th>Episode</th>
              <th>Subject</th>
              <th>Sponsor</th>
              <th>Funding</th>
              <th>ElementReviews</th>
              <th>Delivery state</th>
              <th>Last activity</th>
              {variant === "operations" ? <th /> : null}
            </tr>
          </thead>
          <tbody>
            {episodes.map((episode) => (
              <tr key={episode.id}>
                <td>{episode.label}</td>
                <td>{episode.subject_title ?? "Untitled subject"}</td>
                <td>{episode.sponsor_name ?? "Public"}</td>
                <td>
                  {episode.sponsor_name
                    ? episode.funding_confirmed
                      ? "Funded"
                      : "Funding pending"
                    : "—"}
                </td>
                <td>{episode.element_review_count}</td>
                <td>{deliveryState(episode)}</td>
                <td>
                  {episode.latest_activity_at
                    ? formatDate(episode.latest_activity_at)
                    : "—"}
                </td>
                {variant === "operations" ? (
                  <td>
                    <Link href={`/audit-episodes/${episode.id}`}>Workspace</Link>
                  </td>
                ) : null}
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
