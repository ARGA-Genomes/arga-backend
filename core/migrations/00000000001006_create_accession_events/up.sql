CREATE TABLE accession_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    specimen_id uuid REFERENCES specimens ON DELETE CASCADE NOT NULL,

    event_date date,
    event_time time,
    accession varchar NOT NULL,
    accessioned_by varchar,

    material_sample_id varchar,
    institution_name varchar,
    institution_code varchar,
    type_status varchar
);

CREATE INDEX accession_events_specimen_id ON accession_events (specimen_id);
