CREATE TABLE accession_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    accession varchar,
    material_sample_id varchar,
    institution_name varchar,
    institution_code varchar,
    type_status varchar
);

CREATE INDEX accession_events_dataset_id ON accession_events (dataset_id);
CREATE INDEX accession_events_name_id ON accession_events (name_id);
CREATE INDEX accession_events_event_id ON accession_events (event_id);
