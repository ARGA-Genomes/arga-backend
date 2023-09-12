CREATE TABLE specimens (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    type_status varchar,
    institution_name varchar,
    institution_code varchar,
    collection_code varchar,
    material_sample_id varchar,
    organism_id varchar,
    locality varchar,
    country varchar,
    country_code varchar,
    state_province varchar,
    county varchar,
    municipality varchar,
    latitude float,
    longitude float,
    elevation float,
    depth float,
    elevation_accuracy float,
    depth_accuracy float,
    location_source varchar,
    recorded_by varchar,
    identified_by varchar,
    details varchar,
    remarks varchar,
    identification_remarks varchar
);

CREATE INDEX specimens_dataset_id ON specimens (dataset_id);
CREATE INDEX specimens_name_id ON specimens (name_id);
