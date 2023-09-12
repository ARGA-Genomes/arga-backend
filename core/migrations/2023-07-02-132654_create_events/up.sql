CREATE TABLE events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    field_number varchar,
    event_date date,
    event_time time,
    habitat varchar,
    sampling_protocol varchar,
    sampling_size_value varchar,
    sampling_size_unit varchar,
    sampling_effort varchar,
    field_notes text,
    event_remarks text
);
