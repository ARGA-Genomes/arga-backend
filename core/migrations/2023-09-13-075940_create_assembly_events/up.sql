CREATE TABLE assembly_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    accession varchar,
    name varchar,
    version_status varchar,
    quality varchar
);

CREATE INDEX assembly_events_dataset_id ON assembly_events (dataset_id);
CREATE INDEX assembly_events_name_id ON assembly_events (name_id);
CREATE INDEX assembly_events_event_id ON assembly_events (event_id);
