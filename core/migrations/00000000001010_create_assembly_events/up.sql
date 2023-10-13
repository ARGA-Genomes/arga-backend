CREATE TABLE assembly_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,

    event_date varchar,
    event_time varchar,
    assembled_by varchar,

    name varchar,
    version_status varchar,
    quality varchar,
    assembly_type varchar,
    genome_size bigint
);

CREATE INDEX assembly_events_sequence_id ON assembly_events (sequence_id);
