CREATE TABLE assembly_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,

    event_date date,
    event_time time,
    assembled_by varchar,

    name varchar,
    version_status varchar,
    quality varchar,
    assembly_type varchar,
    genome_size bigint
);

CREATE INDEX assembly_events_sequence_id ON assembly_events (sequence_id);
