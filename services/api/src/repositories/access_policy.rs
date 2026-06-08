pub fn doi_native_display_permitted(
    is_open_access: bool,
    license: Option<&str>,
    pdf_url: Option<&str>,
) -> bool {
    is_open_access && pdf_url.is_some() && has_trusted_open_license(license)
}

pub fn arxiv_native_display_permitted(canonical_url: &str, pdf_url: Option<&str>) -> bool {
    is_trusted_arxiv_url(canonical_url) || pdf_url.is_some_and(is_trusted_arxiv_url)
}

pub fn pmc_native_display_permitted(full_text_url: Option<&str>) -> bool {
    full_text_url.is_some_and(is_trusted_pmc_url)
}

pub fn has_trusted_open_license(license: Option<&str>) -> bool {
    let Some(license) = license else {
        return false;
    };
    let normalized = normalize_license(license);

    matches!(
        normalized.as_str(),
        "cc-by" | "cc0" | "cc-by-sa" | "public-domain" | "public domain"
    ) || normalized.contains("creativecommons.org/licenses/by/")
        || normalized.contains("creativecommons.org/licenses/by-sa/")
        || normalized.contains("creativecommons.org/publicdomain/zero/")
}

fn is_trusted_arxiv_url(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();

    normalized.starts_with("https://arxiv.org/")
        || normalized.starts_with("http://arxiv.org/")
        || normalized.starts_with("https://export.arxiv.org/")
        || normalized.starts_with("http://export.arxiv.org/")
}

fn is_trusted_pmc_url(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();

    normalized.starts_with("https://pmc.ncbi.nlm.nih.gov/articles/pmc")
        || normalized.starts_with("https://www.ncbi.nlm.nih.gov/pmc/articles/pmc")
}

fn normalize_license(value: &str) -> String {
    value
        .trim()
        .trim_end_matches('/')
        .to_ascii_lowercase()
        .replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::{
        arxiv_native_display_permitted, doi_native_display_permitted, has_trusted_open_license,
        pmc_native_display_permitted,
    };

    #[test]
    fn accepts_known_open_licenses() {
        assert!(has_trusted_open_license(Some("cc-by")));
        assert!(has_trusted_open_license(Some(
            "https://creativecommons.org/licenses/by/4.0/"
        )));
        assert!(has_trusted_open_license(Some("CC0")));
        assert!(has_trusted_open_license(Some("cc-by-sa")));
    }

    #[test]
    fn rejects_unclear_or_restricted_licenses() {
        assert!(!has_trusted_open_license(None));
        assert!(!has_trusted_open_license(Some("cc-by-nc")));
        assert!(!has_trusted_open_license(Some(
            "http://www.liebertpub.com/nv/resources-tools/text-and-data-mining-policy/121/"
        )));
    }

    #[test]
    fn doi_native_display_requires_open_access_license_and_pdf() {
        assert!(doi_native_display_permitted(
            true,
            Some("cc-by"),
            Some("https://example.org/article.pdf")
        ));
        assert!(!doi_native_display_permitted(
            true,
            None,
            Some("https://example.org/article.pdf")
        ));
        assert!(!doi_native_display_permitted(
            false,
            Some("cc-by"),
            Some("https://example.org/article.pdf")
        ));
        assert!(!doi_native_display_permitted(true, Some("cc-by"), None));
    }

    #[test]
    fn repository_permissions_require_trusted_hosts() {
        assert!(arxiv_native_display_permitted(
            "https://arxiv.org/abs/2501.12345",
            None
        ));
        assert!(!arxiv_native_display_permitted(
            "https://example.org/abs/2501.12345",
            None
        ));
        assert!(pmc_native_display_permitted(Some(
            "https://pmc.ncbi.nlm.nih.gov/articles/PMC1234567/"
        )));
        assert!(!pmc_native_display_permitted(Some(
            "https://example.org/articles/PMC1234567/"
        )));
    }
}
