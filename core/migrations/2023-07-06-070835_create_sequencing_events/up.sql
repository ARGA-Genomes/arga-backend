CREATE TABLE sequences (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    dna_extract_id uuid REFERENCES dna_extracts ON DELETE CASCADE NOT NULL,

    accession varchar NOT NULL,
    genbank_accession varchar
);

CREATE INDEX sequences_dataset_id ON sequences (dataset_id);
CREATE INDEX sequences_name_id ON sequences (name_id);
CREATE INDEX sequences_dna_extract_id ON sequences (dna_extract_id);


CREATE TABLE sequencing_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,

    sequenced_by varchar,
    material_sample_id varchar,

    concentration float,
    amplicon_size bigint,
    estimated_size bigint,
    bait_set_name varchar,
    bait_set_reference varchar,

    target_gene varchar,
    dna_sequence text
);

CREATE INDEX sequencing_events_sequence_id ON sequencing_events (sequence_id);
CREATE INDEX sequencing_events_event_id ON sequencing_events (event_id);


CREATE TABLE sequencing_run_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequencing_event_id uuid REFERENCES sequencing_events ON DELETE CASCADE NOT NULL,

    trace_id varchar,
    trace_name varchar,
    trace_link varchar,
    sequencing_date timestamp without time zone,
    sequencing_center varchar,
    sequencing_center_code varchar,
    sequencing_method varchar,

    target_gene varchar,
    direction varchar,
    pcr_primer_name_forward varchar,
    pcr_primer_name_reverse varchar,
    sequence_primer_forward_name varchar,
    sequence_primer_reverse_name varchar,

    library_protocol varchar,
    analysis_description varchar,
    analysis_software varchar
);

CREATE INDEX sequencing_run_ev_seq_event_id ON sequencing_run_events (sequencing_event_id);
