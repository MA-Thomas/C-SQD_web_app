/// Streaming fallback for every public route: quiet, registry-toned, no
/// spinner theatrics. Shown while server components fetch from the API.
export default function PublicLoading() {
  return (
    <section className="workspace route-loading" aria-busy="true">
      <p className="eyebrow">C-SQD</p>
      <h1>Loading the registry…</h1>
      <div className="route-loading-bars" aria-hidden>
        <span />
        <span />
        <span />
      </div>
    </section>
  );
}
