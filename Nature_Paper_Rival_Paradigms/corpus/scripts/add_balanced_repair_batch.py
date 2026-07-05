#!/usr/bin/env python3
"""Add a repair-oriented balanced random-base batch.

This helper is deliberately stricter than the earlier expansion helpers. It
freezes overfilled source blocks, opens or completes underfilled blocks for
underrepresented fields, and records direct-route, publisher-page, and
documented OA-alternate acquisition attempts.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import html
import io
import json
import random
import re
import signal
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter, defaultdict
from datetime import date
from pathlib import Path
from typing import Any

from pypdf import PdfReader


ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT.parent
PAPERS_CSV = ROOT / "metadata" / "papers.csv"
SCHEDULE_CSV = ROOT / "sources" / "source_block_schedule_v0.csv"
ACQUISITION_LOG_CSV = ROOT / "sources" / "acquisition_log.csv"
INVENTORIES = ROOT / "source_inventories"
AUDIT_ROOT = ROOT / "sources" / "draw_audits"

ROUND_ID = "round_20260621_balanced_repair_100"
RUN_DATE = date.today().isoformat()
REQUESTED_NEW_PAPERS = 100
TARGET_PER_TUPLE = 10
SEED_BASE = 202606210100
USER_AGENT = (
    "TextDataMining-CorpusExpansion/0.1 "
    "(research corpus; local run; <=1 request/sec)"
)
BROWSER_USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
)

FIELD_SLUG = {
    "Cognitive science / psychology": "cognitive_science_psychology",
    "Computational biology / genomics-ML": "computational_biology",
    "Computational neuroscience": "computational_neuroscience",
    "ML / AI methods proper": "ml_ai_methods",
    "Mechanistic molecular / cell / developmental biology": "molecular_cell_biology",
    "Statistically-oriented physics (precision cosmology, exclusion-limit searches, ML-for-physics)": "statistical_physics",
    "Structure-driven condensed matter / chemistry": "condensed_matter_physics",
    "Structure-driven experimental physics (HEP, neutrino, gravitational waves)": "particle_gravitational_physics",
}

CSV_FIELDS = [
    "source",
    "unit_id",
    "unit_type",
    "year",
    "unit_date",
    "unit_label",
    "unit_url",
    "landing_page_url",
    "source_url",
    "title",
    "summary",
    "authors",
    "oa_marker",
    "pdf_url",
    "doi",
    "openalex_id",
    "source_item_id",
    "paper_type",
    "venue_section",
    "concepts",
    "referenced_works_count",
    "cited_by_count",
]


OPENALEX_SOURCES = {
    "Behavior Research Methods": {
        "slug": "behavior_research_methods",
        "source_id": "https://openalex.org/S137478622",
        "concept_id": "https://openalex.org/C180747234",
        "concept_label": "Cognitive psychology",
        "require_oa": True,
    },
    "Computational Brain & Behavior": {
        "slug": "computational_brain_behavior",
        "source_id": "https://openalex.org/S4210221739",
        "concept_id": "https://openalex.org/C180747234",
        "concept_label": "Cognitive psychology",
        "require_oa": True,
    },
    "Journal of Machine Learning Research": {
        "slug": "jmlr",
        "source_id": "https://openalex.org/S118988714",
        "concept_id": "https://openalex.org/C119857082",
        "concept_label": "Machine learning",
        "require_oa": False,
        "source_known_oa": True,
    },
    "International Conference on Learning Representations": {
        "slug": "iclr",
        "source_id": "https://openalex.org/S4306419637",
        "concept_id": "https://openalex.org/C119857082",
        "concept_label": "Machine learning",
        "require_oa": False,
        "source_known_oa": True,
    },
    "PLOS Biology": {
        "slug": "plos_biology",
        "source_id": "https://openalex.org/S154343897",
        "concept_id": "https://openalex.org/C95444343",
        "concept_label": "Cell biology",
        "require_oa": True,
    },
    "Astronomy and Astrophysics": {
        "slug": "astronomy_astrophysics",
        "source_id": "https://openalex.org/S205231332",
        "concept_id": "https://openalex.org/C26405456",
        "concept_label": "Cosmology",
        "require_oa": True,
    },
    "Chemical Science": {
        "slug": "chemical_science",
        "source_id": "https://openalex.org/S184645833",
        "concept_id": "https://openalex.org/C192562407",
        "concept_label": "Materials science",
        "require_oa": True,
    },
    "npj Computational Materials": {
        "slug": "npj_computational_materials",
        "source_id": "https://openalex.org/S4210232664",
        "concept_id": "https://openalex.org/C26873012",
        "concept_label": "Condensed matter physics",
        "require_oa": True,
    },
}


PLAN = [
    {
        "field": "Cognitive science / psychology",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "specialist",
        "source_block_order": "2",
        "source": "Behavior Research Methods",
        "source_inventory_id": "behavior_research_methods_openalex_cognitive_psychology_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "round_reason": "Underfilled field; new scheduled specialist source block with neutral Cognitive psychology concept filter.",
    },
    {
        "field": "Cognitive science / psychology",
        "era_band": "2015-present_completed_years_2018-2025",
        "year_min": 2018,
        "year_max": 2025,
        "venue_tier": "specialist",
        "source_block_order": "3",
        "source": "Computational Brain & Behavior",
        "source_inventory_id": "computational_brain_behavior_openalex_cognitive_psychology_v1.0.0_2018-2025_20260621",
        "needed": 10,
        "round_reason": "Underfilled field; new scheduled specialist source block with neutral Cognitive psychology concept filter.",
    },
    {
        "field": "ML / AI methods proper",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "top",
        "source_block_order": "2",
        "source": "Journal of Machine Learning Research",
        "source_inventory_id": "jmlr_openalex_machine_learning_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "round_reason": "Underrepresented field; new open archival journal source block with neutral Machine learning concept filter.",
    },
    {
        "field": "ML / AI methods proper",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "top",
        "source_block_order": "3",
        "source": "International Conference on Learning Representations",
        "source_inventory_id": "iclr_openalex_machine_learning_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "round_reason": "Underrepresented field; new open proceedings source block with neutral Machine learning concept filter.",
    },
    {
        "field": "Mechanistic molecular / cell / developmental biology",
        "era_band": "2005-2014_completed_years_2005-2014",
        "year_min": 2005,
        "year_max": 2014,
        "venue_tier": "mid",
        "source_block_order": "2",
        "source": "PLOS Biology",
        "source_inventory_id": "plos_biology_openalex_cell_biology_v1.0.0_2005-2014_20260621",
        "needed": 10,
        "round_reason": "Underrepresented field and era; new born-OA mid/top biology source block with neutral Cell biology concept filter.",
    },
    {
        "field": "Mechanistic molecular / cell / developmental biology",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "mid",
        "source_block_order": "2",
        "source": "PLOS Biology",
        "source_inventory_id": "plos_biology_openalex_cell_biology_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "round_reason": "Underrepresented field; new born-OA mid/top biology source block with neutral Cell biology concept filter.",
    },
    {
        "field": "Statistically-oriented physics (precision cosmology, exclusion-limit searches, ML-for-physics)",
        "era_band": "2005-2014_completed_years_2005-2014",
        "year_min": 2005,
        "year_max": 2014,
        "venue_tier": "specialist",
        "source_block_order": "1",
        "source": "Journal of Cosmology and Astroparticle Physics",
        "source_inventory_id": "jcap_openalex_v1.0.1_through2025_20260620",
        "needed": 10,
        "round_reason": "Already scheduled underfilled historical source block; use publisher-page attempt then arXiv OA alternates when IOP blocks direct PDF.",
    },
    {
        "field": "Statistically-oriented physics (precision cosmology, exclusion-limit searches, ML-for-physics)",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "top",
        "source_block_order": "2",
        "source": "Astronomy and Astrophysics",
        "source_inventory_id": "astronomy_astrophysics_openalex_cosmology_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "round_reason": "Underrepresented field; new top astronomy/cosmology source block with neutral Cosmology concept filter.",
    },
    {
        "field": "Structure-driven condensed matter / chemistry",
        "era_band": "2005-2014_completed_years_2011-2014",
        "year_min": 2011,
        "year_max": 2014,
        "venue_tier": "top",
        "source_block_order": "1",
        "source": "Physical Review X",
        "source_inventory_id": "physical_review_x_openalex_v1.0.1_through2025_20260620",
        "needed": 10,
        "round_reason": "Already scheduled underfilled historical source block; use publisher-page attempt then arXiv OA alternates when APS blocks direct PDF.",
    },
    {
        "field": "Structure-driven condensed matter / chemistry",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "top",
        "source_block_order": "1",
        "source": "Physical Review X",
        "source_inventory_id": "physical_review_x_openalex_v1.0.1_through2025_20260620",
        "needed": 5,
        "round_reason": "Complete the already scheduled underfilled current PRX source block to the 10-paper working target.",
    },
    {
        "field": "Structure-driven condensed matter / chemistry",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "specialist",
        "source_block_order": "2",
        "source": "npj Computational Materials",
        "source_inventory_id": "npj_computational_materials_openalex_condensed_matter_v1.0.0_2015-2025_20260621",
        "needed": 5,
        "round_reason": "Underrepresented field; partially open a gold computational-materials source block to keep the exact 100-paper batch size after Chemical Science low-text failure.",
    },
]

PRIOR_ABORTED_ATTEMPTS = [
    {
        "source": "Physical Review X",
        "reason": "A prior real pass selected a PRX article whose APS direct and publisher-page routes returned 403 and whose OpenAlex locations exposed no arXiv PDF alternate; PRX eligibility was narrowed to rows with documented arXiv PDF alternates.",
    },
    {
        "source": "Chemical Science",
        "reason": "A prior real pass selected a Chemical Science row that returned a PDF but extracted only 2,157 characters; the partial new Chemical Science block was replaced with npj Computational Materials rather than accepting low-text content.",
    },
]


BLOCKED_RECOVERY_CHECKS = [
    {
        "field": "Cognitive science / psychology",
        "source": "Collabra Psychology",
        "era_band": "2015-present_completed_years_2015-2025",
        "source_inventory_id": "collabra_psychology_openalex_v1.0.1_through2025_20260620",
        "reason": "Existing underfilled block remains checked first; prior old Collabra PDF routes now redirect or 403 in this environment.",
    }
]

OPENALEX_LOCATION_CACHE: dict[str, list[str]] = {}


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def write_csv(path: Path, rows: list[dict[str, Any]], fieldnames: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def slugify(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", value.lower()).strip("_")


def max_paper_number(rows: list[dict[str, str]]) -> int:
    nums = []
    for row in rows:
        match = re.fullmatch(r"P(\d{4})", row["paper_id"])
        if match:
            nums.append(int(match.group(1)))
    return max(nums) if nums else 0


def known_keys(papers: list[dict[str, str]]) -> set[tuple[str, str]]:
    keys: set[tuple[str, str]] = set()
    for row in papers:
        for key, col in [
            ("doi", "doi"),
            ("title", "paper_title"),
            ("url", "url"),
            ("pdf", "pdf_url"),
        ]:
            value = row.get(col, "").strip().lower()
            if value:
                keys.add((key, value))
    return keys


def row_keys(row: dict[str, str]) -> set[tuple[str, str]]:
    keys: set[tuple[str, str]] = set()
    for key in ["doi", "title", "landing_page_url", "pdf_url", "openalex_id", "source_item_id"]:
        value = row.get(key, "").strip().lower()
        if value:
            keys.add((key if key != "landing_page_url" else "url", value))
    return keys


def find_inventory(inventory_id: str) -> Path:
    matches = list(INVENTORIES.glob(f"*/{inventory_id}"))
    if len(matches) != 1:
        raise SystemExit(f"Expected one inventory for {inventory_id}, found {len(matches)}")
    return matches[0]


def request_json(url: str) -> dict[str, Any]:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)


def normalize_doi(value: str | None) -> str:
    if not value:
        return ""
    value = value.strip()
    return value.removeprefix("https://doi.org/").removeprefix("http://doi.org/")


def author_string(work: dict[str, Any]) -> str:
    names = []
    for authorship in work.get("authorships") or []:
        author = authorship.get("author") or {}
        name = author.get("display_name")
        if name:
            names.append(name)
    if len(names) > 5:
        return "; ".join(names[:5]) + "; et al."
    return "; ".join(names)


def source_location(work: dict[str, Any]) -> dict[str, Any]:
    return work.get("primary_location") or work.get("best_oa_location") or {}


def all_locations(work: dict[str, Any]) -> list[dict[str, Any]]:
    locations: list[dict[str, Any]] = []
    for loc in [work.get("primary_location"), work.get("best_oa_location")] + (work.get("locations") or []):
        if isinstance(loc, dict):
            locations.append(loc)
    return locations


def looks_pdf_url(url: str) -> bool:
    lowered = url.lower()
    return any(
        token in lowered
        for token in [
            ".pdf",
            "/pdf",
            "pdfdirect",
            "articlepdf",
            "content/pdf",
            "type=printable",
            "arxiv.org/pdf",
            "openreview.net/pdf",
        ]
    )


def candidate_urls_from_work(work: dict[str, Any], source: str) -> list[str]:
    urls: list[str] = []
    for loc in all_locations(work):
        url = loc.get("pdf_url")
        if url:
            urls.append(str(url))
        landing = loc.get("landing_page_url")
        if landing and looks_pdf_url(str(landing)):
            urls.append(str(landing))

    doi = normalize_doi(work.get("doi")).lower()
    if source == "PLOS Biology" and doi.startswith("10.1371/journal.pbio"):
        urls.insert(0, f"https://journals.plos.org/plosbiology/article/file?id={doi}&type=printable")
    if source == "Chemical Science" and doi.startswith("10.1039/"):
        suffix = doi.rsplit("/", 1)[-1]
        year = str(work.get("publication_year") or "")
        if year:
            urls.insert(0, f"https://pubs.rsc.org/en/content/articlepdf/{year}/sc/{suffix}")

    out: list[str] = []
    for url in urls:
        if not url:
            continue
        if url.startswith("http://"):
            out.append("https://" + url[len("http://") :])
        out.append(url)
    return unique(out)


def unit_from_work(source_slug: str, work: dict[str, Any]) -> tuple[str, str, str]:
    year = str(work.get("publication_year") or "unknown")
    biblio = work.get("biblio") or {}
    volume = str(biblio.get("volume") or "").strip()
    issue = str(biblio.get("issue") or "").strip()
    if volume and issue:
        unit = f"{source_slug}_{year}_v{slugify(volume)}_i{slugify(issue)}"
        label = f"vol. {volume}, issue {issue}"
    elif volume:
        unit = f"{source_slug}_{year}_v{slugify(volume)}"
        label = f"vol. {volume}"
    else:
        unit = f"{source_slug}_{year}"
        label = f"year {year}"
    return unit, label, year


def is_research_like(title: str, paper_type: str) -> bool:
    lowered = title.strip().lower()
    if paper_type and paper_type not in {"article", "preprint"}:
        return False
    bad_prefixes = (
        "correction:",
        "erratum:",
        "retraction:",
        "editorial:",
        "comment on",
        "reply to",
        "author correction:",
        "publisher correction:",
    )
    return not lowered.startswith(bad_prefixes)


def build_openalex_inventory(tuple_plan: dict[str, Any]) -> Path:
    source = str(tuple_plan["source"])
    spec = OPENALEX_SOURCES[source]
    inventory_id = str(tuple_plan["source_inventory_id"])
    inv_dir = INVENTORIES / str(spec["slug"]) / inventory_id
    if (
        (inv_dir / "inventory_manifest.json").exists()
        and (inv_dir / "items.csv").exists()
        and (inv_dir / "units.csv").exists()
    ):
        return inv_dir
    inv_dir.mkdir(parents=True, exist_ok=True)

    filters = [
        f"primary_location.source.id:{spec['source_id']}",
        f"from_publication_date:{tuple_plan['year_min']}-01-01",
        f"to_publication_date:{tuple_plan['year_max']}-12-31",
        "type:article",
        f"concepts.id:{spec['concept_id']}",
    ]
    if spec.get("require_oa"):
        filters.append("open_access.is_oa:true")

    select = ",".join(
        [
            "id",
            "doi",
            "display_name",
            "publication_year",
            "publication_date",
            "type",
            "authorships",
            "primary_location",
            "best_oa_location",
            "locations",
            "open_access",
            "concepts",
            "referenced_works_count",
            "cited_by_count",
            "biblio",
        ]
    )
    params = {
        "filter": ",".join(filters),
        "per-page": "200",
        "cursor": "*",
        "select": select,
        "sort": "publication_date:asc",
    }

    rows: list[dict[str, Any]] = []
    archive_pages: list[dict[str, Any]] = []
    cursor = "*"
    page = 0
    while cursor:
        params["cursor"] = cursor
        url = "https://api.openalex.org/works?" + urllib.parse.urlencode(params)
        data = request_json(url)
        page += 1
        archive_pages.append(
            {
                "page_index": page,
                "source_url": url,
                "result_count": len(data.get("results") or []),
            }
        )
        for work in data.get("results") or []:
            title = html.unescape(work.get("display_name") or "").strip()
            if not is_research_like(title, str(work.get("type") or "")):
                continue
            candidates = candidate_urls_from_work(work, source)
            if not candidates:
                continue
            unit_id, unit_label, unit_year = unit_from_work(str(spec["slug"]), work)
            loc = source_location(work)
            landing = loc.get("landing_page_url") or candidates[0]
            oa = work.get("open_access") or {}
            concepts = "; ".join(
                c.get("display_name", "")
                for c in work.get("concepts") or []
                if c.get("display_name")
            )
            rows.append(
                {
                    "source": source,
                    "unit_id": unit_id,
                    "unit_type": "conference_year" if "Conference" in source else "journal_issue_or_year",
                    "year": unit_year,
                    "unit_date": work.get("publication_date") or f"{unit_year}-01-01",
                    "unit_label": unit_label,
                    "unit_url": landing,
                    "landing_page_url": landing,
                    "source_url": url,
                    "title": title,
                    "summary": "",
                    "authors": author_string(work),
                    "oa_marker": (
                        ("source_known_oa;" if spec.get("source_known_oa") else "")
                        + (str(oa.get("oa_status") or "unknown"))
                        + (";is_oa" if oa.get("is_oa") or spec.get("source_known_oa") else "")
                    ),
                    "pdf_url": candidates[0],
                    "doi": normalize_doi(work.get("doi")),
                    "openalex_id": work.get("id") or "",
                    "source_item_id": work.get("id") or "",
                    "paper_type": work.get("type") or "",
                    "venue_section": f"OpenAlex concept filter: {spec['concept_label']}",
                    "concepts": concepts,
                    "referenced_works_count": work.get("referenced_works_count") or "",
                    "cited_by_count": work.get("cited_by_count") or "",
                }
            )
        cursor = (data.get("meta") or {}).get("next_cursor")
        if not data.get("results"):
            break
        time.sleep(1.0)

    units: dict[str, dict[str, Any]] = {}
    for row in rows:
        units.setdefault(
            row["unit_id"],
            {
                "unit_id": row["unit_id"],
                "unit_type": row["unit_type"],
                "year": row["year"],
                "date": row["unit_date"],
                "unit_label": row["unit_label"],
                "unit_url": row["unit_url"],
            },
        )

    write_csv(inv_dir / "archive_pages.csv", archive_pages, ["page_index", "source_url", "result_count"])
    write_csv(
        inv_dir / "units.csv",
        sorted(units.values(), key=lambda r: (r["year"], r["unit_id"])),
        ["unit_id", "unit_type", "year", "date", "unit_label", "unit_url"],
    )
    write_csv(inv_dir / "items.csv", rows, CSV_FIELDS)
    manifest = {
        "inventory_id": inventory_id,
        "source": source,
        "source_id": spec["source_id"],
        "parser_id": "openalex_source_concept_filter",
        "parser_version": "1.0.0",
        "build_date": RUN_DATE,
        "coverage_years": f"{tuple_plan['year_min']}-{tuple_plan['year_max']}",
        "field_filter": {
            "concept_id": spec["concept_id"],
            "concept_label": spec["concept_label"],
            "note": "Neutral OpenAlex concept filter used as the effective field frame for this source block.",
        },
        "oa_filter": "open_access.is_oa:true" if spec.get("require_oa") else "source known open / source-specific PDF route",
        "user_agent": USER_AGENT,
        "rate_limit": "<=1 request/sec",
        "archive_pages": len(archive_pages),
        "items": len(rows),
        "units": len(units),
        "known_gaps": [],
        "completeness_notes": (
            "Complete OpenAlex cursor traversal for the source/year/concept filter at build time; "
            "not a full-source inventory for broad venues."
        ),
    }
    (inv_dir / "inventory_manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    return inv_dir


def ensure_inventories() -> dict[str, Path]:
    paths: dict[str, Path] = {}
    seen: set[str] = set()
    for tuple_plan in PLAN:
        source = str(tuple_plan["source"])
        inventory_id = str(tuple_plan["source_inventory_id"])
        if inventory_id in seen:
            continue
        seen.add(inventory_id)
        if source in OPENALEX_SOURCES:
            paths[inventory_id] = build_openalex_inventory(tuple_plan)
        else:
            paths[inventory_id] = find_inventory(inventory_id)
    return paths


def unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for value in values:
        value = value.strip()
        if value and value not in seen:
            seen.add(value)
            out.append(value)
    return out


def openalex_locations_for_row(row: dict[str, str]) -> list[str]:
    openalex_id = row.get("openalex_id", "").strip()
    if not openalex_id:
        return []
    if openalex_id in OPENALEX_LOCATION_CACHE:
        return OPENALEX_LOCATION_CACHE[openalex_id]
    work_id = openalex_id.rsplit("/", 1)[-1]
    url = "https://api.openalex.org/works/" + work_id + "?" + urllib.parse.urlencode(
        {"select": "primary_location,best_oa_location,locations"}
    )
    try:
        data = request_json(url)
    except Exception:
        return []
    urls: list[str] = []
    for loc in [data.get("primary_location"), data.get("best_oa_location")] + (data.get("locations") or []):
        if isinstance(loc, dict):
            for key in ["pdf_url", "landing_page_url"]:
                value = loc.get(key)
                if value:
                    urls.append(str(value))
    OPENALEX_LOCATION_CACHE[openalex_id] = unique(urls)
    return OPENALEX_LOCATION_CACHE[openalex_id]


def has_arxiv_pdf_alternate(row: dict[str, str]) -> bool:
    return any("arxiv.org/pdf" in url.lower() for url in openalex_locations_for_row(row))


def candidate_urls_for_row(
    row: dict[str, str],
    source: str,
    include_openalex_alternates: bool = False,
) -> list[tuple[str, str]]:
    urls: list[tuple[str, str]] = []
    direct = row.get("pdf_url", "").strip()
    if direct and looks_pdf_url(direct):
        urls.append(("direct_pdf_route", direct))

    doi = row.get("doi", "").strip().lower()
    if source == "PLOS Biology" and doi.startswith("10.1371/journal.pbio"):
        urls.append(("direct_pdf_route", f"https://journals.plos.org/plosbiology/article/file?id={doi}&type=printable"))
    if source == "Chemical Science" and doi.startswith("10.1039/"):
        suffix = doi.rsplit("/", 1)[-1]
        year = row.get("year", "").strip()
        if year:
            urls.append(("direct_pdf_route", f"https://pubs.rsc.org/en/content/articlepdf/{year}/sc/{suffix}"))
    if source == "Journal of Cosmology and Astroparticle Physics" and doi.startswith("10.1088/"):
        urls.append(("direct_pdf_route", f"https://iopscience.iop.org/article/{doi}/pdf"))

    if include_openalex_alternates:
        for url in openalex_locations_for_row(row):
            route = "documented_oa_alternate" if "arxiv.org" in url or "pmc" in url or "osti.gov" in url else "direct_or_documented_location"
            urls.append((route, url))

    out: list[tuple[str, str]] = []
    seen: set[str] = set()
    for route, url in urls:
        candidates = [url]
        if url.startswith("http://"):
            candidates.insert(0, "https://" + url[len("http://") :])
        for candidate in candidates:
            if candidate and candidate not in seen:
                seen.add(candidate)
                out.append((route, candidate))
    return out


def fetch_bytes(url: str, accept: str, referer: str = "") -> tuple[bytes, str, str]:
    headers = {
        "User-Agent": BROWSER_USER_AGENT,
        "Accept": accept,
    }
    if referer:
        headers["Referer"] = referer
    request = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(request, timeout=90) as response:
        return response.read(), response.geturl(), response.headers.get("content-type", "")


def try_pdf_url(url: str, referer: str = "") -> tuple[bytes, str]:
    waits = [3.0, 30.0, 60.0] if "openreview.net" in url.lower() else [1.0, 10.0]
    last_error: Exception | None = None
    for wait in waits:
        time.sleep(wait)
        try:
            data, final_url, content_type = fetch_bytes(url, "application/pdf,text/html,*/*", referer)
            if data.lstrip().startswith(b"%PDF"):
                return data, final_url
            if "application/pdf" in content_type.lower() and len(data) > 1000:
                return data, final_url
            raise ValueError(f"not PDF bytes; content_type={content_type}; final_url={final_url}")
        except urllib.error.HTTPError as exc:
            last_error = exc
            if exc.code != 429:
                raise
        except Exception as exc:  # noqa: BLE001
            last_error = exc
            raise
    assert last_error is not None
    raise last_error


def publisher_page_pdf_candidates(landing_url: str) -> tuple[list[str], str]:
    if not landing_url:
        return [], "no landing page URL"
    try:
        time.sleep(1.0)
        data, final_url, content_type = fetch_bytes(landing_url, "text/html,application/xhtml+xml,*/*")
    except Exception as exc:  # noqa: BLE001
        return [], repr(exc)
    if data.lstrip().startswith(b"%PDF"):
        return [final_url], "landing page returned PDF bytes"
    text = data.decode("utf-8", errors="ignore")
    hrefs = re.findall(r'href=["\']([^"\']+)["\']', text, flags=re.I)
    candidates: list[str] = []
    for href in hrefs:
        lowered = href.lower()
        if any(token in lowered for token in ["pdf", "download", "article-pdf", "pdfdirect"]):
            candidates.append(urllib.parse.urljoin(final_url, href))
    return unique(candidates), f"content_type={content_type}; extracted_pdf_like_links={len(candidates)}"


def acquire_pdf(row: dict[str, str], source: str) -> tuple[bytes, str, str, list[dict[str, str]]]:
    attempts: list[dict[str, str]] = []
    landing = row.get("landing_page_url", "")
    direct_failed = False
    candidates = candidate_urls_for_row(row, source, include_openalex_alternates=True)
    direct_candidates = [item for item in candidates if item[0] == "direct_pdf_route"]
    other_candidates = [item for item in candidates if item[0] != "direct_pdf_route"]

    for route, url in direct_candidates:
        try:
            pdf, final_url = try_pdf_url(url, landing)
            attempts.append({"route": route, "url": url, "result": "success", "final_url": final_url})
            return pdf, final_url, route, attempts
        except Exception as exc:  # noqa: BLE001
            direct_failed = True
            attempts.append({"route": route, "url": url, "result": "failed", "error": repr(exc)})

    if direct_failed:
        candidates, note = publisher_page_pdf_candidates(landing)
        attempts.append(
            {
                "route": "publisher_page_browser_fallback",
                "url": landing,
                "result": "candidate_links_found" if candidates else "failed",
                "note": note,
            }
        )
        for url in candidates:
            try:
                pdf, final_url = try_pdf_url(url, landing)
                attempts.append(
                    {
                        "route": "publisher_page_browser_fallback",
                        "url": url,
                        "result": "success",
                        "final_url": final_url,
                    }
                )
                return pdf, final_url, "publisher_page_browser_fallback", attempts
            except Exception as exc:  # noqa: BLE001
                attempts.append(
                    {
                        "route": "publisher_page_browser_fallback",
                        "url": url,
                        "result": "failed",
                        "error": repr(exc),
                    }
                )

    for route, url in other_candidates:
        try:
            pdf, final_url = try_pdf_url(url, landing)
            attempts.append({"route": route, "url": url, "result": "success", "final_url": final_url})
            return pdf, final_url, route, attempts
        except Exception as exc:  # noqa: BLE001
            attempts.append({"route": route, "url": url, "result": "failed", "error": repr(exc)})

    raise RuntimeError(json.dumps(attempts, ensure_ascii=False))


def page_count(pdf_bytes: bytes) -> int:
    return len(PdfReader(io.BytesIO(pdf_bytes)).pages)


def extract_text(pdf_bytes: bytes) -> str:
    def timeout_handler(signum, frame):  # noqa: ARG001
        raise TimeoutError("PDF text extraction timed out")

    previous = signal.signal(signal.SIGALRM, timeout_handler)
    signal.alarm(60)
    try:
        reader = PdfReader(io.BytesIO(pdf_bytes))
        chunks: list[str] = []
        for idx, page in enumerate(reader.pages, start=1):
            try:
                page_text = page.extract_text() or ""
            except Exception as exc:  # noqa: BLE001
                page_text = f"[Text extraction failed on page {idx}: {exc}]"
            chunks.append(f"\n\n--- Page {idx} ---\n{page_text}")
        return "\n".join(chunks).strip()
    finally:
        signal.alarm(0)
        signal.signal(signal.SIGALRM, previous)


def eligible(row: dict[str, str], tuple_plan: dict[str, Any]) -> bool:
    try:
        year = int(row.get("year", ""))
    except ValueError:
        return False
    if not (int(tuple_plan["year_min"]) <= year <= int(tuple_plan["year_max"])):
        return False
    if not is_research_like(row.get("title", ""), row.get("paper_type", "")):
        return False
    if tuple_plan["source"] == "Physical Review X":
        text = f"{row.get('title', '')} {row.get('concepts', '')}".lower()
        if not any(token in text for token in ["condensed matter", "materials science", "chemistry", "quantum", "topological", "spin", "superconduct"]):
            return False
        if not has_arxiv_pdf_alternate(row):
            return False
    return bool(candidate_urls_for_row(row, str(tuple_plan["source"])))


def draw_for_tuple(
    tuple_plan: dict[str, Any],
    inv_dir: Path,
    papers: list[dict[str, str]],
    taken: set[tuple[str, str]],
    next_id: int,
    tuple_index: int,
    dry_run: bool,
) -> tuple[list[dict[str, Any]], int, list[dict[str, Any]]]:
    source = str(tuple_plan["source"])
    items_path = inv_dir / "items.csv"
    units_path = inv_dir / "units.csv"
    manifest_path = inv_dir / "inventory_manifest.json"
    rows = [r for r in read_csv(items_path) if eligible(r, tuple_plan)]
    units: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in rows:
        units[row["unit_id"]].append(row)
    unit_ids = sorted(uid for uid, unit_rows in units.items() if unit_rows)
    if not unit_ids:
        raise SystemExit(f"No eligible units for {source} {tuple_plan['year_min']}-{tuple_plan['year_max']}")

    seed = SEED_BASE + tuple_index
    rng = random.Random(seed)
    selected: list[dict[str, Any]] = []
    attempts_log: list[dict[str, Any]] = []
    local_taken: set[tuple[str, str]] = set()
    attempts = 0
    max_attempts = max(1000, int(tuple_plan["needed"]) * 300)

    while len(selected) < int(tuple_plan["needed"]):
        attempts += 1
        if attempts > max_attempts:
            raise SystemExit(f"Exceeded {max_attempts} attempts for {source}")
        live_units = [
            uid
            for uid in unit_ids
            if any(not (row_keys(r) & (taken | local_taken)) for r in units[uid])
        ]
        if not live_units:
            raise SystemExit(f"Exhausted available papers for {source}")
        unit_id = rng.choice(live_units)
        available = [r for r in units[unit_id] if not (row_keys(r) & (taken | local_taken))]
        if not available:
            continue
        row = rng.choice(available)
        keys = row_keys(row)
        paper_id = f"P{next_id:04d}"
        field = str(tuple_plan["field"])
        slug = FIELD_SLUG[field]
        pdf_path = ROOT / "pdfs" / slug / f"{paper_id}.pdf"
        text_path = ROOT / "text" / slug / f"{paper_id}.txt"
        p = 1.0 / len(unit_ids) / len(units[unit_id])

        record: dict[str, Any] = {
            "paper_id": paper_id,
            "field": field,
            "source": source,
            "venue_tier": tuple_plan["venue_tier"],
            "era_band": tuple_plan["era_band"],
            "source_block_order": tuple_plan["source_block_order"],
            "draw_idx": len(selected) + 1,
            "seed": seed,
            "inventory_id": tuple_plan["source_inventory_id"],
            "inventory_dir": str(inv_dir.relative_to(WORKSPACE)),
            "inventory_manifest_sha256": sha256_file(manifest_path),
            "items_sha256": sha256_file(items_path),
            "units_sha256": sha256_file(units_path),
            "unit_id": unit_id,
            "unit_year": row.get("year", ""),
            "unit_label": row.get("unit_label", ""),
            "title": row.get("title", ""),
            "authors": row.get("authors", ""),
            "doi": row.get("doi", ""),
            "openalex_id": row.get("openalex_id", ""),
            "source_item_id": row.get("source_item_id", ""),
            "landing_page_url": row.get("landing_page_url", ""),
            "pdf_url": row.get("pdf_url", ""),
            "oa_marker": row.get("oa_marker", ""),
            "paper_type": row.get("paper_type", ""),
            "openalex_cited_by_count": row.get("cited_by_count", ""),
            "eligible_units": len(unit_ids),
            "eligible_papers_in_selected_unit": len(units[unit_id]),
            "selection_probability": p,
            "design_weight": 1.0 / p,
            "pdf_path": str(pdf_path.relative_to(WORKSPACE)),
            "text_path": str(text_path.relative_to(WORKSPACE)),
            "attempts_before_success": attempts,
        }

        if dry_run:
            record["pdf_sha256"] = ""
            record["text_chars"] = ""
            record["pages"] = ""
            record["acquisition_route"] = ""
            selected.append(record)
            local_taken |= keys
            next_id += 1
            continue

        print(f"  trying {paper_id}: {source} | {row.get('year', '')} | {row.get('title', '')[:80]}", flush=True)
        try:
            pdf_bytes, final_url, route, route_attempts = acquire_pdf(row, source)
            text = extract_text(pdf_bytes)
            text_chars = len(text.strip())
            if text_chars < 5000:
                raise ValueError(f"extracted text too short ({text_chars} chars)")
            pages = page_count(pdf_bytes)
        except Exception as exc:  # noqa: BLE001
            attempts_log.append(
                {
                    "paper_id": paper_id,
                    "source": source,
                    "title": row.get("title", ""),
                    "doi": row.get("doi", ""),
                    "result": "aborted_failure",
                    "error": repr(exc),
                }
            )
            raise

        pdf_path.parent.mkdir(parents=True, exist_ok=True)
        text_path.parent.mkdir(parents=True, exist_ok=True)
        pdf_path.write_bytes(pdf_bytes)
        text_path.write_text(text.rstrip() + "\n", encoding="utf-8")
        record["pdf_url"] = final_url
        record["pdf_sha256"] = sha256_bytes(pdf_bytes)
        record["text_chars"] = text_chars
        record["pages"] = pages
        record["acquisition_route"] = route
        selected.append(record)
        attempts_log.append(
            {
                "paper_id": paper_id,
                "source": source,
                "title": row.get("title", ""),
                "doi": row.get("doi", ""),
                "result": "success",
                "route": route,
                "attempts": route_attempts,
            }
        )
        print(f"    acquired {paper_id}: pages={pages} text_chars={text_chars} route={route}", flush=True)
        local_taken |= keys
        next_id += 1

    taken |= local_taken
    return selected, next_id, attempts_log


def blocked_recovery_probe() -> list[dict[str, Any]]:
    results: list[dict[str, Any]] = []
    for check in BLOCKED_RECOVERY_CHECKS:
        inv_dir = find_inventory(check["source_inventory_id"])
        rows = read_csv(inv_dir / "items.csv")
        sample_rows = [
            r
            for r in rows
            if r.get("paper_type") == "article" and 2015 <= int(r.get("year") or 0) <= 2025
        ][:3]
        for row in sample_rows:
            landing = row.get("landing_page_url", "")
            candidates, note = publisher_page_pdf_candidates(landing)
            results.append(
                {
                    **check,
                    "title": row.get("title", ""),
                    "doi": row.get("doi", ""),
                    "landing_page_url": landing,
                    "browser_fallback_result": "candidate_links_found" if candidates else "failed",
                    "browser_fallback_note": note,
                    "candidate_links": ";".join(candidates[:5]),
                }
            )
    return results


def cleanup_outputs(selected: list[dict[str, Any]]) -> None:
    for record in selected:
        for key in ["pdf_path", "text_path"]:
            path = WORKSPACE / str(record.get(key, ""))
            if path.exists():
                path.unlink()


def append_outputs(
    schedule: list[dict[str, str]],
    selected: list[dict[str, Any]],
    route_logs: list[dict[str, Any]],
    blocked_checks: list[dict[str, Any]],
) -> None:
    papers = read_csv(PAPERS_CSV)
    paper_fieldnames = list(papers[0].keys())
    acquisition_rows = read_csv(ACQUISITION_LOG_CSV)
    acquisition_fieldnames = list(acquisition_rows[0].keys())

    for record in selected:
        papers.append(
            {
                "paper_id": record["paper_id"],
                "field": record["field"],
                "venue_tier": record["venue_tier"],
                "paper_title": record["title"],
                "authors": record["authors"],
                "year": record["unit_year"],
                "journal": record["source"],
                "doi": record["doi"],
                "pmid": "",
                "pmcid": "",
                "url": record["landing_page_url"],
                "pdf_url": record["pdf_url"],
                "pdf_path": record["pdf_path"],
                "text_path": record["text_path"],
                "oa_status": record["oa_marker"],
                "download_status": "downloaded_verified_pdf",
                "openalex_cited_by_count": record["openalex_cited_by_count"],
                "source_checked_date": RUN_DATE,
                "notes": (
                    f"{ROUND_ID}; balanced repair Stratum A draw; source={record['source']}; "
                    f"inventory={record['inventory_id']}; unit={record['unit_id']}; "
                    f"route={record.get('acquisition_route', '')}; no overfilled PLOS/Frontiers/EPJ lanes extended"
                ),
                "sample_source": "random_base",
                "selection_probability": f"{record['selection_probability']:.15g}",
                "design_weight": f"{record['design_weight']:.15g}",
            }
        )
        acquisition_rows.append(
            {
                "paper_id": record["paper_id"],
                "source_type": "publisher_or_repository_pdf",
                "source_url": record["pdf_url"],
                "access_status": "oa_accessible_verified",
                "download_status": "downloaded_verified_pdf",
                "local_path": record["pdf_path"],
                "checked_date": RUN_DATE,
                "notes": (
                    f"{ROUND_ID}; source={record['source']}; inventory={record['inventory_id']}; "
                    f"text_path={record['text_path']}; pages={record.get('pages', '')}; "
                    f"text_chars={record.get('text_chars', '')}; route={record.get('acquisition_route', '')}; "
                    f"source_block={record['source']}; source_block_order={record['source_block_order']}"
                ),
            }
        )

    write_csv(PAPERS_CSV, papers, paper_fieldnames)
    write_csv(ACQUISITION_LOG_CSV, acquisition_rows, acquisition_fieldnames)

    schedule_fieldnames = list(schedule[0].keys())
    schedule_by_key = {
        (r["field"], r["era_band"], r["venue_tier"], r["source"]): dict(r)
        for r in schedule
    }
    for tuple_plan in PLAN:
        key = (
            str(tuple_plan["field"]),
            str(tuple_plan["era_band"]),
            str(tuple_plan["venue_tier"]),
            str(tuple_plan["source"]),
        )
        schedule_by_key.setdefault(
            key,
            {
                "field": tuple_plan["field"],
                "era_band": tuple_plan["era_band"],
                "venue_tier": tuple_plan["venue_tier"],
                "source_block_order": tuple_plan["source_block_order"],
                "source": tuple_plan["source"],
                "source_inventory_id": tuple_plan["source_inventory_id"],
                "status": "planned_to_target_10",
                "random_base_papers_in_block": "0",
                "paper_ids": "",
                "design_note": tuple_plan["round_reason"],
            },
        )

    grouped: dict[tuple[str, str, str, str], list[str]] = defaultdict(list)
    for record in selected:
        grouped[
            (
                str(record["field"]),
                str(record["era_band"]),
                str(record["venue_tier"]),
                str(record["source"]),
            )
        ].append(str(record["paper_id"]))

    for key, ids in grouped.items():
        row = schedule_by_key[key]
        existing_ids = [x for x in row.get("paper_ids", "").split(";") if x]
        merged = existing_ids + ids
        row["paper_ids"] = ";".join(merged)
        row["random_base_papers_in_block"] = str(len(merged))
        row["status"] = "completed_target_10" if len(merged) >= TARGET_PER_TUPLE else "partially_completed"
        row["design_note"] = (
            row.get("design_note", "")
            + f" Updated by {ROUND_ID}; added {len(ids)} papers; overfilled lanes frozen; "
            "direct failures used publisher-page fallback before documented OA alternates."
        ).strip()
        schedule_by_key[key] = row

    rows = sorted(
        (
            {field: row.get(field, "") for field in schedule_fieldnames}
            for row in schedule_by_key.values()
        ),
        key=lambda r: (
            r["field"],
            r["era_band"],
            r["venue_tier"],
            int(r.get("source_block_order", "999") or 999),
            r["source"],
        ),
    )
    write_csv(SCHEDULE_CSV, rows, schedule_fieldnames)

    audit_dir = AUDIT_ROOT / ROUND_ID
    audit_dir.mkdir(parents=True, exist_ok=True)
    public_records = [{k: v for k, v in r.items() if k not in {"_inventory_row"}} for r in selected]
    block_counts = Counter((r["field"], r["era_band"], r["source"]) for r in selected)
    audit = {
        "round_id": ROUND_ID,
        "run_date": RUN_DATE,
        "requested_new_papers": REQUESTED_NEW_PAPERS,
        "new_papers_added": len(selected),
        "seed_base": SEED_BASE,
        "target_per_tuple": TARGET_PER_TUPLE,
        "user_agent": USER_AGENT,
        "rate_limit": "<=1 request/sec for OpenAlex and acquisition requests",
        "selection_rationale": (
            "Repair-oriented batch after prior overfilling. PLOS Computational Biology, "
            "Frontiers in Computational Neuroscience, and EPJ C source blocks were frozen. "
            "This batch prioritizes underrepresented fields and underfilled scheduled blocks, "
            "opens additional candidate source blocks from journal_list_v0.md, and records "
            "publisher-page fallback before documented OA alternate use."
        ),
        "plan": PLAN,
        "prior_aborted_attempts": PRIOR_ABORTED_ATTEMPTS,
        "blocked_recovery_checks": blocked_checks,
        "block_counts_added": [
            {"field": k[0], "era_band": k[1], "source": k[2], "added": v}
            for k, v in sorted(block_counts.items())
        ],
        "selected": public_records,
        "route_logs": route_logs,
    }
    (audit_dir / "draw_audit.json").write_text(
        json.dumps(audit, indent=2) + "\n", encoding="utf-8"
    )

    selection_fieldnames = [
        "paper_id",
        "field",
        "source",
        "inventory_id",
        "era_band",
        "draw_idx",
        "seed",
        "unit_id",
        "unit_year",
        "title",
        "doi",
        "eligible_units",
        "eligible_papers_in_selected_unit",
        "selection_probability",
        "design_weight",
        "acquisition_route",
        "pdf_url",
        "pdf_path",
        "text_path",
        "pdf_sha256",
        "text_chars",
    ]
    write_csv(
        audit_dir / "selection_table.csv",
        [
            {
                **{k: r.get(k, "") for k in selection_fieldnames},
                "selection_probability": f"{r['selection_probability']:.15g}",
                "design_weight": f"{r['design_weight']:.15g}",
            }
            for r in public_records
        ],
        selection_fieldnames,
    )
    write_csv(
        audit_dir / "preflight_target_table.csv",
        [
            {
                "field": p["field"],
                "era_band": p["era_band"],
                "venue_tier": p["venue_tier"],
                "source": p["source"],
                "source_inventory_id": p["source_inventory_id"],
                "planned_additions": p["needed"],
                "reason": p["round_reason"],
            }
            for p in PLAN
        ],
        [
            "field",
            "era_band",
            "venue_tier",
            "source",
            "source_inventory_id",
            "planned_additions",
            "reason",
        ],
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    requested = sum(int(p["needed"]) for p in PLAN)
    if requested != REQUESTED_NEW_PAPERS:
        raise SystemExit(f"Plan requests {requested}, expected {REQUESTED_NEW_PAPERS}")

    print("preflight target table", flush=True)
    for p in PLAN:
        print(
            f"  {p['needed']:>2} | {p['field']} | {p['era_band']} | {p['source']} | {p['round_reason']}",
            flush=True,
        )

    inventories = ensure_inventories()
    blocked_checks = blocked_recovery_probe() if not args.dry_run else []
    papers = read_csv(PAPERS_CSV)
    schedule = read_csv(SCHEDULE_CSV)
    taken = known_keys(papers)
    next_id = max_paper_number(papers) + 1
    selected_all: list[dict[str, Any]] = []
    route_logs: list[dict[str, Any]] = []

    try:
        for idx, tuple_plan in enumerate(PLAN, start=1):
            inv_dir = inventories[str(tuple_plan["source_inventory_id"])]
            print(
                f"\n== {tuple_plan['field']} | {tuple_plan['era_band']} | "
                f"{tuple_plan['source']} | needed={tuple_plan['needed']} ==",
                flush=True,
            )
            selected, next_id, logs = draw_for_tuple(
                tuple_plan, inv_dir, papers, taken, next_id, idx, args.dry_run
            )
            selected_all.extend(selected)
            route_logs.extend(logs)
            print(f"{tuple_plan['source']} selected={len(selected)}", flush=True)
    except Exception:
        cleanup_outputs(selected_all)
        raise

    print(f"total_selected={len(selected_all)}", flush=True)
    if args.dry_run:
        return
    append_outputs(schedule, selected_all, route_logs, blocked_checks)


if __name__ == "__main__":
    main()
