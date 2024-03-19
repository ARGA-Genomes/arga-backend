ALTER TABLE taxon_history ADD COLUMN updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp;
ALTER TABLE taxon_history ADD COLUMN act_id uuid REFERENCES nomenclatural_acts NOT NULL;
ALTER TABLE taxon_history ADD COLUMN publication_id uuid REFERENCES name_publications;
ALTER TABLE taxon_history ADD COLUMN source_url varchar;
