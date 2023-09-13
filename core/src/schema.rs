// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "geometry"))]
    pub struct Geometry;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "job_status"))]
    pub struct JobStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "name_list_type"))]
    pub struct NameListType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "region_type"))]
    pub struct RegionType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "taxonomic_status"))]
    pub struct TaxonomicStatus;
}

diesel::table! {
    accession_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        event_id -> Uuid,
        accession -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        institution_name -> Nullable<Varchar>,
        institution_code -> Nullable<Varchar>,
        type_status -> Nullable<Varchar>,
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
        name_id -> Uuid,
        event_id -> Uuid,
        accession -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        version_status -> Nullable<Varchar>,
        quality -> Nullable<Varchar>,
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
    collection_events (id) {
        id -> Uuid,
        event_id -> Uuid,
        specimen_id -> Uuid,
        organism_id -> Nullable<Uuid>,
        accession -> Nullable<Varchar>,
        catalog_number -> Nullable<Varchar>,
        record_number -> Nullable<Varchar>,
        individual_count -> Nullable<Varchar>,
        organism_quantity -> Nullable<Varchar>,
        organism_quantity_type -> Nullable<Varchar>,
        sex -> Nullable<Varchar>,
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
        ref_biomaterial -> Nullable<Varchar>,
        source_mat_id -> Nullable<Varchar>,
        specific_host -> Nullable<Varchar>,
        strain -> Nullable<Varchar>,
        isolate -> Nullable<Varchar>,
    }
}

diesel::table! {
    conservation_statuses (id) {
        id -> Uuid,
        list_id -> Uuid,
        name_id -> Uuid,
        status -> Varchar,
        state -> Nullable<Varchar>,
        source -> Nullable<Varchar>,
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
    dna_extraction_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        event_id -> Uuid,
        accession -> Nullable<Varchar>,
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
    events (id) {
        id -> Uuid,
        field_number -> Nullable<Varchar>,
        event_date -> Nullable<Date>,
        event_time -> Nullable<Time>,
        habitat -> Nullable<Varchar>,
        sampling_protocol -> Nullable<Varchar>,
        sampling_size_value -> Nullable<Varchar>,
        sampling_size_unit -> Nullable<Varchar>,
        sampling_effort -> Nullable<Varchar>,
        field_notes -> Nullable<Text>,
        event_remarks -> Nullable<Text>,
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
    markers (id) {
        id -> Uuid,
        name_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        accession -> Varchar,
        material_sample_id -> Nullable<Varchar>,
        gb_acs -> Nullable<Varchar>,
        marker_code -> Nullable<Varchar>,
        nucleotide -> Nullable<Text>,
        recorded_by -> Nullable<Varchar>,
        list_id -> Uuid,
        version -> Nullable<Varchar>,
        basepairs -> Nullable<Int8>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        shape -> Nullable<Varchar>,
        source_url -> Nullable<Varchar>,
        fasta_url -> Nullable<Varchar>,
        extra_data -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NameListType;

    name_lists (id) {
        id -> Uuid,
        list_type -> NameListType,
        name -> Varchar,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    name_vernacular_names (name_id, vernacular_name_id) {
        name_id -> Uuid,
        vernacular_name_id -> Int8,
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
    sequencing_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        event_id -> Uuid,
        accession -> Nullable<Varchar>,
        genbank_accession -> Nullable<Varchar>,
        sequenced_by -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        concentration -> Nullable<Float8>,
        amplicon_size -> Nullable<Int8>,
        estimated_size -> Nullable<Int8>,
        bait_set_name -> Nullable<Varchar>,
        bait_set_reference -> Nullable<Varchar>,
        target_gene -> Nullable<Varchar>,
        dna_sequence -> Nullable<Text>,
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
    specimens (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        type_status -> Nullable<Varchar>,
        institution_name -> Nullable<Varchar>,
        institution_code -> Nullable<Varchar>,
        collection_code -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        organism_id -> Nullable<Varchar>,
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
        recorded_by -> Nullable<Varchar>,
        identified_by -> Nullable<Varchar>,
        details -> Nullable<Varchar>,
        remarks -> Nullable<Varchar>,
        identification_remarks -> Nullable<Varchar>,
    }
}

diesel::table! {
    subsample_events (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        event_id -> Uuid,
        accession -> Nullable<Varchar>,
        preparation_type -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TaxonomicStatus;

    taxa (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        status -> TaxonomicStatus,
        scientific_name -> Varchar,
        canonical_name -> Varchar,
        kingdom -> Nullable<Varchar>,
        phylum -> Nullable<Varchar>,
        class -> Nullable<Varchar>,
        order -> Nullable<Varchar>,
        family -> Nullable<Varchar>,
        tribe -> Nullable<Varchar>,
        genus -> Nullable<Varchar>,
        specific_epithet -> Nullable<Varchar>,
        subphylum -> Nullable<Varchar>,
        subclass -> Nullable<Varchar>,
        suborder -> Nullable<Varchar>,
        subfamily -> Nullable<Varchar>,
        subtribe -> Nullable<Varchar>,
        subgenus -> Nullable<Varchar>,
        subspecific_epithet -> Nullable<Varchar>,
        superclass -> Nullable<Varchar>,
        superorder -> Nullable<Varchar>,
        superfamily -> Nullable<Varchar>,
        supertribe -> Nullable<Varchar>,
        order_authority -> Nullable<Varchar>,
        family_authority -> Nullable<Varchar>,
        genus_authority -> Nullable<Varchar>,
        species_authority -> Nullable<Varchar>,
    }
}

diesel::table! {
    taxon_history (id) {
        id -> Uuid,
        old_taxon_id -> Uuid,
        new_taxon_id -> Uuid,
        changed_by -> Nullable<Varchar>,
        reason -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    taxon_photos (id) {
        id -> Uuid,
        name_id -> Uuid,
        url -> Varchar,
        source -> Nullable<Varchar>,
        publisher -> Nullable<Varchar>,
        license -> Nullable<Varchar>,
        rights_holder -> Nullable<Varchar>,
    }
}

diesel::table! {
    taxon_remarks (id) {
        id -> Uuid,
        taxon_id -> Uuid,
        remark -> Varchar,
        created_at -> Timestamptz,
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
        id -> Int8,
        vernacular_name -> Varchar,
        language -> Nullable<Varchar>,
    }
}

diesel::joinable!(accession_events -> datasets (dataset_id));
diesel::joinable!(accession_events -> events (event_id));
diesel::joinable!(accession_events -> names (name_id));
diesel::joinable!(assemblies -> names (name_id));
diesel::joinable!(assembly_events -> datasets (dataset_id));
diesel::joinable!(assembly_events -> events (event_id));
diesel::joinable!(assembly_events -> names (name_id));
diesel::joinable!(assembly_stats -> assemblies (assembly_id));
diesel::joinable!(biosamples -> names (name_id));
diesel::joinable!(collection_events -> events (event_id));
diesel::joinable!(collection_events -> organisms (organism_id));
diesel::joinable!(collection_events -> specimens (specimen_id));
diesel::joinable!(conservation_statuses -> name_lists (list_id));
diesel::joinable!(conservation_statuses -> names (name_id));
diesel::joinable!(datasets -> sources (source_id));
diesel::joinable!(dna_extraction_events -> datasets (dataset_id));
diesel::joinable!(dna_extraction_events -> events (event_id));
diesel::joinable!(dna_extraction_events -> names (name_id));
diesel::joinable!(ecology -> datasets (dataset_id));
diesel::joinable!(ecology -> names (name_id));
diesel::joinable!(indigenous_knowledge -> datasets (dataset_id));
diesel::joinable!(indigenous_knowledge -> names (name_id));
diesel::joinable!(markers -> names (name_id));
diesel::joinable!(name_vernacular_names -> names (name_id));
diesel::joinable!(name_vernacular_names -> vernacular_names (vernacular_name_id));
diesel::joinable!(organisms -> names (name_id));
diesel::joinable!(regions -> datasets (dataset_id));
diesel::joinable!(regions -> names (name_id));
diesel::joinable!(sequencing_events -> datasets (dataset_id));
diesel::joinable!(sequencing_events -> events (event_id));
diesel::joinable!(sequencing_events -> names (name_id));
diesel::joinable!(sequencing_run_events -> sequencing_events (sequencing_event_id));
diesel::joinable!(specimens -> datasets (dataset_id));
diesel::joinable!(specimens -> names (name_id));
diesel::joinable!(subsample_events -> datasets (dataset_id));
diesel::joinable!(subsample_events -> events (event_id));
diesel::joinable!(subsample_events -> names (name_id));
diesel::joinable!(taxa -> datasets (dataset_id));
diesel::joinable!(taxa -> names (name_id));
diesel::joinable!(taxon_photos -> names (name_id));
diesel::joinable!(taxon_remarks -> taxa (taxon_id));
diesel::joinable!(trace_files -> names (name_id));

diesel::allow_tables_to_appear_in_same_query!(
    accession_events,
    assemblies,
    assembly_events,
    assembly_stats,
    biosamples,
    collection_events,
    conservation_statuses,
    datasets,
    dna_extraction_events,
    ecology,
    events,
    ibra,
    imcra_mesoscale,
    imcra_provincial,
    indigenous_knowledge,
    jobs,
    markers,
    name_lists,
    name_vernacular_names,
    names,
    organisms,
    regions,
    sequencing_events,
    sequencing_run_events,
    sources,
    specimens,
    subsample_events,
    taxa,
    taxon_history,
    taxon_photos,
    taxon_remarks,
    trace_files,
    users,
    vernacular_names,
);
