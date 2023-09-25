CREATE MATERIALIZED VIEW name_data_summaries AS
SELECT
    names.id AS name_id,
    markers.total::int AS markers,
    genomes.total::int AS genomes,
    specimens.total::int AS specimens,
    other_data.total::int AS other
FROM names
LEFT JOIN (
     SELECT name_id, count(*) AS total
     FROM sequencing_events
     JOIN sequences ON sequencing_events.sequence_id = sequences.id
     WHERE target_gene IS NOT NULL
     GROUP BY name_id
) markers ON markers.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*) AS total
     FROM annotation_events
     JOIN sequences ON annotation_events.sequence_id = sequences.id
     WHERE representation IN ('Full', 'Partial')
     GROUP BY name_id
) genomes ON genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*) AS total
     FROM specimens
     GROUP BY name_id
) specimens ON specimens.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*) as total
     FROM sequences
     LEFT JOIN sequencing_events se on sequences.id = se.sequence_id
     LEFT JOIN annotation_events ae on sequences.id = ae.sequence_id
     WHERE (representation IS NULL OR representation NOT IN ('Full', 'Partial')) AND target_gene IS NULL
     GROUP BY name_id
) other_data ON other_data.name_id = names.id;
