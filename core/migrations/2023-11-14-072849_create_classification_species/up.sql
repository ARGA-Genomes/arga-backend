CREATE VIEW classification_species AS
SELECT
    names.id AS name_id,
    classifications.id AS classification_id,
    names.scientific_name,
    names.canonical_name,
    names.authorship,
    classifications.rank AS classification_rank,
    classifications.scientific_name AS classification_scientific_name,
    classifications.scientific_name_authorship AS classification_authorship,
    classifications.canonical_name AS classification_canonical_name,
    tree.parent_rank,
    tree.parent_scientific_name,
    tree.parent_canonical_name,
    tree.markers,
    tree.genomes,
    tree.specimens,
    tree.other,
    tree.total_genomic
FROM classifications
JOIN (
    SELECT
        classification_dag.id AS tree_id,
        classifications.rank as parent_rank,
        classifications.scientific_name AS parent_scientific_name,
        classifications.canonical_name AS parent_canonical_name,
        name_data_summaries.*
    FROM classification_dag
    JOIN taxa ON taxa.parent_taxon_id = classification_dag.taxon_id
    JOIN name_data_summaries ON taxa.name_id = name_data_summaries.name_id
    JOIN classifications ON classifications.id = classification_dag.parent_id
) tree ON tree.tree_id = classifications.id
JOIN names ON names.id = tree.name_id;
