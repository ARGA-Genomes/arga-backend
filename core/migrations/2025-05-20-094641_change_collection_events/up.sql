-- drop the entire event chain
DROP TABLE IF EXISTS specimens CASCADE;
DROP TABLE IF EXISTS organisms;
DROP TABLE IF EXISTS subsamples CASCADE;
drop table specimens_old cascade;
drop table accession_events cascade;
drop table dna_extracts cascade;
drop table dna_extraction_events cascade;
drop table sequences cascade;
drop table sequencing_events cascade;
drop table sequencing_run_events cascade;
drop table annotation_events cascade;
drop table assembly_events cascade;
drop table deposition_events cascade;
drop table subsample_events;


CREATE TABLE organisms (
    entity_id varchar PRIMARY KEY NOT NULL,
    name_id uuid REFERENCES names ON DELETE CASCADE NOT NULL,
    organism_id varchar NOT NULL,
    sex varchar,
    genotypic_sex varchar,
    phenotypic_sex varchar,
    life_stage varchar,
    reproductive_condition varchar,
    behavior varchar
);


CREATE TABLE specimens (
    entity_id varchar PRIMARY KEY NOT NULL,
    organism_id varchar REFERENCES organisms ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names ON DELETE CASCADE NOT NULL
);


CREATE TABLE collection_events (
    entity_id varchar PRIMARY KEY NOT NULL,
    specimen_id varchar REFERENCES specimens ON DELETE CASCADE NOT NULL,

    name_id uuid REFERENCES names ON DELETE CASCADE NOT NULL,
    organism_id varchar REFERENCES organisms ON DELETE CASCADE NOT NULL,
    field_collecting_id varchar,

    event_date date,
    event_time time without time zone,
    collected_by varchar,
    collection_remarks varchar,
    identified_by varchar,
    identified_date date,
    identification_remarks varchar,

    locality varchar,
    country varchar,
    country_code varchar,
    state_province varchar,
    county varchar,
    municipality varchar,
    latitude float,
    longitude float,
    elevation float,
    depth float,
    elevation_accuracy float,
    depth_accuracy float,
    location_source varchar,

    preparation varchar,
    environment_broad_scale varchar,
    environment_local_scale varchar,
    environment_medium varchar,
    habitat varchar,
    specific_host varchar,
    individual_count varchar,
    organism_quantity varchar,
    organism_quantity_type varchar,

    strain varchar,
    isolate varchar,
    field_notes varchar
);

CREATE INDEX collection_events_specimen_id ON collection_events (specimen_id);
CREATE INDEX collection_events_name_id ON collection_events (name_id);
CREATE INDEX collection_events_organism_id ON collection_events (organism_id);
CREATE INDEX collection_events_field_collecting_id ON collection_events (field_collecting_id);




CREATE MATERIALIZED VIEW collection_event_entities AS
SELECT entity_id FROM collection_event_logs GROUP BY entity_id ORDER BY entity_id;

CREATE UNIQUE INDEX collection_event_entities_entity_id ON collection_event_entities (entity_id);



DROP MATERIALIZED VIEW IF EXISTS specimen_stats;

CREATE MATERIALIZED VIEW specimen_stats AS
SELECT DISTINCT
    specimens.entity_id,
    0 AS sequences,
    0 AS whole_genomes,
    0 AS markers
FROM specimens;




CREATE TABLE specimens_old (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,

    record_id varchar NOT NULL,
    material_sample_id varchar,
    organism_id varchar,

    institution_name varchar,
    institution_code varchar,
    collection_code varchar,
    recorded_by varchar,
    identified_by varchar,
    identified_date varchar,

    type_status varchar,
    locality varchar,
    country varchar,
    country_code varchar,
    state_province varchar,
    county varchar,
    municipality varchar,
    latitude float,
    longitude float,
    elevation float,
    depth float,
    elevation_accuracy float,
    depth_accuracy float,
    location_source varchar,

    details varchar,
    remarks varchar,
    identification_remarks varchar
);



CREATE TABLE accession_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    specimen_id uuid NOT NULL,

    event_date varchar,
    event_time varchar,
    accession varchar NOT NULL,
    accessioned_by varchar,

    material_sample_id varchar,
    institution_name varchar,
    institution_code varchar,
    type_status varchar
);



CREATE TABLE subsamples (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    specimen_id uuid REFERENCES specimens_old NOT NULL,

    record_id varchar NOT NULL,
    material_sample_id varchar,
    institution_name varchar,
    institution_code varchar,
    type_status varchar
);
CREATE TABLE subsample_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    subsample_id uuid REFERENCES subsamples ON DELETE CASCADE NOT NULL,

    event_date varchar,
    event_time varchar,
    subsampled_by varchar,
    preparation_type varchar
);


CREATE TABLE dna_extracts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    subsample_id uuid REFERENCES subsamples ON DELETE CASCADE NOT NULL,

    record_id varchar NOT NULL
);
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




CREATE TABLE sequences (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    dna_extract_id uuid REFERENCES dna_extracts ON DELETE CASCADE NOT NULL,

    record_id varchar NOT NULL
);
CREATE TABLE sequencing_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
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

CREATE TABLE assembly_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,

    event_date varchar,
    event_time varchar,
    assembled_by varchar,

    name varchar,
    version_status varchar,
    quality varchar,
    assembly_type varchar,
    genome_size bigint
);

CREATE TABLE annotation_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,

    event_date varchar,
    event_time varchar,
    annotated_by varchar,

    representation varchar,
    release_type varchar,
    coverage varchar,
    replicons bigint,
    standard_operating_procedures varchar
);

CREATE TABLE deposition_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    sequence_id uuid REFERENCES sequences ON DELETE CASCADE NOT NULL,

    event_date varchar,
    event_time varchar,
    accession varchar,
    submitted_by varchar,

    material_sample_id varchar,
    collection_name varchar,
    collection_code varchar,
    institution_name varchar,

    data_type varchar,
    excluded_from_refseq varchar,
    asm_not_live_date varchar,
    source_uri varchar,

    title varchar,
    url varchar,
    funding_attribution varchar,
    rights_holder varchar,
    access_rights varchar,
    reference varchar,
    last_updated date
);


ALTER TABLE specimens_old ADD COLUMN entity_id varchar;
ALTER TABLE subsamples ADD COLUMN entity_id varchar;
ALTER TABLE dna_extracts ADD COLUMN entity_id varchar;
ALTER TABLE sequences ADD COLUMN entity_id varchar;

ALTER TABLE accession_events ADD COLUMN entity_id varchar;
ALTER TABLE subsample_events ADD COLUMN entity_id varchar;
ALTER TABLE dna_extraction_events ADD COLUMN entity_id varchar;
ALTER TABLE sequencing_events ADD COLUMN entity_id varchar;
ALTER TABLE sequencing_run_events ADD COLUMN entity_id varchar;
ALTER TABLE assembly_events ADD COLUMN entity_id varchar;
ALTER TABLE annotation_events ADD COLUMN entity_id varchar;
ALTER TABLE deposition_events ADD COLUMN entity_id varchar;
