import Link from "next/link";
import { notFound } from "next/navigation";

import { ElementReviewForm } from "../../../../components/element-review-form";
import { getScholarlyObject } from "../../../../lib/csqd-api";
import {
  criterionNodeId,
  factKind,
  getAcademicCweNodes,
  getPublicAuditSummaryForObject,
  payloadRecord,
} from "../../../../lib/public-audit";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
  searchParams: Promise<{
    criterion?: string;
    synthesis?: string;
  }>;
};

export default async function SubmitReviewPage({ params, searchParams }: PageProps) {
  const { id } = await params;
  const { criterion, synthesis } = await searchParams;
  const object = await getScholarlyObject(id);

  if (!object) {
    notFound();
  }

  const [summary, nodes] = await Promise.all([
    getPublicAuditSummaryForObject(object),
    getAcademicCweNodes(),
  ]);
  const elementReviews = summary.facts.filter(
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

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">
            {synthesisMode ? "Submit SynthesisReview" : "Submit ElementReview"}
          </p>
          <h1>{object.title}</h1>
          <p>
            {synthesisMode
              ? "An integrative interpretation of the public audit episode."
              : "A focused review of one criterion — smaller and more composable than traditional peer review."}
          </p>
        </div>
        <Link className="secondary-action" href={`/works/${object.id}`}>
          Back to full coverage
        </Link>
      </header>

      <ElementReviewForm
        auditSubjectId={object.audit_subject_id}
        episodes={summary.episodes.map((episode) => ({
          id: episode.id,
          label: episode.label,
          status: episode.status,
        }))}
        nodes={nodeOptions}
        preselectedCriterion={criterion ?? null}
        subjectPath={`/works/${object.id}`}
        subjectTitle={object.title}
        synthesisMode={synthesisMode}
      />
    </>
  );
}
