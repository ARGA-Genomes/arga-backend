CREATE TABLE sequences (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    dna_extract_id uuid REFERENCES dna_extracts ON DELETE CASCADE NOT NULL,

    record_id varchar NOT NULL
);

CREATE INDEX sequences_dataset_id ON sequences (dataset_id);
CREATE INDEX sequences_name_id ON sequences (name_id);
CREATE INDEX sequences_dna_extract_id ON sequences (dna_extract_id);
