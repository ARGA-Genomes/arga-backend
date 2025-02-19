CREATE MATERIALIZED VIEW sequence_milestones AS
SELECT
    sequences.name_id,
    assembly_events.quality,
    MIN(sequencing_events.event_date) AS sequencing_date,
    MIN(assembly_events.event_date) AS assembly_date,
    MIN(annotation_events.event_date) AS annotation_date,
    MIN(deposition_events.event_date) AS deposition_date
FROM sequences
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
JOIN assembly_events ON sequences.id = assembly_events.sequence_id
JOIN annotation_events ON sequences.id = annotation_events.sequence_id
JOIN deposition_events ON sequences.id = deposition_events.sequence_id
JOIN taxon_names ON sequences.name_id = taxon_names.name_id
GROUP BY sequences.name_id, quality;

CREATE UNIQUE INDEX sequence_milestones_name_quality ON sequence_milestones (name_id, quality);
