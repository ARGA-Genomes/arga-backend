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

        SUM(full_genomes) AS full_genomes,
        SUM(partial_genomes) AS partial_genomes,
        SUM(complete_genomes) AS complete_genomes,
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
        -- if the node is the second taxon in the path then its the direct parent of
        -- leaf node, so we use the value of 1 to allow summing when grouped. this allows
        -- us to determine how many direct children each node has
        CASE WHEN depth = 1 THEN 1 ELSE 0 END AS direct_parent,
        -- pull the the values from the name data summaries
        FIRST_VALUE(loci) OVER tree_paths AS loci,
        FIRST_VALUE(genomes) OVER tree_paths AS genomes,
        FIRST_VALUE(specimens) OVER tree_paths AS specimens,
        FIRST_VALUE(other) OVER tree_paths AS other,
        FIRST_VALUE(total_genomic) OVER tree_paths AS total_genomic,
        FIRST_VALUE(full_genomes) OVER tree_paths AS full_genomes,
        FIRST_VALUE(partial_genomes) OVER tree_paths AS partial_genomes,
        FIRST_VALUE(complete_genomes) OVER tree_paths AS complete_genomes,
        FIRST_VALUE(assembly_chromosomes) OVER tree_paths AS assembly_chromosomes,
        FIRST_VALUE(assembly_scaffolds) OVER tree_paths AS assembly_scaffolds,
        FIRST_VALUE(assembly_contigs) OVER tree_paths AS assembly_contigs,
        -- base values for coverage stats. if there is at least one type of data we consider
        -- it full coverage for the node. this is useful further on when summarising a node
        -- and comparing the total coverage against the amount of children to determine coverage
        -- for a node at any part of the hierarchy without losing that information to aggregation
        CASE WHEN FIRST_VALUE(full_genomes) OVER tree_paths > 0 THEN 1 ELSE 0 END AS full_genomes_coverage,
        CASE WHEN FIRST_VALUE(partial_genomes) OVER tree_paths > 0 THEN 1 ELSE 0 END AS partial_genomes_coverage,
        CASE WHEN FIRST_VALUE(complete_genomes) OVER tree_paths > 0 THEN 1 ELSE 0 END AS complete_genomes_coverage,
        CASE WHEN FIRST_VALUE(assembly_chromosomes) OVER tree_paths > 0 THEN 1 ELSE 0 END AS assembly_chromosomes_coverage,
        CASE WHEN FIRST_VALUE(assembly_scaffolds) OVER tree_paths > 0 THEN 1 ELSE 0 END AS assembly_scaffolds_coverage,
        CASE WHEN FIRST_VALUE(assembly_contigs) OVER tree_paths > 0 THEN 1 ELSE 0 END AS assembly_contigs_coverage
    FROM taxa_tree
    -- a taxon can have multiple alternate names so we group them
    -- up and sum it here otherwise it will cause double counting
    LEFT JOIN taxa_data_summaries ON taxa_data_summaries.taxon_id = taxa_tree.id
    WINDOW tree_paths AS (partition BY path_id ORDER BY depth)
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
        SUM(direct_parent) AS children,
        COUNT(*) - 1 AS descendants,
        SUM(loci) AS loci,
        SUM(genomes) AS genomes,
        SUM(specimens) AS specimens,
        SUM(other) AS other,
        SUM(total_genomic) AS total_genomic,
        SUM(CASE WHEN taxa.rank='species' THEN 1 ELSE 0 END) AS species,
        SUM(full_genomes) AS full_genomes,
        SUM(partial_genomes) AS partial_genomes,
        SUM(complete_genomes) AS complete_genomes,
        SUM(assembly_chromosomes) AS assembly_chromosomes,
        SUM(assembly_scaffolds) AS assembly_scaffolds,
        SUM(assembly_contigs) AS assembly_contigs,
        -- sum up all the coverage for the node and divide it by the amount of children to determine
        -- the total coverage for this specific node
        SUM(full_genomes_coverage) AS total_full_genomes_coverage,
        SUM(partial_genomes_coverage) AS total_partial_genomes_coverage,
        SUM(complete_genomes_coverage) AS total_complete_genomes_coverage,
        SUM(assembly_chromosomes_coverage) AS total_assembly_chromosomes_coverage,
        SUM(assembly_scaffolds_coverage) AS total_assembly_scaffolds_coverage,
        SUM(assembly_contigs_coverage) AS total_assembly_contigs_coverage
    FROM taxon_stats
    JOIN taxa ON taxon_stats.path_id = taxa.id
    GROUP BY taxon_id, taxon_stats.id
)
-- the main query. simply join the tree stats with the taxon and sum the stats for each node
SELECT * FROM stats;

CREATE INDEX taxa_tree_stats_taxon_id ON taxa_tree_stats (taxon_id);
CREATE INDEX taxa_tree_stats_id ON taxa_tree_stats (id);
CREATE UNIQUE INDEX taxa_tree_stats_id_taxon_id ON taxa_tree_stats (id, taxon_id);
