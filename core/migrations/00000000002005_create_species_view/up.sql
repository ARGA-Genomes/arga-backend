CREATE MATERIALIZED VIEW species AS
SELECT
    names.*,
    name_data_summaries.genomes,
    name_data_summaries.markers AS loci,
    name_data_summaries.specimens,
    name_data_summaries.other,
    name_data_summaries.total_genomic,
    taxa.dataset_id AS taxon_dataset_id,
    taxa.status AS taxon_status,
    taxa.rank AS taxon_rank,
    taxa_tree.taxon_id,
    taxa_tree.classification,
    name_attributes.traits
FROM names
JOIN name_data_summaries ON names.id = name_data_summaries.name_id
JOIN taxon_names ON names.id = taxon_names.name_id
JOIN taxa ON taxon_names.taxon_id = taxa.id
JOIN (
  SELECT
      taxon_id,
      jsonb_object_agg(rank, canonical_name) AS classification
  FROM taxa_dag
  GROUP BY taxon_id
) taxa_tree ON taxa.parent_id = taxa_tree.taxon_id
LEFT JOIN (
  SELECT
    name_id,
    array_agg(name::text) filter (WHERE value_type = 'boolean') AS traits
  FROM name_attributes
  GROUP BY name_id
) name_attributes ON names.id = name_attributes.name_id;
