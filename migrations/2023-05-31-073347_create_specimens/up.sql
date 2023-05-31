CREATE TABLE specimens (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id uuid REFERENCES name_lists NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    type_status varchar NOT NULL,
    institution_name varchar,
    organism_id varchar,
    locality varchar,
    latitude float,
    longitude float,
    details varchar,
    remarks varchar
);
