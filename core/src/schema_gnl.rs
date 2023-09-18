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
    species_vernacular_names (id) {
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
        vernacular_names -> Nullable<Array<Text>>,
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
    ranked_taxa (id) {
        id -> Uuid,
        taxa_lists_id -> Uuid,
        name_id -> Uuid,
        scientific_name -> Nullable<Varchar>,
        scientific_name_authorship -> Nullable<Varchar>,
        canonical_name -> Nullable<Varchar>,
        specific_epithet -> Nullable<Varchar>,
        infraspecific_epithet -> Nullable<Varchar>,
        taxon_rank -> Nullable<Text>,
        name_according_to -> Nullable<Text>,
        name_published_in -> Nullable<Text>,
        taxonomic_status -> Nullable<Varchar>,
        taxon_remarks -> Nullable<Text>,
        kingdom -> Nullable<Varchar>,
        phylum -> Nullable<Varchar>,
        class -> Nullable<Varchar>,
        order -> Nullable<Varchar>,
        family -> Nullable<Varchar>,
        genus -> Nullable<Varchar>,
        list_name -> Varchar,
        taxa_priority -> Integer,
    }
}

diesel::table! {
    common_names (id) {
        id -> Uuid,
        vernacular_name -> Varchar,
        vernacular_language -> Nullable<Varchar>,
        scientific_name -> Varchar,
        scientific_name_authorship -> Nullable<Varchar>,
        canonical_name -> Nullable<Varchar>,
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
        ecology -> Nullable<Array<Text>>,
        ibra -> Nullable<Array<Text>>,
        imcra -> Nullable<Array<Text>>,
        state -> Nullable<Array<Text>>,
        drainage_basin -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    whole_genomes (sequence_id) {
        sequence_id -> Uuid,
        dataset_id -> Uuid,
        name_id -> Uuid,
        dna_extract_id -> Uuid,
        dataset_name -> Varchar,
        accession -> Varchar,
        sequenced_by -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        estimated_size -> Nullable<BigInt>,
        assembled_by -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        version_status -> Nullable<Varchar>,
        quality -> Nullable<Varchar>,
        assembly_type -> Nullable<Varchar>,
        genome_size -> Nullable<BigInt>,
        annotated_by -> Nullable<Varchar>,
        representation -> Nullable<Varchar>,
        release_type -> Nullable<Varchar>,
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
        accession -> Varchar,
        sequenced_by -> Nullable<Varchar>,
        material_sample_id -> Nullable<Varchar>,
        target_gene -> Varchar,
    }
}


use super::schema::{datasets, names, assemblies, taxa};

diesel::joinable!(species -> synonyms (id));
diesel::joinable!(species -> species_vernacular_names (id));
diesel::joinable!(taxa -> species (id));
diesel::joinable!(taxa -> synonyms (id));
diesel::joinable!(taxa -> species_vernacular_names (id));
diesel::joinable!(whole_genomes -> datasets (dataset_id));
diesel::joinable!(whole_genomes -> names (name_id));
diesel::joinable!(markers -> datasets (dataset_id));
diesel::joinable!(markers -> names (name_id));
diesel::joinable!(markers -> taxa (name_id));

diesel::joinable!(ranked_taxa -> assemblies (name_id));
diesel::joinable!(ranked_taxa -> names (name_id));

diesel::allow_tables_to_appear_in_same_query!(
    names,
    species,
    synonyms,
    species_vernacular_names,
    undescribed_species,
    whole_genomes,
    markers
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
    species_vernacular_names,
    taxa,
);

diesel::allow_tables_to_appear_in_same_query!(
    ranked_taxa,
    common_names,
);

diesel::allow_tables_to_appear_in_same_query!(
    ranked_taxa,
    names,
);

diesel::allow_tables_to_appear_in_same_query!(
    ranked_taxa,
    assemblies,
);
