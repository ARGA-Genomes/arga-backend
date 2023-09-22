CREATE TABLE subsample_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subsample_id uuid REFERENCES subsamples ON DELETE CASCADE NOT NULL,

    event_date date,
    event_time time,
    subsampled_by varchar,
    preparation_type varchar
);

CREATE INDEX subsample_events_subsample_id ON subsample_events (subsample_id);
