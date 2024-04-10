ALTER TABLE taxon_history ADD COLUMN entity_id varchar;

ALTER TABLE specimens ADD COLUMN entity_id varchar;
ALTER TABLE subsamples ADD COLUMN entity_id varchar;
ALTER TABLE dna_extracts ADD COLUMN entity_id varchar;
ALTER TABLE sequences ADD COLUMN entity_id varchar;

ALTER TABLE collection_events ADD COLUMN entity_id varchar;
ALTER TABLE accession_events ADD COLUMN entity_id varchar;
ALTER TABLE subsample_events ADD COLUMN entity_id varchar;
ALTER TABLE dna_extraction_events ADD COLUMN entity_id varchar;
ALTER TABLE sequencing_events ADD COLUMN entity_id varchar;
ALTER TABLE sequencing_run_events ADD COLUMN entity_id varchar;
ALTER TABLE assembly_events ADD COLUMN entity_id varchar;
ALTER TABLE annotation_events ADD COLUMN entity_id varchar;
ALTER TABLE deposition_events ADD COLUMN entity_id varchar;
