import { AccountAdmin } from "../../components/account-admin";
import { AuthGate } from "../../components/auth-gate";
import { EpisodeConsole } from "../../components/episode-console";
import { InquiryConsole } from "../../components/inquiry-console";

export default function OperationsPage() {
  return (
    <AuthGate
      body="Audit Operations contains solicitation management, draft reports, internal memberships, and commissioned audit workflows."
      eyebrow="Operations"
      returnTo="/operations"
      role="operator"
      title="Audit Operations Requires Sign In"
    >
      <EpisodeConsole variant="operations" />
      <InquiryConsole />
      <AccountAdmin />
    </AuthGate>
  );
}
