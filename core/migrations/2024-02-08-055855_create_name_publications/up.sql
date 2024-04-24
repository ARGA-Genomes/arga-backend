CREATE TABLE name_publications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets NOT NULL,

    citation varchar,
    published_year int,
    source_url varchar,
    type_citation varchar,

    -- these are timestamps from the dataset, not our own timestamps
    record_created_at timestamp with time zone,
    record_updated_at timestamp with time zone
);
