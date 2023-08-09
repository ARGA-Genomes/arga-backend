ALTER TABLE regions ADD COLUMN list_id uuid REFERENCES name_lists NOT NULL;
