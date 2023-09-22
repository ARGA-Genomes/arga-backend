CREATE TABLE annotation_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,

    event_date date,
    event_time time,
    annotated_by varchar,

    representation varchar,
    release_type varchar,
    coverage varchar,
    replicons bigint,
    standard_operating_procedures varchar
);

CREATE INDEX annotation_events_sequence_id ON annotation_events (sequence_id);