CREATE TABLE sequencing_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    event_id uuid REFERENCES events ON DELETE CASCADE NOT NULL,
    organism_id uuid REFERENCES organisms,

    accession varchar,
    genbank_accession varchar,
    target_gene varchar,

    dna_sequence text
);

CREATE INDEX sequencing_events_dataset_id ON sequencing_events (dataset_id);
CREATE INDEX sequencing_events_name_id ON sequencing_events (name_id);
CREATE INDEX sequencing_events_event_id ON sequencing_events (event_id);
CREATE INDEX sequencing_events_organism_id ON sequencing_events (organism_id);


CREATE TABLE sequencing_run_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequencing_event_id uuid REFERENCES sequencing_events ON DELETE CASCADE NOT NULL,

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

CREATE INDEX sequencing_run_ev_seq_event_id ON sequencing_run_events (sequencing_event_id);
