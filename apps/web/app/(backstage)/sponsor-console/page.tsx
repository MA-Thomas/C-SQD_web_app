import { AuthGate } from "../../components/auth-gate";
import { EpisodeConsole } from "../../components/episode-console";

export default function SponsorConsolePage() {
  return (
    <AuthGate
      body="Sponsor Console contains private funding, scope, assignment progress, delivery state, and audit deliverable information for commissioned work."
      eyebrow="Sponsor operations"
      returnTo="/sponsor-console"
      role="sponsor"
      title="Sponsor Console Requires Sign In"
    >
      <EpisodeConsole variant="sponsor" />
    </AuthGate>
  );
}
