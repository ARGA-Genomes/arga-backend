CREATE MATERIALIZED VIEW taxa_tree_stats AS
-- combines the name_data_summaries of names that are linked to one another.
-- this allows us to present data from dark taxa
WITH taxa_data_summaries AS (
     SELECT
        taxon_id,
        SUM(markers) AS loci,
        SUM(genomes) AS genomes,
        SUM(specimens) AS specimens,
        SUM(other) AS other,
        SUM(total_genomic) AS total_genomic,
        SUM(complete_genomes) AS complete_genomes,
        SUM(partial_genomes) AS partial_genomes,
        SUM(assembly_chromosomes) AS assembly_chromosomes,
        SUM(assembly_scaffolds) AS assembly_scaffolds,
        SUM(assembly_contigs) AS assembly_contigs
    FROM name_data_summaries
    JOIN taxon_names ON taxon_names.name_id = name_data_summaries.name_id
    GROUP BY taxon_id
),
-- the linked name_data_summaries joined with the taxa_tree to get stats that
-- aggregate up the taxa hierarchy for each taxon
taxon_stats AS (
    SELECT
        taxa_tree.taxon_id,
        id,
        path_id,
        depth,
        FIRST_VALUE(loci) OVER tree_paths AS loci,
        FIRST_VALUE(genomes) OVER tree_paths AS genomes,
        FIRST_VALUE(specimens) OVER tree_paths AS specimens,
        FIRST_VALUE(other) OVER tree_paths AS other,
        FIRST_VALUE(total_genomic) OVER tree_paths AS total_genomic,
        FIRST_VALUE(complete_genomes) OVER tree_paths AS complete_genomes,
        FIRST_VALUE(partial_genomes) OVER tree_paths AS partial_genomes,
        FIRST_VALUE(assembly_chromosomes) OVER tree_paths AS assembly_chromosomes,
        FIRST_VALUE(assembly_scaffolds) OVER tree_paths AS assembly_scaffolds,
        FIRST_VALUE(assembly_contigs) OVER tree_paths AS assembly_contigs
    FROM taxa_tree
    -- a taxon can have multiple alternate names so we group them
    -- up and sum it here otherwise it will cause double counting
    LEFT JOIN taxa_data_summaries ON taxa_data_summaries.taxon_id = taxa_tree.id
    WINDOW tree_paths AS (partition by path_id order by depth)
    ORDER BY path_id, depth
),
-- the grouped up stats for higher taxonomy. it joins the taxon_stats on the
-- path_id to get the accumulated amounts of all descendents and then groups
-- by the taxon id itself to ensure that all paths are folded in to the parent taxon
stats AS (
    SELECT
        taxon_id,
        taxon_stats.id,
        MAX(depth) AS tree_depth,
        SUM(loci) AS loci,
        SUM(genomes) AS genomes,
        SUM(specimens) AS specimens,
        SUM(other) AS other,
        SUM(total_genomic) AS total_genomic,
        SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END) AS species,
        SUM(complete_genomes) AS complete_genomes,
        SUM(partial_genomes) AS partial_genomes,
        SUM(assembly_chromosomes) AS assembly_chromosomes,
        SUM(assembly_scaffolds) AS assembly_scaffolds,
        SUM(assembly_contigs) AS assembly_contigs
    FROM taxon_stats
    JOIN taxa ON taxon_stats.path_id = taxa.id
    GROUP BY taxon_id, taxon_stats.id
)
-- the main query. simply join the tree stats with the taxon and sum the stats for each node
SELECT
    stats.*,
    CASE WHEN species > 0 THEN (complete_genomes / species) ELSE 0 END AS complete_genomes_coverage,
    CASE WHEN species > 0 THEN (partial_genomes / species) ELSE 0 END AS partial_genomes_coverageb,
    CASE WHEN species > 0 THEN (assembly_chromosomes / species) ELSE 0 END AS assembly_chromosomes_coverage,
    CASE WHEN species > 0 THEN (assembly_scaffolds / species) ELSE 0 END AS assembly_scaffolds_coverage,
    CASE WHEN species > 0 THEN (assembly_contigs / species) ELSE 0 END AS assembly_contigs_coverage
FROM stats
;

CREATE INDEX taxa_tree_stats_taxon_id ON taxa_tree_stats (taxon_id);
CREATE INDEX taxa_tree_stats_id ON taxa_tree_stats (id);
