UPDATE scholarly_objects
SET
    native_display_permitted = CASE
        WHEN lower(trim(both '/' FROM COALESCE(license, ''))) IN (
            'cc-by',
            'cc0',
            'cc-by-sa',
            'public-domain',
            'public domain'
        ) THEN true
        WHEN lower(COALESCE(license, '')) LIKE '%creativecommons.org/licenses/by/%' THEN true
        WHEN lower(COALESCE(license, '')) LIKE '%creativecommons.org/licenses/by-sa/%' THEN true
        WHEN lower(COALESCE(license, '')) LIKE '%creativecommons.org/publicdomain/zero/%' THEN true
        WHEN metadata_provenance->>'source' = 'arxiv' THEN true
        WHEN EXISTS (
            SELECT 1
            FROM external_article_locations
            WHERE external_article_locations.scholarly_object_id = scholarly_objects.id
              AND external_article_locations.location_type = 'full_text'
              AND lower(external_article_locations.url) LIKE 'https://pmc.ncbi.nlm.nih.gov/articles/pmc%'
        ) THEN true
        ELSE false
    END,
    updated_at = now()
WHERE native_display_permitted = true
   OR EXISTS (
       SELECT 1
       FROM external_article_locations
       WHERE external_article_locations.scholarly_object_id = scholarly_objects.id
         AND external_article_locations.location_type IN ('pdf', 'full_text')
   );
