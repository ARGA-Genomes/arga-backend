-- we have to drop all the views that depend on name_data_summaries and recreate them.
-- fortunately we usually want this anyway since adding or removing fields normally needs
-- to be propagated to the dependent views
DROP MATERIALIZED VIEW taxa_tree_stats;
DROP MATERIALIZED VIEW species;
DROP MATERIALIZED VIEW name_data_summaries;


CREATE MATERIALIZED VIEW name_data_summaries AS
SELECT
    names.id AS name_id,
    COALESCE(markers.total, 0) AS markers,
    COALESCE(genomes.total, 0) AS genomes,
    COALESCE(specimens.total, 0) AS specimens,
    COALESCE(other_data.total, 0) AS other,
    COALESCE(markers.total, 0) + COALESCE(genomes.total, 0) + COALESCE(other_data.total, 0) AS total_genomic,

    COALESCE(full_genomes.total, 0) AS full_genomes,
    COALESCE(partial_genomes.total, 0) AS partial_genomes,
    COALESCE(complete_genomes.total, 0) AS complete_genomes,
    COALESCE(assembly_chromosomes.total, 0) AS assembly_chromosomes,
    COALESCE(assembly_scaffolds.total, 0) AS assembly_scaffolds,
    COALESCE(assembly_contigs.total, 0) AS assembly_contigs
FROM names
LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM sequencing_events
     JOIN sequences ON sequencing_events.sequence_id = sequences.id
     WHERE target_gene IS NOT NULL
     GROUP BY name_id
) markers ON markers.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     GROUP BY name_id
) genomes ON genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM specimens
     GROUP BY name_id
) specimens ON specimens.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int as total
     FROM sequences
     LEFT JOIN sequencing_events se on sequences.id = se.sequence_id
     LEFT JOIN assembly_events on sequences.id = assembly_events.sequence_id
     LEFT JOIN annotation_events ae on sequences.id = ae.sequence_id
     WHERE assembly_events.id IS NULL AND target_gene IS NULL
     GROUP BY name_id
) other_data ON other_data.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM annotation_events
     JOIN sequences ON annotation_events.sequence_id = sequences.id
     WHERE representation = 'Full'
     GROUP BY name_id
) full_genomes ON full_genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM annotation_events
     JOIN sequences ON annotation_events.sequence_id = sequences.id
     WHERE representation = 'Partial'
     GROUP BY name_id
) partial_genomes ON partial_genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     WHERE quality = 'Complete Genome'
     GROUP BY name_id
) complete_genomes ON complete_genomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     WHERE quality = 'Chromosome'
     GROUP BY name_id
) assembly_chromosomes ON assembly_chromosomes.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     WHERE quality = 'Scaffold'
     GROUP BY name_id
) assembly_scaffolds ON assembly_scaffolds.name_id = names.id

LEFT JOIN (
     SELECT name_id, count(*)::int AS total
     FROM assembly_events
     JOIN sequences ON assembly_events.sequence_id = sequences.id
     WHERE quality = 'Contig'
     GROUP BY name_id
) assembly_contigs ON assembly_contigs.name_id = names.id;


CREATE UNIQUE INDEX name_data_summaries_name_id ON name_data_summaries (name_id);
