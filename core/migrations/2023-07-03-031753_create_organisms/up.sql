CREATE TABLE organisms (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name_id uuid REFERENCES names NOT NULL,

    organism_id varchar,
    organism_name varchar,
    organism_scope varchar,
    associated_organisms varchar,
    previous_identifications varchar,
    remarks text
);
