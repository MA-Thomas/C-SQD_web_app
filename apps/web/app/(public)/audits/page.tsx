import Link from "next/link";

import { WorkCard } from "../../components/work-card";
import { SectionRail, WorkListRow } from "../../components/work-list-row";
import { getScholarlyObjects } from "../../lib/csqd-api";
import {
  formatDate,
  getPublicAuditSummariesForObjects,
  groupScholarlyObjects,
  type ScholarlyWorkGroup,
} from "../../lib/public-audit";

/// Delivered public outputs: audit reports first, then subjects with review
/// depth, then contested audits. This page shows C-SQD's value before a
/// visitor has a particular paper in mind.
export default async function AuditsPage() {
  const objects = await getScholarlyObjects();
  const groups = groupScholarlyObjects(objects);
  const summaries = await getPublicAuditSummariesForObjects(
    groups.map((group) => group.primaryVersion),
  );
  const summaryOf = (group: ScholarlyWorkGroup) =>
    summaries.get(group.primaryVersion.id) ?? null;

  const reportGroups = groups
    .filter((group) => summaryOf(group)?.latestReport)
    .sort((left, right) => {
      const leftTime = new Date(
        summaryOf(left)?.latestReport?.authored_at ?? 0,
      ).getTime();
      const rightTime = new Date(
        summaryOf(right)?.latestReport?.authored_at ?? 0,
      ).getTime();

      return rightTime - leftTime;
    });
  const reviewDepthGroups = groups
    .filter((group) => (summaryOf(group)?.elementReviewCount ?? 0) > 0)
    .sort(
      (left, right) =>
        (summaryOf(right)?.elementReviewCount ?? 0) -
        (summaryOf(left)?.elementReviewCount ?? 0),
    );
  const challengedGroups = groups
    .filter((group) => (summaryOf(group)?.challengeCount ?? 0) > 0)
    .sort(
      (left, right) =>
        (summaryOf(right)?.challengeCount ?? 0) -
        (summaryOf(left)?.challengeCount ?? 0),
    );

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">Audit reports</p>
          <h1>Delivered Public Audits</h1>
          <p>
            Public SynthesisReviews, subjects with meaningful review depth, and
            contested audit claims.
          </p>
        </div>
        <Link className="secondary-action" href="/discover">
          Discover works
        </Link>
      </header>

      <div className="pub-stat-strip" aria-label="Public audit metrics">
        <span>
          <strong>{reportGroups.length}</strong> audit reports
        </span>
        <span>
          <strong>{reviewDepthGroups.length}</strong> reviewed subjects
        </span>
        <span>
          <strong>{challengedGroups.length}</strong> challenged subjects
        </span>
      </div>

      <section className="pub-section" id="reports">
        <div className="pub-section-head">
          <h2>Audit Reports</h2>
          <Link href="/method#synthesisreviews">How reports work</Link>
        </div>
        {reportGroups.length === 0 ? (
          <div className="pub-empty">
            <h3>No public audit reports yet</h3>
            <p>
              Public SynthesisReviews will appear here once they are published.
            </p>
          </div>
        ) : (
          <div className="pub-grid">
            {reportGroups.map((group) => {
              const summary = summaryOf(group);

              return (
                <article className="pub-card" key={group.id}>
                  <div className="pub-card-kicker">
                    <span>Report</span>
                    {summary?.latestReport ? (
                      <span>{formatDate(summary.latestReport.authored_at)}</span>
                    ) : null}
                  </div>
                  <h3>
                    <Link href={`/works/${group.primaryVersion.id}#latest-report`}>
                      {group.title}
                    </Link>
                  </h3>
                  <p className="pub-lead-deck">
                    {summary?.latestReport?.summary ?? "Public audit report available"}
                  </p>
                  <div className="pub-card-actions">
                    <Link href={`/works/${group.primaryVersion.id}#latest-report`}>
                      Read the report
                    </Link>
                    <Link href={`/works/${group.primaryVersion.id}`}>
                      Full coverage
                    </Link>
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="pub-section" id="review-depth">
        <div className="pub-section-head">
          <h2>Subjects With Review Depth</h2>
          <Link href="/criteria">Browse criteria</Link>
        </div>
        {reviewDepthGroups.length === 0 ? (
          <div className="pub-empty">
            <h3>No public ElementReviews yet</h3>
            <p>
              Focused reviews will appear here once reviewers submit them
              against criteria.
            </p>
          </div>
        ) : (
          <div className="pub-grid">
            {reviewDepthGroups.map((group) => (
              <WorkCard group={group} key={group.id} summary={summaryOf(group)} />
            ))}
          </div>
        )}
      </section>

      <section className="pub-section" id="challenges">
        <div className="pub-section-head">
          <h2>Contested Audit Claims</h2>
          <Link href="/method#challenges">How challenges work</Link>
        </div>
        {challengedGroups.length === 0 ? (
          <div className="pub-empty">
            <h3>No public challenges recorded</h3>
            <p>
              This surface holds provenance-bearing contestations of
              ElementReviews and SynthesisReviews.
            </p>
          </div>
        ) : (
          <div className="pub-grid">
            <SectionRail title="Challenged subjects">
              {challengedGroups.map((group) => (
                <WorkListRow
                  detail={
                    <span>
                      {summaryOf(group)?.challengeCount} challenge
                      {(summaryOf(group)?.challengeCount ?? 0) === 1 ? "" : "s"}
                    </span>
                  }
                  group={group}
                  key={group.id}
                  summary={summaryOf(group)}
                />
              ))}
            </SectionRail>
          </div>
        )}
      </section>
    </>
  );
}
