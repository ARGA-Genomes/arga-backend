CREATE VIEW species AS
SELECT *
FROM (
  SELECT
    *,
    array_agg(canonical_name)
      FILTER(WHERE subspecific_epithet IS NOT NULL)
      OVER (PARTITION BY genus, specific_epithet ORDER BY subspecific_epithet ASC) AS subspecies,
    rank() OVER (PARTITION BY genus, specific_epithet ORDER BY canonical_name ASC) AS window_rank
  FROM taxa
) tbl
WHERE window_rank = 1;
