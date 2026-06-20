import { AuthGate } from "../../components/auth-gate";
import { EpisodeConsole } from "../../components/episode-console";

export default function ReviewerQueuePage() {
  return (
    <AuthGate
      body="Reviewer Queue contains assigned commissioned ElementReviews, criterion-specific task briefs, due dates, compensation state, and submission status."
      eyebrow="Reviewer operations"
      returnTo="/reviewer-queue"
      role="reviewer"
      title="Reviewer Queue Requires Sign In"
    >
      <EpisodeConsole variant="reviewer" />
    </AuthGate>
  );
}
