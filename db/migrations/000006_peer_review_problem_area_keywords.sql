UPDATE cwe_nodes
SET source_metadata = source_metadata || jsonb_build_object(
    'browse_keywords',
    jsonb_build_array(
        'method',
        'methods',
        'methodology',
        'study design',
        'protocol',
        'reproducibility',
        'replication'
    )
)
WHERE id = '00000000-0000-0000-0000-000000000601';

UPDATE cwe_nodes
SET source_metadata = source_metadata || jsonb_build_object(
    'browse_keywords',
    jsonb_build_array(
        'statistics',
        'statistical',
        'biostatistics',
        'uncertainty',
        'power',
        'confidence interval',
        'p value'
    )
)
WHERE id = '00000000-0000-0000-0000-000000000602';

UPDATE cwe_nodes
SET source_metadata = source_metadata || jsonb_build_object(
    'browse_keywords',
    jsonb_build_array(
        'data',
        'code',
        'materials',
        'availability',
        'open data',
        'repository',
        'reproducibility'
    )
)
WHERE id = '00000000-0000-0000-0000-000000000603';

UPDATE cwe_nodes
SET source_metadata = source_metadata || jsonb_build_object(
    'browse_keywords',
    jsonb_build_array(
        'interpretation',
        'conclusion',
        'claim strength',
        'causal claim',
        'inference',
        'evidence strength'
    )
)
WHERE id = '00000000-0000-0000-0000-000000000604';

UPDATE cwe_nodes
SET source_metadata = source_metadata || jsonb_build_object(
    'browse_keywords',
    jsonb_build_array(
        'ethics',
        'ethical',
        'consent',
        'privacy',
        'risk',
        'harm',
        'equity'
    )
)
WHERE id = '00000000-0000-0000-0000-000000000605';
