CREATE TABLE annotation_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    representation varchar,
    release_type varchar,
    coverage varchar,
    replicons bigint,
    standard_operating_procedures varchar
);

CREATE INDEX annotation_events_sequence_id ON annotation_events (sequence_id);
CREATE INDEX annotation_events_event_id ON annotation_events (event_id);
