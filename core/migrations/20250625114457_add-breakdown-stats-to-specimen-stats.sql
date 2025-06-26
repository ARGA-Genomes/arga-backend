DROP MATERIALIZED VIEW IF EXISTS specimen_stats;

-- Statistics for all specimens
CREATE MATERIALIZED VIEW specimen_stats AS
SELECT DISTINCT
    specimens.entity_id,
    SUM(CASE WHEN sequences.record_id IS NOT NULL THEN 1 ELSE 0 END) OVER entities AS sequences,
    SUM(CASE WHEN annotation_events.representation IN ('Full', 'Partial') THEN 1 ELSE 0 END) OVER entities AS whole_genomes,
    SUM(CASE WHEN sequencing_events.target_gene IS NOT NULL THEN 1 ELSE 0 END) OVER entities AS loci,
    SUM(CASE WHEN sequencing_events.target_gene IS NULL AND assembly_events.id IS NULL THEN 1 ELSE 0 END) OVER entities AS other_genomic,
    SUM(CASE WHEN annotation_events.representation = 'Full' THEN 1 ELSE 0 END) OVER entities AS full_genomes,
    SUM(CASE WHEN annotation_events.representation = 'Partial' THEN 1 ELSE 0 END) OVER entities AS partial_genomes,
    SUM(CASE WHEN assembly_events.quality = 'Complete Genome' THEN 1 ELSE 0 END) OVER entities AS complete_genomes,
    SUM(CASE WHEN assembly_events.quality = 'Chromosome' THEN 1 ELSE 0 END) OVER entities AS assembly_chromosomes,
    SUM(CASE WHEN assembly_events.quality = 'Scaffold' THEN 1 ELSE 0 END) OVER entities AS assembly_scaffolds,
    SUM(CASE WHEN assembly_events.quality = 'Contig' THEN 1 ELSE 0 END) OVER entities AS assembly_contigs
FROM specimens
LEFT JOIN subsamples ON subsamples.specimen_id = specimens.entity_id
LEFT JOIN dna_extracts ON dna_extracts.subsample_id = subsamples.id
LEFT JOIN sequences ON sequences.dna_extract_id = dna_extracts.id
LEFT JOIN sequencing_events ON sequencing_events.sequence_id = sequences.id
LEFT JOIN annotation_events ON annotation_events.sequence_id = sequences.id
LEFT JOIN assembly_events ON assembly_events.sequence_id = sequences.id
WINDOW entities AS (partition BY specimens.entity_id ORDER BY specimens.entity_id DESC);

CREATE UNIQUE INDEX specimen_stats_entity_id ON specimen_stats(entity_id);
