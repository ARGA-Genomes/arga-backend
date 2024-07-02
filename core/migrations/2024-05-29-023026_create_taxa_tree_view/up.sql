CREATE MATERIALIZED VIEW taxa_tree AS
WITH RECURSIVE tree (
    taxon_id,
    path_id,
    id,
    parent_id,
    depth,
    path
) AS (
    SELECT
        taxon_id,
        id AS path_id,
        id,
        parent_id,
        0,
        ARRAY[id]
    FROM taxa_dag_down
UNION
    SELECT
        tree.taxon_id,
        tree.path_id,
        t.id,
        t.parent_id,
        tree.depth + 1,
        path || t.id
    FROM tree, taxa t
    WHERE tree.parent_id = t.id
    AND NOT tree.taxon_id = ANY(path)
)
SELECT taxon_id, path_id, id, parent_id, depth
FROM tree
ORDER BY path_id, depth DESC;

COMMENT ON MATERIALIZED VIEW taxa_tree IS 'A denormalised, exhaustive tree containing all paths that descend from every taxon';
COMMENT ON COLUMN taxa_tree.taxon_id IS 'The root taxon that a descending tree is available for';
COMMENT ON COLUMN taxa_tree.path_id IS 'The taxon that this particular path starts from';

CREATE INDEX taxa_tree_taxon_id ON taxa_tree (taxon_id);
CREATE INDEX taxa_tree_path_id ON taxa_tree (path_id);
