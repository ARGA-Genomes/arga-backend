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
    genomic_components (sequence_id) {
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
        release_date -> Nullable<Varchar>,
        deposited_by -> Nullable<Varchar>,
        data_type -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        url -> Nullable<Varchar>,
        source_uri -> Nullable<Varchar>,
        funding_attribution -> Nullable<Varchar>,
        rights_holder -> Nullable<Varchar>,
        access_rights -> Nullable<Varchar>,
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
        markers -> Integer,
        genomes -> Integer,
        specimens -> Integer,
        other -> Integer,
        total_genomic -> Integer,
    }
}

diesel::table! {
    taxa_dag (id) {
        taxon_id -> Uuid,
        taxon_scientific_name -> Varchar,
        taxon_canonical_name -> Varchar,
        id -> Uuid,
        parent_id -> Nullable<Uuid>,
        rank -> crate::schema::sql_types::TaxonomicRank,
        scientific_name -> Varchar,
        canonical_name -> Varchar,
        depth -> Integer,
    }
}

diesel::table! {
    species (id) {
        id -> Uuid,
        scientific_name -> Varchar,
        canonical_name -> Varchar,
        authorship -> Nullable<Varchar>,
        dataset_id -> Uuid,
        status -> crate::schema::sql_types::TaxonomicStatus,
        rank -> crate::schema::sql_types::TaxonomicRank,
        classification -> Jsonb,
        genomes -> BigInt,
        loci -> BigInt,
        specimens -> BigInt,
        other -> BigInt,
        total_genomic -> BigInt,
        traits -> Nullable<Array<Varchar>>,
        vernacular_names -> Nullable<Array<Varchar>>,
    }
}

diesel::table! {
    overview (category, name) {
        category -> Varchar,
        name -> Varchar,
        total -> BigInt,
    }
}


use super::schema::{datasets, names, taxa, specimens, accession_events, name_attributes, taxon_names};

diesel::joinable!(species -> taxa (id));
diesel::joinable!(whole_genomes -> datasets (dataset_id));
diesel::joinable!(whole_genomes -> names (name_id));
diesel::joinable!(markers -> datasets (dataset_id));
diesel::joinable!(markers -> names (name_id));
diesel::joinable!(markers -> taxa (name_id));
diesel::joinable!(specimen_stats -> specimens (id));
diesel::joinable!(name_data_summaries -> names (name_id));
diesel::joinable!(taxon_names -> species (taxon_id));

diesel::allow_tables_to_appear_in_same_query!(
    names,
    species,
    whole_genomes,
    markers,
    name_data_summaries,
    taxa_dag,
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
    species,
    taxon_names,
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
    taxa_dag,
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

diesel::allow_tables_to_appear_in_same_query!(
    name_attributes,
    species,
);

diesel::allow_tables_to_appear_in_same_query!(
    datasets,
    species,
);
