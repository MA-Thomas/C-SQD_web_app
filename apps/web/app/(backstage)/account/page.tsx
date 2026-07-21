import { AccountPanel } from "../../components/account-panel";
import { AuthGate } from "../../components/auth-gate";

type PageProps = {
  searchParams: Promise<{
    welcome?: string;
  }>;
};

export default async function AccountPage({ searchParams }: PageProps) {
  const { welcome } = await searchParams;

  return (
    <AuthGate
      body="Account settings cover your display name — how authorship appears on the audit record — and your role state."
      eyebrow="Account"
      returnTo="/account"
      title="Account Settings Require Sign In"
    >
      <AccountPanel welcome={welcome === "1"} />
    </AuthGate>
  );
}
