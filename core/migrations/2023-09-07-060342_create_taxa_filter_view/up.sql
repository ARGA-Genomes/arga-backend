CREATE MATERIALIZED VIEW taxa_filter AS
SELECT
    taxa.*,
    classification_tree.hierarchy,
    classification_tree.classification,
    name_data_summaries.genomes,
    name_data_summaries.markers,
    name_data_summaries.specimens,
    name_data_summaries.other,
    ecology.values AS ecology,
    regions.ibra,
    regions.imcra,
    regions.state,
    regions.drainage_basin,
    name_attributes.traits
FROM taxa
JOIN name_data_summaries ON taxa.name_id = name_data_summaries.name_id
LEFT JOIN (
  SELECT
      taxon_id,
      array_agg(canonical_name::text) as hierarchy,
      jsonb_object_agg(rank, canonical_name) as classification
  FROM classification_dag
  GROUP BY taxon_id
) classification_tree ON taxa.parent_taxon_id = classification_tree.taxon_id
LEFT JOIN ecology ON taxa.name_id = ecology.name_id
LEFT JOIN (
  SELECT
    name_id,
    array_agg(value) filter (WHERE region_type = 'ibra') AS ibra,
    array_agg(value) filter (WHERE region_type = 'imcra') AS imcra,
    array_agg(value) filter (WHERE region_type = 'state') AS state,
    array_agg(value) filter (WHERE region_type = 'drainage_basin') AS drainage_basin
  FROM regions, unnest(values) AS value
  GROUP BY name_id
) regions ON taxa.name_id = regions.name_id
LEFT JOIN (
  SELECT
    name_id,
    array_agg(name::text) filter (WHERE value_type = 'boolean') AS traits
  FROM name_attributes
  GROUP BY name_id
) name_attributes ON taxa.name_id = name_attributes.name_id;


CREATE INDEX taxa_filter_ecology ON taxa_filter USING GIN(ecology);
