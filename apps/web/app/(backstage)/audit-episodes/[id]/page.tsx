import { AuthGate } from "../../../components/auth-gate";
import { CommercialPanel } from "../../../components/commercial-panel";

type PageProps = {
  params: Promise<{
    id: string;
  }>;
};

export default async function AuditEpisodeWorkspacePage({ params }: PageProps) {
  const { id } = await params;

  return (
    <AuthGate
      body="Episode Workspace contains solicitation management, draft reports, internal memberships, private facts, and commissioned audit operations."
      eyebrow="Audit operations"
      returnTo={`/audit-episodes/${id}`}
      role="operator"
      title="Episode Workspace Requires Sign In"
    >
      <CommercialPanel episodeId={id} />
    </AuthGate>
  );
}
