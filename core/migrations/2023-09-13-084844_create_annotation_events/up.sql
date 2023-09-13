CREATE TABLE annotation_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    accession varchar,
    representation varchar,
    release_type varchar,
    coverage varchar,
    replicons bigint,
    standard_operating_procedures varchar
);

CREATE INDEX annotation_events_dataset_id ON annotation_events (dataset_id);
CREATE INDEX annotation_events_name_id ON annotation_events (name_id);
CREATE INDEX annotation_events_event_id ON annotation_events (event_id);
