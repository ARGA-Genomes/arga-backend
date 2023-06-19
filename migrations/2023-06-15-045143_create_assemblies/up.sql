CREATE TABLE assemblies (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name_id uuid REFERENCES names NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp,
    updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp,

    accession varchar NOT NULL,
    bioproject_id varchar,
    biosample_id varchar,
    material_sample_id varchar,

    nuccore varchar,
    refseq_category varchar,
    specific_host varchar,
    clone_strain varchar,
    version_status varchar,
    contam_screen_input varchar,
    release_type varchar,
    genome_rep varchar,
    gbrs_paired_asm varchar,
    paired_asm_comp varchar,
    excluded_from_refseq varchar,
    relation_to_type_material varchar,
    asm_not_live_date varchar,

    other_catalog_numbers varchar,
    recorded_by varchar,

    genetic_accession_uri varchar,
    event_date varchar
);
