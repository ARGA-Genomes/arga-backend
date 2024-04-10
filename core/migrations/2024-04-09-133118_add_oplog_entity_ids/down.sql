ALTER TABLE taxon_history DROP COLUMN entity_id;

ALTER TABLE specimens DROP COLUMN entity_id;
ALTER TABLE subsamples DROP COLUMN entity_id;
ALTER TABLE dna_extracts DROP COLUMN entity_id;
ALTER TABLE sequences DROP COLUMN entity_id;

ALTER TABLE collection_events DROP COLUMN entity_id;
ALTER TABLE accession_events DROP COLUMN entity_id;
ALTER TABLE subsample_events DROP COLUMN entity_id;
ALTER TABLE dna_extraction_events DROP COLUMN entity_id;
ALTER TABLE sequencing_events DROP COLUMN entity_id;
ALTER TABLE sequencing_run_events DROP COLUMN entity_id;
ALTER TABLE assembly_events DROP COLUMN entity_id;
ALTER TABLE annotation_events DROP COLUMN entity_id;
ALTER TABLE deposition_events DROP COLUMN entity_id;
