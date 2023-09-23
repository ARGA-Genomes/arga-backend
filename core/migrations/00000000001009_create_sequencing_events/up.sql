CREATE TABLE sequencing_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,

    event_date varchar,
    event_time varchar,
    sequenced_by varchar,
    material_sample_id varchar,

    concentration float,
    amplicon_size bigint,
    estimated_size varchar,
    bait_set_name varchar,
    bait_set_reference varchar,

    target_gene varchar,
    dna_sequence text
);

CREATE INDEX sequencing_events_sequence_id ON sequencing_events (sequence_id);


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
