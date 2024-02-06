CREATE VIEW markers AS
SELECT DISTINCT
    sequences.id AS sequence_id,
    sequences.dataset_id,
    sequences.name_id,
    sequences.dna_extract_id,
    datasets.name AS dataset_name,
    sequences.record_id,
    specimens.latitude,
    specimens.longitude,
    deposition_events.accession,
    sequencing_events.sequenced_by,
    sequencing_events.material_sample_id,
    sequencing_events.target_gene,
    deposition_events.event_date AS release_date
FROM sequences
JOIN datasets ON sequences.dataset_id = datasets.id
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
LEFT JOIN deposition_events ON sequences.id = deposition_events.sequence_id
LEFT JOIN dna_extracts ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN subsamples ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN specimens ON subsamples.specimen_id = specimens.id
WHERE sequencing_events.target_gene IS NOT NULL;
