import Link from "next/link";

import { LeadStoryCard } from "../components/lead-story-card";
import { TupleBadge } from "../components/tuple-badge";
import { TUPLE_ITEMS } from "../lib/tuple-items";
import { WorkCard } from "../components/work-card";
import { SectionRail, WorkListRow } from "../components/work-list-row";
import { getScholarlyObjects } from "../lib/csqd-api";
import {
  formatCount,
  getPublicAuditSummariesForObjects,
  groupScholarlyObjects,
  publicAuditStatusLabel,
  type PublicAuditSummary,
  type ScholarlyWorkGroup,
} from "../lib/public-audit";

/// Briefing homepage: the lead audit report as the top story, then rails
/// answering "what changed?" — recently challenged, gaining scrutiny,
/// awaiting review — and a card grid across the registry.
export default async function HomePage() {
  const objects = await getScholarlyObjects();
  const groups = groupScholarlyObjects(objects);
  const summaries = await getPublicAuditSummariesForObjects(
    groups.map((group) => group.primaryVersion),
  );
  const summaryOf = (group: ScholarlyWorkGroup) =>
    summaries.get(group.primaryVersion.id) ?? null;

  const reportGroups = groups
    .filter((group) => summaryOf(group)?.latestReport)
    .sort(
      (left, right) => reportTime(summaryOf(right)) - reportTime(summaryOf(left)),
    );
  const lead = reportGroups[0] ?? null;
  const moreReports = reportGroups.slice(1, 6);

  const challenged = groups
    .filter((group) => (summaryOf(group)?.challengeCount ?? 0) > 0)
    .sort(
      (left, right) =>
        (summaryOf(right)?.challengeCount ?? 0) -
        (summaryOf(left)?.challengeCount ?? 0),
    )
    .slice(0, 5);

  const gainingScrutiny = groups
    .filter(
      (group) =>
        group.id !== lead?.id &&
        (summaryOf(group)?.tuple?.scrutinyDepth ?? 0) > 0,
    )
    .sort(
      (left, right) =>
        (summaryOf(right)?.tuple?.scrutinyDepth ?? 0) -
        (summaryOf(left)?.tuple?.scrutinyDepth ?? 0),
    )
    .slice(0, 5);

  const awaitingReview = groups
    .filter((group) => {
      const label = publicAuditStatusLabel(group.primaryVersion, summaryOf(group));

      return label === "Unaudited" || label === "Registered for audit";
    })
    .slice(0, 5);

  const gridGroups = groups
    .filter((group) => group.id !== lead?.id)
    .sort(
      (left, right) =>
        (summaryOf(right)?.elementReviewCount ?? 0) -
        (summaryOf(left)?.elementReviewCount ?? 0),
    )
    .slice(0, 6);

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">Public audit registry</p>
          <h1>Today in audits</h1>
          <p>
            Commissioned and public audits of scientific and technical claims —
            reports, criterion-level reviews, and challenges, all on the record.
          </p>
        </div>
        <Link className="secondary-action" href="/method">
          How the method works
        </Link>
      </header>

      {lead ? (
        <div className="pub-lead-cluster">
          <LeadStoryCard group={lead} summary={summaryOf(lead)} />
          <SectionRail
            footerHref="/audits"
            footerLabel="All audit reports"
            title="More reports"
          >
            {moreReports.length === 0 ? (
              <li>
                <p className="pub-row-meta">No other public reports yet.</p>
              </li>
            ) : (
              moreReports.map((group) => (
                <WorkListRow group={group} key={group.id} summary={summaryOf(group)} />
              ))
            )}
          </SectionRail>
        </div>
      ) : (
        <div className="pub-empty">
          <h3>No public audit reports yet</h3>
          <p>
            Public SynthesisReviews will appear here as audit reports are
            delivered. Explore registered works in Discover meanwhile.
          </p>
        </div>
      )}

      <section className="pub-section">
        <div className="pub-grid">
          <SectionRail
            footerHref="/audits#challenges"
            footerLabel="All challenged audits"
            title="Recently challenged"
          >
            {challenged.length === 0 ? (
              <li>
                <p className="pub-row-meta">No open public challenges.</p>
              </li>
            ) : (
              challenged.map((group) => (
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
              ))
            )}
          </SectionRail>

          <SectionRail
            footerHref="/discover?sort=scrutiny"
            footerLabel="Sort Discover by scrutiny"
            title="Gaining scrutiny"
          >
            {gainingScrutiny.length === 0 ? (
              <li>
                <p className="pub-row-meta">No scrutiny activity yet.</p>
              </li>
            ) : (
              gainingScrutiny.map((group) => (
                <WorkListRow
                  detail={
                    <span>
                      depth {formatCount(summaryOf(group)?.tuple?.scrutinyDepth ?? 0)}
                    </span>
                  }
                  group={group}
                  key={group.id}
                  summary={summaryOf(group)}
                />
              ))
            )}
          </SectionRail>

          <SectionRail
            footerHref="/discover?status=unaudited"
            footerLabel="Find works to review"
            title="Awaiting review"
          >
            {awaitingReview.length === 0 ? (
              <li>
                <p className="pub-row-meta">
                  Every registered work has review activity.
                </p>
              </li>
            ) : (
              awaitingReview.map((group) => (
                <WorkListRow group={group} key={group.id} summary={summaryOf(group)} />
              ))
            )}
          </SectionRail>
        </div>
      </section>

      <section className="pub-section">
        <div className="pub-section-head">
          <h2>Across the registry</h2>
          <Link href="/discover">Open Discover</Link>
        </div>
        {gridGroups.length === 0 ? (
          <div className="pub-empty">
            <h3>No registered works yet</h3>
            <p>Register a scholarly work to open its public audit record.</p>
          </div>
        ) : (
          <div className="pub-grid">
            {gridGroups.map((group) => (
              <WorkCard group={group} key={group.id} summary={summaryOf(group)} />
            ))}
          </div>
        )}
      </section>

      <section className="pub-section">
        <div className="pub-section-head">
          <h2>How to read the claim audit tuple</h2>
          <Link href="/method#evaluation-tuple">Read the method</Link>
        </div>
        <div className="pub-tuple-explainer">
          <figure>
            <TupleBadge
              tuple={{
                problems: 2,
                ethicalConcerns: 0,
                stakes: 3,
                scrutinyDepth: 4,
                uptake: 12,
              }}
            />
            <figcaption>
              Example. Concern criteria turn red when reviewers have upheld
              problems; the dots give magnitude at a glance.
            </figcaption>
          </figure>
          <dl className="pub-def-list">
            {TUPLE_ITEMS.map((item) => (
              <div key={item.key}>
                <dt>{item.label}</dt>
                <dd>{item.definition}</dd>
              </div>
            ))}
          </dl>
          <p className="pub-filter-note">
            The tuple is recomputable for any reviewer community and reference
            time — a derived view over the immutable audit record, not a
            mysterious score.
          </p>
        </div>
      </section>
    </>
  );
}

function reportTime(summary: PublicAuditSummary | null) {
  const value = summary?.latestReport?.authored_at;

  if (!value) {
    return 0;
  }

  const date = new Date(value);

  return Number.isNaN(date.getTime()) ? 0 : date.getTime();
}
