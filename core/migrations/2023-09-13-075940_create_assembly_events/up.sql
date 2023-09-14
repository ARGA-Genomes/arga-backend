CREATE TABLE assembly_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    name varchar,
    version_status varchar,
    quality varchar
);

CREATE INDEX assembly_events_sequence_id ON assembly_events (sequence_id);
CREATE INDEX assembly_events_event_id ON assembly_events (event_id);
