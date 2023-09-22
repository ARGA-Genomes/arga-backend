CREATE MATERIALIZED VIEW specimen_stats AS
SELECT DISTINCT
    specimens.id,
    SUM(CASE WHEN sequences.record_id IS NOT NULL THEN 1 ELSE 0 END) over (partition BY specimens.record_id ORDER BY specimens.record_id DESC) AS sequences,
    SUM(CASE WHEN annotation_events.representation IN ('Full', 'Partial') THEN 1 ELSE 0 END) over (partition BY specimens.record_id ORDER BY specimens.record_id DESC) AS whole_genomes,
    SUM(CASE WHEN sequencing_events.target_gene IS NOT NULL THEN 1 ELSE 0 END) over (partition BY specimens.record_id ORDER BY specimens.record_id DESC) AS markers
FROM specimens
LEFT JOIN subsamples ON subsamples.specimen_id = specimens.id
LEFT JOIN dna_extracts ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN sequences ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN sequencing_events ON sequencing_events.sequence_id = sequences.id
LEFT JOIN annotation_events ON annotation_events.sequence_id = sequences.id;
