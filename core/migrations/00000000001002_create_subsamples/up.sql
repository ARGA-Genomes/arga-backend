CREATE TABLE subsamples (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    specimen_id uuid REFERENCES specimens NOT NULL,

    record_id varchar NOT NULL,
    material_sample_id varchar,
    institution_name varchar,
    institution_code varchar,
    type_status varchar
);

CREATE INDEX subsamples_dataset_id ON subsamples (dataset_id);
CREATE INDEX subsamples_name_id ON subsamples (name_id);
CREATE INDEX subsamples_specimen_id ON subsamples (specimen_id);
