// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "attribute_category"))]
    pub struct AttributeCategory;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "attribute_value_type"))]
    pub struct AttributeValueType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "geometry"))]
    pub struct Geometry;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "job_status"))]
    pub struct JobStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "nomenclatural_act_type"))]
    pub struct NomenclaturalActType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "operation_action"))]
    pub struct OperationAction;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "region_type"))]
    pub struct RegionType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "taxonomic_act_type"))]
    pub struct TaxonomicActType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "taxonomic_rank"))]
    pub struct TaxonomicRank;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "taxonomic_status"))]
    pub struct TaxonomicStatus;
}

diesel::table! {
    accession_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        specimen_id -> Uuid,
        event_date -> Nullable<Varchar>,
        event_time -> Nullable<Varchar>,
        accession -> Varchar,
        accessioned_by -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        institution_name -> Nullable<Varchar>,
        institution_code -> Nullable<Varchar>,
        type_status -> Nullable<Varchar>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    admin_media (id) {
        id -> Uuid,
        name_id -> Uuid,
        image_source -> Varchar,
        url -> Varchar,
        width -> Nullable<Int4>,
        height -> Nullable<Int4>,
        reference_url -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        source -> Nullable<Varchar>,
        creator -> Nullable<Varchar>,
        publisher -> Nullable<Varchar>,
        license -> Nullable<Varchar>,
        rights_holder -> Nullable<Varchar>,
    }
}

diesel::table! {
    annotation_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        sequence_id -> Uuid,
        event_date -> Nullable<Varchar>,
        event_time -> Nullable<Varchar>,
        annotated_by -> Nullable<Varchar>,
        representation -> Nullable<Varchar>,
        release_type -> Nullable<Varchar>,
        coverage -> Nullable<Varchar>,
        replicons -> Nullable<Int8>,
        standard_operating_procedures -> Nullable<Varchar>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    assemblies (id) {
        id -> Uuid,
        name_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        accession -> Varchar,
        bioproject_id -> Nullable<Varchar>,
        biosample_id -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        nuccore -> Nullable<Varchar>,
        refseq_category -> Nullable<Varchar>,
        specific_host -> Nullable<Varchar>,
        clone_strain -> Nullable<Varchar>,
        version_status -> Nullable<Varchar>,
        contam_screen_input -> Nullable<Varchar>,
        release_type -> Nullable<Varchar>,
        genome_rep -> Nullable<Varchar>,
        gbrs_paired_asm -> Nullable<Varchar>,
        paired_asm_comp -> Nullable<Varchar>,
        excluded_from_refseq -> Nullable<Varchar>,
        relation_to_type_material -> Nullable<Varchar>,
        asm_not_live_date -> Nullable<Varchar>,
        other_catalog_numbers -> Nullable<Varchar>,
        recorded_by -> Nullable<Varchar>,
        genetic_accession_uri -> Nullable<Varchar>,
        event_date -> Nullable<Varchar>,
    }
}

diesel::table! {
    assembly_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        sequence_id -> Uuid,
        event_date -> Nullable<Varchar>,
        event_time -> Nullable<Varchar>,
        assembled_by -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        version_status -> Nullable<Varchar>,
        quality -> Nullable<Varchar>,
        assembly_type -> Nullable<Varchar>,
        genome_size -> Nullable<Int8>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    assembly_stats (id) {
        id -> Uuid,
        assembly_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        total_length -> Nullable<Int4>,
        spanned_gaps -> Nullable<Int4>,
        unspanned_gaps -> Nullable<Int4>,
        region_count -> Nullable<Int4>,
        scaffold_count -> Nullable<Int4>,
        scaffold_n50 -> Nullable<Int4>,
        scaffold_l50 -> Nullable<Int4>,
        scaffold_n75 -> Nullable<Int4>,
        scaffold_n90 -> Nullable<Int4>,
        contig_count -> Nullable<Int4>,
        contig_n50 -> Nullable<Int4>,
        contig_l50 -> Nullable<Int4>,
        total_gap_length -> Nullable<Int4>,
        molecule_count -> Nullable<Int4>,
        top_level_count -> Nullable<Int4>,
        component_count -> Nullable<Int4>,
        gc_perc -> Nullable<Int4>,
    }
}

diesel::table! {
    biosamples (id) {
        id -> Uuid,
        name_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        accession -> Varchar,
        sra -> Nullable<Varchar>,
        submission_date -> Nullable<Varchar>,
        publication_date -> Nullable<Varchar>,
        last_update -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        owner -> Nullable<Varchar>,
        attributes -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OperationAction;

    collection_event_logs (operation_id) {
        operation_id -> Numeric,
        parent_id -> Numeric,
        entity_id -> Varchar,
        dataset_version_id -> Uuid,
        action -> OperationAction,
        atom -> Jsonb,
    }
}

diesel::table! {
    collection_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        specimen_id -> Uuid,
        event_date -> Nullable<Varchar>,
        event_time -> Nullable<Varchar>,
        collected_by -> Nullable<Varchar>,
        field_number -> Nullable<Varchar>,
        catalog_number -> Nullable<Varchar>,
        record_number -> Nullable<Varchar>,
        individual_count -> Nullable<Varchar>,
        organism_quantity -> Nullable<Varchar>,
        organism_quantity_type -> Nullable<Varchar>,
        sex -> Nullable<Varchar>,
        genotypic_sex -> Nullable<Varchar>,
        phenotypic_sex -> Nullable<Varchar>,
        life_stage -> Nullable<Varchar>,
        reproductive_condition -> Nullable<Varchar>,
        behavior -> Nullable<Varchar>,
        establishment_means -> Nullable<Varchar>,
        degree_of_establishment -> Nullable<Varchar>,
        pathway -> Nullable<Varchar>,
        occurrence_status -> Nullable<Varchar>,
        preparation -> Nullable<Varchar>,
        other_catalog_numbers -> Nullable<Varchar>,
        env_broad_scale -> Nullable<Varchar>,
        env_local_scale -> Nullable<Varchar>,
        env_medium -> Nullable<Varchar>,
        habitat -> Nullable<Varchar>,
        ref_biomaterial -> Nullable<Varchar>,
        source_mat_id -> Nullable<Varchar>,
        specific_host -> Nullable<Varchar>,
        strain -> Nullable<Varchar>,
        isolate -> Nullable<Varchar>,
        field_notes -> Nullable<Varchar>,
        remarks -> Nullable<Varchar>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    dataset_versions (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        version -> Varchar,
        created_at -> Timestamptz,
        imported_at -> Timestamptz,
    }
}

diesel::table! {
    datasets (id) {
        id -> Uuid,
        source_id -> Uuid,
        global_id -> Varchar,
        name -> Varchar,
        short_name -> Nullable<Varchar>,
        description -> Nullable<Text>,
        url -> Nullable<Varchar>,
        citation -> Nullable<Varchar>,
        license -> Nullable<Varchar>,
        rights_holder -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    deposition_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        sequence_id -> Uuid,
        event_date -> Nullable<Varchar>,
        event_time -> Nullable<Varchar>,
        accession -> Nullable<Varchar>,
        submitted_by -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        collection_name -> Nullable<Varchar>,
        collection_code -> Nullable<Varchar>,
        institution_name -> Nullable<Varchar>,
        data_type -> Nullable<Varchar>,
        excluded_from_refseq -> Nullable<Varchar>,
        asm_not_live_date -> Nullable<Varchar>,
        source_uri -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        url -> Nullable<Varchar>,
        funding_attribution -> Nullable<Varchar>,
        rights_holder -> Nullable<Varchar>,
        access_rights -> Nullable<Varchar>,
        reference -> Nullable<Varchar>,
        last_updated -> Nullable<Date>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    dna_extraction_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        dna_extract_id -> Uuid,
        event_date -> Nullable<Varchar>,
        event_time -> Nullable<Varchar>,
        extracted_by -> Nullable<Varchar>,
        preservation_type -> Nullable<Varchar>,
        preparation_type -> Nullable<Varchar>,
        extraction_method -> Nullable<Varchar>,
        measurement_method -> Nullable<Varchar>,
        concentration_method -> Nullable<Varchar>,
        quality -> Nullable<Varchar>,
        concentration -> Nullable<Float8>,
        absorbance_260_230 -> Nullable<Float8>,
        absorbance_260_280 -> Nullable<Float8>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    dna_extracts (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        subsample_id -> Uuid,
        record_id -> Varchar,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    ecology (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        values -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Geometry;

    ibra (ogc_fid) {
        ogc_fid -> Int4,
        reg_code_7 -> Nullable<Varchar>,
        reg_name_7 -> Nullable<Varchar>,
        hectares -> Nullable<Float8>,
        sq_km -> Nullable<Float8>,
        rec_id -> Nullable<Int4>,
        reg_code_6 -> Nullable<Varchar>,
        reg_name_6 -> Nullable<Varchar>,
        reg_no_61 -> Nullable<Float8>,
        feat_id -> Nullable<Varchar>,
        shape_leng -> Nullable<Float8>,
        shape_area -> Nullable<Float8>,
        wkb_geometry -> Nullable<Geometry>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Geometry;

    imcra_mesoscale (ogc_fid) {
        ogc_fid -> Int4,
        meso_name -> Nullable<Varchar>,
        meso_num -> Nullable<Int4>,
        meso_abbr -> Nullable<Varchar>,
        water_type -> Nullable<Varchar>,
        area_km2 -> Nullable<Float8>,
        wkb_geometry -> Nullable<Geometry>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Geometry;

    imcra_provincial (ogc_fid) {
        ogc_fid -> Int4,
        pb_name -> Nullable<Varchar>,
        pb_num -> Nullable<Int4>,
        water_type -> Nullable<Varchar>,
        area_km2 -> Nullable<Float8>,
        wkb_geometry -> Nullable<Geometry>,
    }
}

diesel::table! {
    indigenous_knowledge (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        name -> Varchar,
        food_use -> Bool,
        medicinal_use -> Bool,
        cultural_connection -> Bool,
        last_updated -> Timestamptz,
        source_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JobStatus;

    jobs (id) {
        id -> Uuid,
        status -> JobStatus,
        #[max_length = 255]
        worker -> Varchar,
        payload -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AttributeCategory;
    use super::sql_types::AttributeValueType;

    name_attributes (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        name -> Varchar,
        category -> AttributeCategory,
        value_type -> AttributeValueType,
        value_bool -> Nullable<Bool>,
        value_int -> Nullable<Int8>,
        value_decimal -> Nullable<Numeric>,
        value_str -> Nullable<Varchar>,
        value_timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    name_publications (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        citation -> Nullable<Varchar>,
        published_year -> Nullable<Int4>,
        source_url -> Nullable<Varchar>,
        type_citation -> Nullable<Varchar>,
        record_created_at -> Nullable<Timestamptz>,
        record_updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    names (id) {
        id -> Uuid,
        scientific_name -> Varchar,
        canonical_name -> Varchar,
        authorship -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OperationAction;

    nomenclatural_act_logs (operation_id) {
        operation_id -> Numeric,
        parent_id -> Numeric,
        entity_id -> Varchar,
        dataset_version_id -> Uuid,
        action -> OperationAction,
        atom -> Jsonb,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NomenclaturalActType;

    nomenclatural_acts (id) {
        id -> Uuid,
        entity_id -> Varchar,
        publication_id -> Uuid,
        name_id -> Uuid,
        acted_on_id -> Uuid,
        act -> NomenclaturalActType,
        source_url -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    organisms (id) {
        id -> Uuid,
        name_id -> Uuid,
        organism_id -> Nullable<Varchar>,
        organism_name -> Nullable<Varchar>,
        organism_scope -> Nullable<Varchar>,
        associated_organisms -> Nullable<Varchar>,
        previous_identifications -> Nullable<Varchar>,
        remarks -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RegionType;

    regions (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        region_type -> RegionType,
        values -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    sequences (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        dna_extract_id -> Uuid,
        record_id -> Varchar,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    sequencing_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        sequence_id -> Uuid,
        event_date -> Nullable<Varchar>,
        event_time -> Nullable<Varchar>,
        sequenced_by -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        concentration -> Nullable<Float8>,
        amplicon_size -> Nullable<Int8>,
        estimated_size -> Nullable<Varchar>,
        bait_set_name -> Nullable<Varchar>,
        bait_set_reference -> Nullable<Varchar>,
        target_gene -> Nullable<Varchar>,
        dna_sequence -> Nullable<Text>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    sequencing_run_events (id) {
        id -> Uuid,
        sequencing_event_id -> Uuid,
        trace_id -> Nullable<Varchar>,
        trace_name -> Nullable<Varchar>,
        trace_link -> Nullable<Varchar>,
        sequencing_date -> Nullable<Timestamp>,
        sequencing_center -> Nullable<Varchar>,
        sequencing_center_code -> Nullable<Varchar>,
        sequencing_method -> Nullable<Varchar>,
        target_gene -> Nullable<Varchar>,
        direction -> Nullable<Varchar>,
        pcr_primer_name_forward -> Nullable<Varchar>,
        pcr_primer_name_reverse -> Nullable<Varchar>,
        sequence_primer_forward_name -> Nullable<Varchar>,
        sequence_primer_reverse_name -> Nullable<Varchar>,
        library_protocol -> Nullable<Varchar>,
        analysis_description -> Nullable<Varchar>,
        analysis_software -> Nullable<Varchar>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    sources (id) {
        id -> Uuid,
        name -> Varchar,
        author -> Varchar,
        rights_holder -> Varchar,
        access_rights -> Varchar,
        license -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OperationAction;

    specimen_logs (operation_id) {
        operation_id -> Numeric,
        parent_id -> Numeric,
        entity_id -> Varchar,
        dataset_version_id -> Uuid,
        action -> OperationAction,
        atom -> Jsonb,
    }
}

diesel::table! {
    specimens (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        record_id -> Varchar,
        material_sample_id -> Nullable<Varchar>,
        organism_id -> Nullable<Varchar>,
        institution_name -> Nullable<Varchar>,
        institution_code -> Nullable<Varchar>,
        collection_code -> Nullable<Varchar>,
        recorded_by -> Nullable<Varchar>,
        identified_by -> Nullable<Varchar>,
        identified_date -> Nullable<Varchar>,
        type_status -> Nullable<Varchar>,
        locality -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        country_code -> Nullable<Varchar>,
        state_province -> Nullable<Varchar>,
        county -> Nullable<Varchar>,
        municipality -> Nullable<Varchar>,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        elevation -> Nullable<Float8>,
        depth -> Nullable<Float8>,
        elevation_accuracy -> Nullable<Float8>,
        depth_accuracy -> Nullable<Float8>,
        location_source -> Nullable<Varchar>,
        details -> Nullable<Varchar>,
        remarks -> Nullable<Varchar>,
        identification_remarks -> Nullable<Varchar>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    subsample_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        subsample_id -> Uuid,
        event_date -> Nullable<Varchar>,
        event_time -> Nullable<Varchar>,
        subsampled_by -> Nullable<Varchar>,
        preparation_type -> Nullable<Varchar>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    subsamples (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        specimen_id -> Uuid,
        record_id -> Varchar,
        material_sample_id -> Nullable<Varchar>,
        institution_name -> Nullable<Varchar>,
        institution_code -> Nullable<Varchar>,
        type_status -> Nullable<Varchar>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TaxonomicStatus;
    use super::sql_types::TaxonomicRank;

    taxa (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        status -> TaxonomicStatus,
        rank -> TaxonomicRank,
        scientific_name -> Varchar,
        canonical_name -> Varchar,
        authorship -> Nullable<Varchar>,
        nomenclatural_code -> Varchar,
        citation -> Nullable<Varchar>,
        vernacular_names -> Nullable<Array<Nullable<Text>>>,
        description -> Nullable<Text>,
        remarks -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OperationAction;

    taxa_logs (operation_id) {
        operation_id -> Numeric,
        parent_id -> Numeric,
        entity_id -> Varchar,
        dataset_version_id -> Uuid,
        action -> OperationAction,
        atom -> Jsonb,
    }
}

diesel::table! {
    taxon_history (id) {
        id -> Uuid,
        acted_on -> Uuid,
        taxon_id -> Uuid,
        dataset_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        publication_id -> Nullable<Uuid>,
        source_url -> Nullable<Varchar>,
        entity_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    taxon_names (taxon_id, name_id) {
        taxon_id -> Uuid,
        name_id -> Uuid,
    }
}

diesel::table! {
    taxon_photos (id) {
        id -> Uuid,
        taxon_id -> Uuid,
        url -> Varchar,
        source -> Nullable<Varchar>,
        publisher -> Nullable<Varchar>,
        license -> Nullable<Varchar>,
        rights_holder -> Nullable<Varchar>,
        priority -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OperationAction;

    taxonomic_act_logs (operation_id) {
        operation_id -> Numeric,
        parent_id -> Numeric,
        entity_id -> Varchar,
        dataset_version_id -> Uuid,
        action -> OperationAction,
        atom -> Jsonb,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TaxonomicActType;

    taxonomic_acts (id) {
        id -> Uuid,
        entity_id -> Varchar,
        taxon_id -> Uuid,
        accepted_taxon_id -> Nullable<Uuid>,
        act -> TaxonomicActType,
        source_url -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    trace_files (id) {
        id -> Uuid,
        name_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        metadata -> Jsonb,
        peak_locations_user -> Nullable<Array<Nullable<Int4>>>,
        peak_locations_basecaller -> Nullable<Array<Nullable<Int4>>>,
        quality_values_user -> Nullable<Array<Nullable<Int4>>>,
        quality_values_basecaller -> Nullable<Array<Nullable<Int4>>>,
        sequences_user -> Nullable<Array<Nullable<Int4>>>,
        sequences_basecaller -> Nullable<Array<Nullable<Int4>>>,
        measurements_voltage -> Nullable<Array<Nullable<Int4>>>,
        measurements_current -> Nullable<Array<Nullable<Int4>>>,
        measurements_power -> Nullable<Array<Nullable<Int4>>>,
        measurements_temperature -> Nullable<Array<Nullable<Int4>>>,
        analyzed_g -> Nullable<Array<Nullable<Int4>>>,
        analyzed_a -> Nullable<Array<Nullable<Int4>>>,
        analyzed_t -> Nullable<Array<Nullable<Int4>>>,
        analyzed_c -> Nullable<Array<Nullable<Int4>>>,
        raw_g -> Nullable<Array<Nullable<Int4>>>,
        raw_a -> Nullable<Array<Nullable<Int4>>>,
        raw_t -> Nullable<Array<Nullable<Int4>>>,
        raw_c -> Nullable<Array<Nullable<Int4>>>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        user_role -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 255]
        password_salt -> Varchar,
    }
}

diesel::table! {
    vernacular_names (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        vernacular_name -> Varchar,
        citation -> Nullable<Varchar>,
        source_url -> Nullable<Varchar>,
    }
}

diesel::joinable!(accession_events -> datasets (dataset_id));
diesel::joinable!(accession_events -> specimens (specimen_id));
diesel::joinable!(admin_media -> names (name_id));
diesel::joinable!(annotation_events -> datasets (dataset_id));
diesel::joinable!(annotation_events -> sequences (sequence_id));
diesel::joinable!(assemblies -> names (name_id));
diesel::joinable!(assembly_events -> datasets (dataset_id));
diesel::joinable!(assembly_events -> sequences (sequence_id));
diesel::joinable!(assembly_stats -> assemblies (assembly_id));
diesel::joinable!(biosamples -> names (name_id));
diesel::joinable!(collection_event_logs -> dataset_versions (dataset_version_id));
diesel::joinable!(collection_events -> datasets (dataset_id));
diesel::joinable!(collection_events -> specimens (specimen_id));
diesel::joinable!(dataset_versions -> datasets (dataset_id));
diesel::joinable!(datasets -> sources (source_id));
diesel::joinable!(deposition_events -> datasets (dataset_id));
diesel::joinable!(deposition_events -> sequences (sequence_id));
diesel::joinable!(dna_extraction_events -> datasets (dataset_id));
diesel::joinable!(dna_extraction_events -> dna_extracts (dna_extract_id));
diesel::joinable!(dna_extracts -> datasets (dataset_id));
diesel::joinable!(dna_extracts -> names (name_id));
diesel::joinable!(dna_extracts -> subsamples (subsample_id));
diesel::joinable!(ecology -> datasets (dataset_id));
diesel::joinable!(ecology -> names (name_id));
diesel::joinable!(indigenous_knowledge -> datasets (dataset_id));
diesel::joinable!(indigenous_knowledge -> names (name_id));
diesel::joinable!(name_attributes -> datasets (dataset_id));
diesel::joinable!(name_attributes -> names (name_id));
diesel::joinable!(name_publications -> datasets (dataset_id));
diesel::joinable!(nomenclatural_act_logs -> dataset_versions (dataset_version_id));
diesel::joinable!(nomenclatural_acts -> name_publications (publication_id));
diesel::joinable!(organisms -> names (name_id));
diesel::joinable!(regions -> datasets (dataset_id));
diesel::joinable!(regions -> names (name_id));
diesel::joinable!(sequences -> datasets (dataset_id));
diesel::joinable!(sequences -> dna_extracts (dna_extract_id));
diesel::joinable!(sequences -> names (name_id));
diesel::joinable!(sequencing_events -> datasets (dataset_id));
diesel::joinable!(sequencing_events -> sequences (sequence_id));
diesel::joinable!(sequencing_run_events -> sequencing_events (sequencing_event_id));
diesel::joinable!(specimen_logs -> dataset_versions (dataset_version_id));
diesel::joinable!(specimens -> datasets (dataset_id));
diesel::joinable!(specimens -> names (name_id));
diesel::joinable!(subsample_events -> datasets (dataset_id));
diesel::joinable!(subsample_events -> subsamples (subsample_id));
diesel::joinable!(subsamples -> datasets (dataset_id));
diesel::joinable!(subsamples -> names (name_id));
diesel::joinable!(subsamples -> specimens (specimen_id));
diesel::joinable!(taxa -> datasets (dataset_id));
diesel::joinable!(taxa_logs -> dataset_versions (dataset_version_id));
diesel::joinable!(taxon_history -> datasets (dataset_id));
diesel::joinable!(taxon_history -> name_publications (publication_id));
diesel::joinable!(taxon_names -> names (name_id));
diesel::joinable!(taxon_names -> taxa (taxon_id));
diesel::joinable!(taxon_photos -> taxa (taxon_id));
diesel::joinable!(taxonomic_act_logs -> dataset_versions (dataset_version_id));
diesel::joinable!(trace_files -> names (name_id));
diesel::joinable!(vernacular_names -> datasets (dataset_id));
diesel::joinable!(vernacular_names -> names (name_id));

diesel::allow_tables_to_appear_in_same_query!(
    accession_events,
    admin_media,
    annotation_events,
    assemblies,
    assembly_events,
    assembly_stats,
    biosamples,
    collection_event_logs,
    collection_events,
    dataset_versions,
    datasets,
    deposition_events,
    dna_extraction_events,
    dna_extracts,
    ecology,
    ibra,
    imcra_mesoscale,
    imcra_provincial,
    indigenous_knowledge,
    jobs,
    name_attributes,
    name_publications,
    names,
    nomenclatural_act_logs,
    nomenclatural_acts,
    organisms,
    regions,
    sequences,
    sequencing_events,
    sequencing_run_events,
    sources,
    specimen_logs,
    specimens,
    subsample_events,
    subsamples,
    taxa,
    taxa_logs,
    taxon_history,
    taxon_names,
    taxon_photos,
    taxonomic_act_logs,
    taxonomic_acts,
    trace_files,
    users,
    vernacular_names,
);
