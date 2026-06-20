import Link from "next/link";

export default function NotFound() {
  return (
    <section className="workspace empty-state route-not-found">
      <h1>Page not found</h1>
      <p>
        This record may have moved, or the address may be mistyped. The audit
        record itself is immutable — nothing here is ever deleted.
      </p>
      <p>
        <Link href="/">Back to the registry</Link> ·{" "}
        <Link href="/discover">Discover audit subjects</Link>
      </p>
    </section>
  );
}
