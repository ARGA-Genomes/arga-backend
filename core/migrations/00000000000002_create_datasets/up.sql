CREATE TABLE datasets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id uuid REFERENCES sources ON DELETE CASCADE NOT NULL,
    global_id varchar NOT NULL,
    name varchar NOT NULL,
    short_name varchar,
    description text,
    url varchar,
    citation varchar,
    license varchar,
    rights_holder varchar,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);
COMMENT ON TABLE sources IS 'Information and metadata about imported data';

CREATE UNIQUE INDEX dataset_global_id ON datasets (global_id);
