CREATE MATERIALIZED VIEW taxa_filter AS
SELECT
    taxa.*,
    ecology.values AS ecology,
    regions.ibra,
    regions.imcra,
    regions.state,
    regions.drainage_basin
FROM taxa
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
) regions ON taxa.name_id = regions.name_id;

CREATE INDEX taxa_filter_ecology ON taxa_filter USING GIN(ecology);
