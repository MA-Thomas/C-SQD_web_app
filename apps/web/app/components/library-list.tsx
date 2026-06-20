"use client";

import Link from "next/link";
import { useEffect, useState } from "react";

import { getLibraryItems, type LibraryItemSummary } from "../lib/csqd-api";
import { formatDate } from "../lib/public-audit";

export function LibraryList() {
  const [items, setItems] = useState<LibraryItemSummary[] | null>(null);

  useEffect(() => {
    let cancelled = false;

    void getLibraryItems().then((result) => {
      if (!cancelled) {
        setItems(result);
      }
    });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="workspace-section first-workspace-section">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Saved subjects and watched audit activity</p>
          <h2>Library / Watchlist</h2>
        </div>
      </div>
      {items === null ? (
        <p className="muted-copy">Loading library…</p>
      ) : items.length === 0 ? (
        <div className="empty-state">
          <h2>Your library is empty</h2>
          <p>
            Save works from Discover or any public audit subject page to watch
            their audit activity.
          </p>
          <div className="source-actions">
            <Link className="secondary-action" href="/discover">
              Open Discover
            </Link>
          </div>
        </div>
      ) : (
        <div className="object-list">
          {items.map((item) => (
            <Link
              className="report-row"
              href={`/scholarly-objects/${item.scholarly_object.id}`}
              key={item.id}
            >
              <div>
                <strong>{item.scholarly_object.title}</strong>
                <span>Saved {formatDate(item.added_at)}</span>
              </div>
            </Link>
          ))}
        </div>
      )}
    </section>
  );
}
