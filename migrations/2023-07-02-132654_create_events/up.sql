CREATE TABLE events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_event_id uuid REFERENCES events,

    event_id varchar,
    field_number varchar,
    event_date date,
    habitat varchar,
    sampling_protocol varchar,
    sampling_size_value varchar,
    sampling_size_unit varchar,
    sampling_effort varchar,
    field_notes text,
    event_remarks text
);

CREATE INDEX events_parent_event_id ON events (parent_event_id);
