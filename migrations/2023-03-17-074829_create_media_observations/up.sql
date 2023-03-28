CREATE TABLE media_observations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    media_id bigint,
    scientific_name varchar(255),
    basis_of_record varchar(255),
    institution_code varchar(255),
    collection_code varchar(255),
    dataset_name varchar(255),
    captive varchar(255),
    event_date timestamp with time zone,
    license varchar(255),
    rights_holder varchar(255)
);
