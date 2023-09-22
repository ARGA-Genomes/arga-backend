CREATE TABLE dna_extracts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    subsample_id uuid REFERENCES subsamples ON DELETE CASCADE NOT NULL,

    record_id varchar NOT NULL
);

CREATE INDEX dna_extracts_dataset_id ON dna_extracts (dataset_id);
CREATE INDEX dna_extracts_name_id ON dna_extracts (name_id);
CREATE INDEX dna_extracts_subsample_id ON dna_extracts (subsample_id);
