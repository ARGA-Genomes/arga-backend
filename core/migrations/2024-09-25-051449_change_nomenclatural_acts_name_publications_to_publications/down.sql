ALTER TABLE nomenclatural_acts DROP COLUMN publication_id;
ALTER TABLE nomenclatural_acts ADD COLUMN publication_id uuid REFERENCES name_publications NOT NULL;
