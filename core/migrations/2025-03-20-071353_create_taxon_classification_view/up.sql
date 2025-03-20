CREATE MATERIALIZED VIEW taxon_classification AS
SELECT
    taxon_id,
    array_agg(taxa_dag.canonical_name ORDER BY depth DESC) AS hierarchy,
    jsonb_object_agg(rank, canonical_name) AS ranks
FROM taxa_dag
GROUP BY taxon_id;

CREATE UNIQUE INDEX taxon_classification_taxon_id ON taxon_classification (taxon_id);
