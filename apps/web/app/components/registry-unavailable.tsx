/// Rendered when the C-SQD API cannot be reached. Deliberately distinct
/// from empty states: "no works registered" is a true statement about the
/// registry, while this is a statement about infrastructure — conflating
/// the two misleads readers (and once misled the operator).
export function RegistryUnavailable() {
  return (
    <div className="pub-empty pub-unavailable" role="alert">
      <h3>The audit registry is unreachable</h3>
      <p>
        This is not an empty registry — the site could not reach the C-SQD
        API, so no audit records can be shown. If you run this instance,
        check that the API and database are up (locally:{" "}
        <code>scripts/setup_db.sh</code>, then <code>npm run dev:api</code>).
        Otherwise, try again shortly.
      </p>
    </div>
  );
}
