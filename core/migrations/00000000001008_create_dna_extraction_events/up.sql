CREATE TABLE dna_extraction_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    dna_extract_id uuid REFERENCES dna_extracts ON DELETE CASCADE NOT NULL,

    event_date varchar,
    event_time varchar,
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
