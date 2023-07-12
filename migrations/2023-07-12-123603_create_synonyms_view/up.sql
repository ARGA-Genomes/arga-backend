CREATE VIEW synonyms AS
SELECT *
FROM (
  SELECT
    taxa.*,
    array_agg(synonym.scientific_name)
      OVER (PARTITION BY taxa.id ORDER BY synonym.scientific_name DESC) AS names,
    rank() OVER (PARTITION BY taxa.id ORDER BY synonym.scientific_name ASC) AS window_rank
  FROM taxa
  JOIN taxon_history ON taxa.id = taxon_history.new_taxon_id
  JOIN taxa synonym ON synonym.id = taxon_history.old_taxon_id
) tbl
WHERE window_rank = 1
