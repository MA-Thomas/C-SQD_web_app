#!/usr/bin/env python3
"""Add a protocol-aligned balancing batch after the clinical big-5 pass.

This batch targets underfilled fields and the mid/top + historical imbalances
without extending already-overrepresented specialist blocks. Source frames are
publisher/official archives: PLOS API for PLOS ONE, RSC issue endpoints for
Chemical Science, and A&A issue pages for Astronomy & Astrophysics.
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

from lxml import html as lxml_html
from pypdf import PdfReader


ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT.parent
PAPERS_CSV = ROOT / "metadata" / "papers.csv"
SCHEDULE_CSV = ROOT / "sources" / "source_block_schedule_v0.csv"
ACQUISITION_LOG_CSV = ROOT / "sources" / "acquisition_log.csv"
INVENTORIES = ROOT / "source_inventories"
AUDIT_ROOT = ROOT / "sources" / "draw_audits"

ROUND_ID = "round_20260621_balanced_protocol_top_mid_historical_70"
RUN_DATE = date.today().isoformat()
REQUESTED_NEW_PAPERS = 70
TARGET_PER_TUPLE = 10
SEED_BASE = 202606211800
USER_AGENT = "TextDataMining-CorpusExpansion/0.1 (research corpus; local run; <=1 request/sec)"
BROWSER_USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
)

FIELD_SLUG = {
    "Cognitive science / psychology": "cognitive_science_psychology",
    "Mechanistic molecular / cell / developmental biology": "molecular_cell_biology",
    "Statistically-oriented physics (precision cosmology, exclusion-limit searches, ML-for-physics)": "statistical_physics",
    "Structure-driven condensed matter / chemistry": "condensed_matter_physics",
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

PLAN: list[dict[str, Any]] = [
    {
        "field": "Cognitive science / psychology",
        "era_band": "2005-2014_completed_years_2006-2014",
        "year_min": 2006,
        "year_max": 2014,
        "venue_tier": "mid",
        "source_block_order": "4",
        "source": "PLOS ONE",
        "source_inventory_id": "plos_one_plos_api_cognitive_psychology_v1.0.0_2006-2014_20260621",
        "needed": 10,
        "builder": "plos_one",
        "plos_subject": "Cognitive psychology",
        "round_reason": (
            "Underfilled Field 8; open a general born-OA mid source block using "
            "the official PLOS API and PLOS subject taxonomy as the neutral "
            "Cognitive psychology field filter."
        ),
    },
    {
        "field": "Cognitive science / psychology",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "mid",
        "source_block_order": "5",
        "source": "PLOS ONE",
        "source_inventory_id": "plos_one_plos_api_cognitive_psychology_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "builder": "plos_one",
        "plos_subject": "Cognitive psychology",
        "round_reason": (
            "Underfilled Field 8; add a second mid-tier source block from the "
            "same official PLOS ONE frame for the modern completed years."
        ),
    },
    {
        "field": "Mechanistic molecular / cell / developmental biology",
        "era_band": "2005-2014_completed_years_2006-2014",
        "year_min": 2006,
        "year_max": 2014,
        "venue_tier": "mid",
        "source_block_order": "3",
        "source": "PLOS ONE",
        "source_inventory_id": "plos_one_plos_api_developmental_biology_v1.0.0_2006-2014_20260621",
        "needed": 10,
        "builder": "plos_one",
        "plos_subject": "Developmental biology",
        "round_reason": (
            "Underfilled Field 5; open a general born-OA mid source block using "
            "PLOS Developmental biology subject tags as a neutral mechanistic "
            "biology field filter."
        ),
    },
    {
        "field": "Mechanistic molecular / cell / developmental biology",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "mid",
        "source_block_order": "3",
        "source": "PLOS ONE",
        "source_inventory_id": "plos_one_plos_api_developmental_biology_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "builder": "plos_one",
        "plos_subject": "Developmental biology",
        "round_reason": (
            "Underfilled Field 5; add modern mid-tier developmental-biology "
            "coverage from the official PLOS ONE frame."
        ),
    },
    {
        "field": "Structure-driven condensed matter / chemistry",
        "era_band": "2005-2014_completed_years_2010-2014",
        "year_min": 2010,
        "year_max": 2014,
        "venue_tier": "mid",
        "source_block_order": "3",
        "source": "Chemical Science",
        "source_inventory_id": "chemical_science_rsc_issue_archive_research_v1.0.0_2010-2014_20260621",
        "needed": 10,
        "builder": "chemical_science",
        "round_reason": (
            "Underfilled Field 6; open a mid-tier born-OA chemistry source block "
            "from official RSC issue endpoints, restricted to RSC Edge Article "
            "research content."
        ),
    },
    {
        "field": "Structure-driven condensed matter / chemistry",
        "era_band": "2015-present_completed_years_2015-2025",
        "year_min": 2015,
        "year_max": 2025,
        "venue_tier": "mid",
        "source_block_order": "3",
        "source": "Chemical Science",
        "source_inventory_id": "chemical_science_rsc_issue_archive_research_v1.0.0_2015-2025_20260621",
        "needed": 10,
        "builder": "chemical_science",
        "round_reason": (
            "Underfilled Field 6 and mid-tier imbalance; add modern Chemical "
            "Science research articles from the official RSC issue frame."
        ),
    },
    {
        "field": "Statistically-oriented physics (precision cosmology, exclusion-limit searches, ML-for-physics)",
        "era_band": "2005-2014_completed_years_2005-2014",
        "year_min": 2005,
        "year_max": 2014,
        "venue_tier": "top",
        "source_block_order": "2",
        "source": "Astronomy and Astrophysics",
        "source_inventory_id": "astronomy_astrophysics_official_issues_cosmology_v1.0.0_2005-2014_20260621",
        "needed": 10,
        "builder": "aanda",
        "round_reason": (
            "Underfilled historical top-tier Field 4 cell; use official A&A "
            "issue pages with section/title filters for cosmology and large-scale "
            "structure, plus direct A&A PDF routes."
        ),
    },
]

EXCLUDED_TITLE_RE = re.compile(
    r"(^|\b)(correction|corrigendum|erratum|retraction|editorial|author correction|"
    r"publisher correction):|systematic review|meta-analysis|meta analysis|"
    r"scoping review|narrative review|overview of reviews|study protocol|"
    r"trial protocol|\bprotocol\b|\bguideline\b",
    re.IGNORECASE,
)

AANDA_COSMOLOGY_RE = re.compile(
    r"\b(cosmolog|large-scale structure|large scale structure|dark energy|dark matter|"
    r"cmb|cosmic microwave background|baryon acoustic|bao|hubble diagram|"
    r"supernova hubble|weak lensing|strong lensing|galaxy cluster|clusters of galaxies|"
    r"redshift survey|lambda[- ]?cdm|lcdm|intracluster medium)\b",
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


def normalize_space(value: str) -> str:
    return html.unescape(re.sub(r"\s+", " ", value or "")).strip()


def normalize_doi(value: str | None) -> str:
    if not value:
        return ""
    value = value.strip()
    value = value.removeprefix("https://doi.org/").removeprefix("http://doi.org/")
    return value.lower()


def slugify(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", value.lower()).strip("_")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


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
        for key, col in [("doi", "doi"), ("title", "paper_title"), ("url", "url"), ("pdf", "pdf_url")]:
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
    data: bytes | None = None,
) -> tuple[bytes, str, str]:
    headers = {
        "User-Agent": BROWSER_USER_AGENT if browser else USER_AGENT,
        "Accept": accept,
    }
    if referer:
        headers["Referer"] = referer
    if data is not None:
        headers["Content-Type"] = "application/x-www-form-urlencoded"
    request = urllib.request.Request(url, data=data, headers=headers)
    with urllib.request.urlopen(request, timeout=90) as response:
        return response.read(), response.geturl(), response.headers.get("content-type", "")


def request_json(url: str) -> dict[str, Any]:
    data, _, _ = request_bytes(url, "application/json,*/*")
    return json.loads(data.decode("utf-8"))


def post_html(url: str, params: dict[str, str]) -> tuple[str, str, str]:
    data = urllib.parse.urlencode(params).encode()
    raw, final_url, content_type = request_bytes(
        url,
        "text/html,application/xhtml+xml,*/*",
        browser=True,
        referer="https://pubs.rsc.org/en/journals/journalissues/sc",
        data=data,
    )
    return raw.decode("utf-8", errors="ignore"), final_url, content_type


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
    return bool(EXCLUDED_TITLE_RE.search(title.strip()))


def author_string(authors: list[str]) -> str:
    authors = [normalize_space(a) for a in authors if normalize_space(a)]
    if len(authors) > 5:
        return "; ".join(authors[:5]) + "; et al."
    return "; ".join(authors)


def plos_article_url(doi: str) -> str:
    return f"https://journals.plos.org/plosone/article?id={doi}"


def plos_pdf_url(doi: str) -> str:
    return f"https://journals.plos.org/plosone/article/file?id={doi}&type=printable"


def plos_issue_url(volume: str, issue: str) -> str:
    if volume and issue:
        return f"https://journals.plos.org/plosone/issue?id=10.1371/issue.pone.v{int(volume):02d}.i{int(issue):02d}"
    return "https://journals.plos.org/plosone"


def build_plos_one_inventory(tuple_plan: dict[str, Any], page_sleep: float) -> Path:
    inventory_id = str(tuple_plan["source_inventory_id"])
    inv_dir = INVENTORIES / "plos_one" / inventory_id
    if (
        (inv_dir / "inventory_manifest.json").exists()
        and csv_has_data_rows(inv_dir / "items.csv")
        and csv_has_data_rows(inv_dir / "units.csv")
    ):
        return inv_dir
    inv_dir.mkdir(parents=True, exist_ok=True)

    subject = str(tuple_plan["plos_subject"])
    query = f'journal:"PLOS ONE" AND article_type:"Research Article" AND subject:"{subject}" AND doc_type:full'
    date_filter = (
        f"publication_date:[{tuple_plan['year_min']}-01-01T00:00:00Z TO "
        f"{tuple_plan['year_max']}-12-31T23:59:59Z]"
    )
    fields = "id,title,publication_date,article_type,author,subject,volume,issue,doc_type"
    rows: list[dict[str, str]] = []
    archive_pages: list[dict[str, Any]] = []
    seen: set[str] = set()
    start = 0
    rows_per_page = 500
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
                "page_index": start // rows_per_page,
                "source_url": url,
                "result_count": len(docs),
                "num_found": response.get("numFound", ""),
                "start": start,
            }
        )
        for doc in docs:
            doi = normalize_doi(doc.get("id"))
            if not doi or doi in seen or "/" in doi.removeprefix("10.1371/journal.pone."):
                continue
            title = normalize_space(str(doc.get("title") or ""))
            if not title or is_excluded_title(title):
                continue
            pub_date = str(doc.get("publication_date") or "")[:10]
            year = pub_date[:4]
            volume = str(doc.get("volume") or "")
            issue = str(doc.get("issue") or "")
            if volume and issue:
                unit_id = f"plos_one_v{int(volume):02d}_i{int(issue):02d}"
                unit_label = f"vol. {volume}, issue {issue}"
            else:
                unit_id = f"plos_one_{pub_date[:7].replace('-', '_')}"
                unit_label = pub_date[:7]
            subjects = doc.get("subject") or []
            rows.append(
                {
                    "source": "PLOS ONE",
                    "unit_id": unit_id,
                    "unit_type": "official_journal_issue",
                    "year": year,
                    "unit_date": pub_date,
                    "unit_label": unit_label,
                    "unit_url": plos_issue_url(volume, issue),
                    "landing_page_url": plos_article_url(doi),
                    "source_url": url,
                    "title": title,
                    "summary": "",
                    "authors": author_string(doc.get("author") or []),
                    "oa_marker": "gold;source_known_oa;plos_api_doc_type_full",
                    "pdf_url": plos_pdf_url(doi),
                    "doi": doi,
                    "openalex_id": "",
                    "source_item_id": doi,
                    "paper_type": str(doc.get("article_type") or ""),
                    "venue_section": f"PLOS subject filter: {subject}",
                    "concepts": "; ".join(subjects[:20]),
                    "referenced_works_count": "",
                    "cited_by_count": "",
                }
            )
            seen.add(doi)
        print(
            f"inventory {inventory_id}: start={start} docs={len(docs)} eligible_total={len(rows)}",
            flush=True,
        )
        start += rows_per_page
        if start >= int(response.get("numFound") or 0) or not docs:
            break
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
    write_csv(inv_dir / "archive_pages.csv", archive_pages, ["page_index", "source_url", "result_count", "num_found", "start"])
    write_csv(inv_dir / "units.csv", sorted(units.values(), key=lambda r: (r["year"], r["unit_id"])), ["unit_id", "unit_type", "year", "date", "unit_label", "unit_url"])
    write_csv(inv_dir / "items.csv", rows, CSV_FIELDS)
    manifest = {
        "inventory_id": inventory_id,
        "source": "PLOS ONE",
        "source_urls": ["https://api.plos.org/search", "https://journals.plos.org/plosone"],
        "parser_id": "plos_api_subject_doc_type_full",
        "parser_version": "1.0.0",
        "build_date": RUN_DATE,
        "coverage_years": f"{tuple_plan['year_min']}-{tuple_plan['year_max']}",
        "field_filter": {
            "field": tuple_plan["field"],
            "rule": f'official PLOS subject:"{subject}" plus article_type Research Article and doc_type full',
        },
        "oa_filter": "source_known_gold_oa",
        "user_agent": USER_AGENT,
        "rate_limit": "<=1 request/sec",
        "archive_pages": len(archive_pages),
        "items": len(rows),
        "units": len(units),
        "known_gaps": [],
    }
    (inv_dir / "inventory_manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return inv_dir


def rsc_volume_for_year(year: int) -> int:
    return year - 2009


def rsc_issue_ids_for_year(year: int, page_sleep: float) -> tuple[list[dict[str, str]], dict[str, Any]]:
    volume = rsc_volume_for_year(year)
    root = f"{year}#{volume}#current#2041-6520#SC#2041-6539#volname"
    text, final_url, content_type = post_html("https://pubs.rsc.org/en/journals/JournalIssuesForYear", {"root": root})
    doc = lxml_html.fromstring(text)
    issues: list[dict[str, str]] = []
    for link in doc.xpath('//a[@data-issueid]'):
        issue_id = (link.get("data-issueid") or "").lower()
        if not issue_id:
            continue
        issues.append(
            {
                "year": str(year),
                "volume": str(volume),
                "issue_id": issue_id,
                "issue_label": normalize_space(link.text_content()),
                "issue_type": link.get("data-type") or "current",
                "sercode": link.get("data-sercode") or "sc",
                "issnprint": link.get("data-issnprint") or "2041-6520",
                "issnonline": link.get("data-issnonline") or "2041-6539",
            }
        )
    time.sleep(max(page_sleep, 1.0))
    return issues, {"year": year, "source_url": final_url, "content_type": content_type, "result_count": len(issues)}


def rsc_issue_html(issue: dict[str, str]) -> tuple[str, str, str]:
    params = {
        "name": "SC",
        "issueid": issue["issue_id"],
        "jname": "Chemical Science",
        "pageno": "1",
        "isarchive": "False",
        "issnprint": "2041-6520",
        "issnonline": "2041-6539",
        "iscontentavailable": "True",
        "publishOnlyVolume": "False",
        "latestissueid": "SC017023",
        "category": "advancearticles",
        "duration": "",
    }
    return post_html("https://pubs.rsc.org/en/journals/issues", params)


def build_chemical_science_inventory(tuple_plan: dict[str, Any], page_sleep: float) -> Path:
    inventory_id = str(tuple_plan["source_inventory_id"])
    inv_dir = INVENTORIES / "chemical_science" / inventory_id
    if (
        (inv_dir / "inventory_manifest.json").exists()
        and csv_has_data_rows(inv_dir / "items.csv")
        and csv_has_data_rows(inv_dir / "units.csv")
    ):
        return inv_dir
    inv_dir.mkdir(parents=True, exist_ok=True)

    rows: list[dict[str, str]] = []
    archive_pages: list[dict[str, Any]] = []
    units_rows: list[dict[str, Any]] = []
    seen: set[str] = set()
    allowed_types = {"edge article"}
    for year in range(int(tuple_plan["year_min"]), int(tuple_plan["year_max"]) + 1):
        issues, year_page = rsc_issue_ids_for_year(year, page_sleep)
        archive_pages.append({"page_type": "issue_list", **year_page})
        for issue in issues:
            text, final_url, content_type = rsc_issue_html(issue)
            doc = lxml_html.fromstring(text)
            unit_id = f"chemical_science_{issue['issue_id']}"
            units_rows.append(
                {
                    "unit_id": unit_id,
                    "unit_type": "official_journal_issue",
                    "year": issue["year"],
                    "date": issue["year"],
                    "unit_label": issue["issue_label"],
                    "unit_url": f"https://pubs.rsc.org/en/journals/journal/sc?issueid={issue['issue_id']}",
                }
            )
            cards = doc.xpath('//div[contains(@class,"capsule--article")]')
            eligible_in_issue = 0
            for card in cards:
                context = normalize_space(" ".join(card.xpath('.//*[contains(@class,"capsule__context")]//text()'))
                )
                paper_type = context.replace("Open Access", "").strip()
                if paper_type.lower() not in allowed_types:
                    continue
                landing_links = card.xpath('.//a[contains(@href,"/content/articlelanding/")]/@href')
                pdf_links = card.xpath('.//a[contains(@href,"/content/articlepdf/")]/@href')
                doi_links = card.xpath('.//a[starts-with(@href,"https://doi.org/10.1039")]/@href')
                if not landing_links or not pdf_links or not doi_links:
                    continue
                title = normalize_space(" ".join(card.xpath('.//*[contains(@class,"capsule__title")]//text()')))
                if not title or is_excluded_title(title):
                    continue
                doi = normalize_doi(doi_links[0])
                if not doi or doi in seen:
                    continue
                authors = normalize_space(" ".join(card.xpath('.//*[contains(@class,"capsule__authors")]//text()')))
                landing = urllib.parse.urljoin(final_url, landing_links[0])
                pdf = urllib.parse.urljoin(final_url, pdf_links[0])
                rows.append(
                    {
                        "source": "Chemical Science",
                        "unit_id": unit_id,
                        "unit_type": "official_journal_issue",
                        "year": issue["year"],
                        "unit_date": issue["year"],
                        "unit_label": issue["issue_label"],
                        "unit_url": f"https://pubs.rsc.org/en/journals/journal/sc?issueid={issue['issue_id']}",
                        "landing_page_url": landing,
                        "source_url": final_url,
                        "title": title,
                        "summary": "",
                        "authors": authors,
                        "oa_marker": "gold;source_known_oa;rsc_open_access",
                        "pdf_url": pdf,
                        "doi": doi,
                        "openalex_id": "",
                        "source_item_id": doi,
                        "paper_type": paper_type,
                        "venue_section": "RSC issue card type: Edge Article",
                        "concepts": "",
                        "referenced_works_count": "",
                        "cited_by_count": "",
                    }
                )
                seen.add(doi)
                eligible_in_issue += 1
            archive_pages.append(
                {
                    "page_type": "issue",
                    "year": issue["year"],
                    "source_url": final_url,
                    "content_type": content_type,
                    "result_count": len(cards),
                    "eligible_items_added_total": len(rows),
                    "eligible_items_in_issue": eligible_in_issue,
                }
            )
            print(
                f"inventory {inventory_id}: issue={issue['issue_id']} cards={len(cards)} eligible_total={len(rows)}",
                flush=True,
            )
            time.sleep(max(page_sleep, 1.0))

    rows.sort(key=lambda r: (r["year"], r["unit_id"], r["title"]))
    write_csv(inv_dir / "archive_pages.csv", archive_pages, ["page_type", "year", "source_url", "content_type", "result_count", "eligible_items_added_total", "eligible_items_in_issue"])
    write_csv(inv_dir / "units.csv", units_rows, ["unit_id", "unit_type", "year", "date", "unit_label", "unit_url"])
    write_csv(inv_dir / "items.csv", rows, CSV_FIELDS)
    manifest = {
        "inventory_id": inventory_id,
        "source": "Chemical Science",
        "source_urls": ["https://pubs.rsc.org/en/journals/journalissues/sc", "https://pubs.rsc.org/en/journals/issues"],
        "parser_id": "rsc_chemical_science_issue_endpoint_edge_articles",
        "parser_version": "1.0.0",
        "build_date": RUN_DATE,
        "coverage_years": f"{tuple_plan['year_min']}-{tuple_plan['year_max']}",
        "field_filter": {"field": tuple_plan["field"], "rule": "field-specific chemistry venue; RSC Edge Article research content only"},
        "oa_filter": "source_known_gold_oa",
        "user_agent": BROWSER_USER_AGENT,
        "rate_limit": "<=1 request/sec",
        "archive_pages": len(archive_pages),
        "items": len(rows),
        "units": len(units_rows),
        "known_gaps": [],
    }
    (inv_dir / "inventory_manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return inv_dir


def aanda_all_issue_links() -> list[dict[str, str]]:
    url = "https://www.aanda.org/component/issues/?task=all"
    data, final_url, _ = request_bytes(url, "text/html,application/xhtml+xml,*/*", browser=True)
    doc = lxml_html.fromstring(data.decode("utf-8", errors="ignore"))
    issues: list[dict[str, str]] = []
    for link in doc.xpath('//a[contains(@href,"/articles/aa/abs/") and contains(@href,"/contents/contents.html")]'):
        href = urllib.parse.urljoin(final_url, link.get("href") or "")
        match = re.search(r"/abs/(\d{4})/([^/]+)/contents/contents\.html", href)
        if not match:
            continue
        label = normalize_space(link.text_content())
        issues.append({"year": match.group(1), "issue_code": match.group(2), "unit_url": href, "unit_label": label})
    return issues


def aanda_pdf_from_html(full_html_url: str) -> str:
    return re.sub(r"/full_html/(\d{4}/[^/]+)/([^/]+)/\2\.html$", r"/pdf/\1/\2.pdf", full_html_url)


def build_aanda_inventory(tuple_plan: dict[str, Any], page_sleep: float) -> Path:
    inventory_id = str(tuple_plan["source_inventory_id"])
    inv_dir = INVENTORIES / "astronomy_astrophysics" / inventory_id
    if (
        (inv_dir / "inventory_manifest.json").exists()
        and csv_has_data_rows(inv_dir / "items.csv")
        and csv_has_data_rows(inv_dir / "units.csv")
    ):
        return inv_dir
    inv_dir.mkdir(parents=True, exist_ok=True)

    rows: list[dict[str, str]] = []
    archive_pages: list[dict[str, Any]] = []
    units_rows: list[dict[str, Any]] = []
    seen: set[str] = set()
    issues = [
        i
        for i in aanda_all_issue_links()
        if int(tuple_plan["year_min"]) <= int(i["year"]) <= int(tuple_plan["year_max"])
    ]
    issues.sort(key=lambda r: (r["year"], r["issue_code"]))
    for issue in issues:
        data, final_url, content_type = request_bytes(issue["unit_url"], "text/html,application/xhtml+xml,*/*", browser=True)
        doc = lxml_html.fromstring(data.decode("utf-8", errors="ignore"))
        unit_id = f"aanda_{issue['year']}_{slugify(issue['issue_code'])}"
        units_rows.append(
            {
                "unit_id": unit_id,
                "unit_type": "official_journal_issue",
                "year": issue["year"],
                "date": issue["year"],
                "unit_label": issue["unit_label"],
                "unit_url": issue["unit_url"],
            }
        )
        current_section = ""
        eligible_in_issue = 0
        for el in doc.xpath('//h2 | //article[contains(@class,"science")]'):
            if el.tag == "h2" and (el.get("id") or "").startswith("section_"):
                current_section = normalize_space(el.text_content())
                continue
            if el.tag != "article":
                continue
            title_links = el.xpath('.//a[contains(@class,"article_title")]/@href')
            title = normalize_space(" ".join(el.xpath('.//a[contains(@class,"article_title")]//text()')))
            doi_links = el.xpath('.//div[contains(@class,"article_doi")]//a[starts-with(@href,"https://doi.org/")]/@href')
            pdf_links = el.xpath('.//a[contains(@href,"/articles/aa/pdf/")]/@href')
            if not title_links or not doi_links or not pdf_links:
                continue
            if is_excluded_title(title):
                continue
            section_match = "cosmology" in current_section.lower() or "clusters of galaxies" in current_section.lower()
            title_match = bool(AANDA_COSMOLOGY_RE.search(title))
            if not section_match and not title_match:
                continue
            doi = normalize_doi(doi_links[0])
            if not doi or doi in seen:
                continue
            landing = urllib.parse.urljoin(final_url, title_links[0])
            pdf = urllib.parse.urljoin(final_url, pdf_links[0])
            authors = author_string(el.xpath('.//*[contains(@class,"article-authors")]//*[contains(@class,"author")]//text()'))
            rows.append(
                {
                    "source": "Astronomy and Astrophysics",
                    "unit_id": unit_id,
                    "unit_type": "official_journal_issue",
                    "year": issue["year"],
                    "unit_date": issue["year"],
                    "unit_label": issue["unit_label"],
                    "unit_url": issue["unit_url"],
                    "landing_page_url": landing,
                    "source_url": final_url,
                    "title": title,
                    "summary": "",
                    "authors": authors,
                    "oa_marker": "free_access_or_s2o_archive;publisher_pdf_available",
                    "pdf_url": pdf,
                    "doi": doi,
                    "openalex_id": "",
                    "source_item_id": doi,
                    "paper_type": "article",
                    "venue_section": current_section,
                    "concepts": "A&A cosmology section/title filter",
                    "referenced_works_count": "",
                    "cited_by_count": "",
                }
            )
            seen.add(doi)
            eligible_in_issue += 1
        archive_pages.append(
            {
                "page_index": len(archive_pages),
                "source_url": final_url,
                "result_count": len(doc.xpath('//article[contains(@class,"science")]')),
                "eligible_items_added_total": len(rows),
                "eligible_items_in_issue": eligible_in_issue,
                "content_type": content_type,
            }
        )
        print(
            f"inventory {inventory_id}: issue={issue['year']}/{issue['issue_code']} eligible_total={len(rows)}",
            flush=True,
        )
        time.sleep(max(page_sleep, 1.0))

    rows.sort(key=lambda r: (r["year"], r["unit_id"], r["title"]))
    write_csv(inv_dir / "archive_pages.csv", archive_pages, ["page_index", "source_url", "result_count", "eligible_items_added_total", "eligible_items_in_issue", "content_type"])
    write_csv(inv_dir / "units.csv", units_rows, ["unit_id", "unit_type", "year", "date", "unit_label", "unit_url"])
    write_csv(inv_dir / "items.csv", rows, CSV_FIELDS)
    manifest = {
        "inventory_id": inventory_id,
        "source": "Astronomy and Astrophysics",
        "source_urls": ["https://www.aanda.org/component/issues/?task=all"],
        "parser_id": "aanda_official_issues_cosmology_section_title_filter",
        "parser_version": "1.0.0",
        "build_date": RUN_DATE,
        "coverage_years": f"{tuple_plan['year_min']}-{tuple_plan['year_max']}",
        "field_filter": {
            "field": tuple_plan["field"],
            "rule": "A&A section contains Cosmology/clusters of galaxies or title matches the predeclared cosmology/large-scale-structure regex.",
            "title_regex": AANDA_COSMOLOGY_RE.pattern,
        },
        "oa_filter": "publisher_pdf_available",
        "user_agent": BROWSER_USER_AGENT,
        "rate_limit": "<=1 request/sec",
        "archive_pages": len(archive_pages),
        "items": len(rows),
        "units": len(units_rows),
        "known_gaps": [],
    }
    (inv_dir / "inventory_manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return inv_dir


def ensure_inventory(tuple_plan: dict[str, Any], page_sleep: float) -> Path:
    builder = tuple_plan["builder"]
    if builder == "plos_one":
        return build_plos_one_inventory(tuple_plan, page_sleep)
    if builder == "chemical_science":
        return build_chemical_science_inventory(tuple_plan, page_sleep)
    if builder == "aanda":
        return build_aanda_inventory(tuple_plan, page_sleep)
    raise SystemExit(f"Unknown builder: {builder}")


def eligible(row: dict[str, str], tuple_plan: dict[str, Any]) -> bool:
    try:
        year = int(row.get("year", ""))
    except ValueError:
        return False
    if not (int(tuple_plan["year_min"]) <= year <= int(tuple_plan["year_max"])):
        return False
    if is_excluded_title(row.get("title", "")):
        return False
    return bool(row.get("pdf_url")) and bool(row.get("landing_page_url"))


def looks_pdf_url(url: str) -> bool:
    lowered = url.lower()
    return any(token in lowered for token in [".pdf", "/pdf", "articlepdf", "type=printable"])


def try_pdf_url(url: str, referer: str = "") -> tuple[bytes, str]:
    data, final_url, content_type = request_bytes(url, "application/pdf,text/html,*/*", browser=True, referer=referer)
    if data.lstrip().startswith(b"%PDF"):
        return data, final_url
    if "application/pdf" in content_type.lower() and len(data) > 1000:
        return data, final_url
    raise ValueError(f"not PDF bytes; content_type={content_type}; final_url={final_url}")


def publisher_page_pdf_candidates(landing_url: str) -> tuple[list[str], str]:
    try:
        data, final_url, content_type = request_bytes(landing_url, "text/html,application/xhtml+xml,*/*", browser=True)
    except Exception as exc:  # noqa: BLE001
        return [], repr(exc)
    doc = lxml_html.fromstring(data.decode("utf-8", errors="ignore"))
    candidates = doc.xpath('//meta[@name="citation_pdf_url"]/@content')
    candidates += doc.xpath('//a[contains(translate(@href, "PDFDOWNLOAD", "pdfdownload"), "pdf")]/@href')
    candidates = [urllib.parse.urljoin(final_url, href) for href in candidates]
    return unique(candidates), f"content_type={content_type}; extracted_pdf_like_links={len(candidates)}"


def acquire_pdf(row: dict[str, str]) -> tuple[bytes, str, str, list[dict[str, str]]]:
    attempts: list[dict[str, str]] = []
    landing = row.get("landing_page_url", "")
    for url in unique([row.get("pdf_url", "")]):
        if not looks_pdf_url(url):
            continue
        try:
            pdf, final_url = try_pdf_url(url, landing)
            attempts.append({"route": "direct_pdf_route", "url": url, "result": "success", "final_url": final_url})
            return pdf, final_url, "direct_pdf_route", attempts
        except Exception as exc:  # noqa: BLE001
            attempts.append({"route": "direct_pdf_route", "url": url, "result": "failed", "error": repr(exc)})

    candidates, note = publisher_page_pdf_candidates(landing)
    attempts.append({"route": "publisher_page_browser_fallback", "url": landing, "result": "candidate_links_found" if candidates else "failed", "note": note})
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


def draw_for_tuple(
    tuple_plan: dict[str, Any],
    inv_dir: Path,
    papers: list[dict[str, str]],
    taken: set[tuple[str, str]],
    next_id: int,
    tuple_index: int,
    dry_run: bool,
) -> tuple[list[dict[str, Any]], int, list[dict[str, Any]]]:
    items_path = inv_dir / "items.csv"
    units_path = inv_dir / "units.csv"
    manifest_path = inv_dir / "inventory_manifest.json"
    rows = [r for r in read_csv(items_path) if eligible(r, tuple_plan)]
    units: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in rows:
        units[row["unit_id"]].append(row)
    unit_ids = sorted(uid for uid, unit_rows in units.items() if unit_rows)
    if not unit_ids:
        raise SystemExit(f"No eligible units for {tuple_plan['source']} {tuple_plan['source_inventory_id']}")

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
            raise SystemExit(f"Exceeded {max_attempts} attempts for {tuple_plan['source']}")
        live_units = [
            uid
            for uid in unit_ids
            if any(not (row_keys(r) & (taken | local_taken)) for r in units[uid])
        ]
        if not live_units:
            raise SystemExit(f"Exhausted available papers for {tuple_plan['source']}")
        unit_id = rng.choice(live_units)
        available = [r for r in units[unit_id] if not (row_keys(r) & (taken | local_taken))]
        row = rng.choice(available)
        keys = row_keys(row)
        paper_id = f"P{next_id:04d}"
        field_slug = FIELD_SLUG[str(tuple_plan["field"])]
        pdf_path = ROOT / "pdfs" / field_slug / f"{paper_id}.pdf"
        text_path = ROOT / "text" / field_slug / f"{paper_id}.txt"
        p = 1.0 / len(unit_ids) / len(units[unit_id])
        record: dict[str, Any] = {
            "paper_id": paper_id,
            "field": tuple_plan["field"],
            "source": tuple_plan["source"],
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
            "venue_section": row.get("venue_section", ""),
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

        print(f"  trying {paper_id}: {tuple_plan['source']} | {row.get('year', '')} | {row.get('doi', '')}", flush=True)
        try:
            cited_by_count, openalex_id = openalex_count_for_doi(str(record["doi"]))
            record["openalex_cited_by_count"] = cited_by_count
            record["openalex_id"] = openalex_id
            pdf_bytes, final_url, route, route_attempts = acquire_pdf(row)
            text = extract_text(pdf_bytes)
            text_chars = len(text.strip())
            if text_chars < 5000:
                raise ValueError(f"extracted text too short ({text_chars} chars)")
            pages = page_count(pdf_bytes)
        except Exception as exc:  # noqa: BLE001
            attempts_log.append(
                {
                    "paper_id": paper_id,
                    "source": tuple_plan["source"],
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
                "source": tuple_plan["source"],
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


def audit_dir() -> Path:
    return AUDIT_ROOT / ROUND_ID


def write_preflight(papers: list[dict[str, str]] | None = None) -> None:
    directory = audit_dir()
    directory.mkdir(parents=True, exist_ok=True)
    current_counts: Counter[str] = Counter()
    if papers is not None:
        current_counts.update(r.get("field", "") for r in papers if r.get("sample_source") == "random_base")
    rows = [
        {
            "field": p["field"],
            "era_band": p["era_band"],
            "venue_tier": p["venue_tier"],
            "source": p["source"],
            "source_inventory_id": p["source_inventory_id"],
            "planned_additions": p["needed"],
            "current_random_base_field_count": current_counts.get(str(p["field"]), ""),
            "status": "planned_to_target_10",
            "reason": p["round_reason"],
        }
        for p in PLAN
    ]
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
        "selection_rationale": (
            "Address raw-count imbalances by freezing the already-overrepresented "
            "specialist-heavy source blocks and opening official-source mid/top "
            "blocks for underfilled fields. The batch prioritizes 2005-2014 where "
            "clean source frames exist, with paired modern blocks only for fields "
            "that are still below the field-count floor."
        ),
        "frozen_overrepresented_blocks": [
            "The European Physical Journal C",
            "Frontiers in Computational Neuroscience",
            "PLOS Computational Biology",
        ],
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
                    f"{ROUND_ID}; balanced Stratum A draw; source={record['source']}; "
                    f"inventory={record['inventory_id']}; unit={record['unit_id']}; "
                    f"route={record.get('acquisition_route', '')}; official source frame used; "
                    "resolvers not used as replacement sampling frames"
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
        key = (tuple_plan["field"], tuple_plan["era_band"], tuple_plan["venue_tier"], tuple_plan["source"])
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
        grouped[(record["field"], record["era_band"], record["venue_tier"], record["source"])].append(record["paper_id"])
    for key, ids in grouped.items():
        row = schedule_by_key[key]
        existing_ids = [x for x in row.get("paper_ids", "").split(";") if x]
        merged = existing_ids + ids
        row["paper_ids"] = ";".join(merged)
        row["random_base_papers_in_block"] = str(len(merged))
        row["status"] = "completed_target_10" if len(merged) >= TARGET_PER_TUPLE else "partially_completed"
        row["design_note"] = (
            row.get("design_note", "")
            + f" Updated by {ROUND_ID}; added {len(ids)} balancing papers; "
            "direct PDF routes succeeded or publisher-page fallback was attempted before any legal alternate route."
        ).strip()
        schedule_by_key[key] = row
    rows = sorted(
        ({field: row.get(field, "") for field in schedule_fieldnames} for row in schedule_by_key.values()),
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
            "Balance underfilled fields, mid/top tiers, and historical coverage "
            "while freezing overrepresented specialist blocks."
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


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--preflight-only", action="store_true")
    parser.add_argument("--page-sleep", type=float, default=1.0)
    args = parser.parse_args()

    requested = sum(int(p["needed"]) for p in PLAN)
    if requested != REQUESTED_NEW_PAPERS:
        raise SystemExit(f"Plan requests {requested}, expected {REQUESTED_NEW_PAPERS}")
    papers = read_csv(PAPERS_CSV)
    write_preflight(papers)
    print("preflight target table", flush=True)
    for p in PLAN:
        print(f"  {p['needed']:>2} | {p['field']} | {p['era_band']} | {p['venue_tier']} | {p['source']}", flush=True)
    if args.preflight_only:
        print(f"wrote {audit_dir() / 'preflight_target_table.csv'}", flush=True)
        return

    inventories: dict[str, Path] = {}
    for p in PLAN:
        inventory_id = str(p["source_inventory_id"])
        inventories[inventory_id] = ensure_inventory(p, args.page_sleep)

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
            selected, next_id, logs = draw_for_tuple(tuple_plan, inv_dir, papers, taken, next_id, idx, args.dry_run)
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
