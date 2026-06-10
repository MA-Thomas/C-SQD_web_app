import { redirect } from "next/navigation";

type PageProps = {
  searchParams: Promise<{
    include_preprints?: string;
    q?: string;
  }>;
};

export default async function RetrieveRedirectPage({ searchParams }: PageProps) {
  const { include_preprints, q } = await searchParams;
  const params = new URLSearchParams();

  if (q) {
    params.set("q", q);
  }

  if (include_preprints) {
    params.set("include_preprints", include_preprints);
  }

  const query = params.toString();

  redirect(query ? `/intake?${query}` : "/intake");
}
