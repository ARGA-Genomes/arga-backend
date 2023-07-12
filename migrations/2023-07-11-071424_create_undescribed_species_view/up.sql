CREATE VIEW undescribed_species AS
SELECT
  genus,
  genus_authority,
  names
FROM (
  SELECT
    *,
    array_agg(scientific_name) OVER (PARTITION BY genus, genus_authority ORDER BY scientific_name ASC) AS names,
    rank() OVER (partition BY genus, genus_authority ORDER BY scientific_name DESC) AS window_rank
  FROM taxa
  WHERE status IN ('undescribed', 'hybrid')
) tbl
WHERE window_rank = 1;
