-- Recreated dropped views

-- a convenience view that joins all the sequencing events. Will return duplicates of
-- a sequence if there are multiple events of the same type for a sequence
CREATE OR REPLACE VIEW whole_genomes AS
SELECT
    sequences.id AS sequence_id,
    sequences.dataset_id,
    sequences.name_id,
    sequences.dna_extract_id,
    datasets.name AS dataset_name,
    sequences.record_id,
    collection_events.latitude,
    collection_events.longitude,
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
LEFT JOIN collection_events ON collection_events.specimen_id = subsamples.specimen_id;


-- All loci data
CREATE OR REPLACE VIEW markers AS
SELECT DISTINCT
    sequences.id AS sequence_id,
    sequences.dataset_id,
    sequences.name_id,
    sequences.dna_extract_id,
    datasets.name AS dataset_name,
    sequences.record_id,
    collection_events.latitude,
    collection_events.longitude,
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
LEFT JOIN collection_events ON collection_events.specimen_id = subsamples.specimen_id
WHERE sequencing_events.target_gene IS NOT NULL;


-- All data regarded as a genomic component
CREATE OR REPLACE VIEW genomic_components AS
SELECT DISTINCT
    sequences.id AS sequence_id,
    sequences.dataset_id,
    sequences.name_id,
    sequences.dna_extract_id,
    datasets.name AS dataset_name,
    sequences.record_id,
    collection_events.latitude,
    collection_events.longitude,
    deposition_events.accession,
    sequencing_events.sequenced_by,
    sequencing_events.material_sample_id,
    sequencing_events.estimated_size,
    deposition_events.event_date AS release_date,
    deposition_events.submitted_by AS deposited_by,
    deposition_events.data_type,
    deposition_events.title,
    deposition_events.url,
    deposition_events.source_uri,
    deposition_events.funding_attribution,
    deposition_events.rights_holder,
    deposition_events.access_rights
FROM sequences
JOIN datasets ON sequences.dataset_id = datasets.id
JOIN sequencing_events ON sequences.id = sequencing_events.sequence_id
JOIN deposition_events ON sequences.id = deposition_events.sequence_id
LEFT JOIN assembly_events on sequences.id = assembly_events.sequence_id
LEFT JOIN dna_extracts ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN subsamples ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN collection_events ON collection_events.specimen_id = subsamples.specimen_id
WHERE assembly_events.id IS NULL AND target_gene IS NULL;


-- Statistics for all specimens
CREATE MATERIALIZED VIEW IF NOT EXISTS specimen_stats AS
SELECT DISTINCT
    specimens.entity_id,
    SUM(CASE WHEN sequences.record_id IS NOT NULL THEN 1 ELSE 0 END) OVER (partition BY specimens.entity_id ORDER BY specimens.entity_id DESC) AS sequences,
    SUM(CASE WHEN annotation_events.representation IN ('Full', 'Partial') THEN 1 ELSE 0 END) OVER (partition BY specimens.entity_id ORDER BY specimens.entity_id DESC) AS whole_genomes,
    SUM(CASE WHEN sequencing_events.target_gene IS NOT NULL THEN 1 ELSE 0 END) OVER (partition BY specimens.entity_id ORDER BY specimens.entity_id DESC) AS markers
FROM specimens
LEFT JOIN subsamples ON subsamples.specimen_id = specimens.entity_id
LEFT JOIN dna_extracts ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN sequences ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN sequencing_events ON sequencing_events.sequence_id = sequences.id
LEFT JOIN annotation_events ON annotation_events.sequence_id = sequences.id;


-- Very high level stats for all data in the ARGA database
CREATE MATERIALIZED VIEW IF NOT EXISTS overview AS
SELECT 'data_type' AS category, 'sequences' AS name, count(*) AS total FROM sequences
UNION ALL SELECT 'data_type' AS category, 'whole_genomes' AS name, count(*) AS total FROM whole_genomes
UNION ALL SELECT 'data_type' AS category, 'loci' AS name, count(*) AS total FROM markers
UNION ALL SELECT 'data_type' AS category, 'specimens' AS name, count(*) AS total FROM specimens

UNION ALL

SELECT 'source' AS category, sources.name, count(distinct name_id) as total FROM sources
LEFT JOIN datasets ON source_id=sources.id
LEFT JOIN name_attributes ON datasets.id = name_attributes.dataset_id
GROUP BY sources.name

UNION ALL

SELECT 'dataset' AS category, datasets.name, count(*) AS total FROM name_attributes
JOIN datasets ON name_attributes.dataset_id = datasets.id
WHERE name_attributes.name = 'last_updated'
GROUP BY datasets.name
