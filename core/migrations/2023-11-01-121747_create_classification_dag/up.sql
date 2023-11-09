CREATE MATERIALIZED VIEW classification_dag AS
WITH RECURSIVE dag(
    taxon_id,
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
        id,
        parent_id,
        rank,
        scientific_name,
        canonical_name,
        0,
        false,
        ARRAY[id]
    FROM classifications
UNION
    SELECT
        dag.taxon_id,
        c.id,
        c.parent_id,
        c.rank,
        c.scientific_name,
        c.canonical_name,
        dag.depth + 1,
        c.id = ANY(path),
        path || c.id
    FROM dag, classifications c
    WHERE dag.parent_id = c.id
      AND dag.id != dag.parent_id
      AND NOT is_cycle
)
SELECT taxon_id, id, parent_id, rank, scientific_name, canonical_name, depth
FROM dag
ORDER BY taxon_id ASC, depth ASC;

CREATE INDEX classification_dag_id ON classification_dag (taxon_id);
