CREATE TABLE dna_extraction_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    accession varchar,
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

CREATE INDEX dna_extraction_events_dataset_id ON dna_extraction_events (dataset_id);
CREATE INDEX dna_extraction_events_name_id ON dna_extraction_events (name_id);
CREATE INDEX dna_extraction_events_event_id ON dna_extraction_events (event_id);
