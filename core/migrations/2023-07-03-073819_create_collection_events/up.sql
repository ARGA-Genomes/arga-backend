CREATE TABLE collection_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id uuid REFERENCES events NOT NULL,
    specimen_id uuid REFERENCES specimens NOT NULL,
    organism_id uuid REFERENCES organisms,

    occurrence_id varchar,
    catalog_number varchar,
    record_number varchar,
    individual_count varchar,
    organism_quantity varchar,
    organism_quantity_type varchar,
    sex varchar,
    life_stage varchar,
    reproductive_condition varchar,
    behavior varchar,
    establishment_means varchar,
    degree_of_establishment varchar,
    pathway varchar,
    occurrence_status varchar,
    preparation varchar,
    other_catalog_numbers varchar
);
