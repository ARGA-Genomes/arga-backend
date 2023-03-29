CREATE VIEW gnl AS
SELECT
    id,
    scientific_name,
    scientific_name_authorship,
    canonical_name,
    specific_epithet,
    infraspecific_epithet,
    taxon_rank,
    name_according_to,
    name_published_in,
    taxonomic_status,
    taxon_remarks,
    kingdom,
    phylum,
    class,
    "order",
    family,
    genus,
    'user_taxa' AS source,
    taxa_lists_id
FROM user_taxa
UNION ALL
SELECT
    id,
    scientific_name,
    scientific_name_authorship,
    canonical_name,
    specific_epithet,
    infraspecific_epithet,
    taxon_rank,
    name_according_to,
    name_published_in,
    taxonomic_status,
    taxon_remarks,
    kingdom,
    phylum,
    class,
    "order",
    family,
    genus,
    'gbif' AS source,
    NULL as taxa_lists_id
FROM taxa;
