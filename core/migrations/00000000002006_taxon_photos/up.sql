CREATE TABLE taxon_photos (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    taxon_id uuid REFERENCES taxa NOT NULL,
    url varchar NOT NULL,
    source varchar,
    publisher varchar,
    license varchar,
    rights_holder varchar,
    priority int NOT NULL DEFAULT 1
);
