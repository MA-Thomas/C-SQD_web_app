# Source Inventories

Reusable source inventories live here. They are sampling-frame caches, not acquired corpus
papers.

Use one versioned directory per source inventory:

`<source_slug>/<inventory_id>/`

`<inventory_id>` should encode the parser id, parser version, coverage range, and build date.

Each inventory should contain:

- `inventory_manifest.json` — source identifiers, source URLs, parser id/version, build
  date, coverage, user agent, rate limit, request summary, hashes, and known gaps.
- `archive_pages.csv` — official archive pages such as journal volume pages, conference-year
  pages, proceedings indexes, or collaboration publication-list pages.
- `units.csv` — archival units such as journal issues, conference-years, tracks,
  proceedings volumes, article batches, or declared publication-list years.
- `items.csv` — paper rows with unit id, article/paper URL, title, section/type, date,
  summary or abstract snippet, authors when exposed, OA marker when exposed, DOI/arXiv id
  when exposed, and source URL.
- `field_assignments.csv` — optional derived field labels and neutral evidence. Keep field
  assignment separate from raw source inventory, and never include paradigm-orientation
  judgments.

Draw audit logs must record the exact inventory id, manifest path, manifest hash, table
hashes, parser version, and any field-assignment table/version used.
