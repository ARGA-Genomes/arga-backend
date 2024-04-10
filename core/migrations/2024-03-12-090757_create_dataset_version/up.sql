CREATE TABLE dataset_versions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    version varchar NOT NULL,
    created_at timestamp WITH time zone NOT NULL,
    imported_at timestamp WITH time zone NOT NULL
);
