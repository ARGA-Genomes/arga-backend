CREATE VIEW whole_genomes AS
SELECT
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
    sequencing_events.estimated_size,
    assembly_events.assembled_by,
    assembly_events.name,
    assembly_events.version_status,
    assembly_events.quality,
    assembly_events.assembly_type,
    assembly_events.genome_size,
    annotation_events.annotated_by,
    annotation_events.representation,
    annotation_events.release_type,
    deposition_events.event_date AS release_date,
    deposition_events.submitted_by AS deposited_by,
    deposition_events.data_type,
    deposition_events.excluded_from_refseq
FROM sequences
JOIN datasets ON sequences.dataset_id = datasets.id
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
JOIN assembly_events ON sequences.id = assembly_events.sequence_id
JOIN annotation_events ON sequences.id = annotation_events.sequence_id
JOIN deposition_events ON sequences.id = deposition_events.sequence_id
LEFT JOIN dna_extracts ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN subsamples ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN specimens ON subsamples.specimen_id = specimens.id
WHERE annotation_events.representation IN ('Complete', 'Full', 'Partial');
