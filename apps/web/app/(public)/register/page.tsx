import { cookies } from "next/headers";
import Link from "next/link";

import { RegistryUnavailable } from "../../components/registry-unavailable";
import {
  formatLabel,
  getSession,
  isApiReachable,
  retrieveArticle,
  searchScholarlyObjects,
} from "../../lib/csqd-api";
import {
  groupScholarlyObjects,
  publicAuditStatusLabel,
  versionKindForObject,
} from "../../lib/public-audit";
import { StatusPill } from "../../components/status-pill";

type PageProps = {
  searchParams: Promise<{
    include_preprints?: string;
    q?: string;
  }>;
};

/// Contribution ramp: search local records and external sources (DOI, arXiv,
/// PubMed, title), pull missing metadata in, and land on the work's public
/// audit record. Registration creates or reuses the durable AuditSubject.
export default async function RegisterPage({ searchParams }: PageProps) {
  const { include_preprints, q } = await searchParams;
  const query = q?.trim() ?? "";
  const includePreprints = isCheckedSearchParam(include_preprints);
  let objects = query ? await searchScholarlyObjects(query) : [];
  let groups = groupScholarlyObjects(objects);
  let retrievalError: string | null = null;
  let needsSignInForRetrieval = false;
  const shouldRetrieve =
    query &&
    (groups.length === 0 ||
      (includePreprints &&
        !groups.some((group) =>
          group.versions.some(
            (version) => versionKindForObject(version) === "preprint",
          ),
        )));

  if (shouldRetrieve) {
    // External retrieval registers durable audit subjects, so the API
    // requires an identified session; forward the request cookies.
    const cookieStore = await cookies();
    const cookieHeader = cookieStore
      .getAll()
      .map((cookie) => `${cookie.name}=${cookie.value}`)
      .join("; ");
    const session = await getSession(cookieHeader);

    if (!session) {
      needsSignInForRetrieval = true;
    } else {
      const retrieval = await retrieveArticle(
        query,
        { includePreprints },
        cookieHeader,
      );

      if (retrieval.error) {
        retrievalError = retrieval.error;
      } else {
        objects = await searchScholarlyObjects(query);
        groups = groupScholarlyObjects(objects);
      }
    }
  }

  const registryDown = groups.length === 0 && !(await isApiReachable());

  return (
    <>
      <header className="pub-page-head">
        <div>
          <p className="pub-kicker">Academic Peer Review</p>
          <h1>Register a Scholarly Work</h1>
          <p>
            Search existing records and external sources. Missing works are
            retrieved and registered as durable audit subjects — the entry ramp
            into the public audit graph.
          </p>
        </div>
        <Link className="secondary-action" href="/discover">
          Back to Discover
        </Link>
      </header>

      <form action="/register" className="pub-filterbar">
        <label>
          Work
          <input
            defaultValue={query}
            name="q"
            placeholder="Title, DOI, PMID, PMCID, arXiv ID, article URL, author, or venue"
            type="search"
          />
        </label>
        <label className="pub-checkbox-inline">
          <input
            defaultChecked={includePreprints}
            name="include_preprints"
            type="checkbox"
            value="true"
          />
          Include matching preprints
        </label>
        <button type="submit">Search</button>
      </form>

      {groups.length > 0 ? (
        <div className="pub-stat-strip">
          <span>
            <strong>{groups.length}</strong> matching works
          </span>
          <span>
            <strong>{objects.length}</strong> versions
          </span>
        </div>
      ) : null}

      {registryDown ? (
        <RegistryUnavailable />
      ) : needsSignInForRetrieval && groups.length === 0 ? (
        <div className="pub-empty">
          <h3>Sign in to retrieve this work</h3>
          <p>
            No local record matched. Retrieving from external sources (DOI,
            arXiv, PubMed, title search) registers a durable audit subject,
            so it requires a signed-in identity. Local search stays public.
          </p>
          <Link
            className="primary-action"
            href={`/sign-in?return_to=${encodeURIComponent(
              `/register?q=${encodeURIComponent(query)}${includePreprints ? "&include_preprints=true" : ""}`,
            )}`}
          >
            Sign in to continue
          </Link>
        </div>
      ) : groups.length === 0 ? (
        <div className="pub-empty">
          <h3>{query ? "No work found" : "Search scholarly works"}</h3>
          <p>
            {query
              ? (retrievalError ??
                "No local or retrievable record matched this query.")
              : "Use a title, DOI, PMID, PMCID, arXiv ID, article URL, author, or venue."}
          </p>
        </div>
      ) : (
        <div className="pub-grid">
          {groups.map((group) => {
            const object = group.primaryVersion;
            const subjectId = object.audit_subject_id;

            return (
              <article className="pub-card" key={group.id}>
                <div className="pub-card-kicker">
                  <StatusPill status={publicAuditStatusLabel(object, null)} />
                  <span>
                    {group.versionCount} version{group.versionCount === 1 ? "" : "s"}
                  </span>
                  {object.publication_year ? <span>{object.publication_year}</span> : null}
                </div>
                <h3>
                  <Link href={`/works/${object.id}`}>{group.title}</Link>
                </h3>
                <p className="pub-source-line">{object.authors.join(", ")}</p>
                <div className="pub-version-list">
                  {group.versions.map((version) => (
                    <Link
                      className="pub-version-row"
                      href={`/works/${version.id}`}
                      key={version.id}
                    >
                      <strong>{formatLabel(versionKindForObject(version))}</strong>
                      <span>
                        {version.source_name}
                        {version.publication_year
                          ? ` · ${version.publication_year}`
                          : ""}
                      </span>
                    </Link>
                  ))}
                </div>
                <div className="pub-card-actions">
                  <Link href={`/works/${object.id}`}>Open public audit record</Link>
                  {subjectId ? (
                    <Link href={`/commission?subject_id=${subjectId}`}>
                      Commission audit
                    </Link>
                  ) : null}
                </div>
              </article>
            );
          })}
        </div>
      )}
    </>
  );
}

function isCheckedSearchParam(value: string | undefined) {
  return value === "true" || value === "on" || value === "1";
}
