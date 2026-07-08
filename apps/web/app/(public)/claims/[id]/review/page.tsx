import Link from "next/link";
import { notFound } from "next/navigation";

import { ElementReviewForm } from "../../../../components/element-review-form";
import {
  getAuditEpisodesForSubject,
  getAuditSubject,
  getDomainInstantiation,
  getEvidenceArtifacts,
  getFactsForSubject,
  type EvidenceArtifactSummary,
} from "../../../../lib/csqd-api";
import {
  criterionNodeId,
  factKind,
  getAcademicCweNodes,
  payloadRecord,
} from "../../../../lib/public-audit";
import { warrantsFromFacts, type WarrantSummary } from "../../../../lib/warrants";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
  searchParams: Promise<{
    criterion?: string;
    synthesis?: string;
    warrant?: string;
  }>;
};

/// Review submission for a claim-scoped audit subject. Same unsolicited
/// contribution flow as /works/[id]/review, but the subject is the bounded
/// target claim, and the review can target an attached evidence artifact or a
/// specific warrant link — the unit the audit question turns on.
export default async function SubmitClaimReviewPage({
  params,
  searchParams,
}: PageProps) {
  const { id } = await params;
  const { criterion, synthesis, warrant } = await searchParams;
  const subject = await getAuditSubject(id);

  if (!subject) {
    notFound();
  }

  const [episodes, facts, domain, fallbackNodes] = await Promise.all([
    getAuditEpisodesForSubject(subject.id),
    getFactsForSubject(subject.id),
    getDomainInstantiation(subject.domain_instantiation_id).catch(() => null),
    getAcademicCweNodes(),
  ]);
  const nodes = domain?.cwe_nodes?.length ? domain.cwe_nodes : fallbackNodes;
  const elementReviews = facts.filter(
    (fact) => factKind(fact) === "element_review",
  );
  const nodeOptions = nodes.map((node) => ({
    id: node.id,
    label: node.label,
    description: node.description,
    existingReviewCount: elementReviews.filter(
      (fact) => criterionNodeId(payloadRecord(fact, "element_review")) === node.id,
    ).length,
  }));
  const synthesisMode = synthesis === "1";
  const openEpisode =
    episodes.find((episode) => episode.status === "active") ?? null;
  const evidenceArtifacts = openEpisode
    ? await getEvidenceArtifacts(openEpisode.id)
    : [];
  const warrants = warrantsFromFacts(facts);
  const subjectTitle =
    subject.claim_statement ?? subject.title ?? "Untitled audit subject";

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">
            {synthesisMode ? "Submit SynthesisReview" : "Submit ElementReview"}
          </p>
          <h1>{subjectTitle}</h1>
          <p>
            {synthesisMode
              ? "An integrative interpretation of the audit episode for this claim."
              : "A focused review of one criterion — optionally targeting one attached artifact or one warrant link."}
          </p>
        </div>
        <Link className="secondary-action" href={`/claims/${subject.id}`}>
          Back to the claim audit
        </Link>
      </header>

      <ElementReviewForm
        auditSubjectId={subject.id}
        episodes={episodes.map((episode) => ({
          id: episode.id,
          label: episode.label,
          status: episode.status,
        }))}
        evidenceArtifacts={evidenceArtifacts.map((artifact) => ({
          id: artifact.artifact.id,
          title: artifact.title,
        }))}
        nodes={nodeOptions}
        preselectedCriterion={criterion ?? null}
        preselectedWarrant={warrant ?? null}
        subjectPath={`/claims/${subject.id}`}
        subjectTitle={subjectTitle}
        synthesisMode={synthesisMode}
        warrants={warrants.map((item) => ({
          id: item.factId,
          label: warrantOptionLabel(item, evidenceArtifacts),
          evidenceArtifactId: item.evidenceArtifactId,
        }))}
      />
    </>
  );
}

/// "<artifact title>: <artifact claim>" truncated to stay legible in a
/// native select.
function warrantOptionLabel(
  warrant: WarrantSummary,
  artifacts: EvidenceArtifactSummary[],
) {
  const artifactTitle = artifacts.find(
    (artifact) => artifact.artifact.id === warrant.evidenceArtifactId,
  )?.title;
  const label = artifactTitle
    ? `${artifactTitle} — ${warrant.artifactClaim}`
    : warrant.artifactClaim;

  return label.length > 110 ? `${label.slice(0, 107)}…` : label;
}
