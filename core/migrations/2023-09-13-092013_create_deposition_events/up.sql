CREATE TABLE deposition_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    material_sample_id varchar,
    submitted_by varchar,

    collection_name varchar,
    collection_code varchar,
    institution_name varchar,

    data_type varchar,
    excluded_from_refseq varchar,
    asm_not_live_date varchar,
    source_uri varchar,

    title varchar,
    url varchar,
    funding_attribution varchar,
    rights_holder varchar,
    access_rights varchar,
    reference varchar,
    last_updated date
);

CREATE INDEX deposition_events_sequence_id ON deposition_events (sequence_id);
CREATE INDEX deposition_events_event_id ON deposition_events (event_id);
