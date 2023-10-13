CREATE TABLE collection_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    specimen_id uuid REFERENCES specimens ON DELETE CASCADE NOT NULL,

    event_date varchar,
    event_time varchar,
    collected_by varchar,

    field_number varchar,
    catalog_number varchar,
    record_number varchar,
    individual_count varchar,
    organism_quantity varchar,
    organism_quantity_type varchar,
    sex varchar,
    genotypic_sex varchar,
    phenotypic_sex varchar,
    life_stage varchar,
    reproductive_condition varchar,
    behavior varchar,
    establishment_means varchar,
    degree_of_establishment varchar,
    pathway varchar,
    occurrence_status varchar,
    preparation varchar,
    other_catalog_numbers varchar,

    env_broad_scale varchar,
    env_local_scale varchar,
    env_medium varchar,
    habitat varchar,
    ref_biomaterial varchar,
    source_mat_id varchar,
    specific_host varchar,
    strain varchar,
    isolate varchar,

    field_notes varchar,
    remarks varchar
);

CREATE INDEX collection_events_specimen_id ON collection_events (specimen_id);
