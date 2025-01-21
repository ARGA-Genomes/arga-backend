-- drop dependent views and recreate them all
DROP MATERIALIZED VIEW taxa_tree_stats;
DROP MATERIALIZED VIEW species;
DROP MATERIALIZED VIEW name_data_summaries;



CREATE MATERIALIZED VIEW name_data_summaries AS
SELECT
    names.id AS name_id,
    COALESCE(markers.total, 0) AS markers,
    COALESCE(genomes.total, 0) AS genomes,
    COALESCE(specimens.total, 0) AS specimens,
    COALESCE(other_data.total, 0) AS other,
    COALESCE(markers.total, 0) + COALESCE(genomes.total, 0) + COALESCE(other_data.total, 0) AS total_genomic
FROM names
LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM sequencing_events
     JOIN sequences ON sequencing_events.sequence_id = sequences.id
     WHERE target_gene IS NOT NULL
     GROUP BY name_id
) markers ON markers.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     GROUP BY name_id
) genomes ON genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM specimens
     GROUP BY name_id
) specimens ON specimens.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int as total
     FROM sequences
     LEFT JOIN sequencing_events se on sequences.id = se.sequence_id
     LEFT JOIN assembly_events on sequences.id = assembly_events.sequence_id
     LEFT JOIN annotation_events ae on sequences.id = ae.sequence_id
     WHERE assembly_events.id IS NULL AND target_gene IS NULL
     GROUP BY name_id
) other_data ON other_data.name_id = names.id;




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




CREATE MATERIALIZED VIEW species AS
SELECT
    taxa.id,
    taxa.scientific_name,
    taxa.canonical_name,
    taxa.authorship,
    taxa.dataset_id,
    taxa.status,
    taxa.rank,
    taxa_tree.classification,
    summaries.genomes,
    summaries.loci,
    summaries.specimens,
    summaries.other,
    summaries.total_genomic,
    name_attributes.traits,
    vernacular_names.names AS vernacular_names
FROM taxa
JOIN (
  SELECT
      taxon_id,
      jsonb_object_agg(rank, canonical_name) AS classification
  FROM taxa_dag
  GROUP BY taxon_id
) taxa_tree ON taxa.parent_id = taxa_tree.taxon_id
JOIN (
  SELECT
      taxon_id,
      SUM(genomes) AS genomes,
      SUM(markers) AS loci,
      SUM(specimens) AS specimens,
      SUM(other) AS other,
      SUM(total_genomic) AS total_genomic
  FROM name_data_summaries
  JOIN taxon_names ON taxon_names.name_id = name_data_summaries.name_id
  GROUP BY taxon_id
) summaries ON taxa.id = summaries.taxon_id
LEFT JOIN (
  SELECT
    taxon_id,
    array_agg(name::text) filter (WHERE value_type = 'boolean') AS traits
  FROM name_attributes
  JOIN taxon_names ON taxon_names.name_id = name_attributes.name_id
  GROUP BY taxon_id
) name_attributes ON taxa.id = name_attributes.taxon_id
LEFT JOIN (
  SELECT
    taxon_id,
    array_agg(DISTINCT vernacular_name) as names
  FROM vernacular_names
  JOIN taxon_names ON taxon_names.name_id = vernacular_names.name_id
  GROUP BY taxon_id
) vernacular_names ON taxa.id = vernacular_names.taxon_id;
