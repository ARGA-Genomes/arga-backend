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
        0,         -- depth
        false,     -- is_cycle
        ARRAY[id]  -- path
    FROM taxa
UNION
    -- for each row that the intermediate table spits out we join
    -- on all taxa that has a parent_id matching the rows that were output.
    -- this lets us go down the tree as each time we will output more rows
    -- until we reach the leafs (the row has no other rows linking to it via parent_id).
    SELECT
        dag.taxon_id,
        t.id,
        t.parent_id,
        dag.depth + 1,     -- depth
        t.id = ANY(path),  -- is_cycle
        path || t.id       -- path
    FROM dag, taxa t
    WHERE dag.id = t.parent_id
      -- because we are traversing down the tree we don't need to check for a terminus such as a parent
      -- null check. instead we just want to make sure we aren't infinitely trying to traverse the root
      -- so we only do a cyclic check.
      AND NOT is_cycle
)
SELECT taxon_id, id, parent_id, depth
FROM dag
ORDER BY taxon_id ASC, depth ASC;

COMMENT ON MATERIALIZED VIEW taxa_dag_down IS 'A denormalised graph of all descendents for every taxon';

-- index on the 'query' column. this is how most queries are going to hit the view. specifically to get a list of
-- descendant nodes for a particular taxon
CREATE INDEX taxa_dag_down_taxon_id ON taxa_dag_down (taxon_id);

-- because the underlying taxa tree is a DAG we know that a taxon can only ever appear once for each taxon_id 'query'.
-- by creating a uniqueness constraint on taxon_id and id we can concurrently update the tree without locking the table
CREATE UNIQUE INDEX taxa_dag_down_taxon_id_id ON taxa_dag_down (taxon_id, id, depth);
