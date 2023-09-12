CREATE TABLE subsample_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    accession varchar,
    preparation_type varchar
);

CREATE INDEX subsample_events_dataset_id ON subsample_events (dataset_id);
CREATE INDEX subsample_events_name_id ON subsample_events (name_id);
CREATE INDEX subsample_events_event_id ON subsample_events (event_id);
