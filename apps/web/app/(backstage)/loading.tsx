/// Streaming fallback for backstage routes.
export default function BackstageLoading() {
  return (
    <section className="workspace route-loading" aria-busy="true">
      <p className="eyebrow">Backstage</p>
      <h1>Loading…</h1>
      <div className="route-loading-bars" aria-hidden>
        <span />
        <span />
        <span />
      </div>
    </section>
  );
}
