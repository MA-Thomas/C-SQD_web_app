import Link from "next/link";

import { PublicWorkCard } from "../components/public-work-card";
import { getScholarlyObjects } from "../lib/csqd-api";
import {
  formatDate,
  getAcademicCweNodes,
  getPublicAuditSummariesForObjects,
  groupScholarlyObjects,
  publicAuditStatusLabel,
} from "../lib/public-audit";

export default async function PublicRegistryHomePage() {
  const [objects, cweNodes] = await Promise.all([
    getScholarlyObjects(),
    getAcademicCweNodes(),
  ]);
  const groups = groupScholarlyObjects(objects);
  const publicSummaries = await getPublicAuditSummariesForObjects(
    groups.map((group) => group.primaryVersion),
  );
  const reportGroups = groups.filter((group) => {
    const summary = publicSummaries.get(group.primaryVersion.id);

    return (summary?.synthesisReviewCount ?? group.synthesisReviewCount) > 0;
  });
  const needingReview = groups.filter((group) => {
    const summary = publicSummaries.get(group.primaryVersion.id);

    return (
      publicAuditStatusLabel(group.primaryVersion, summary) !==
      "Audit report available"
    );
  });
  const elementReviewCount = groups.reduce((sum, group) => {
    const summary = publicSummaries.get(group.primaryVersion.id);

    return sum + (summary?.elementReviewCount ?? group.elementReviewCount);
  }, 0);
  const synthesisReviewCount = groups.reduce((sum, group) => {
    const summary = publicSummaries.get(group.primaryVersion.id);

    return sum + (summary?.synthesisReviewCount ?? group.synthesisReviewCount);
  }, 0);

  return (
    <section className="workspace registry-home">
      <header className="registry-header">
        <div>
          <p className="eyebrow">Public audit registry</p>
          <h1>C-SQD</h1>
          <p>
            Discover scholarly works, inspect public audit activity, read
            SynthesisReviews, and follow focused ElementReviews by criterion.
          </p>
        </div>
        <Link className="status-pill" href="/method">
          How the method works
        </Link>
      </header>

      <form className="retrieval-form registry-search" action="/discover">
        <label htmlFor="registry-query">Search public audit subjects</label>
        <div className="retrieval-controls">
          <input
            id="registry-query"
            name="q"
            placeholder="Title, DOI, arXiv, PubMed, author, venue, or keyword"
            type="search"
          />
          <button type="submit">Search</button>
        </div>
      </form>

      <section
        className="metric-grid four-metric-grid"
        aria-label="Public registry metrics"
      >
        <div className="metric">
          <span>Scholarly works</span>
          <strong>{groups.length}</strong>
        </div>
        <div className="metric">
          <span>CRWE criteria</span>
          <strong>{cweNodes.length}</strong>
        </div>
        <div className="metric">
          <span>ElementReviews</span>
          <strong>{elementReviewCount}</strong>
        </div>
        <div className="metric">
          <span>SynthesisReviews</span>
          <strong>{synthesisReviewCount}</strong>
        </div>
      </section>

      <section className="registry-band" aria-label="Recent public reports">
        <div className="section-heading">
          <div>
            <p className="eyebrow">Public Audits</p>
            <h2>Recent Reports</h2>
          </div>
          <Link className="secondary-action" href="/public-audits">
            View all
          </Link>
        </div>
        {reportGroups.length === 0 ? (
          <div className="empty-state">
            <h2>No public reports yet</h2>
            <p>
              Public SynthesisReviews will appear here as audit reports are
              published.
            </p>
          </div>
        ) : (
          <div className="report-list">
            {reportGroups.slice(0, 3).map((group) => {
              const summary = publicSummaries.get(group.primaryVersion.id);

              return (
                <Link
                  className="report-row"
                  href={`/scholarly-objects/${group.primaryVersion.id}#latest-report`}
                  key={group.id}
                >
                  <div>
                    <strong>{group.title}</strong>
                    <span>
                      {summary?.latestReport
                        ? summary.latestReport.summary
                        : "Audit report available"}
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

      <section className="registry-band" aria-label="Public audit subjects">
        <div className="section-heading">
          <div>
            <p className="eyebrow">Discover</p>
            <h2>Works With Public Audit Activity</h2>
          </div>
          <Link className="secondary-action" href="/discover">
            Open Discover
          </Link>
        </div>
        <div className="object-list">
          {(reportGroups.length > 0 ? reportGroups : needingReview)
            .slice(0, 4)
            .map((group) => (
              <PublicWorkCard
                group={group}
                key={group.id}
                summary={publicSummaries.get(group.primaryVersion.id) ?? null}
              />
            ))}
        </div>
      </section>

      <section className="registry-band" aria-label="Evaluation tuple explainer">
        <div className="section-heading">
          <div>
            <p className="eyebrow">Method</p>
            <h2>What The Evaluation Tuple Means</h2>
          </div>
          <Link className="secondary-action" href="/method">
            Read the method
          </Link>
        </div>
        <div className="tuple-explainer">
          <p>
            Every public audit subject carries a derived five-part summary of
            its audit record: <strong>Problems</strong> found by reviewers,{" "}
            <strong>Ethical concerns</strong>, the <strong>Stakes</strong> of
            the work, the <strong>Scrutiny depth</strong> it has received, and
            its <strong>Uptake</strong>. The tuple is recomputable for any
            reviewer community and reference time — a derived view over the
            immutable audit record, not a mysterious score.
          </p>
        </div>
      </section>
    </section>
  );
}
