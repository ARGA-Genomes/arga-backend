CREATE TABLE biosamples (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name_id uuid REFERENCES names NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp,
    updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp,

    accession varchar NOT NULL,
    sra varchar,

    submission_date varchar,
    publication_date varchar,
    last_update varchar,

    title varchar,
    owner varchar,

    attributes jsonb
);
