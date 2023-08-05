CREATE TABLE sequencing_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id uuid REFERENCES events NOT NULL,
    specimen_id uuid REFERENCES specimens NOT NULL,
    organism_id uuid REFERENCES organisms,

    sequence_id varchar,
    genbank_accession varchar,
    target_gene varchar,

    dna_sequence text
);

CREATE TABLE sequencing_run_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequencing_event_id uuid REFERENCES sequencing_events NOT NULL,

    trace_id varchar,
    trace_name varchar,
    trace_link varchar,
    sequencing_date timestamp without time zone,
    sequencing_center varchar,

    target_gene varchar,
    direction varchar,
    pcr_primer_name_forward varchar,
    pcr_primer_name_reverse varchar,
    sequence_primer_forward_name varchar,
    sequence_primer_reverse_name varchar
);
