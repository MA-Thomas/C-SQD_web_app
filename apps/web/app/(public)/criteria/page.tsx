import Link from "next/link";

import {
  formatLabel,
  getDomainInstantiation,
  getDomainInstantiations,
} from "../../lib/csqd-api";

/// Taxonomy reference for the active domain's criteria (CRWE for Academic
/// Peer Review). Exploration happens in Discover; this page explains the
/// lens. Criteria are per-domain configuration, so everything here is
/// fetched from the domain registry, never hard-coded.
export default async function CriteriaPage() {
  const domains = await getDomainInstantiations();
  const academicDomain =
    domains.find((domain) => domain.domain_type === "academic_publishing") ??
    domains[0] ??
    null;
  const detail = academicDomain
    ? await getDomainInstantiation(academicDomain.id)
    : null;
  const nodes = detail?.cwe_nodes ?? [];
  const isAcademic = academicDomain?.domain_type === "academic_publishing";

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">
            {academicDomain ? formatLabel(academicDomain.domain_type) : "Criteria"}
          </p>
          <h1>{isAcademic ? "CRWE Criteria" : "Audit Criteria"}</h1>
          <p>
            {isAcademic
              ? "The Common Research Weakness Enumeration: the public taxonomy this domain uses to attach reviews to explicit criteria instead of vague overall impressions."
              : "The criteria this domain uses to attach reviews to explicit, auditable dimensions."}{" "}
            Each criterion links to Discover filtered to works with related
            audit activity.
          </p>
        </div>
        <Link className="secondary-action" href="/method#crwe">
          Criteria in the method
        </Link>
      </header>

      <div className="pub-stat-strip">
        <span>
          <strong>{nodes.length}</strong> criteria in{" "}
          {detail?.name ?? "the active domain"}
        </span>
        <span>
          Missing a criterion? Petition for one from any work's audit record.
        </span>
      </div>

      {nodes.length === 0 ? (
        <div className="pub-empty">
          <h3>No criteria configured</h3>
          <p>This domain has no configured criteria yet.</p>
        </div>
      ) : (
        <div className="pub-criteria-grid">
          {nodes.map((node) => (
            <article className="pub-criterion-card" key={node.id}>
              <h3>{node.label}</h3>
              <p>{node.description}</p>
              <Link href={`/discover?criterion=${encodeURIComponent(node.id)}`}>
                Works reviewed under this criterion
              </Link>
            </article>
          ))}
        </div>
      )}
    </>
  );
}
