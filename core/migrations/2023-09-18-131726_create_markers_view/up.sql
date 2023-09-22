CREATE VIEW markers AS
SELECT
    sequences.id AS sequence_id,
    sequences.dataset_id,
    sequences.name_id,
    sequences.dna_extract_id,
    datasets.name AS dataset_name,
    sequences.record_id,
    deposition_events.accession,
    sequencing_events.sequenced_by,
    sequencing_events.material_sample_id,
    sequencing_events.target_gene
FROM sequences
JOIN datasets on sequences.dataset_id = datasets.id
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
LEFT JOIN deposition_events ON sequences.id = deposition_events.sequence_id
WHERE sequencing_events.target_gene IS NOT NULL;
