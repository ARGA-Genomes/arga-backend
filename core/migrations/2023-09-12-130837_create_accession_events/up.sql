CREATE TABLE accession_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    specimen_id uuid REFERENCES specimens ON DELETE CASCADE NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    material_sample_id varchar,
    institution_name varchar,
    institution_code varchar,
    type_status varchar
);

CREATE INDEX accession_events_specimen_id ON accession_events (specimen_id);
CREATE INDEX accession_events_event_id ON accession_events (event_id);
