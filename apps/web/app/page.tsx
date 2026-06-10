import Link from "next/link";

import { AppSidebar } from "./components/app-sidebar";
import {
  formatLabel,
  getAuditEpisodes,
  type AuditEpisodeSummary,
} from "./lib/csqd-api";

export default async function AuditConsolePage() {
  const episodes = await getAuditEpisodes();
  const activeEpisodes = episodes.filter((episode) => episode.status === "active");
  const synthesisReady = episodes.filter((episode) => episode.synthesis_ready);
  const factCount = episodes.reduce((sum, episode) => sum + episode.fact_count, 0);

  return (
    <main className="app-shell">
      <AppSidebar activeItem="console" />

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Commissioned audit operations</p>
            <h1>Audit Console</h1>
          </div>
          <Link className="status-pill" href="/commission">
            Commission audit
          </Link>
        </header>

        <section className="metric-grid" aria-label="Audit console metrics">
          <div className="metric">
            <span>Episodes</span>
            <strong>{episodes.length}</strong>
          </div>
          <div className="metric">
            <span>Active</span>
            <strong>{activeEpisodes.length}</strong>
          </div>
          <div className="metric">
            <span>Facts</span>
            <strong>{factCount}</strong>
          </div>
        </section>

        <section className="object-list" aria-label="Commissioned audit episodes">
          {episodes.length === 0 ? (
            <div className="empty-state">
              <h2>No commissioned audits yet</h2>
              <p>Register an audit subject and commission the first scoped episode.</p>
            </div>
          ) : (
            episodes.map((episode) => (
              <EpisodeRow episode={episode} key={episode.id} />
            ))
          )}
        </section>

        {synthesisReady.length > 0 ? (
          <section className="detail-panels audit-console-secondary">
            <article className="panel">
              <div className="panel-heading">
                <div>
                  <p className="eyebrow">Synthesis queue</p>
                  <h2>Ready For Interpretation</h2>
                </div>
                <span className="access-badge">{synthesisReady.length}</span>
              </div>
              <div className="version-context-list">
                {synthesisReady.map((episode) => (
                  <Link
                    className="version-context-row"
                    href={`/audit-episodes/${episode.id}`}
                    key={episode.id}
                  >
                    <div>
                      <strong>{episode.label}</strong>
                      <span>{episode.subject_title ?? "Untitled subject"}</span>
                    </div>
                    <span>{episode.element_review_count} facts</span>
                  </Link>
                ))}
              </div>
            </article>
          </section>
        ) : null}
      </section>
    </main>
  );
}

function EpisodeRow({ episode }: { episode: AuditEpisodeSummary }) {
  return (
    <article className="object-card work-card">
      <div className="object-main">
        <div className="object-kicker">
          <span>{formatLabel(episode.status)}</span>
          <span>{formatLabel(episode.subject_type)}</span>
          {episode.sponsor_name ? <span>{episode.sponsor_name}</span> : null}
        </div>
        <h2>{episode.label}</h2>
        <p>{episode.subject_title ?? "Untitled audit subject"}</p>
        <div className="object-actions">
          <Link href={`/audit-episodes/${episode.id}`}>Open workspace</Link>
          <Link href={`/commission?subject_id=${episode.subject_id}`}>
            Commission related audit
          </Link>
        </div>
      </div>
      <dl className="object-facts">
        <div>
          <dt>Facts</dt>
          <dd>{episode.fact_count}</dd>
        </div>
        <div>
          <dt>Element reviews</dt>
          <dd>{episode.element_review_count}</dd>
        </div>
        <div>
          <dt>Synthesis</dt>
          <dd>{episode.synthesis_ready ? "Ready" : "Pending"}</dd>
        </div>
        <div>
          <dt>Latest activity</dt>
          <dd>{formatDate(episode.latest_activity_at ?? episode.authored_at)}</dd>
        </div>
      </dl>
    </article>
  );
}

function formatDate(value: string) {
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
