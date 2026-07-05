#!/usr/bin/env python3
"""Add an initial protocol-compliant Field 9 random-base batch.

This batch starts the clinical medicine / epidemiology / evidence-based
medicine field from declared Field 9 source blocks in journal_list_v1.md. It
uses publisher/source archive routes as the sampling frame, records a preflight
table before registry mutation, and preserves the source-block audit trail.
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
from datetime import date, datetime
from pathlib import Path
from typing import Any

from lxml import html as lxml_html
from pypdf import PdfReader


ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT.parent
PAPERS_CSV = ROOT / "metadata" / "papers.csv"
SCHEDULE_CSV = ROOT / "sources" / "source_block_schedule_v0.csv"
ACQUISITION_LOG_CSV = ROOT / "sources" / "acquisition_log.csv"
INVENTORIES = ROOT / "source_inventories"
AUDIT_ROOT = ROOT / "sources" / "draw_audits"

ROUND_ID = "round_20260621_clinical_medicine_initial_30"
RUN_DATE = date.today().isoformat()
REQUESTED_NEW_PAPERS = 30
TARGET_PER_TUPLE = 10
SEED_BASE = 202606210900
USER_AGENT = (
    "TextDataMining-CorpusExpansion/0.1 "
    "(research corpus; local run; <=1 request/sec)"
)
BROWSER_USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
)

FIELD = "Clinical medicine / epidemiology / evidence-based medicine"
FIELD_SLUG = "clinical_medicine"

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

PLAN: list[dict[str, Any]] = [
    {
        "field": FIELD,
        "era_band": "2005-2014_completed_years_2005-2014",
        "year_min": 2005,
        "year_max": 2014,
        "venue_tier": "mid",
        "source_block_order": "1",
        "source": "PLOS Medicine",
        "source_inventory_id": "plos_medicine_plos_api_v1.0.0_2005-2014_20260621",
        "needed": 10,
        "round_reason": (
            "Initial Field 9 source block from journal_list_v1.md; PLOS Medicine is "
            "listed as top/mid and operationalized as mid pending final tier split. "
            "Official PLOS API/issue metadata define the completed 2005-2014 frame."
        ),
    },
    {
        "field": FIELD,
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "mid",
        "source_block_order": "1",
        "source": "PLOS Medicine",
        "source_inventory_id": "plos_medicine_plos_api_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "round_reason": (
            "Initial Field 9 modern broad-OA clinical block from journal_list_v1.md. "
            "Official PLOS API/issue metadata define the completed 2015-2025 frame."
        ),
    },
    {
        "field": FIELD,
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "mid",
        "source_block_order": "2",
        "source": "BMC Medicine",
        "source_inventory_id": "bmc_medicine_springer_archive_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "round_reason": (
            "Initial Field 9 broad-OA clinical companion block from journal_list_v1.md. "
            "Springer/BMC publisher article archive pages define the completed 2015-2025 frame."
        ),
    },
]


def round_id() -> str:
    return ROUND_ID


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def write_csv(path: Path, rows: list[dict[str, Any]], fieldnames: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def csv_has_data_rows(path: Path) -> bool:
    if not path.exists():
        return False
    with path.open(newline="", encoding="utf-8") as f:
        reader = csv.reader(f)
        next(reader, None)
        return any(True for _ in reader)


def csv_has_nonblank_column(path: Path, column: str) -> bool:
    if not path.exists():
        return False
    with path.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        return any((row.get(column) or "").strip() for row in reader)


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
    for raw_key, mapped_key in [
        ("doi", "doi"),
        ("title", "title"),
        ("landing_page_url", "url"),
        ("pdf_url", "pdf"),
        ("openalex_id", "openalex_id"),
        ("source_item_id", "source_item_id"),
    ]:
        value = row.get(raw_key, "").strip().lower()
        if value:
            keys.add((mapped_key, value))
    return keys


def normalize_doi(value: str | None) -> str:
    if not value:
        return ""
    value = value.strip()
    value = value.removeprefix("https://doi.org/").removeprefix("http://doi.org/")
    return value.lower()


def request_bytes(url: str, accept: str, browser: bool = False, referer: str = "") -> tuple[bytes, str, str]:
    headers = {
        "User-Agent": BROWSER_USER_AGENT if browser else USER_AGENT,
        "Accept": accept,
    }
    if referer:
        headers["Referer"] = referer
    request = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(request, timeout=90) as response:
        return response.read(), response.geturl(), response.headers.get("content-type", "")


def request_json(url: str) -> dict[str, Any]:
    data, _, _ = request_bytes(url, "application/json,*/*")
    return json.loads(data.decode("utf-8"))


def unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for value in values:
        value = value.strip()
        if value and value not in seen:
            seen.add(value)
            out.append(value)
    return out


def is_excluded_title(title: str) -> bool:
    lowered = title.strip().lower()
    excluded_prefixes = (
        "correction:",
        "erratum:",
        "retraction:",
        "editorial:",
        "author correction:",
        "publisher correction:",
        "case report:",
    )
    excluded_terms = (
        " protocol",
        "study protocol",
        "trial protocol",
        "systematic review",
        "meta-analysis",
        "meta analysis",
        "scoping review",
        "narrative review",
        "overview of reviews",
        "guideline",
    )
    return lowered.startswith(excluded_prefixes) or any(term in lowered for term in excluded_terms)


def author_string(authors: list[str]) -> str:
    authors = [html.unescape(a).strip() for a in authors if a and a.strip()]
    if len(authors) > 5:
        return "; ".join(authors[:5]) + "; et al."
    return "; ".join(authors)


def plos_article_url(doi: str) -> str:
    return f"https://journals.plos.org/plosmedicine/article?id={doi}"


def plos_pdf_url(doi: str) -> str:
    return f"https://journals.plos.org/plosmedicine/article/file?id={doi}&type=printable"


def plos_issue_url(volume: str, issue: str) -> str:
    if volume and issue:
        return f"https://journals.plos.org/plosmedicine/issue?id=10.1371/issue.pmed.v{int(volume):02d}.i{int(issue):02d}"
    return "https://journals.plos.org/plosmedicine"


def title_map_from_plos_issue(unit_url: str) -> dict[str, str]:
    data, final_url, _ = request_bytes(unit_url, "text/html,application/xhtml+xml,*/*")
    doc = lxml_html.fromstring(data.decode("utf-8", errors="ignore"))
    out: dict[str, str] = {}
    links = doc.xpath('//a[contains(@href,"article?id=10.1371/journal.pmed")]')
    for link in links:
        href = urllib.parse.urljoin(final_url, link.get("href") or "")
        match = re.search(r"id=(10\.1371/journal\.pmed\.[^&#]+)", href)
        if not match:
            continue
        title = html.unescape(re.sub(r"\s+", " ", link.text_content()).strip())
        if title:
            out[normalize_doi(match.group(1))] = title
    return out


def enrich_plos_titles(rows: list[dict[str, Any]]) -> tuple[int, int]:
    titles_found = 0
    pages_checked = 0
    cache: dict[str, dict[str, str]] = {}
    for row in rows:
        unit_url = str(row.get("unit_url") or "")
        if not unit_url:
            continue
        if unit_url not in cache:
            cache[unit_url] = title_map_from_plos_issue(unit_url)
            pages_checked += 1
            time.sleep(1.0)
        title = cache[unit_url].get(normalize_doi(str(row.get("doi") or "")), "")
        if title:
            row["title"] = title
            titles_found += 1
    return titles_found, pages_checked


def build_plos_inventory(tuple_plan: dict[str, Any]) -> Path:
    inventory_id = str(tuple_plan["source_inventory_id"])
    inv_dir = INVENTORIES / "plos_medicine" / inventory_id
    if (
        (inv_dir / "inventory_manifest.json").exists()
        and (inv_dir / "items.csv").exists()
        and (inv_dir / "units.csv").exists()
        and csv_has_data_rows(inv_dir / "items.csv")
        and csv_has_data_rows(inv_dir / "units.csv")
        and csv_has_nonblank_column(inv_dir / "items.csv", "title")
    ):
        return inv_dir
    inv_dir.mkdir(parents=True, exist_ok=True)

    start = 0
    rows_per_page = 500
    docs_by_doi: dict[str, dict[str, Any]] = {}
    archive_pages: list[dict[str, Any]] = []
    query = 'journal:"PLOS Medicine" AND article_type:"Research Article"'
    date_filter = (
        f"publication_date:[{tuple_plan['year_min']}-01-01T00:00:00Z TO "
        f"{tuple_plan['year_max']}-12-31T23:59:59Z]"
    )
    fields = "id,journal,publication_date,article_type,author,volume,issue"
    while True:
        params = {
            "q": query,
            "fq": date_filter,
            "fl": fields,
            "rows": str(rows_per_page),
            "start": str(start),
            "wt": "json",
        }
        url = "https://api.plos.org/search?" + urllib.parse.urlencode(params)
        data = request_json(url)
        response = data.get("response") or {}
        docs = response.get("docs") or []
        archive_pages.append(
            {
                "page_index": len(archive_pages) + 1,
                "source_url": url,
                "result_count": len(docs),
                "raw_num_found": response.get("numFound", ""),
            }
        )
        for doc in docs:
            raw_id = str(doc.get("id") or "")
            match = re.match(r"(10\.1371/journal\.pmed\.[^/]+)", raw_id)
            doi = normalize_doi(match.group(1) if match else "")
            if not doi or not doi.startswith("10.1371/journal.pmed."):
                continue
            if doi in docs_by_doi:
                continue
            docs_by_doi[doi] = doc
        start += rows_per_page
        if start >= int(response.get("numFound") or 0) or not docs:
            break
        time.sleep(1.0)

    rows: list[dict[str, Any]] = []
    for doi, doc in sorted(docs_by_doi.items()):
        date_text = str(doc.get("publication_date") or "")
        year = date_text[:4]
        if not year:
            continue
        volume = str(doc.get("volume") or "")
        issue = str(doc.get("issue") or "")
        unit_id = f"plos_medicine_{year}_v{slugify(volume)}_i{slugify(issue)}" if volume and issue else f"plos_medicine_{year}"
        unit_label = f"vol. {volume}, issue {issue}" if volume and issue else f"year {year}"
        unit_url = plos_issue_url(volume, issue)
        rows.append(
            {
                "source": "PLOS Medicine",
                "unit_id": unit_id,
                "unit_type": "journal_issue",
                "year": year,
                "unit_date": date_text[:10],
                "unit_label": unit_label,
                "unit_url": unit_url,
                "landing_page_url": plos_article_url(doi),
                "source_url": unit_url,
                "title": "",
                "summary": "",
                "authors": author_string(list(doc.get("author") or [])),
                "oa_marker": "gold;source_known_oa;plos_research_article",
                "pdf_url": plos_pdf_url(doi),
                "doi": doi,
                "openalex_id": "",
                "source_item_id": doi,
                "paper_type": "Research Article",
                "venue_section": "PLOS API article_type: Research Article",
                "concepts": "",
                "referenced_works_count": "",
                "cited_by_count": "",
            }
        )

    titles_found, issue_pages_checked = enrich_plos_titles(rows)

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

    write_csv(inv_dir / "archive_pages.csv", archive_pages, ["page_index", "source_url", "result_count", "raw_num_found"])
    write_csv(inv_dir / "units.csv", sorted(units.values(), key=lambda r: (r["year"], r["unit_id"])), ["unit_id", "unit_type", "year", "date", "unit_label", "unit_url"])
    write_csv(inv_dir / "items.csv", rows, CSV_FIELDS)
    manifest = {
        "inventory_id": inventory_id,
        "source": "PLOS Medicine",
        "source_urls": ["https://api.plos.org/search", "https://journals.plos.org/plosmedicine/"],
        "parser_id": "plos_api_research_article_inventory",
        "parser_version": "1.0.0",
        "build_date": RUN_DATE,
        "coverage_years": f"{tuple_plan['year_min']}-{tuple_plan['year_max']}",
        "field_filter": {
            "source": "PLOS Medicine",
            "note": "Declared Field 9 source from journal_list_v1.md; PLOS API article_type=Research Article is the article-type filter.",
        },
        "oa_filter": "source_known_oa_gold",
        "user_agent": USER_AGENT,
        "rate_limit": "<=1 request/sec",
        "archive_pages": len(archive_pages),
        "items": len(rows),
        "units": len(units),
        "title_enrichment": {
            "source": "official PLOS issue pages",
            "issue_pages_checked": issue_pages_checked,
            "rows_with_titles": titles_found,
        },
        "known_gaps": [],
        "completeness_notes": (
            "Complete PLOS API pagination for the journal/date/article_type filter at build time. "
            "Rows are collapsed to one item per DOI because the API returns section-level records. "
            "Titles are enriched from official issue pages so title-based article-type exclusions "
            "can be applied before drawing."
        ),
    }
    (inv_dir / "inventory_manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return inv_dir


def parse_date_from_text(text: str) -> tuple[str, str]:
    match = re.search(r"(\d{1,2} [A-Z][a-z]+ 20\d{2})", text)
    if not match:
        return "", ""
    parsed = datetime.strptime(match.group(1), "%d %B %Y")
    return parsed.date().isoformat(), str(parsed.year)


def springer_archive_page_url(source: str, page: int) -> str:
    journal_id = {"BMC Medicine": "12916"}[source]
    return f"https://link.springer.com/journal/{journal_id}/articles?searchType=journalSearch&sort=PubDate&page={page}"


def build_bmc_inventory(tuple_plan: dict[str, Any]) -> Path:
    inventory_id = str(tuple_plan["source_inventory_id"])
    inv_dir = INVENTORIES / "bmc_medicine" / inventory_id
    if (
        (inv_dir / "inventory_manifest.json").exists()
        and (inv_dir / "items.csv").exists()
        and (inv_dir / "units.csv").exists()
        and csv_has_data_rows(inv_dir / "items.csv")
        and csv_has_data_rows(inv_dir / "units.csv")
    ):
        return inv_dir
    inv_dir.mkdir(parents=True, exist_ok=True)

    rows: list[dict[str, Any]] = []
    archive_pages: list[dict[str, Any]] = []
    seen_dois: set[str] = set()
    page = 1
    while True:
        url = springer_archive_page_url("BMC Medicine", page)
        data, final_url, content_type = request_bytes(url, "text/html,application/xhtml+xml,*/*", browser=True)
        text = data.decode("utf-8", errors="ignore")
        doc = lxml_html.fromstring(text)
        articles = doc.xpath('//article[contains(@class,"app-card-open")]')
        archive_pages.append(
            {
                "page_index": page,
                "source_url": final_url,
                "result_count": len(articles),
                "content_type": content_type,
            }
        )
        if not articles:
            break

        page_years: list[int] = []
        for article in articles:
            article_text = " ".join(article.text_content().split())
            unit_date, year = parse_date_from_text(article_text)
            if not year:
                continue
            year_int = int(year)
            page_years.append(year_int)
            if year_int > int(tuple_plan["year_max"]):
                continue
            if year_int < int(tuple_plan["year_min"]):
                continue
            title = " ".join(article.xpath('.//h2//text()')).strip()
            title = html.unescape(re.sub(r"\s+", " ", title))
            if not title or is_excluded_title(title):
                continue
            if " Research Open access " not in f" {article_text} ":
                continue
            hrefs = article.xpath('.//a[contains(@href,"/article/10.")]/@href')
            if not hrefs:
                continue
            landing = urllib.parse.urljoin(final_url, hrefs[0])
            doi_match = re.search(r"/article/(10\.[^?#]+)", landing)
            doi = normalize_doi(doi_match.group(1) if doi_match else "")
            if not doi or doi in seen_dois:
                continue
            seen_dois.add(doi)
            month = unit_date[:7] if unit_date else year
            unit_id = f"bmc_medicine_{month.replace('-', '_')}"
            rows.append(
                {
                    "source": "BMC Medicine",
                    "unit_id": unit_id,
                    "unit_type": "official_month_article_batch",
                    "year": year,
                    "unit_date": unit_date,
                    "unit_label": month,
                    "unit_url": url,
                    "landing_page_url": landing,
                    "source_url": final_url,
                    "title": title,
                    "summary": "",
                    "authors": "",
                    "oa_marker": "gold;source_known_oa;springer_research_open_access",
                    "pdf_url": f"https://link.springer.com/content/pdf/{doi}.pdf",
                    "doi": doi,
                    "openalex_id": "",
                    "source_item_id": doi,
                    "paper_type": "Research",
                    "venue_section": "Springer article card: Research Open access",
                    "concepts": "",
                    "referenced_works_count": "",
                    "cited_by_count": "",
                }
            )

        if page_years and min(page_years) < int(tuple_plan["year_min"]):
            break
        page += 1
        if page > 500:
            raise RuntimeError("BMC Medicine archive crawl exceeded 500 pages")
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

    write_csv(inv_dir / "archive_pages.csv", archive_pages, ["page_index", "source_url", "result_count", "content_type"])
    write_csv(inv_dir / "units.csv", sorted(units.values(), key=lambda r: (r["year"], r["unit_id"])), ["unit_id", "unit_type", "year", "date", "unit_label", "unit_url"])
    write_csv(inv_dir / "items.csv", rows, CSV_FIELDS)
    manifest = {
        "inventory_id": inventory_id,
        "source": "BMC Medicine",
        "source_urls": ["https://link.springer.com/journal/12916/articles"],
        "parser_id": "springer_journal_archive_research_open_access",
        "parser_version": "1.0.0",
        "build_date": RUN_DATE,
        "coverage_years": f"{tuple_plan['year_min']}-{tuple_plan['year_max']}",
        "field_filter": {
            "source": "BMC Medicine",
            "note": "Declared Field 9 source from journal_list_v1.md; publisher article cards marked Research Open access define the article-type frame.",
        },
        "oa_filter": "source_known_oa_gold",
        "user_agent": BROWSER_USER_AGENT,
        "rate_limit": "<=1 request/sec",
        "archive_pages": len(archive_pages),
        "items": len(rows),
        "units": len(units),
        "known_gaps": [],
        "completeness_notes": (
            "Publisher archive pages were crawled in reverse publication-date order until the "
            "completed-year lower bound was crossed; 2026/current-year rows were excluded."
        ),
    }
    (inv_dir / "inventory_manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return inv_dir


def ensure_inventories() -> dict[str, Path]:
    paths: dict[str, Path] = {}
    for tuple_plan in PLAN:
        inventory_id = str(tuple_plan["source_inventory_id"])
        if inventory_id in paths:
            continue
        if tuple_plan["source"] == "PLOS Medicine":
            paths[inventory_id] = build_plos_inventory(tuple_plan)
        elif tuple_plan["source"] == "BMC Medicine":
            paths[inventory_id] = build_bmc_inventory(tuple_plan)
        else:
            raise SystemExit(f"Unsupported source: {tuple_plan['source']}")
    return paths


def metadata_from_article_page(row: dict[str, str]) -> dict[str, Any]:
    landing = row.get("landing_page_url", "")
    data, final_url, _ = request_bytes(landing, "text/html,application/xhtml+xml,*/*", browser=True)
    doc = lxml_html.fromstring(data.decode("utf-8", errors="ignore"))
    title = doc.xpath('//meta[@name="citation_title"]/@content')
    authors = doc.xpath('//meta[@name="citation_author"]/@content')
    pdfs = doc.xpath('//meta[@name="citation_pdf_url"]/@content')
    dois = doc.xpath('//meta[@name="citation_doi"]/@content')
    pmids = doc.xpath('//meta[@name="citation_pmid"]/@content')
    pmcids = doc.xpath('//meta[@name="citation_pmcid"]/@content')
    publication_dates = doc.xpath('//meta[@name="citation_publication_date"]/@content')
    return {
        "landing_page_url": final_url,
        "title": html.unescape(title[0]).strip() if title else row.get("title", ""),
        "authors": author_string(authors) if authors else row.get("authors", ""),
        "pdf_url": pdfs[0].strip() if pdfs else row.get("pdf_url", ""),
        "doi": normalize_doi(dois[0] if dois else row.get("doi", "")),
        "pmid": pmids[0].strip() if pmids else "",
        "pmcid": pmcids[0].strip() if pmcids else "",
        "publication_date": publication_dates[0].strip() if publication_dates else "",
    }


def openalex_count_for_doi(doi: str) -> tuple[str, str]:
    if not doi:
        return "", ""
    url = "https://api.openalex.org/works/doi:" + urllib.parse.quote(f"https://doi.org/{doi}", safe="") + "?" + urllib.parse.urlencode(
        {"select": "id,cited_by_count"}
    )
    try:
        data = request_json(url)
    except Exception:
        return "", ""
    return str(data.get("cited_by_count") or ""), str(data.get("id") or "")


def looks_pdf_url(url: str) -> bool:
    lowered = url.lower()
    return any(token in lowered for token in [".pdf", "/pdf", "type=printable", "content/pdf", "article/file"])


def try_pdf_url(url: str, referer: str = "") -> tuple[bytes, str]:
    data, final_url, content_type = request_bytes(url, "application/pdf,text/html,*/*", browser=True, referer=referer)
    if data.lstrip().startswith(b"%PDF"):
        return data, final_url
    if "application/pdf" in content_type.lower() and len(data) > 1000:
        return data, final_url
    raise ValueError(f"not PDF bytes; content_type={content_type}; final_url={final_url}")


def publisher_page_pdf_candidates(landing_url: str) -> tuple[list[str], str]:
    if not landing_url:
        return [], "no landing page URL"
    try:
        data, final_url, content_type = request_bytes(landing_url, "text/html,application/xhtml+xml,*/*", browser=True)
    except Exception as exc:  # noqa: BLE001
        return [], repr(exc)
    if data.lstrip().startswith(b"%PDF"):
        return [final_url], "landing page returned PDF bytes"
    doc = lxml_html.fromstring(data.decode("utf-8", errors="ignore"))
    candidates = doc.xpath('//meta[@name="citation_pdf_url"]/@content')
    candidates += doc.xpath('//a[contains(translate(@href, "PDFDOWNLOAD", "pdfdownload"), "pdf")]/@href')
    candidates = [urllib.parse.urljoin(final_url, href) for href in candidates]
    return unique(candidates), f"content_type={content_type}; extracted_pdf_like_links={len(candidates)}"


def acquire_pdf(row: dict[str, str], metadata: dict[str, Any]) -> tuple[bytes, str, str, list[dict[str, str]]]:
    attempts: list[dict[str, str]] = []
    landing = str(metadata.get("landing_page_url") or row.get("landing_page_url") or "")
    direct_candidates = unique([
        str(metadata.get("pdf_url") or ""),
        row.get("pdf_url", ""),
    ])

    for url in direct_candidates:
        if not looks_pdf_url(url):
            continue
        try:
            pdf, final_url = try_pdf_url(url, landing)
            attempts.append({"route": "direct_pdf_route", "url": url, "result": "success", "final_url": final_url})
            return pdf, final_url, "direct_pdf_route", attempts
        except Exception as exc:  # noqa: BLE001
            attempts.append({"route": "direct_pdf_route", "url": url, "result": "failed", "error": repr(exc)})

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
            attempts.append({"route": "publisher_page_browser_fallback", "url": url, "result": "success", "final_url": final_url})
            return pdf, final_url, "publisher_page_browser_fallback", attempts
        except Exception as exc:  # noqa: BLE001
            attempts.append({"route": "publisher_page_browser_fallback", "url": url, "result": "failed", "error": repr(exc)})

    raise RuntimeError(json.dumps(attempts, ensure_ascii=False))


def page_count(pdf_bytes: bytes) -> int:
    return len(PdfReader(io.BytesIO(pdf_bytes)).pages)


def extract_text(pdf_bytes: bytes) -> str:
    def timeout_handler(signum, frame):  # noqa: ARG001
        raise TimeoutError("PDF text extraction timed out")

    previous = signal.signal(signal.SIGALRM, timeout_handler)
    signal.alarm(90)
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
    title = row.get("title", "")
    if title and is_excluded_title(title):
        return False
    paper_type = row.get("paper_type", "").strip().lower()
    if tuple_plan["source"] == "PLOS Medicine":
        return (
            paper_type == "research article"
            and bool(row.get("pdf_url"))
            and bool(row.get("title", "").strip())
            and not is_excluded_title(row.get("title", ""))
        )
    if tuple_plan["source"] == "BMC Medicine":
        return paper_type == "research" and bool(row.get("pdf_url"))
    return False


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
        pdf_path = ROOT / "pdfs" / FIELD_SLUG / f"{paper_id}.pdf"
        text_path = ROOT / "text" / FIELD_SLUG / f"{paper_id}.txt"
        p = 1.0 / len(unit_ids) / len(units[unit_id])

        record: dict[str, Any] = {
            "paper_id": paper_id,
            "field": FIELD,
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
            "pmid": "",
            "pmcid": "",
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
            selected.append(record)
            local_taken |= keys
            next_id += 1
            continue

        print(f"  trying {paper_id}: {source} | {row.get('year', '')} | {row.get('doi', '')}", flush=True)
        try:
            metadata = metadata_from_article_page(row)
            if metadata.get("title"):
                record["title"] = metadata["title"]
            if metadata.get("authors"):
                record["authors"] = metadata["authors"]
            if metadata.get("doi"):
                record["doi"] = metadata["doi"]
            if metadata.get("pmid"):
                record["pmid"] = metadata["pmid"]
            if metadata.get("pmcid"):
                record["pmcid"] = metadata["pmcid"]
            if metadata.get("landing_page_url"):
                record["landing_page_url"] = metadata["landing_page_url"]
            if metadata.get("pdf_url"):
                record["pdf_url"] = metadata["pdf_url"]
            cited_by_count, openalex_id = openalex_count_for_doi(str(record["doi"]))
            record["openalex_cited_by_count"] = cited_by_count
            record["openalex_id"] = openalex_id

            pdf_bytes, final_url, route, route_attempts = acquire_pdf(row, record)
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
                "title": record.get("title", ""),
                "doi": record.get("doi", ""),
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


def cleanup_outputs(selected: list[dict[str, Any]]) -> None:
    for record in selected:
        for key in ["pdf_path", "text_path"]:
            path = WORKSPACE / str(record.get(key, ""))
            if path.exists():
                path.unlink()


def source_plan_from_schedule_row(row: dict[str, str], needed: int) -> dict[str, Any]:
    era_band = row["era_band"]
    if "2005-2014" in era_band:
        year_min, year_max = 2005, 2014
    elif "2015-present" in era_band:
        year_min, year_max = 2015, 2025
    else:
        raise ValueError(f"Unknown era band for repair: {era_band}")
    return {
        "field": row["field"],
        "era_band": era_band,
        "year_min": year_min,
        "year_max": year_max,
        "venue_tier": row["venue_tier"],
        "source_block_order": row["source_block_order"],
        "source": row["source"],
        "source_inventory_id": row["source_inventory_id"],
        "needed": needed,
        "round_reason": (
            "Repair draw for protocol title exclusions discovered after PLOS title enrichment; "
            "replacement drawn from the same corrected source-block frame."
        ),
    }


def schedule_rows_for_papers(schedule: list[dict[str, str]], paper_ids: set[str]) -> dict[str, dict[str, str]]:
    out: dict[str, dict[str, str]] = {}
    for row in schedule:
        ids = [item for item in row.get("paper_ids", "").split(";") if item]
        for paper_id in ids:
            if paper_id in paper_ids:
                out[paper_id] = row
    return out


def title_exclusion_rows(papers: list[dict[str, str]]) -> list[dict[str, str]]:
    return [
        row
        for row in papers
        if row.get("field") == FIELD
        and row.get("sample_source") == "random_base"
        and is_excluded_title(row.get("paper_title", ""))
    ]


def remove_bad_rows_and_files(
    bad_rows: list[dict[str, str]],
    schedule: list[dict[str, str]],
) -> list[dict[str, str]]:
    bad_ids = {row["paper_id"] for row in bad_rows}
    for row in bad_rows:
        for key in ["pdf_path", "text_path"]:
            path = WORKSPACE / row.get(key, "")
            if path.exists():
                path.unlink()

    papers = [row for row in read_csv(PAPERS_CSV) if row.get("paper_id") not in bad_ids]
    acquisition_rows = [
        row for row in read_csv(ACQUISITION_LOG_CSV) if row.get("paper_id") not in bad_ids
    ]
    write_csv(PAPERS_CSV, papers, list(papers[0].keys()))
    write_csv(ACQUISITION_LOG_CSV, acquisition_rows, list(acquisition_rows[0].keys()))

    cleaned: list[dict[str, str]] = []
    for row in schedule:
        row = dict(row)
        ids = [item for item in row.get("paper_ids", "").split(";") if item and item not in bad_ids]
        if len(ids) != len([item for item in row.get("paper_ids", "").split(";") if item]):
            row["paper_ids"] = ";".join(ids)
            row["random_base_papers_in_block"] = str(len(ids))
            row["status"] = "completed_target_10" if len(ids) >= TARGET_PER_TUPLE else "partially_completed"
            row["design_note"] = (
                row.get("design_note", "")
                + f" Removed {len(bad_ids)} protocol-ineligible title-exclusion row(s) before repair append."
            ).strip()
        cleaned.append(row)
    return cleaned


def audit_dir() -> Path:
    return AUDIT_ROOT / ROUND_ID


def write_preflight(papers: list[dict[str, str]] | None = None) -> None:
    directory = audit_dir()
    directory.mkdir(parents=True, exist_ok=True)
    current_counts: Counter[str] = Counter()
    if papers is not None:
        current_counts.update(r.get("field", "") for r in papers if r.get("sample_source") == "random_base")
    write_csv(
        directory / "preflight_target_table.csv",
        [
            {
                "field": p["field"],
                "era_band": p["era_band"],
                "venue_tier": p["venue_tier"],
                "source": p["source"],
                "source_inventory_id": p["source_inventory_id"],
                "planned_additions": p["needed"],
                "current_random_base_field_count": current_counts.get(str(p["field"]), ""),
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
            "current_random_base_field_count",
            "reason",
        ],
    )
    preflight = {
        "round_id": ROUND_ID,
        "run_date": RUN_DATE,
        "status": "preflight_declared_before_registry_mutation",
        "requested_new_papers": REQUESTED_NEW_PAPERS,
        "target_per_tuple": TARGET_PER_TUPLE,
        "field_priority": FIELD,
        "selection_rationale": (
            "Clinical medicine has zero registry rows. This initial Field 9 batch opens "
            "declared journal_list_v1.md source blocks and fills three eligible source "
            "block tuples to the protocol working target of 10 papers each, bringing the "
            "field into the 25-30 paper range of the smallest existing fields without "
            "using PubMed/PMC/Crossref as replacement sampling frames."
        ),
        "plan": PLAN,
    }
    (directory / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n", encoding="utf-8")


def append_outputs(
    schedule: list[dict[str, str]],
    selected: list[dict[str, Any]],
    route_logs: list[dict[str, Any]],
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
                "pmid": record.get("pmid", ""),
                "pmcid": record.get("pmcid", ""),
                "url": record["landing_page_url"],
                "pdf_url": record["pdf_url"],
                "pdf_path": record["pdf_path"],
                "text_path": record["text_path"],
                "oa_status": record["oa_marker"],
                "download_status": "downloaded_verified_pdf",
                "openalex_cited_by_count": record["openalex_cited_by_count"],
                "source_checked_date": RUN_DATE,
                "notes": (
                    f"{ROUND_ID}; Field 9 initial Stratum A draw; source={record['source']}; "
                    f"inventory={record['inventory_id']}; unit={record['unit_id']}; "
                    f"route={record.get('acquisition_route', '')}; clinical source blocks "
                    "declared from journal_list_v1.md; PubMed/PMC/Crossref not used as sampling frame"
                ),
                "sample_source": "random_base",
                "selection_probability": f"{record['selection_probability']:.15g}",
                "design_weight": f"{record['design_weight']:.15g}",
            }
        )
        acquisition_rows.append(
            {
                "paper_id": record["paper_id"],
                "source_type": "publisher_pdf",
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
            + f" Updated by {ROUND_ID}; added {len(ids)} clinical medicine papers; "
            "direct PDF routes succeeded or publisher-page fallback was attempted before any alternate route."
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

    directory = audit_dir()
    directory.mkdir(parents=True, exist_ok=True)
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
        "rate_limit": "<=1 request/sec for inventory and acquisition requests",
        "selection_rationale": (
            "Initial clinical medicine batch. Field 9 had zero corpus rows, so the batch "
            "prioritized declared clinical source blocks from journal_list_v1.md while "
            "capping each eligible tuple at the protocol working target of 10."
        ),
        "plan": PLAN,
        "block_counts_added": [
            {"field": k[0], "era_band": k[1], "source": k[2], "added": v}
            for k, v in sorted(block_counts.items())
        ],
        "selected": public_records,
        "route_logs": route_logs,
    }
    (directory / "draw_audit.json").write_text(json.dumps(audit, indent=2) + "\n", encoding="utf-8")

    selection_fieldnames = [
        "paper_id",
        "field",
        "source",
        "inventory_id",
        "era_band",
        "venue_tier",
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
        directory / "selection_table.csv",
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


def run_title_exclusion_repair(dry_run: bool) -> None:
    global PLAN, REQUESTED_NEW_PAPERS, ROUND_ID

    original_round_id = ROUND_ID
    original_requested = REQUESTED_NEW_PAPERS
    original_plan = PLAN

    papers = read_csv(PAPERS_CSV)
    schedule = read_csv(SCHEDULE_CSV)
    bad_rows = title_exclusion_rows(papers)
    if not bad_rows:
        print("No clinical title-exclusion rows found.", flush=True)
        return
    bad_ids = {row["paper_id"] for row in bad_rows}
    schedule_for_bad = schedule_rows_for_papers(schedule, bad_ids)
    missing = bad_ids - set(schedule_for_bad)
    if missing:
        raise SystemExit(f"Could not map bad paper IDs to source schedule rows: {sorted(missing)}")

    grouped: dict[tuple[str, str, str, str], dict[str, Any]] = {}
    for paper_id, schedule_row in schedule_for_bad.items():
        key = (
            schedule_row["field"],
            schedule_row["era_band"],
            schedule_row["venue_tier"],
            schedule_row["source"],
        )
        grouped.setdefault(
            key,
            {
                "row": schedule_row,
                "paper_ids": [],
            },
        )["paper_ids"].append(paper_id)

    repair_plan = [
        source_plan_from_schedule_row(group["row"], len(group["paper_ids"]))
        for group in grouped.values()
    ]
    repair_plan.sort(key=lambda p: (p["era_band"], p["source"], p["source_block_order"]))

    ROUND_ID = "round_20260621_clinical_medicine_title_exclusion_repair_01"
    REQUESTED_NEW_PAPERS = sum(int(p["needed"]) for p in repair_plan)
    PLAN = repair_plan
    try:
        write_preflight(papers)
        directory = audit_dir()
        write_csv(
            directory / "title_exclusion_removals.csv",
            [
                {
                    "paper_id": row["paper_id"],
                    "year": row["year"],
                    "journal": row["journal"],
                    "doi": row["doi"],
                    "paper_title": row["paper_title"],
                    "reason": "default Field 9 exclusion: protocols/guidelines/reviews/meta-analyses are not in the random-base draw",
                }
                for row in bad_rows
            ],
            ["paper_id", "year", "journal", "doi", "paper_title", "reason"],
        )

        inventories = ensure_inventories()
        taken = known_keys(papers)
        next_id = max_paper_number(papers) + 1
        selected_all: list[dict[str, Any]] = []
        route_logs: list[dict[str, Any]] = []
        try:
            for idx, tuple_plan in enumerate(PLAN, start=1):
                inv_dir = inventories[str(tuple_plan["source_inventory_id"])]
                print(
                    f"\n== repair | {tuple_plan['field']} | {tuple_plan['era_band']} | "
                    f"{tuple_plan['source']} | needed={tuple_plan['needed']} ==",
                    flush=True,
                )
                selected, next_id, logs = draw_for_tuple(
                    tuple_plan, inv_dir, papers, taken, next_id, idx, dry_run
                )
                selected_all.extend(selected)
                route_logs.extend(logs)
                print(f"{tuple_plan['source']} repair_selected={len(selected)}", flush=True)
        except Exception:
            cleanup_outputs(selected_all)
            raise

        print(f"repair_total_selected={len(selected_all)}", flush=True)
        if dry_run:
            return
        cleaned_schedule = remove_bad_rows_and_files(bad_rows, schedule)
        append_outputs(cleaned_schedule, selected_all, route_logs)
    finally:
        ROUND_ID = original_round_id
        REQUESTED_NEW_PAPERS = original_requested
        PLAN = original_plan


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--preflight-only", action="store_true")
    parser.add_argument("--repair-title-exclusions", action="store_true")
    args = parser.parse_args()

    if args.repair_title_exclusions:
        run_title_exclusion_repair(args.dry_run)
        return

    requested = sum(int(p["needed"]) for p in PLAN)
    if requested != REQUESTED_NEW_PAPERS:
        raise SystemExit(f"Plan requests {requested}, expected {REQUESTED_NEW_PAPERS}")

    papers = read_csv(PAPERS_CSV)
    write_preflight(papers)
    print("preflight target table", flush=True)
    for p in PLAN:
        print(
            f"  {p['needed']:>2} | {p['field']} | {p['era_band']} | {p['venue_tier']} | "
            f"{p['source']} | {p['round_reason']}",
            flush=True,
        )
    if args.preflight_only:
        print(f"wrote {audit_dir() / 'preflight_target_table.csv'}", flush=True)
        return

    inventories = ensure_inventories()
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
    append_outputs(schedule, selected_all, route_logs)


if __name__ == "__main__":
    main()
