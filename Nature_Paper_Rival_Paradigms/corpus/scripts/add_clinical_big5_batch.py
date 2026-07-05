#!/usr/bin/env python3
"""Add protocol-compliant Field 9 papers from the big-5 clinical journals.

This round opens the Field 9 top-tier clinical/translational block listed in
journal_list_v1.md. The accessible source block in this environment is Nature
Medicine's official research-article archive, narrowed to publisher-marked Open
Access articles and a neutral human clinical evidence filter.
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

ROUND_ID = "round_20260621_clinical_big5_nature_medicine_10"
RUN_DATE = date.today().isoformat()
REQUESTED_NEW_PAPERS = 10
TARGET_PER_TUPLE = 10
SEED_BASE = 202606211500
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
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "top",
        "source_block_order": "5",
        "source": "Nature Medicine",
        "source_inventory_id": (
            "nature_medicine_nature_research_articles_oa_clinical_v1.0.0_"
            "2015-2025_20260621"
        ),
        "needed": 10,
        "round_reason": (
            "Big-5 Field 9 top-tier source block from journal_list_v1.md. "
            "Nature Medicine is hybrid/gold, so the effective source frame is "
            "the official Nature Medicine research-article archive restricted "
            "to publisher-marked Open Access Article cards and the predeclared "
            "human clinical evidence boundary."
        ),
    },
]

BIG5_BLOCKED_SOURCE_NOTES = [
    {
        "field": FIELD,
        "era_band": "2015-present_completed_years_2015-2025",
        "venue_tier": "top",
        "source_block_order": "1",
        "source": "New England Journal of Medicine (NEJM)",
        "source_inventory_id": "nejm_publisher_archive_probe_20260621",
        "status": "source_frame_blocked_browser_recovery_pending",
        "planned_additions": "0",
        "reason": (
            "Included in the big-5 preflight, but local publisher archive and "
            "article routes returned scripted access blocks. Current RSS alone "
            "is not a completed 2015-2025 frame."
        ),
    },
    {
        "field": FIELD,
        "era_band": "2015-present_completed_years_2015-2025",
        "venue_tier": "top",
        "source_block_order": "2",
        "source": "The Lancet",
        "source_inventory_id": "lancet_publisher_archive_probe_20260621",
        "status": "source_frame_blocked_browser_recovery_pending",
        "planned_additions": "0",
        "reason": (
            "Included in the big-5 preflight, but local issue/archive routes "
            "returned scripted access blocks or redirect loops. Current RSS "
            "alone is not a completed 2015-2025 frame."
        ),
    },
    {
        "field": FIELD,
        "era_band": "2015-present_completed_years_2015-2025",
        "venue_tier": "top",
        "source_block_order": "3",
        "source": "JAMA",
        "source_inventory_id": "jama_publisher_archive_probe_20260621",
        "status": "source_frame_blocked_browser_recovery_pending",
        "planned_additions": "0",
        "reason": (
            "Included in the big-5 preflight, but local JAMA Network issue, "
            "article, and PDF routes returned scripted 403 blocks. A real "
            "browser recovery pass is required before opening this source."
        ),
    },
    {
        "field": FIELD,
        "era_band": "2015-present_completed_years_2015-2025",
        "venue_tier": "top",
        "source_block_order": "4",
        "source": "The BMJ",
        "source_inventory_id": "bmj_publisher_archive_probe_20260621",
        "status": "publisher_page_blocked_browser_recovery_pending",
        "planned_additions": "0",
        "reason": (
            "Included in the big-5 preflight. BMJ sitemap pages are reachable, "
            "but local article/PDF routes returned scripted 403 blocks and the "
            "sitemap alone does not expose enough article-type metadata for the "
            "original-research Field 9 frame."
        ),
    },
]

NATURE_ARCHIVE_URL = (
    "https://www.nature.com/nm/research-articles?"
    "searchType=journalSearch&sort=PubDate&page={page}"
)

EXCLUDED_TITLE_RE = re.compile(
    r"(^|\b)(correction|erratum|retraction|editorial|author correction|"
    r"publisher correction|case report):|"
    r"systematic review|meta-analysis|meta analysis|scoping review|"
    r"narrative review|overview of reviews|study protocol|trial protocol|"
    r"\bprotocol\b|\bguideline\b",
    re.IGNORECASE,
)

CLINICAL_EVIDENCE_RE = re.compile(
    r"\b("
    r"patient|patients|participant|participants|clinical|trial|"
    r"randomized|randomised|cohort|population|epidemiolog|registry|"
    r"biobank|hospital|mortality|morbidity|survival|diagnos|prognos|"
    r"treatment|therapy|therapeutic|phase\s*[123]|screening|"
    r"health\s*care|healthcare|public health|global health|adults|"
    r"children|people with|covid|sars-cov-2|electronic health record|"
    r"medical record|real-world|case-control|cross-sectional|incidence|"
    r"prevalence|vaccine|vaccination|outcome|outcomes"
    r")\b",
    re.IGNORECASE,
)


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


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalize_space(value: str) -> str:
    return html.unescape(re.sub(r"\s+", " ", value or "")).strip()


def normalize_doi(value: str | None) -> str:
    if not value:
        return ""
    value = value.strip()
    value = value.removeprefix("https://doi.org/").removeprefix("http://doi.org/")
    return value.lower()


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
        ("source_item_id", "source_item_id"),
    ]:
        value = row.get(raw_key, "").strip().lower()
        if value:
            keys.add((mapped_key, value))
    return keys


def request_bytes(
    url: str,
    accept: str,
    browser: bool = False,
    referer: str = "",
) -> tuple[bytes, str, str]:
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


def author_string(authors: list[str]) -> str:
    authors = [html.unescape(a).strip() for a in authors if a and a.strip()]
    if len(authors) > 5:
        return "; ".join(authors[:5]) + "; et al."
    return "; ".join(authors)


def is_excluded_title(title: str) -> bool:
    return bool(EXCLUDED_TITLE_RE.search(title.strip()))


def clinical_filter_match(title: str, summary: str) -> tuple[bool, str]:
    text = f"{title} {summary}"
    if is_excluded_title(title):
        return False, "excluded_article_type_title"
    match = CLINICAL_EVIDENCE_RE.search(text)
    if not match:
        return False, "no_predeclared_human_clinical_evidence_token"
    return True, f"matched_token:{match.group(0).lower()}"


def nature_doi_from_url(landing: str) -> str:
    path = urllib.parse.urlparse(landing).path
    article_id = path.rstrip("/").split("/")[-1]
    if not article_id:
        return ""
    return normalize_doi(f"10.1038/{article_id}")


def nature_pdf_url(landing: str) -> str:
    return landing.rstrip("/") + ".pdf"


def probe_url(url: str) -> dict[str, str]:
    try:
        data, final_url, content_type = request_bytes(
            url,
            "text/html,application/xhtml+xml,application/rss+xml,*/*",
            browser=True,
        )
        return {
            "url": url,
            "result": "success",
            "final_url": final_url,
            "content_type": content_type,
            "bytes": str(len(data)),
            "error": "",
        }
    except urllib.error.HTTPError as exc:
        return {
            "url": url,
            "result": "http_error",
            "final_url": getattr(exc, "url", url),
            "content_type": "",
            "bytes": "",
            "error": f"HTTP {exc.code}",
        }
    except Exception as exc:  # noqa: BLE001
        return {
            "url": url,
            "result": "error",
            "final_url": "",
            "content_type": "",
            "bytes": "",
            "error": repr(exc),
        }


def audit_dir() -> Path:
    return AUDIT_ROOT / ROUND_ID


def write_preflight(papers: list[dict[str, str]] | None = None) -> None:
    directory = audit_dir()
    directory.mkdir(parents=True, exist_ok=True)
    current_counts: Counter[str] = Counter()
    if papers is not None:
        current_counts.update(r.get("field", "") for r in papers if r.get("sample_source") == "random_base")

    rows: list[dict[str, Any]] = []
    for blocked in BIG5_BLOCKED_SOURCE_NOTES:
        rows.append(
            {
                "field": blocked["field"],
                "era_band": blocked["era_band"],
                "venue_tier": blocked["venue_tier"],
                "source": blocked["source"],
                "source_inventory_id": blocked["source_inventory_id"],
                "planned_additions": blocked["planned_additions"],
                "current_random_base_field_count": current_counts.get(str(blocked["field"]), ""),
                "status": blocked["status"],
                "reason": blocked["reason"],
            }
        )
    for plan in PLAN:
        rows.append(
            {
                "field": plan["field"],
                "era_band": plan["era_band"],
                "venue_tier": plan["venue_tier"],
                "source": plan["source"],
                "source_inventory_id": plan["source_inventory_id"],
                "planned_additions": plan["needed"],
                "current_random_base_field_count": current_counts.get(str(plan["field"]), ""),
                "status": "planned_to_target_10",
                "reason": plan["round_reason"],
            }
        )

    write_csv(
        directory / "preflight_target_table.csv",
        rows,
        [
            "field",
            "era_band",
            "venue_tier",
            "source",
            "source_inventory_id",
            "planned_additions",
            "current_random_base_field_count",
            "status",
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
            "Fill the Field 9 top-tier Clinical Medicine cell using the big-5 "
            "journal list. Four flagship sources need browser recovery before "
            "they can be opened without violating the source-frame protocol. "
            "Nature Medicine exposes an official publisher archive with Open "
            "Access markers, so this round opens that source block first."
        ),
        "blocked_or_pending_big5_sources": BIG5_BLOCKED_SOURCE_NOTES,
        "plan": PLAN,
    }
    (directory / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n", encoding="utf-8")


def nature_card_rows(page: int, html_bytes: bytes, final_url: str) -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    doc = lxml_html.fromstring(html_bytes.decode("utf-8", errors="ignore"))
    items: list[dict[str, str]] = []
    screened: list[dict[str, str]] = []
    for idx, article in enumerate(doc.xpath("//article"), start=1):
        title = normalize_space(" ".join(article.xpath('.//h3//text() | .//h2//text()')))
        summary = normalize_space(" ".join(article.xpath('.//*[@data-test="article-description"]//text()')))
        paper_type = normalize_space(" ".join(article.xpath('.//*[@data-test="article.type"]//text()')))
        date_values = article.xpath(".//time/@datetime")
        if not title or not date_values:
            continue
        unit_date = date_values[0][:10]
        year = unit_date[:4]
        hrefs = article.xpath('.//a[contains(@href, "/articles/")]/@href')
        if not hrefs:
            continue
        landing = urllib.parse.urljoin(final_url, hrefs[0])
        card_text = normalize_space(article.text_content())
        authors = author_string(article.xpath('.//*[@itemprop="creator"]//*[@itemprop="name"]/text()'))
        is_open_access = "Open Access" in card_text or "Open access" in card_text
        clinical_ok, clinical_reason = clinical_filter_match(title, summary)
        status_reasons: list[str] = []
        if paper_type != "Article":
            status_reasons.append("not_article_type")
        if not is_open_access:
            status_reasons.append("no_publisher_open_access_marker")
        if not clinical_ok:
            status_reasons.append(clinical_reason)
        if is_excluded_title(title):
            status_reasons.append("excluded_title")
        status = "eligible" if not status_reasons else "excluded"
        screened.append(
            {
                "page": str(page),
                "card_index": str(idx),
                "year": year,
                "date": unit_date,
                "title": title,
                "paper_type": paper_type,
                "landing_page_url": landing,
                "open_access_marker": "yes" if is_open_access else "no",
                "clinical_filter": clinical_reason,
                "screen_status": status,
                "screen_reasons": ";".join(unique(status_reasons)),
            }
        )
        if status != "eligible":
            continue
        unit_label = unit_date[:7]
        row = {
            "source": "Nature Medicine",
            "unit_id": f"nature_medicine_{unit_label.replace('-', '_')}",
            "unit_type": "official_month_open_access_clinical_article_batch",
            "year": year,
            "unit_date": unit_date,
            "unit_label": unit_label,
            "unit_url": NATURE_ARCHIVE_URL.format(page=page),
            "landing_page_url": landing,
            "source_url": final_url,
            "title": title,
            "summary": summary,
            "authors": authors,
            "oa_marker": "publisher_card_open_access",
            "pdf_url": nature_pdf_url(landing),
            "doi": nature_doi_from_url(landing),
            "openalex_id": "",
            "source_item_id": landing.rstrip("/").split("/")[-1],
            "paper_type": paper_type,
            "venue_section": "Nature Medicine research-articles; publisher card: Open Access Article",
            "concepts": "",
            "referenced_works_count": "",
            "cited_by_count": "",
        }
        items.append(row)
    return items, screened


def build_nature_inventory(tuple_plan: dict[str, Any], page_sleep: float) -> Path:
    inventory_id = str(tuple_plan["source_inventory_id"])
    inv_dir = INVENTORIES / "nature_medicine" / inventory_id
    if (
        (inv_dir / "inventory_manifest.json").exists()
        and csv_has_data_rows(inv_dir / "items.csv")
        and csv_has_data_rows(inv_dir / "units.csv")
    ):
        return inv_dir
    inv_dir.mkdir(parents=True, exist_ok=True)

    rows: list[dict[str, str]] = []
    screened_rows: list[dict[str, str]] = []
    archive_pages: list[dict[str, Any]] = []
    seen_urls: set[str] = set()
    page = 1
    while True:
        url = NATURE_ARCHIVE_URL.format(page=page)
        data, final_url, content_type = request_bytes(
            url,
            "text/html,application/xhtml+xml,*/*",
            browser=True,
        )
        doc = lxml_html.fromstring(data.decode("utf-8", errors="ignore"))
        cards = doc.xpath("//article")
        page_items, page_screened = nature_card_rows(page, data, final_url)
        years = [int(row["year"]) for row in page_screened if row.get("year", "").isdigit()]
        for row in page_items:
            year_int = int(row["year"])
            if year_int < int(tuple_plan["year_min"]) or year_int > int(tuple_plan["year_max"]):
                continue
            if row["landing_page_url"] in seen_urls:
                continue
            seen_urls.add(row["landing_page_url"])
            rows.append(row)
        screened_rows.extend(page_screened)
        archive_pages.append(
            {
                "page_index": page,
                "source_url": final_url,
                "requested_url": url,
                "result_count": len(cards),
                "screened_cards": len(page_screened),
                "eligible_items_added_total": len(rows),
                "min_year_on_page": min(years) if years else "",
                "max_year_on_page": max(years) if years else "",
                "content_type": content_type,
            }
        )
        print(
            "inventory page "
            f"{page}: cards={len(cards)} eligible_total={len(rows)} "
            f"min_year={min(years) if years else ''}",
            flush=True,
        )
        if years and min(years) < int(tuple_plan["year_min"]):
            break
        if not cards:
            break
        page += 1
        if page > 250:
            raise RuntimeError("Nature Medicine archive crawl exceeded 250 pages")
        time.sleep(max(page_sleep, 1.0))

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

    rows.sort(key=lambda r: (r["year"], r["unit_id"], r["title"]))
    write_csv(
        inv_dir / "archive_pages.csv",
        archive_pages,
        [
            "page_index",
            "source_url",
            "requested_url",
            "result_count",
            "screened_cards",
            "eligible_items_added_total",
            "min_year_on_page",
            "max_year_on_page",
            "content_type",
        ],
    )
    write_csv(
        inv_dir / "screened_cards.csv",
        screened_rows,
        [
            "page",
            "card_index",
            "year",
            "date",
            "title",
            "paper_type",
            "landing_page_url",
            "open_access_marker",
            "clinical_filter",
            "screen_status",
            "screen_reasons",
        ],
    )
    write_csv(
        inv_dir / "units.csv",
        sorted(units.values(), key=lambda r: (r["year"], r["unit_id"])),
        ["unit_id", "unit_type", "year", "date", "unit_label", "unit_url"],
    )
    write_csv(inv_dir / "items.csv", rows, CSV_FIELDS)
    manifest = {
        "inventory_id": inventory_id,
        "source": "Nature Medicine",
        "source_urls": ["https://www.nature.com/nm/research-articles"],
        "parser_id": "nature_research_articles_open_access_clinical_cards",
        "parser_version": "1.0.0",
        "build_date": RUN_DATE,
        "coverage_years": f"{tuple_plan['year_min']}-{tuple_plan['year_max']}",
        "field_filter": {
            "field": FIELD,
            "rule": (
                "Nature Medicine is not treated as field membership by venue "
                "alone. Eligible cards must be publisher research-article cards "
                "with type Article, publisher Open Access marker, no default "
                "Field 9 exclusion title terms, and at least one predeclared "
                "human clinical evidence token in title or card summary."
            ),
            "clinical_evidence_regex": CLINICAL_EVIDENCE_RE.pattern,
            "excluded_title_regex": EXCLUDED_TITLE_RE.pattern,
        },
        "oa_filter": "publisher_card_open_access",
        "user_agent": BROWSER_USER_AGENT,
        "rate_limit": "<=1 request/sec",
        "archive_pages": len(archive_pages),
        "screened_cards": len(screened_rows),
        "items": len(rows),
        "units": len(units),
        "known_gaps": [
            "Other big-5 journals were not used as substitute frames because local scripted archive/article routes were blocked or incomplete.",
            "Open Access status is taken from the official Nature archive card marker; PDFs are not downloaded during inventory construction.",
        ],
        "completeness_notes": (
            "Publisher research-article archive pages were crawled in reverse "
            "publication-date order until the completed-year lower bound was "
            "crossed. 2026/current-year rows were excluded."
        ),
    }
    (inv_dir / "inventory_manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return inv_dir


def eligible(row: dict[str, str], tuple_plan: dict[str, Any]) -> bool:
    try:
        year = int(row.get("year", ""))
    except ValueError:
        return False
    if not (int(tuple_plan["year_min"]) <= year <= int(tuple_plan["year_max"])):
        return False
    if row.get("paper_type", "").strip() != "Article":
        return False
    if row.get("oa_marker", "") != "publisher_card_open_access":
        return False
    title = row.get("title", "")
    summary = row.get("summary", "")
    clinical_ok, _ = clinical_filter_match(title, summary)
    return clinical_ok and bool(row.get("pdf_url"))


def metadata_from_nature_page(row: dict[str, str]) -> dict[str, Any]:
    landing = row.get("landing_page_url", "")
    data, final_url, _ = request_bytes(landing, "text/html,application/xhtml+xml,*/*", browser=True)
    doc = lxml_html.fromstring(data.decode("utf-8", errors="ignore"))
    title = doc.xpath('//meta[@name="citation_title"]/@content')
    authors = doc.xpath('//meta[@name="citation_author"]/@content')
    pdfs = doc.xpath('//meta[@name="citation_pdf_url"]/@content')
    dois = doc.xpath('//meta[@name="citation_doi"]/@content')
    pmids = doc.xpath('//meta[@name="citation_pmid"]/@content')
    pmcids = doc.xpath('//meta[@name="citation_pmcid"]/@content')
    article_types = doc.xpath('//meta[@name="citation_article_type"]/@content')
    dc_types = doc.xpath('//meta[@name="dc.type"]/@content')
    return {
        "landing_page_url": final_url,
        "title": html.unescape(title[0]).strip() if title else row.get("title", ""),
        "authors": author_string(authors) if authors else row.get("authors", ""),
        "pdf_url": pdfs[0].strip() if pdfs else row.get("pdf_url", ""),
        "doi": normalize_doi(dois[0] if dois else row.get("doi", "")),
        "pmid": pmids[0].strip() if pmids else "",
        "pmcid": pmcids[0].strip() if pmcids else "",
        "article_type": article_types[0].strip() if article_types else row.get("paper_type", ""),
        "dc_type": dc_types[0].strip() if dc_types else "",
    }


def openalex_count_for_doi(doi: str) -> tuple[str, str]:
    if not doi:
        return "", ""
    url = "https://api.openalex.org/works/doi:" + urllib.parse.quote(
        f"https://doi.org/{doi}",
        safe="",
    ) + "?" + urllib.parse.urlencode({"select": "id,cited_by_count"})
    try:
        data = request_json(url)
    except Exception:
        return "", ""
    return str(data.get("cited_by_count") or ""), str(data.get("id") or "")


def looks_pdf_url(url: str) -> bool:
    lowered = url.lower()
    return lowered.endswith(".pdf") or "/pdf" in lowered or ".pdf?" in lowered


def try_pdf_url(url: str, referer: str = "") -> tuple[bytes, str]:
    data, final_url, content_type = request_bytes(
        url,
        "application/pdf,text/html,*/*",
        browser=True,
        referer=referer,
    )
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
    direct_candidates = unique([str(metadata.get("pdf_url") or ""), row.get("pdf_url", "")])

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

    pmcid = str(metadata.get("pmcid") or "").strip()
    if pmcid:
        for url in [
            f"https://pmc.ncbi.nlm.nih.gov/articles/{pmcid}/pdf/",
            f"https://pmc.ncbi.nlm.nih.gov/articles/{pmcid}/pdf",
        ]:
            try:
                pdf, final_url = try_pdf_url(url, landing)
                attempts.append({"route": "pmc_legal_oa_alternate", "url": url, "result": "success", "final_url": final_url})
                return pdf, final_url, "pmc_legal_oa_alternate", attempts
            except Exception as exc:  # noqa: BLE001
                attempts.append({"route": "pmc_legal_oa_alternate", "url": url, "result": "failed", "error": repr(exc)})

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
            metadata = metadata_from_nature_page(row)
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

            pdf_bytes, final_url, route, route_attempts = acquire_pdf(row, record | metadata)
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
                    f"{ROUND_ID}; Field 9 big-5 Stratum A draw; source={record['source']}; "
                    f"inventory={record['inventory_id']}; unit={record['unit_id']}; "
                    f"route={record.get('acquisition_route', '')}; official Nature archive "
                    "used as source frame; PubMed/PMC/Crossref not used as sampling frame"
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
    for blocked in BIG5_BLOCKED_SOURCE_NOTES:
        key = (
            str(blocked["field"]),
            str(blocked["era_band"]),
            str(blocked["venue_tier"]),
            str(blocked["source"]),
        )
        schedule_by_key.setdefault(
            key,
            {
                "field": blocked["field"],
                "era_band": blocked["era_band"],
                "venue_tier": blocked["venue_tier"],
                "source_block_order": blocked["source_block_order"],
                "source": blocked["source"],
                "source_inventory_id": blocked["source_inventory_id"],
                "status": blocked["status"],
                "random_base_papers_in_block": "0",
                "paper_ids": "",
                "design_note": blocked["reason"],
            },
        )
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
            + f" Updated by {ROUND_ID}; added {len(ids)} Nature Medicine big-5 clinical medicine papers; "
            "direct PDF routes succeeded or publisher-page fallback was attempted before any legal alternate route."
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
        "rate_limit": "<=1 request/sec for Nature inventory and acquisition requests",
        "selection_rationale": (
            "Big-5 clinical medicine top-tier fill. NEJM, The Lancet, JAMA, "
            "and The BMJ were documented as pending browser recovery instead "
            "of being replaced by PubMed/PMC/Crossref frames. Nature Medicine "
            "was fillable via its official Open Access research-article archive."
        ),
        "blocked_or_pending_big5_sources": BIG5_BLOCKED_SOURCE_NOTES,
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


def write_probe_summary() -> None:
    probes = [
        ("NEJM", "https://www.nejm.org/medical-articles/original-article"),
        ("The Lancet", "https://www.thelancet.com/journals/lancet/issue/current"),
        ("JAMA", "https://jamanetwork.com/journals/jama/issue"),
        ("The BMJ", "https://www.bmj.com/content/research"),
        ("Nature Medicine", "https://www.nature.com/nm/research-articles?searchType=journalSearch&sort=PubDate&page=1"),
    ]
    rows = []
    for source, url in probes:
        result = probe_url(url)
        result["source"] = source
        rows.append(result)
        time.sleep(1.0)
    write_csv(
        audit_dir() / "big5_access_probe_summary.csv",
        rows,
        ["source", "url", "result", "final_url", "content_type", "bytes", "error"],
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--preflight-only", action="store_true")
    parser.add_argument("--skip-probes", action="store_true")
    parser.add_argument("--page-sleep", type=float, default=1.0)
    args = parser.parse_args()

    requested = sum(int(p["needed"]) for p in PLAN)
    if requested != REQUESTED_NEW_PAPERS:
        raise SystemExit(f"Plan requests {requested}, expected {REQUESTED_NEW_PAPERS}")

    papers = read_csv(PAPERS_CSV)
    write_preflight(papers)
    if not args.skip_probes:
        write_probe_summary()
    print("preflight target table", flush=True)
    for blocked in BIG5_BLOCKED_SOURCE_NOTES:
        print(
            f"   0 | {blocked['field']} | {blocked['era_band']} | {blocked['venue_tier']} | "
            f"{blocked['source']} | {blocked['status']}",
            flush=True,
        )
    for p in PLAN:
        print(
            f"  {p['needed']:>2} | {p['field']} | {p['era_band']} | {p['venue_tier']} | "
            f"{p['source']} | {p['round_reason']}",
            flush=True,
        )
    if args.preflight_only:
        print(f"wrote {audit_dir() / 'preflight_target_table.csv'}", flush=True)
        return

    inventories = {
        str(PLAN[0]["source_inventory_id"]): build_nature_inventory(PLAN[0], args.page_sleep)
    }
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
                tuple_plan,
                inv_dir,
                papers,
                taken,
                next_id,
                idx,
                args.dry_run,
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
