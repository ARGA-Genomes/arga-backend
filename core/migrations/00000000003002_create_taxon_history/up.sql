CREATE TABLE taxon_history (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    old_taxon_id uuid REFERENCES taxa NOT NULL,
    new_taxon_id uuid REFERENCES taxa NOT NULL,
    dataset_id uuid REFERENCES datasets NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX taxon_history_unique_link ON taxon_history (old_taxon_id, new_taxon_id);
