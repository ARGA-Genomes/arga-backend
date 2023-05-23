CREATE TABLE taxon_photos (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name_id uuid REFERENCES names NOT NULL,
    url varchar NOT NULL,
    source varchar,
    publisher varchar,
    license varchar,
    rights_holder varchar
);
