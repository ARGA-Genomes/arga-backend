CREATE TABLE dna_extracts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    subsample_id uuid REFERENCES subsamples ON DELETE CASCADE NOT NULL,

    accession varchar NOT NULL
);

CREATE INDEX dna_extracts_dataset_id ON dna_extracts (dataset_id);
CREATE INDEX dna_extracts_name_id ON dna_extracts (name_id);
CREATE INDEX dna_extracts_subsample_id ON dna_extracts (subsample_id);


CREATE TABLE dna_extraction_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dna_extract_id uuid REFERENCES dna_extracts ON DELETE CASCADE NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    extracted_by varchar,

    preservation_type varchar,
    preparation_type varchar,
    extraction_method varchar,
    measurement_method varchar,
    concentration_method varchar,
    quality varchar,

    concentration float,
    absorbance_260_230 float,
    absorbance_260_280 float
);

CREATE INDEX dna_extraction_events_dna_extracts_id ON dna_extraction_events (dna_extract_id);
CREATE INDEX dna_extraction_events_event_id ON dna_extraction_events (event_id);
