diesel::table! {
    species (id) {
        id -> Uuid,
        source -> Uuid,
        name_id -> Uuid,
        status -> crate::schema::sql_types::TaxonomicStatus,
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
        subspecies -> Nullable<Array<Text>>,
        window_rank -> BigInt,
    }
}

diesel::table! {
    synonyms (id) {
        id -> Uuid,
        source -> Uuid,
        name_id -> Uuid,
        status -> crate::schema::sql_types::TaxonomicStatus,
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
        names -> Nullable<Array<Text>>,
        window_rank -> BigInt,
    }
}

diesel::table! {
    common_names (id) {
        id -> Uuid,
        source -> Uuid,
        name_id -> Uuid,
        status -> crate::schema::sql_types::TaxonomicStatus,
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
        names -> Nullable<Array<Text>>,
        window_rank -> BigInt,
    }
}

diesel::table! {
    undescribed_species (genus) {
        genus -> Varchar,
        genus_authority -> Nullable<Varchar>,
        names -> Array<Text>,
    }
}


diesel::table! {
    taxa_filter (id) {
        id -> Uuid,
        name_id -> Uuid,
        status -> crate::schema::sql_types::TaxonomicStatus,
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
        species_authority -> Nullable<Varchar>,
        genomes -> Nullable<Integer>,
        markers -> Nullable<Integer>,
        specimens -> Nullable<Integer>,
        other -> Nullable<Integer>,
        ecology -> Nullable<Array<Text>>,
        ibra -> Nullable<Array<Text>>,
        imcra -> Nullable<Array<Text>>,
        state -> Nullable<Array<Text>>,
        drainage_basin -> Nullable<Array<Text>>,
        traits -> Nullable<Array<Varchar>>,
    }
}

diesel::table! {
    whole_genomes (sequence_id) {
        sequence_id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        dna_extract_id -> Uuid,
        dataset_name -> Varchar,
        record_id -> Varchar,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        accession -> Nullable<Varchar>,
        sequenced_by -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        estimated_size -> Nullable<Varchar>,
        assembled_by -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        version_status -> Nullable<Varchar>,
        quality -> Nullable<Varchar>,
        assembly_type -> Nullable<Varchar>,
        genome_size -> Nullable<BigInt>,
        annotated_by -> Nullable<Varchar>,
        representation -> Nullable<Varchar>,
        release_type -> Nullable<Varchar>,
        release_date -> Nullable<Varchar>,
        deposited_by -> Nullable<Varchar>,
        data_type -> Nullable<Varchar>,
        excluded_from_refseq -> Nullable<Varchar>,
    }
}

diesel::table! {
    markers (sequence_id) {
        sequence_id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        dna_extract_id -> Uuid,
        dataset_name -> Varchar,
        record_id -> Varchar,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        accession -> Nullable<Varchar>,
        sequenced_by -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        target_gene -> Varchar,
        release_date -> Nullable<Varchar>,
    }
}

diesel::table! {
    specimen_stats (id) {
        id -> Uuid,
        sequences -> BigInt,
        whole_genomes -> BigInt,
        markers -> BigInt,
    }
}

diesel::table! {
    name_data_summaries (name_id) {
        name_id -> Uuid,
        // TODO: these are actually BigInt. switch to using
        // bigdecimal and numeric to stop worrying about this
        markers -> Nullable<Integer>,
        genomes -> Nullable<Integer>,
        specimens -> Nullable<Integer>,
        other -> Nullable<Integer>,
    }
}


use super::schema::{datasets, names, taxa, specimens, accession_events};

diesel::joinable!(species -> synonyms (id));
diesel::joinable!(species -> common_names (id));
diesel::joinable!(taxa -> species (id));
diesel::joinable!(taxa -> synonyms (id));
diesel::joinable!(taxa -> common_names (id));
diesel::joinable!(whole_genomes -> datasets (dataset_id));
diesel::joinable!(whole_genomes -> names (name_id));
diesel::joinable!(markers -> datasets (dataset_id));
diesel::joinable!(markers -> names (name_id));
diesel::joinable!(markers -> taxa (name_id));
diesel::joinable!(specimen_stats -> specimens (id));
diesel::joinable!(name_data_summaries -> names (name_id));
diesel::joinable!(taxa_filter -> names (name_id));
diesel::joinable!(name_data_summaries -> taxa_filter (name_id));

diesel::allow_tables_to_appear_in_same_query!(
    names,
    species,
    synonyms,
    common_names,
    undescribed_species,
    whole_genomes,
    markers,
    name_data_summaries,
    taxa_filter,
);

diesel::allow_tables_to_appear_in_same_query!(
    datasets,
    whole_genomes,
);

diesel::allow_tables_to_appear_in_same_query!(
    datasets,
    markers,
);

diesel::allow_tables_to_appear_in_same_query!(
    taxa,
    markers,
);

diesel::allow_tables_to_appear_in_same_query!(
    species,
    taxa,
);

diesel::allow_tables_to_appear_in_same_query!(
    synonyms,
    taxa,
);

diesel::allow_tables_to_appear_in_same_query!(
    common_names,
    taxa,
);

diesel::allow_tables_to_appear_in_same_query!(
    name_data_summaries,
    taxa,
);

diesel::allow_tables_to_appear_in_same_query!(
    whole_genomes,
    taxa,
);

diesel::allow_tables_to_appear_in_same_query!(
    specimen_stats,
    specimens,
);

diesel::allow_tables_to_appear_in_same_query!(
    specimen_stats,
    datasets,
);

diesel::allow_tables_to_appear_in_same_query!(
    specimen_stats,
    accession_events,
);
