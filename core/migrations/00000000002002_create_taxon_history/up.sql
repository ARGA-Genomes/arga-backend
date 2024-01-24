CREATE TABLE taxon_history (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    old_taxon_id uuid REFERENCES taxa NOT NULL,
    new_taxon_id uuid REFERENCES taxa NOT NULL,

    changed_by varchar,
    reason varchar,

    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp
);
