import { AuthGate } from "../../components/auth-gate";
import { LibraryList } from "../../components/library-list";

export default function LibraryPage() {
  return (
    <AuthGate
      body="Library and Watchlist are account surfaces for saved subjects, watched audit activity, and private follow-up workflows."
      eyebrow="Account"
      returnTo="/library"
      role="member"
      title="Library Requires Sign In"
    >
      <LibraryList />
    </AuthGate>
  );
}
