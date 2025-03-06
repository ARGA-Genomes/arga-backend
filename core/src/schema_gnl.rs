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
        full_genomes -> Integer,
        partial_genomes -> Integer,
        complete_genomes -> Integer,
        assembly_chromosomes -> Integer,
        assembly_scaffolds -> Integer,
        assembly_contigs -> Integer,
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

diesel::table! {
    taxa_tree_stats (taxon_id, id) {
        taxon_id -> Uuid,
        id -> Uuid,
        tree_depth -> Integer,
        children -> BigInt,
        descendants -> BigInt,
        loci -> Nullable<Numeric>,
        genomes -> Nullable<Numeric>,
        specimens -> Nullable<Numeric>,
        other -> Nullable<Numeric>,
        total_genomic -> Nullable<Numeric>,
        species -> Nullable<BigInt>,

        full_genomes -> Nullable<Numeric>,
        partial_genomes -> Nullable<Numeric>,
        complete_genomes -> Nullable<Numeric>,
        assembly_chromosomes -> Nullable<Numeric>,
        assembly_scaffolds -> Nullable<Numeric>,
        assembly_contigs -> Nullable<Numeric>,

        total_full_genomes_coverage -> BigInt,
        total_partial_genomes_coverage -> BigInt,
        total_complete_genomes_coverage -> BigInt,
        total_assembly_chromosomes_coverage -> BigInt,
        total_assembly_scaffolds_coverage -> BigInt,
        total_assembly_contigs_coverage -> BigInt,
    }
}

diesel::table! {
    taxa_tree (id, path_id, id) {
        taxon_id -> Uuid,
        path_id -> Uuid,
        id -> Uuid,
        parent_id -> Uuid,
        depth -> BigInt,
    }
}

diesel::table! {
    sequence_milestones (name_id, representation) {
        name_id -> Uuid,
        representation -> Varchar,
        sequencing_date -> Nullable<Varchar>,
        assembly_date -> Nullable<Varchar>,
        annotation_date -> Nullable<Varchar>,
        deposition_date -> Nullable<Varchar>,
    }
}


use super::schema::{
    accession_events,
    assembly_events,
    datasets,
    deposition_events,
    name_attributes,
    names,
    sequences,
    specimens,
    taxa,
    taxon_names,
};

diesel::joinable!(species -> taxa (id));
diesel::joinable!(whole_genomes -> datasets (dataset_id));
diesel::joinable!(whole_genomes -> names (name_id));
diesel::joinable!(markers -> datasets (dataset_id));
diesel::joinable!(markers -> names (name_id));
diesel::joinable!(markers -> taxa (name_id));
diesel::joinable!(specimen_stats -> specimens (id));
diesel::joinable!(name_data_summaries -> names (name_id));
diesel::joinable!(taxon_names -> species (taxon_id));
diesel::joinable!(taxa_tree_stats -> taxa (taxon_id));

diesel::allow_tables_to_appear_in_same_query!(
    names,
    species,
    whole_genomes,
    markers,
    name_data_summaries,
    taxa_dag,
    taxa_tree,
    taxa_tree_stats,
    sequence_milestones,
);

diesel::allow_tables_to_appear_in_same_query!(datasets, whole_genomes);
diesel::allow_tables_to_appear_in_same_query!(datasets, markers);
diesel::allow_tables_to_appear_in_same_query!(datasets, species);
diesel::allow_tables_to_appear_in_same_query!(datasets, specimen_stats);

diesel::allow_tables_to_appear_in_same_query!(species, taxon_names);
diesel::allow_tables_to_appear_in_same_query!(species, assembly_events);
diesel::allow_tables_to_appear_in_same_query!(species, deposition_events);
diesel::allow_tables_to_appear_in_same_query!(species, sequences);
diesel::allow_tables_to_appear_in_same_query!(specimen_stats, specimens);
diesel::allow_tables_to_appear_in_same_query!(specimen_stats, accession_events);

diesel::allow_tables_to_appear_in_same_query!(name_attributes, species);

diesel::allow_tables_to_appear_in_same_query!(taxa, taxa_dag);
diesel::allow_tables_to_appear_in_same_query!(taxa, taxa_tree);
diesel::allow_tables_to_appear_in_same_query!(taxa, taxa_tree_stats);
diesel::allow_tables_to_appear_in_same_query!(taxa, species);
diesel::allow_tables_to_appear_in_same_query!(taxa, markers);
diesel::allow_tables_to_appear_in_same_query!(taxa, whole_genomes);
diesel::allow_tables_to_appear_in_same_query!(taxa, name_data_summaries);

diesel::allow_tables_to_appear_in_same_query!(sequence_milestones, taxon_names);
diesel::allow_tables_to_appear_in_same_query!(sequence_milestones, datasets);
