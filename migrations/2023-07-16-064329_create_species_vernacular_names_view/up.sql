CREATE VIEW species_vernacular_names AS
SELECT *
FROM (
  SELECT
    taxa.*,
    array_agg(vernacular_name)
      OVER (PARTITION BY genus, specific_epithet ORDER BY vernacular_name ASC) AS vernacular_names,
    rank() OVER (PARTITION BY genus, specific_epithet ORDER BY vernacular_name DESC) AS window_rank
  FROM names
  INNER JOIN name_vernacular_names nvn ON names.id = nvn.name_id
  INNER JOIN vernacular_names ON nvn.vernacular_name_id = vernacular_names.id
  INNER JOIN taxa ON names.id = taxa.name_id
) tbl
WHERE window_rank = 1;
