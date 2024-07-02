CREATE MATERIALIZED VIEW taxa_dag_down AS
WITH RECURSIVE dag(
    taxon_id,
    id,
    parent_id,
    depth,
    is_cycle,
    path
) AS (
    SELECT
        id AS taxon_id,
        id,
        parent_id,
        0,
        false,
        ARRAY[id]
    FROM taxa
UNION
    SELECT
        dag.taxon_id,
        t.id,
        t.parent_id,
        dag.depth + 1,
        t.id = ANY(path),
        path || t.id
    FROM dag, taxa t
    WHERE dag.id = t.parent_id
      AND dag.id != dag.parent_id
      AND dag.parent_id IS NOT NULL
      AND NOT is_cycle
)
SELECT taxon_id, id, parent_id, depth
FROM dag
ORDER BY taxon_id ASC, depth ASC;

COMMENT ON MATERIALIZED VIEW taxa_dag_down IS 'A denormalised graph of all descendents for every taxon';

CREATE INDEX taxa_dag_down_taxon_id ON taxa_dag_down (taxon_id);
