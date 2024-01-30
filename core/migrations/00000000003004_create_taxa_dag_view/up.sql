CREATE MATERIALIZED VIEW taxa_dag AS
WITH RECURSIVE dag(
    taxon_id,
    taxon_scientific_name,
    taxon_canonical_name,
    id,
    parent_id,
    rank,
    scientific_name,
    canonical_name,
    depth,
    is_cycle,
    path
) AS (
    SELECT
        id AS taxon_id,
        scientific_name AS taxon_scientific_name,
        canonical_name AS taxon_canonical_name,
        id,
        parent_id,
        rank,
        scientific_name,
        canonical_name,
        0,
        false,
        ARRAY[id]
    FROM taxa
UNION
    SELECT
        dag.taxon_id,
        dag.taxon_scientific_name,
        dag.taxon_canonical_name,
        t.id,
        t.parent_id,
        t.rank,
        t.scientific_name,
        t.canonical_name,
        dag.depth + 1,
        t.id = ANY(path),
        path || t.id
    FROM dag, taxa t
    WHERE dag.parent_id = t.id
      AND dag.id != dag.parent_id
      AND dag.parent_id IS NOT NULL
      AND NOT is_cycle
)
SELECT taxon_id, taxon_scientific_name, taxon_canonical_name, id, parent_id, rank, scientific_name, canonical_name, depth
FROM dag
ORDER BY taxon_id ASC, depth ASC;

CREATE INDEX taxa_dag_id ON taxa_dag (taxon_id);
