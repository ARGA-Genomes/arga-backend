CREATE MATERIALIZED VIEW ranked_taxa AS
SELECT *
FROM (
  SELECT
    user_taxa.*,
    user_taxa_lists.name as list_name,
    user_taxa_lists.priority as taxa_priority,
    rank() OVER (PARTITION BY scientific_name ORDER BY priority ASC) AS rank
  FROM user_taxa
  JOIN user_taxa_lists ON user_taxa.taxa_lists_id = user_taxa_lists.id
) ranked
WHERE rank = 1;
