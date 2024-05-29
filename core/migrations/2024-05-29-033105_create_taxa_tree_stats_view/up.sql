CREATE MATERIALIZED VIEW taxa_tree_stats AS
SELECT
    taxon_id,
    id,
    scientific_name,
    rank,
    SUM(markers) AS markers,
    SUM(genomes) AS genomes,
    SUM(specimens) AS specimens,
    SUM(other) AS other,
    SUM(total_genomic) AS total_genomic
FROM (
    SELECT taxa_tree.taxon_id, id, taxa_tree.scientific_name, taxa_tree.rank,
        FIRST_VALUE(markers) OVER tree_paths AS markers,
        FIRST_VALUE(genomes) OVER tree_paths AS genomes,
        FIRST_VALUE(specimens) OVER tree_paths AS specimens,
        FIRST_VALUE(other) OVER tree_paths AS other,
        FIRST_VALUE(total_genomic) OVER tree_paths AS total_genomic
    FROM taxa_tree
    -- a taxon can have multiple alternate names so we group them
    -- up and sum it here otherwise it will cause double counting
    LEFT JOIN (
        SELECT
            taxon_id,
            SUM(markers) AS markers,
            SUM(genomes) AS genomes,
            SUM(specimens) AS specimens,
            SUM(other) AS other,
            SUM(total_genomic) AS total_genomic
        FROM name_data_summaries
        JOIN taxon_names ON taxon_names.name_id = name_data_summaries.name_id
        GROUP BY taxon_id
    ) summed ON summed.taxon_id = taxa_tree.id
    WINDOW tree_paths AS (partition by path_id order by depth)
    ORDER BY path_id, depth
) taxon_stats
GROUP BY taxon_id, id, scientific_name, rank;

CREATE INDEX taxa_tree_stats_taxon_id ON taxa_tree_stats (taxon_id);
