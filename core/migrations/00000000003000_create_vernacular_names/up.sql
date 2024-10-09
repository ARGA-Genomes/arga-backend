CREATE TABLE IF NOT EXISTS vernacular_names (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    vernacular_name varchar NOT NULL,
    citation varchar,
    source_url varchar
);

CREATE UNIQUE INDEX IF NOT EXISTS vernacular_names_unique_name ON vernacular_names (dataset_id, name_id, vernacular_name);
