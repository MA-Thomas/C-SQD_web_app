import Link from "next/link";

import { PublicWorkCard } from "../../components/public-work-card";
import { getScholarlyObjects } from "../../lib/csqd-api";
import {
  formatDate,
  getPublicAuditSummariesForObjects,
  groupScholarlyObjects,
} from "../../lib/public-audit";

export default async function PublicAuditsPage() {
  const objects = await getScholarlyObjects();
  const groups = groupScholarlyObjects(objects);
  const summaries = await getPublicAuditSummariesForObjects(
    groups.map((group) => group.primaryVersion),
  );
  const reportGroups = groups.filter(
    (group) =>
      (summaries.get(group.primaryVersion.id)?.synthesisReviewCount ??
        group.synthesisReviewCount) > 0,
  );
  const elementReviewGroups = [...groups]
    .filter(
      (group) =>
        (summaries.get(group.primaryVersion.id)?.elementReviewCount ??
          group.elementReviewCount) > 0,
    )
    .sort(
      (left, right) =>
        (summaries.get(right.primaryVersion.id)?.elementReviewCount ??
          right.elementReviewCount) -
        (summaries.get(left.primaryVersion.id)?.elementReviewCount ??
          left.elementReviewCount),
    );
  const challengeCount = Array.from(summaries.values()).reduce(
    (sum, summary) => sum + summary.challengeCount,
    0,
  );

  return (
          <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Public audit outputs</p>
            <h1>Public Audits</h1>
          </div>
          <Link className="status-pill" href="/discover">
            Discover works
          </Link>
        </header>

        <section className="metric-grid" aria-label="Public audit metrics">
          <div className="metric">
            <span>Audit reports</span>
            <strong>{reportGroups.length}</strong>
          </div>
          <div className="metric">
            <span>Reviewed subjects</span>
            <strong>{elementReviewGroups.length}</strong>
          </div>
          <div className="metric">
            <span>Open challenges</span>
            <strong>{challengeCount}</strong>
          </div>
        </section>

        <section
          className="workspace-section first-workspace-section"
          id="synthesis-reviews"
        >
          <div className="section-heading">
            <div>
              <p className="eyebrow">SynthesisReviews</p>
              <h2>Audit Reports</h2>
            </div>
            <Link className="secondary-action" href="/method#synthesisreviews">
              Method
            </Link>
          </div>
          {reportGroups.length === 0 ? (
            <div className="empty-state">
              <h2>No public audit reports yet</h2>
              <p>
                Current or draft SynthesisReviews will appear here once they are
                published.
              </p>
            </div>
          ) : (
            <div className="report-list">
              {reportGroups.map((group) => {
                const summary = summaries.get(group.primaryVersion.id);

                return (
                  <Link
                    className="report-row"
                    href={`/scholarly-objects/${group.primaryVersion.id}#latest-report`}
                    key={group.id}
                  >
                    <div>
                      <strong>{group.title}</strong>
                      <span>
                        {summary?.latestReport?.summary ??
                          "Public audit report available"}
                      </span>
                    </div>
                    <span>
                      {summary?.latestReport
                        ? formatDate(summary.latestReport.authored_at)
                        : "Current"}
                    </span>
                  </Link>
                );
              })}
            </div>
          )}
        </section>

        <section className="workspace-section" id="element-reviews">
          <div className="section-heading">
            <div>
              <p className="eyebrow">ElementReviews</p>
              <h2>Subjects With Review Depth</h2>
            </div>
            <Link className="secondary-action" href="/browse">
              Browse CRWE
            </Link>
          </div>
          <div className="object-list">
            {elementReviewGroups.length === 0 ? (
              <div className="empty-state">
                <h2>No public ElementReviews yet</h2>
                <p>
                  Focused reviews will appear here once reviewers submit them
                  against CRWE criteria.
                </p>
              </div>
            ) : (
              elementReviewGroups.map((group) => (
                <PublicWorkCard
                  group={group}
                  key={group.id}
                  summary={summaries.get(group.primaryVersion.id) ?? null}
                />
              ))
            )}
          </div>
        </section>

        <section className="workspace-section" id="challenges">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Challenges</p>
              <h2>Contested Audit Claims</h2>
            </div>
            <Link className="secondary-action" href="/method#challenges">
              Challenge method
            </Link>
          </div>
          <div className="empty-state">
            <h2>No public challenges recorded</h2>
            <p>
              The UI reserves this public surface for provenance-bearing
              contestations of ElementReviews and SynthesisReviews.
            </p>
          </div>
        </section>
      </section>
  );
}
