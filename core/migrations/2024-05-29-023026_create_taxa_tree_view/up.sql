CREATE MATERIALIZED VIEW taxa_tree AS
WITH RECURSIVE tree (
    taxon_id,
    taxon_scientific_name,
    taxon_canonical_name,
    path_id,
    path_scientific_name,
    path_canonical_name,
    id,
    parent_id,
    rank,
    scientific_name,
    canonical_name,
    depth,
    path
) AS (
    SELECT
        taxon_id,
        taxon_scientific_name,
        taxon_canonical_name,
        id AS path_id,
        scientific_name AS path_scientific_name,
        canonical_name AS path_canonical_name,
        id,
        parent_id,
        rank,
        scientific_name,
        canonical_name,
        0,
        ARRAY[id]
    FROM taxa_dag_down
UNION
    SELECT
        tree.taxon_id,
        tree.taxon_scientific_name,
        tree.taxon_canonical_name,
        tree.path_id,
        tree.path_scientific_name,
        tree.path_canonical_name,
        t.id,
        t.parent_id,
        t.rank,
        t.scientific_name,
        t.canonical_name,
        tree.depth + 1,
        path || t.id
    FROM tree, taxa t
    WHERE tree.parent_id = t.id
    AND NOT tree.taxon_id = ANY(path)
)
SELECT *
FROM tree
ORDER BY path_id, depth DESC;

COMMENT ON MATERIALIZED VIEW taxa_tree IS 'A denormalised, exhaustive tree containing all paths that descend from every taxon';
COMMENT ON COLUMN taxa_tree.taxon_id IS 'The root taxon that a descending tree is available for';
COMMENT ON COLUMN taxa_tree.path_id IS 'The taxon that this particular path starts from';

CREATE INDEX taxa_tree_taxon_id ON taxa_tree (taxon_id);
CREATE INDEX taxa_tree_path_id ON taxa_tree (path_id);
