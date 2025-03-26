DROP MATERIALIZED VIEW overview;
CREATE MATERIALIZED VIEW overview AS
SELECT 'data_type' AS category, 'sequences' AS name, count(*) AS total FROM sequences
UNION ALL SELECT 'data_type' AS category, 'whole_genomes' AS name, count(*) AS total FROM whole_genomes
UNION ALL SELECT 'data_type' AS category, 'loci' AS name, count(*) AS total FROM markers
UNION ALL SELECT 'data_type' AS category, 'specimens' AS name, count(*) AS total FROM specimens

UNION ALL

SELECT 'source' AS category, sources.name, count(distinct name_id) as total FROM sources
LEFT JOIN datasets ON source_id=sources.id
LEFT JOIN name_attributes ON datasets.id = name_attributes.dataset_id
GROUP BY sources.name

UNION ALL

SELECT 'dataset' AS category, datasets.name, count(*) AS total FROM name_attributes
JOIN datasets ON name_attributes.dataset_id = datasets.id
WHERE name_attributes.name = 'last_updated'
GROUP BY datasets.name
