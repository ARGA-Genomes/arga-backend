CREATE EXTENSION IF NOT EXISTS postgis;



CREATE TABLE sources (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar NOT NULL,
    author varchar NOT NULL,
    rights_holder varchar NOT NULL,
    access_rights varchar NOT NULL,
    license varchar NOT NULL
);
COMMENT ON TABLE sources IS 'Metadata of the curated and processed datasets';

CREATE UNIQUE INDEX sources_name ON sources (name);
