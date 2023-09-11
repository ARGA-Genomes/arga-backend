CREATE TABLE specimens (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    type_status varchar,
    institution_name varchar,
    institution_code varchar,
    collection_code varchar,
    catalog_number varchar,
    organism_id varchar,
    locality varchar,
    latitude float,
    longitude float,
    recorded_by varchar,
    details varchar,
    remarks varchar
);
