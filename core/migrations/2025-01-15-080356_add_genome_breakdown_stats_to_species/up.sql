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
    summaries.full_genomes,
    summaries.partial_genomes,
    summaries.complete_genomes,
    summaries.assembly_chromosomes,
    summaries.assembly_scaffolds,
    summaries.assembly_contigs,
    name_attributes.traits,
    name_attributes.attributes,
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
) summaries ON taxa.id = summaries.taxon_id
LEFT JOIN (
  SELECT
    taxon_id,
    array_agg(name::text) filter (WHERE value_type = 'boolean') AS traits,
    jsonb_agg(CASE
     WHEN value_type = 'boolean' THEN jsonb_build_object('name', name, 'value', value_bool)
     WHEN value_type = 'string' THEN jsonb_build_object('name', name, 'value', value_str)
     WHEN value_type = 'integer' THEN jsonb_build_object('name', name, 'value', value_int)
     WHEN value_type = 'decimal' THEN jsonb_build_object('name', name, 'value', value_decimal)
     WHEN value_type = 'timestamp' THEN jsonb_build_object('name', name, 'value', value_timestamp)
    END) AS attributes
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
