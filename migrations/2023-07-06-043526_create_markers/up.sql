CREATE TABLE markers (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name_id uuid REFERENCES names NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp,
    updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp,

    accession varchar NOT NULL,
    material_sample_id varchar,

    gb_acs varchar,
    marker_code varchar,
    nucleotide text,

    recorded_by varchar
);
