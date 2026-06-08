import Link from "next/link";

import { AppSidebar } from "../components/app-sidebar";
import { formatLabel, getReviewAssignments } from "../lib/csqd-api";

function formatDueAt(value: string | null) {
  if (!value) {
    return "Unscheduled";
  }

  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat("en", {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(date);
}

export default async function AssignmentsPage() {
  const assignments = await getReviewAssignments();
  const activeAssignments = assignments.filter((assignment) =>
    ["accepted", "in_progress"].includes(assignment.state),
  );
  const compensationEligible = assignments.filter(
    (assignment) => assignment.compensation_status === "eligible",
  );

  return (
    <main className="app-shell">
      <AppSidebar activeItem="assignments" />

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Academic Peer Review domain</p>
            <h1>Assignments</h1>
          </div>
          <div className="status-pill">Local demo</div>
        </header>

        <section className="metric-grid" aria-label="Assignment metrics">
          <div className="metric">
            <span>Assignments</span>
            <strong>{assignments.length}</strong>
          </div>
          <div className="metric">
            <span>Active</span>
            <strong>{activeAssignments.length}</strong>
          </div>
          <div className="metric">
            <span>Comp eligible</span>
            <strong>{compensationEligible.length}</strong>
          </div>
        </section>

        <section className="assignment-list" aria-label="Review assignments">
          {assignments.length === 0 ? (
            <div className="empty-state">
              <h2>API connection pending</h2>
              <p>Start the Rust API to load demo review assignments.</p>
            </div>
          ) : (
            assignments.map((assignment) => (
              <article className="object-card assignment-card" key={assignment.id}>
                <div className="object-main">
                  <div className="object-kicker">
                    <span>{formatLabel(assignment.assignment_type)}</span>
                    <span>{formatLabel(assignment.state)}</span>
                  </div>
                  <h2>
                    <Link href={`/scholarly-objects/${assignment.scholarly_object_id}`}>
                      {assignment.scholarly_object_title}
                    </Link>
                  </h2>
                  <p>Reviewer: {assignment.reviewer_display_name}</p>
                  <div className="object-actions">
                    <Link href={`/scholarly-objects/${assignment.scholarly_object_id}`}>
                      Open object
                    </Link>
                    <a
                      href={assignment.scholarly_object_canonical_url}
                      rel="noreferrer"
                      target="_blank"
                    >
                      Open source
                    </a>
                  </div>
                </div>
                <dl className="object-facts assignment-facts">
                  <div>
                    <dt>Due</dt>
                    <dd>{formatDueAt(assignment.due_at)}</dd>
                  </div>
                  <div>
                    <dt>Compensation</dt>
                    <dd>{formatLabel(assignment.compensation_status)}</dd>
                  </div>
                  <div>
                    <dt>Assignment ID</dt>
                    <dd>{assignment.id}</dd>
                  </div>
                </dl>
              </article>
            ))
          )}
        </section>
      </section>
    </main>
  );
}
