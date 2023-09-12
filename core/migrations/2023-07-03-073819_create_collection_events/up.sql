CREATE TABLE collection_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,
    specimen_id uuid REFERENCES specimens ON DELETE CASCADE NOT NULL ,
    organism_id uuid REFERENCES organisms,

    accession varchar,
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
    other_catalog_numbers varchar,

    env_broad_scale varchar,
    ref_biomaterial varchar,
    source_mat_id varchar,
    specific_host varchar,
    strain varchar,
    isolate varchar
);

CREATE INDEX collection_events_event_id ON collection_events (event_id);
CREATE INDEX collection_events_specimen_id ON collection_events (specimen_id);
CREATE INDEX collection_events_organism_id ON collection_events (organism_id);
