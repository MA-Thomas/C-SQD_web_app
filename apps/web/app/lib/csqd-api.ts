export type ScholarlyObjectSummary = {
  id: string;
  audit_subject_id: string | null;
  object_type: string;
  work_group: ArticleVersionGroupSummary | null;
  version_kind: string;
  title: string;
  authors: string[];
  source_name: string;
  publication_year: number | null;
  canonical_url: string;
  license: string | null;
  audit_status: string;
  audit_episode_count: number;
  fact_count: number;
  element_review_fact_count: number;
  synthesis_review_count: number;
};

export type LibraryItemSummary = {
  id: string;
  user_id: string;
  subject_id: string;
  added_reason: string;
  added_at: string;
  scholarly_object: ScholarlyObjectSummary;
};

export type ProblemAreaWorkSummary = {
  scholarly_object: ScholarlyObjectSummary;
  problem_fact_count: number;
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

export type ArticleRetrievalResult = {
  source: string;
  source_identifier: string;
  work_group: ArticleVersionGroupSummary;
  version_kind: string;
  scholarly_object_id: string;
  audit_subject_id: string;
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

export type AuditSubject = {
  id: string;
  domain_instantiation_id: string;
  subject_type: string;
  title: string | null;
  external_refs: unknown[];
  registered_by: unknown;
  registered_at: string;
};

export type AuditEpisode = {
  id: string;
  subject_id: string;
  domain_instantiation_id: string;
  label: string;
  status: string;
  authored_by: unknown;
  authored_at: string;
  notes: string | null;
};

export type AuditEpisodeSummary = AuditEpisode & {
  subject_title: string | null;
  subject_type: string;
  sponsor_name: string | null;
  sponsor_organization_type: string | null;
  fact_count: number;
  element_review_count: number;
  synthesis_review_count: number;
  latest_activity_at: string | null;
  synthesis_ready: boolean;
};

export type Money = {
  amount: number;
  currency: string;
};

export type CreateAuditSubjectRequest = {
  domain_instantiation_id: string;
  subject_type: string;
  title: string | null;
  external_refs?: unknown[];
  registered_by?: unknown;
};

export type CommissionAuditEpisodeRequest = {
  label: string;
  sponsor_organization_name: string;
  sponsor_organization_type: string;
  funding: Money;
  scope_cwe_node_ids: string[];
  deadline: string | null;
  confidential: boolean;
  notes: string | null;
};

export type CommissionAuditEpisodeResult = {
  organization: {
    id: string;
    name: string;
    org_type: string;
    created_at: string;
  };
  episode: AuditEpisode;
  commission_fact: Fact;
};

export type CreateEpisodeElementReviewRequest = {
  cwe_node_id: string;
  submitted_by?: string | null;
  solicitation?: string | null;
  finding: string;
  severity: string | null;
  confidence: string | null;
  limitations: string | null;
  recommendations: string | null;
  content: string;
  featured: boolean;
};

export type PaymentScheme = {
  amount: Money;
  currency: string;
  condition: string | { tiered: { on_submission: Money; on_acceptance: Money } };
};

export type CreateEpisodeSolicitationRequest = {
  issued_to: string | null;
  cwe_node_id: string;
  commission_fact_id: string | null;
  payment_scheme: PaymentScheme;
};

export type CreateEpisodeSolicitationEventRequest = {
  solicitation_fact_id: string;
  event_type: string;
  principal: unknown | null;
  note: string | null;
};

export type CreateSynthesisReviewSectionRequest = {
  section_type: string;
  content: string;
  referenced_facts: string[];
};

export type CreateSynthesisReviewRequest = {
  submitted_by?: string | null;
  status: string;
  summary: string;
  sections: CreateSynthesisReviewSectionRequest[];
  featured: boolean;
  unsolicited?: boolean;
};

export type SynthesisReviewSection = {
  id: string;
  review_id: string;
  section_type: string;
  content: string;
  referenced_facts: string[];
};

export type SynthesisReview = {
  unsolicited?: boolean;
  id: string;
  episode_id: string;
  submitted_by: string;
  authored_at: string;
  status: string;
  summary: string;
  sections: SynthesisReviewSection[];
  featured: boolean;
};

export type EvalTuple = {
  n: number;
  m: number;
  s: number;
  l: number;
  u: number;
  computed_at: string;
  community_filter: {
    tags: string[];
    domain_instantiation_id: string | null;
    min_endorsements: number | null;
  };
};

export type Fact = {
  id: string;
  subject_id: string;
  domain_instantiation_id: string;
  occurred_at: string;
  payload: unknown;
  status: string;
  provenance: unknown;
  external_refs: unknown[];
};

export type ArticleRetrievalOptions = {
  includePreprints?: boolean;
};

const apiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";

export async function getAuditEpisodes(): Promise<AuditEpisodeSummary[]> {
  return fetchJson<AuditEpisodeSummary[]>("/api/audit-episodes", []);
}

export async function getAuditSubjects(): Promise<AuditSubject[]> {
  return fetchJson<AuditSubject[]>("/api/audit-subjects", []);
}

export async function getAuditSubject(id: string): Promise<AuditSubject | null> {
  return fetchJson<AuditSubject | null>(`/api/audit-subjects/${id}`, null);
}

export async function createAuditSubject(
  request: CreateAuditSubjectRequest,
): Promise<AuditSubject | null> {
  return postJson<AuditSubject>("/api/audit-subjects", request);
}

export async function getAuditEpisodesForSubject(
  subjectId: string,
): Promise<AuditEpisode[]> {
  return fetchJson<AuditEpisode[]>(
    `/api/audit-subjects/${subjectId}/audit-episodes`,
    [],
  );
}

export async function commissionAuditEpisode(
  subjectId: string,
  request: CommissionAuditEpisodeRequest,
): Promise<CommissionAuditEpisodeResult | null> {
  return postJson<CommissionAuditEpisodeResult>(
    `/api/audit-subjects/${subjectId}/audit-episodes`,
    request,
  );
}

export async function getAuditEpisode(id: string): Promise<AuditEpisode | null> {
  return fetchJson<AuditEpisode | null>(`/api/audit-episodes/${id}`, null);
}

export async function getFactsForEpisode(episodeId: string): Promise<Fact[]> {
  return fetchJson<Fact[]>(`/api/audit-episodes/${episodeId}/facts`, []);
}

export async function getFactsForSubject(subjectId: string): Promise<Fact[]> {
  return fetchJson<Fact[]>(`/api/audit-subjects/${subjectId}/facts`, []);
}

export async function createEpisodeElementReview(
  episodeId: string,
  request: CreateEpisodeElementReviewRequest,
): Promise<Fact | null> {
  return postJson<Fact>(
    `/api/audit-episodes/${episodeId}/facts/element-review`,
    request,
  );
}

export async function createEpisodeSolicitation(
  episodeId: string,
  request: CreateEpisodeSolicitationRequest,
): Promise<Fact | null> {
  return postJson<Fact>(
    `/api/audit-episodes/${episodeId}/facts/solicitation`,
    request,
  );
}

export async function createEpisodeSolicitationEvent(
  episodeId: string,
  request: CreateEpisodeSolicitationEventRequest,
): Promise<Fact | null> {
  return postJson<Fact>(
    `/api/audit-episodes/${episodeId}/facts/solicitation-event`,
    request,
  );
}

export async function getSynthesisReviews(
  episodeId: string,
): Promise<SynthesisReview[]> {
  return fetchJson<SynthesisReview[]>(
    `/api/audit-episodes/${episodeId}/synthesis-reviews`,
    [],
  );
}

export async function createSynthesisReview(
  episodeId: string,
  request: CreateSynthesisReviewRequest,
): Promise<SynthesisReview | null> {
  return postJson<SynthesisReview>(
    `/api/audit-episodes/${episodeId}/synthesis-reviews`,
    request,
  );
}

export async function getEvalTuple(episodeId: string): Promise<EvalTuple | null> {
  return fetchJson<EvalTuple | null>(
    `/api/audit-episodes/${episodeId}/eval-tuple`,
    null,
  );
}

export async function getScholarlyObjects(): Promise<ScholarlyObjectSummary[]> {
  return fetchJson<ScholarlyObjectSummary[]>("/api/scholarly-objects", []);
}

export async function searchScholarlyObjects(
  query: string,
): Promise<ScholarlyObjectSummary[]> {
  const params = new URLSearchParams({ query });
  return fetchJson<ScholarlyObjectSummary[]>(
    `/api/work-search?${params.toString()}`,
    [],
  );
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

  return fetchJson<ProblemAreaWorkSummary[]>(
    `/api/peer-review/problem-area-works?${params.toString()}`,
    [],
  );
}

export async function getLibraryItems(): Promise<LibraryItemSummary[]> {
  return fetchJson<LibraryItemSummary[]>("/api/library-items", []);
}

export async function addLibraryItem(
  scholarlyObjectId: string,
): Promise<LibraryItemSummary | null> {
  return postJson<LibraryItemSummary>("/api/library-items", {
    scholarly_object_id: scholarlyObjectId,
  });
}

export async function getDomainInstantiations(): Promise<
  DomainInstantiationSummary[]
> {
  return fetchJson<DomainInstantiationSummary[]>("/api/domain-instantiations", []);
}

export async function getDomainInstantiation(
  id: string,
): Promise<DomainInstantiationDetail | null> {
  return fetchJson<DomainInstantiationDetail | null>(
    `/api/domain-instantiations/${id}`,
    null,
  );
}

export async function getScholarlyObject(
  id: string,
): Promise<ScholarlyObjectDetail | null> {
  return fetchJson<ScholarlyObjectDetail | null>(
    `/api/scholarly-objects/${id}`,
    null,
  );
}

export async function getArticleAccess(
  id: string,
): Promise<ArticleAccessSummary | null> {
  return fetchJson<ArticleAccessSummary | null>(
    `/api/scholarly-objects/${id}/article-access`,
    null,
  );
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

async function fetchJson<T>(endpoint: string, fallback: T): Promise<T> {
  try {
    const response = await fetch(`${apiBaseUrl}${endpoint}`, {
      cache: "no-store",
      credentials: "include",
    });

    if (!response.ok) {
      return fallback;
    }

    return response.json();
  } catch {
    return fallback;
  }
}

async function postJson<T>(endpoint: string, body: unknown): Promise<T | null> {
  try {
    const response = await fetch(`${apiBaseUrl}${endpoint}`, {
      body: JSON.stringify(body),
      cache: "no-store",
      credentials: "include",
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

// ── Public audit summaries (B3 backend) ─────────────────────────

export type PublicTupleSummaryApi = {
  problems: number;
  ethical_concerns: number;
  stakes: number;
  scrutiny_depth: number;
  uptake: number;
};

export type PublicSubjectSummaryApi = {
  scholarly_object_id: string | null;
  audit_subject_id: string | null;
  status_label: string;
  tuple: PublicTupleSummaryApi | null;
  crwe_reviewed_node_ids: string[];
  element_review_count: number;
  synthesis_review_count: number;
  challenge_count: number;
  latest_report: SynthesisReview | null;
  episodes: AuditEpisode[];
};

export async function getPublicSummaryForScholarlyObject(
  id: string,
): Promise<PublicSubjectSummaryApi | null> {
  return apiGet<PublicSubjectSummaryApi>(
    `/api/public/scholarly-objects/${id}/summary`,
  );
}

export async function getPublicSummariesForScholarlyObjects(
  ids: string[],
): Promise<PublicSubjectSummaryApi[]> {
  if (ids.length === 0) {
    return [];
  }

  const summaries: PublicSubjectSummaryApi[] = [];

  for (let start = 0; start < ids.length; start += 100) {
    const batch = ids.slice(start, start + 100);
    const result = await apiGet<PublicSubjectSummaryApi[]>(
      `/api/public/scholarly-objects/summaries?ids=${batch
        .map(encodeURIComponent)
        .join(",")}`,
    );

    summaries.push(...(result ?? []));
  }

  return summaries;
}

// ── Evaluation tuple recomputation ──────────────────────────────

export type EvalTupleParams = {
  tEval?: string;
  tags?: string[];
  minEndorsements?: number;
};

export async function getEvalTupleWithParams(
  episodeId: string,
  params: EvalTupleParams,
): Promise<EvalTuple | null> {
  const query = new URLSearchParams();

  if (params.tEval) {
    query.set("t_eval", params.tEval);
  }

  if (params.tags && params.tags.length > 0) {
    query.set("tags", params.tags.join(","));
  }

  if (params.minEndorsements !== undefined) {
    query.set("min_endorsements", String(params.minEndorsements));
  }

  const suffix = query.toString() ? `?${query.toString()}` : "";

  return apiGet<EvalTuple>(`/api/audit-episodes/${episodeId}/eval-tuple${suffix}`);
}

// ── Timeline ────────────────────────────────────────────────────

export type TimelineEntry =
  | { entry_type: "fact"; fact: Fact }
  | { entry_type: "membership"; membership: Record<string, unknown> }
  | { entry_type: "episode_relation"; relation: EpisodeRelation }
  | { entry_type: "synthesis_review"; review: SynthesisReview }
  | { entry_type: "synthesis_relation"; relation: SynthesisReviewRelation };

export async function getEpisodeTimeline(
  episodeId: string,
): Promise<TimelineEntry[]> {
  return (await apiGet<TimelineEntry[]>(
    `/api/audit-episodes/${episodeId}/timeline`,
  )) ?? [];
}

// ── Relations / challenges ──────────────────────────────────────

export type EpisodeRelation = {
  id: string;
  source_episode_id: string;
  target_episode_id: string;
  relation_type: unknown;
  asserted_by: unknown;
  asserted_at: string;
};

export type SynthesisReviewRelation = {
  id: string;
  source: string;
  target: string;
  relation_type:
    | "supersedes"
    | "related_to"
    | { contests: { scope: "partial" | "full"; rationale: string | null } };
  asserted_by: unknown;
  asserted_at: string;
};

export async function getSynthesisReviewRelations(
  reviewId: string,
): Promise<SynthesisReviewRelation[]> {
  return (await apiGet<SynthesisReviewRelation[]>(
    `/api/synthesis-reviews/${reviewId}/relations`,
  )) ?? [];
}

export async function contestSynthesisReview(
  reviewId: string,
  target: string,
  scope: "partial" | "full",
  rationale: string,
): Promise<SynthesisReviewRelation | null> {
  return apiSend<SynthesisReviewRelation>(
    `/api/synthesis-reviews/${reviewId}/relations`,
    {
      target,
      relation_type: { contests: { scope, rationale } },
    },
  );
}

// ── Session / auth ──────────────────────────────────────────────

export type SessionUser = {
  user_id: string;
  display_name: string;
  email: string;
  roles: Array<"member" | "sponsor" | "reviewer" | "operator">;
};

export async function requestMagicLink(email: string): Promise<{
  email: string;
  expires_at: string;
  sign_in_url: string;
} | null> {
  return apiSend(`/api/auth/request-link`, { email });
}

export async function completeMagicLink(
  token: string,
): Promise<{ user: SessionUser } | null> {
  return apiSend(`/api/auth/complete`, { token });
}

export async function getSession(): Promise<SessionUser | null> {
  const result = await apiGet<{ user: SessionUser | null }>(`/api/auth/session`);

  return result?.user ?? null;
}

export async function signOut(): Promise<void> {
  await apiSend(`/api/auth/sign-out`, {});
}

// ── Participation, petitions, responses ─────────────────────────

export async function startPublicEpisode(
  subjectId: string,
  label: string,
  notes?: string,
): Promise<{ episode: AuditEpisode; participation_fact: Fact } | null> {
  return apiSend(`/api/audit-subjects/${subjectId}/public-episodes`, {
    label,
    notes: notes ?? null,
  });
}

export async function joinPublicEpisode(
  episodeId: string,
  note?: string,
): Promise<Fact | null> {
  return apiSend(`/api/audit-episodes/${episodeId}/join`, {
    note: note ?? null,
  });
}

export async function submitFeaturePetition(
  episodeId: string,
  elementReviewFactId: string,
  rationale: string,
): Promise<Fact | null> {
  return apiSend(`/api/audit-episodes/${episodeId}/facts/feature-petition`, {
    element_review: elementReviewFactId,
    rationale,
  });
}

export async function submitCwePetition(
  episodeId: string,
  body: {
    kind: "new_element" | "applicability";
    cwe_node?: string;
    proposed_label?: string;
    rationale: string;
  },
): Promise<Fact | null> {
  return apiSend(`/api/audit-episodes/${episodeId}/facts/cwe-petition`, {
    kind: body.kind,
    cwe_node: body.cwe_node ?? null,
    proposed_label: body.proposed_label ?? null,
    rationale: body.rationale,
  });
}

export async function submitChallengeResponse(
  episodeId: string,
  respondingTo: string[],
  responseType: "accepts" | "contests" | "partially_accepts" | "revises_and_responds",
  content: string,
): Promise<Fact | null> {
  return apiSend(`/api/audit-episodes/${episodeId}/facts/submitter-response`, {
    responding_to: respondingTo,
    response_type: responseType,
    content,
  });
}

// ── Shared credentialed fetch helpers ───────────────────────────

async function apiGet<T>(endpoint: string): Promise<T | null> {
  try {
    const response = await fetch(`${apiBaseUrl}${endpoint}`, {
      cache: "no-store",
      credentials: "include",
    });

    if (!response.ok) {
      return null;
    }

    return (await response.json()) as T;
  } catch {
    return null;
  }
}

async function apiSend<T>(endpoint: string, body: unknown): Promise<T | null> {
  try {
    const response = await fetch(`${apiBaseUrl}${endpoint}`, {
      method: "POST",
      cache: "no-store",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      const detail = await response.json().catch(() => null);
      const message =
        detail && typeof detail === "object" && "error" in detail
          ? String((detail as { error: unknown }).error)
          : `request failed (${response.status})`;

      throw new Error(message);
    }

    return (await response.json()) as T;
  } catch (error) {
    if (error instanceof Error) {
      throw error;
    }

    return null;
  }
}

export async function submitElementReview(
  episodeId: string,
  request: CreateEpisodeElementReviewRequest,
): Promise<Fact | null> {
  return apiSend<Fact>(
    `/api/audit-episodes/${episodeId}/facts/element-review`,
    request,
  );
}

export async function submitSynthesisReview(
  episodeId: string,
  request: CreateSynthesisReviewRequest & { unsolicited?: boolean },
): Promise<SynthesisReview | null> {
  return apiSend<SynthesisReview>(
    `/api/audit-episodes/${episodeId}/synthesis-reviews`,
    request,
  );
}
