export type ScholarlyObjectSummary = {
  id: string;
  audit_object_id: string | null;
  object_type: string;
  work_group: ArticleVersionGroupSummary | null;
  version_kind: string;
  title: string;
  authors: string[];
  source_name: string;
  publication_year: number | null;
  canonical_url: string;
  license: string | null;
  review_status: string;
  evaluation_fact_count: number;
  review_event_count: number;
  active_element_review_count: number;
  active_synthesis_review_count: number;
};

export type LibraryItemSummary = {
  id: string;
  user_id: string;
  audit_object_id: string;
  added_reason: string;
  added_at: string;
  scholarly_object: ScholarlyObjectSummary;
};

export type ProblemAreaWorkSummary = {
  scholarly_object: ScholarlyObjectSummary;
  problem_review_event_count: number;
  relevance: string;
};

export type DomainInstantiationSummary = {
  id: string;
  domain_type: string;
  name: string;
  created_at: string;
  governed_by: unknown;
};

export type CWENode = {
  id: string;
  domain_instantiation_id: string;
  parent: string | null;
  label: string;
  description: string;
  source: string;
};

export type DomainInstantiationDetail = DomainInstantiationSummary & {
  config: unknown;
  cwe_nodes: CWENode[];
};

export type ExternalArticleLocation = {
  id: string;
  location_type: string;
  url: string;
  license: string | null;
  is_canonical: boolean;
};

export type ScholarlyObjectDetail = ScholarlyObjectSummary & {
  work_group: ArticleVersionGroupSummary | null;
  version_kind: string;
  versions: ArticleVersionSummary[];
  doi: string | null;
  abstract_text: string | null;
  publication_date: string | null;
  native_display_permitted: boolean;
  external_locations: ExternalArticleLocation[];
};

export type ArticleVersionSummary = {
  scholarly_object_id: string;
  title: string;
  version_kind: string;
  doi: string | null;
  source_name: string;
  canonical_url: string;
  native_display_permitted: boolean;
  is_current: boolean;
  is_primary: boolean;
};

export type ArticleAccessSummary = {
  scholarly_object_id: string;
  doi: string | null;
  source_name: string;
  publication_date: string | null;
  license: string | null;
  canonical_url: string;
  display_strategy: string;
  rights_status: string;
  native_display_permitted: boolean;
  canonical_location: ExternalArticleLocation | null;
  preferred_source: ExternalArticleLocation | null;
  external_locations: ExternalArticleLocation[];
};

export type ReviewAssignmentSummary = {
  id: string;
  scholarly_object_id: string;
  scholarly_object_title: string;
  scholarly_object_canonical_url: string;
  reviewer_display_name: string;
  assignment_type: string;
  compensation_status: string;
  state: string;
  due_at: string | null;
};

export type ArticleRetrievalResult = {
  source: string;
  source_identifier: string;
  work_group: ArticleVersionGroupSummary;
  version_kind: string;
  scholarly_object_id: string;
  title: string;
  authors: string[];
  abstract_text: string | null;
  canonical_url: string;
  pdf_url: string | null;
  doi: string | null;
  was_created: boolean;
  article_access: ArticleAccessSummary;
};

export type ArticleVersionGroupSummary = {
  id: string;
  title: string;
  primary_scholarly_object_id: string | null;
  version_count: number;
};

export type ArticleRetrievalSet = {
  results: ArticleRetrievalResult[];
};

export type ArticleRetrievalResponse = {
  result: ArticleRetrievalResult | null;
  results: ArticleRetrievalResult[];
  error: string | null;
};

export type CreateElementReviewRequest = {
  cwe_node_id: string;
  finding: string;
  severity: string | null;
  confidence: string | null;
  content: string;
  solicitation: string | null;
};

export type ReviewEvent = {
  id: string;
  audit_object_id: string;
  domain_instantiation_id: string;
  occurred_at: string;
  payload: unknown;
  status: string;
  provenance: unknown;
};

export type ArticleRetrievalOptions = {
  includePreprints?: boolean;
};

const apiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";

export async function getScholarlyObjects(): Promise<ScholarlyObjectSummary[]> {
  try {
    const response = await fetch(`${apiBaseUrl}/api/scholarly-objects`, {
      cache: "no-store",
    });

    if (!response.ok) {
      return [];
    }

    return response.json();
  } catch {
    return [];
  }
}

export async function searchScholarlyObjects(
  query: string,
): Promise<ScholarlyObjectSummary[]> {
  try {
    const params = new URLSearchParams({ query });
    const response = await fetch(`${apiBaseUrl}/api/work-search?${params.toString()}`, {
      cache: "no-store",
    });

    if (!response.ok) {
      return [];
    }

    return response.json();
  } catch {
    return [];
  }
}

export async function browseProblemAreaWorks(options: {
  cweNodeId?: string;
  query?: string;
}): Promise<ProblemAreaWorkSummary[]> {
  const params = new URLSearchParams();

  if (options.query?.trim()) {
    params.set("query", options.query.trim());
  }

  if (options.cweNodeId?.trim()) {
    params.set("cwe_node_id", options.cweNodeId.trim());
  }

  try {
    const response = await fetch(
      `${apiBaseUrl}/api/peer-review/problem-area-works?${params.toString()}`,
      {
        cache: "no-store",
      },
    );

    if (!response.ok) {
      return [];
    }

    return response.json();
  } catch {
    return [];
  }
}

export async function getLibraryItems(): Promise<LibraryItemSummary[]> {
  try {
    const response = await fetch(`${apiBaseUrl}/api/library-items`, {
      cache: "no-store",
    });

    if (!response.ok) {
      return [];
    }

    return response.json();
  } catch {
    return [];
  }
}

export async function addLibraryItem(
  scholarlyObjectId: string,
): Promise<LibraryItemSummary | null> {
  try {
    const response = await fetch(`${apiBaseUrl}/api/library-items`, {
      body: JSON.stringify({ scholarly_object_id: scholarlyObjectId }),
      cache: "no-store",
      headers: {
        "Content-Type": "application/json",
      },
      method: "POST",
    });

    if (!response.ok) {
      return null;
    }

    return response.json();
  } catch {
    return null;
  }
}

export async function getDomainInstantiations(): Promise<
  DomainInstantiationSummary[]
> {
  try {
    const response = await fetch(`${apiBaseUrl}/api/domain-instantiations`, {
      cache: "no-store",
    });

    if (!response.ok) {
      return [];
    }

    return response.json();
  } catch {
    return [];
  }
}

export async function getDomainInstantiation(
  id: string,
): Promise<DomainInstantiationDetail | null> {
  try {
    const response = await fetch(`${apiBaseUrl}/api/domain-instantiations/${id}`, {
      cache: "no-store",
    });

    if (!response.ok) {
      return null;
    }

    return response.json();
  } catch {
    return null;
  }
}

export async function getScholarlyObject(
  id: string,
): Promise<ScholarlyObjectDetail | null> {
  try {
    const response = await fetch(`${apiBaseUrl}/api/scholarly-objects/${id}`, {
      cache: "no-store",
    });

    if (!response.ok) {
      return null;
    }

    return response.json();
  } catch {
    return null;
  }
}

export async function getArticleAccess(
  id: string,
): Promise<ArticleAccessSummary | null> {
  try {
    const response = await fetch(
      `${apiBaseUrl}/api/scholarly-objects/${id}/article-access`,
      {
        cache: "no-store",
      },
    );

    if (!response.ok) {
      return null;
    }

    return response.json();
  } catch {
    return null;
  }
}

export async function createElementReview(
  scholarlyObjectId: string,
  request: CreateElementReviewRequest,
): Promise<ReviewEvent | null> {
  try {
    const response = await fetch(
      `${apiBaseUrl}/api/scholarly-objects/${scholarlyObjectId}/review-events/element-review`,
      {
        body: JSON.stringify(request),
        cache: "no-store",
        headers: {
          "Content-Type": "application/json",
        },
        method: "POST",
      },
    );

    if (!response.ok) {
      return null;
    }

    return response.json();
  } catch {
    return null;
  }
}

export async function getReviewAssignments(): Promise<ReviewAssignmentSummary[]> {
  try {
    const response = await fetch(`${apiBaseUrl}/api/review-assignments`, {
      cache: "no-store",
    });

    if (!response.ok) {
      return [];
    }

    return response.json();
  } catch {
    return [];
  }
}

export async function retrieveArxivArticle(
  query: string,
): Promise<ArticleRetrievalResponse> {
  return retrieveArticleFromEndpoint("/api/article-retrieval/arxiv", query);
}

export async function retrieveDoiArticle(
  query: string,
): Promise<ArticleRetrievalResponse> {
  return retrieveArticleFromEndpoint("/api/article-retrieval/doi", query);
}

export async function retrievePubmedArticle(
  query: string,
): Promise<ArticleRetrievalResponse> {
  return retrieveArticleFromEndpoint("/api/article-retrieval/pubmed", query);
}

export async function retrieveTitleArticle(
  query: string,
  options: ArticleRetrievalOptions = {},
): Promise<ArticleRetrievalResponse> {
  return retrieveArticleFromEndpoint("/api/article-retrieval/title", query, {
    include_preprints: options.includePreprints,
  });
}

export async function retrieveArticle(
  query: string,
  options: ArticleRetrievalOptions = {},
): Promise<ArticleRetrievalResponse> {
  if (queryContainsDoi(query)) {
    return retrieveDoiArticle(query);
  }

  if (queryContainsArxivIdentifier(query)) {
    return retrieveArxivArticle(query);
  }

  if (queryContainsPubmedIdentifier(query)) {
    return retrievePubmedArticle(query);
  }

  return retrieveTitleArticle(query, options);
}

async function retrieveArticleFromEndpoint(
  endpoint: string,
  query: string,
  extraParams: Record<string, boolean | string | undefined> = {},
): Promise<ArticleRetrievalResponse> {
  const params = new URLSearchParams({ query });
  for (const [key, value] of Object.entries(extraParams)) {
    if (value !== undefined) {
      params.set(key, String(value));
    }
  }

  try {
    const response = await fetch(`${apiBaseUrl}${endpoint}?${params.toString()}`, {
      cache: "no-store",
    });

    if (!response.ok) {
      const body = (await response.json().catch(() => null)) as {
        error?: string;
      } | null;

      return {
        result: null,
        results: [],
        error: body?.error ?? `Article retrieval failed with status ${response.status}`,
      };
    }

    const body = (await response.json()) as ArticleRetrievalResult | ArticleRetrievalSet;
    const results = isArticleRetrievalSet(body) ? body.results : [body];

    return {
      result: results[0] ?? null,
      results,
      error: null,
    };
  } catch {
    return {
      result: null,
      results: [],
      error: "Article retrieval failed before the API responded.",
    };
  }
}

function isArticleRetrievalSet(
  body: ArticleRetrievalResult | ArticleRetrievalSet,
): body is ArticleRetrievalSet {
  return "results" in body && Array.isArray(body.results);
}

function queryContainsDoi(query: string) {
  const trimmed = query.trim().toLowerCase();

  return (
    trimmed.startsWith("10.") ||
    trimmed.startsWith("doi:10.") ||
    trimmed.includes("doi.org/10.") ||
    trimmed.includes("dx.doi.org/10.")
  );
}

function queryContainsArxivIdentifier(query: string) {
  const trimmed = query.trim().toLowerCase();

  return (
    trimmed.includes("arxiv.org/abs/") ||
    trimmed.includes("arxiv.org/pdf/") ||
    /^arxiv:\s*\d{4}\.\d{4,5}(v\d+)?$/.test(trimmed) ||
    /^\d{4}\.\d{4,5}(v\d+)?$/.test(trimmed) ||
    /^[a-z-]+(\.[a-z]{2})?\/\d{7}(v\d+)?$/.test(trimmed)
  );
}

function queryContainsPubmedIdentifier(query: string) {
  const trimmed = query.trim().toLowerCase();

  return (
    /^pmid:\s*\d+$/.test(trimmed) ||
    /^pmcid:\s*pmc\d+$/.test(trimmed) ||
    /^pmc\d+$/.test(trimmed) ||
    /^\d+$/.test(trimmed) ||
    trimmed.includes("pubmed.ncbi.nlm.nih.gov/") ||
    trimmed.includes("pmc.ncbi.nlm.nih.gov/articles/") ||
    trimmed.includes("ncbi.nlm.nih.gov/pmc/articles/")
  );
}

export function formatLabel(value: string) {
  return value
    .split("_")
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(" ");
}
