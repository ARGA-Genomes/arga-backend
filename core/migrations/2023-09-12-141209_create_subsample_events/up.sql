CREATE TABLE subsamples (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    specimen_id uuid REFERENCES specimens NOT NULL,

    accession varchar NOT NULL,
    material_sample_id varchar,
    institution_name varchar,
    institution_code varchar,
    type_status varchar
);

CREATE INDEX subsamples_dataset_id ON subsamples (dataset_id);
CREATE INDEX subsamples_name_id ON subsamples (name_id);
CREATE INDEX subsamples_specimen_id ON subsamples (specimen_id);


CREATE TABLE subsample_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subsample_id uuid REFERENCES subsamples ON DELETE CASCADE NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,
    preparation_type varchar
);

CREATE INDEX subsample_events_subsample_id ON subsample_events (subsample_id);
CREATE INDEX subsample_events_event_id ON subsample_events (event_id);
