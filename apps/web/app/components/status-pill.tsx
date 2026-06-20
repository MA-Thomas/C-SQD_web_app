const STATUS_CLASS: Record<string, string> = {
  Unaudited: "status-neutral",
  "Registered for audit": "status-info",
  "ElementReviews submitted": "status-info",
  "In synthesis": "status-progress",
  "Audit report available": "status-positive",
  Challenged: "status-warning",
  Superseded: "status-superseded",
};

export function StatusPill({ status }: { status: string }) {
  const variant = STATUS_CLASS[status] ?? "status-neutral";

  return <span className={`status-pill-badge ${variant}`}>{status}</span>;
}
