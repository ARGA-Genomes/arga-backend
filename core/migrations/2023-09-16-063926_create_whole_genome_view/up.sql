CREATE VIEW whole_genomes AS
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
    deposition_events.event_date as release_date,
    deposition_events.submitted_by AS deposited_by,
    deposition_events.data_type,
    deposition_events.excluded_from_refseq
FROM sequences
JOIN datasets on sequences.dataset_id = datasets.id
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
JOIN assembly_events ON sequences.id = assembly_events.sequence_id
JOIN annotation_events ON sequences.id = annotation_events.sequence_id
JOIN deposition_events ON sequences.id = deposition_events.sequence_id
WHERE annotation_events.representation IN ('Full', 'Partial');
