CREATE MATERIALIZED VIEW taxa_tree_stats AS
SELECT
    taxon_id,
    taxon_stats.id,
    SUM(loci) AS loci,
    SUM(genomes) AS genomes,
    SUM(specimens) AS specimens,
    SUM(other) AS other,
    SUM(total_genomic) AS total_genomic,
    SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END) AS species
FROM (
    SELECT
        taxa_tree.taxon_id,
        id,
        path_id,
        FIRST_VALUE(loci) OVER tree_paths AS loci,
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
            SUM(markers) AS loci,
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
JOIN taxa ON taxon_stats.path_id = taxa.id
GROUP BY taxon_id, taxon_stats.id;

CREATE INDEX taxa_tree_stats_taxon_id ON taxa_tree_stats (taxon_id);
CREATE INDEX taxa_tree_stats_id ON taxa_tree_stats (id);
