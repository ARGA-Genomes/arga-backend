CREATE VIEW user_taxa_objects AS
SELECT
    objects.id as object_id,
    objects.entity_id,
    objects.attribute_id,
    objects.value_id,
    attributes.name AS attribute_name,
    attributes.data_type as attribute_data_type,
    user_taxa.taxa_lists_id,
    user_taxa.scientific_name,
    user_taxa.scientific_name_authorship,
    user_taxa.canonical_name,
    user_taxa.taxon_rank,
    user_taxa.taxonomic_status
FROM objects
JOIN attributes ON attribute_id=attributes.id
JOIN user_taxa ON entity_id=user_taxa.id;
